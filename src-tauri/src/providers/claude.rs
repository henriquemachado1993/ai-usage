use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode};
use serde_json::{Value, json};
use tokio::sync::Mutex;

use super::{Fetched, ProviderError, ProviderResult, capitalize, home, resolve_dir};
use crate::model::{Account, UsageWindow};

const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
const TOKEN_URL: &str = "https://platform.claude.com/v1/oauth/token";
const CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const CREDENTIALS_FILE: &str = ".credentials.json";
const EXPIRY_SKEW_MS: i64 = 60_000;

static REFRESH_LOCK: Mutex<()> = Mutex::const_new(());
const WINDOWS: &[(&str, &str)] = &[
    ("five_hour", "Sessão (5h)"),
    ("seven_day", "Semanal"),
    ("seven_day_opus", "Semanal Opus"),
    ("seven_day_sonnet", "Semanal Sonnet"),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Source {
    File,
    Keychain,
}

#[derive(Debug)]
struct Credentials {
    access_token: String,
    refresh_token: Option<String>,
    expires_at: Option<i64>,
    scopes: Vec<String>,
    subscription: Option<String>,
    source: Source,
}

impl Credentials {
    fn is_expired(&self, now_ms: i64) -> bool {
        self.expires_at
            .is_some_and(|exp| exp - EXPIRY_SKEW_MS <= now_ms)
    }
}

fn config_dir(account: Option<&Account>) -> (PathBuf, bool) {
    let custom = account.and_then(|a| a.config_dir.as_deref());
    let is_custom = custom.is_some_and(|c| !c.trim().is_empty())
        || std::env::var("CLAUDE_CONFIG_DIR").is_ok_and(|v| !v.is_empty());
    (
        resolve_dir(custom, "CLAUDE_CONFIG_DIR", ".claude"),
        is_custom,
    )
}

pub fn detect() -> bool {
    let (dir, _) = config_dir(None);
    dir.join(CREDENTIALS_FILE).exists() || (cfg!(target_os = "macos") && dir.exists())
}

pub async fn fetch(client: &Client, account: &Account) -> ProviderResult {
    let (dir, is_custom) = config_dir(Some(account));
    let mut creds = read_credentials(&dir, is_custom).await.ok_or_else(|| {
        ProviderError::Other(format!(
            "Credenciais do Claude Code não encontradas em {}",
            dir.display()
        ))
    })?;

    if creds.is_expired(Utc::now().timestamp_millis()) {
        creds = refresh(client, &dir, creds).await?;
    }

    let mut resp = request_usage(client, &creds.access_token).await?;
    if resp.status() == StatusCode::UNAUTHORIZED && creds.source == Source::File {
        creds = refresh(client, &dir, creds).await?;
        resp = request_usage(client, &creds.access_token).await?;
    }

    match resp.status() {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            return Err(ProviderError::Expired(
                "Sessão inválida — rode `claude` no terminal e faça /login".into(),
            ));
        }
        s if !s.is_success() => return Err(ProviderError::Other(format!("API respondeu {s}"))),
        _ => {}
    }

    let body: Value = resp.json().await?;
    Ok(Fetched {
        identity: read_identity(&dir, is_custom),
        plan: creds.subscription.map(|s| capitalize(&s)),
        windows: parse_usage(&body),
        stale_reason: None,
    })
}

async fn request_usage(client: &Client, token: &str) -> Result<reqwest::Response, ProviderError> {
    Ok(client
        .get(USAGE_URL)
        .bearer_auth(token)
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("Accept", "application/json")
        .send()
        .await?)
}

async fn refresh(
    client: &Client,
    dir: &Path,
    creds: Credentials,
) -> Result<Credentials, ProviderError> {
    let relogin = || {
        ProviderError::Expired(
            "Login do Claude expirado — rode `claude` no terminal e faça /login".into(),
        )
    };
    if creds.source != Source::File {
        return Err(ProviderError::Expired(
            "Token expirado — rode `claude` no terminal para renová-lo".into(),
        ));
    }

    let _guard = REFRESH_LOCK.lock().await;
    let path = dir.join(CREDENTIALS_FILE);
    let text = std::fs::read_to_string(&path).map_err(|e| ProviderError::Other(e.to_string()))?;
    let current = parse_credentials(&text, Source::File).ok_or_else(relogin)?;
    if current.access_token != creds.access_token
        && !current.is_expired(Utc::now().timestamp_millis())
    {
        return Ok(current);
    }
    let refresh_token = current.refresh_token.clone().ok_or_else(relogin)?;

    let mut body = json!({
        "grant_type": "refresh_token",
        "refresh_token": refresh_token,
        "client_id": CLIENT_ID,
    });
    if !current.scopes.is_empty() {
        body["scope"] = current.scopes.join(" ").into();
    }
    let resp = client.post(TOKEN_URL).json(&body).send().await?;
    match resp.status() {
        StatusCode::BAD_REQUEST | StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            return Err(relogin());
        }
        s if !s.is_success() => {
            return Err(ProviderError::Other(format!(
                "Falha ao renovar o token do Claude ({s})"
            )));
        }
        _ => {}
    }

    let tokens: Value = resp.json().await?;
    let updated =
        apply_refresh(&text, &tokens, Utc::now().timestamp_millis()).ok_or_else(|| {
            ProviderError::Other("Resposta inesperada ao renovar o token do Claude".into())
        })?;
    write_atomic(&path, &updated)
        .map_err(|e| ProviderError::Other(format!("Falha ao salvar o token renovado: {e}")))?;
    parse_credentials(&updated, Source::File).ok_or_else(relogin)
}

fn apply_refresh(text: &str, tokens: &Value, now_ms: i64) -> Option<String> {
    let mut root: Value = serde_json::from_str(text).ok()?;
    let oauth = root.get_mut("claudeAiOauth")?.as_object_mut()?;
    let access = tokens.get("access_token")?.as_str()?;
    oauth.insert("accessToken".into(), access.into());
    if let Some(refresh) = tokens.get("refresh_token").and_then(Value::as_str) {
        oauth.insert("refreshToken".into(), refresh.into());
    }
    if let Some(expires_in) = tokens.get("expires_in").and_then(Value::as_i64) {
        oauth.insert("expiresAt".into(), (now_ms + expires_in * 1000).into());
    }
    if let Some(scope) = tokens.get("scope").and_then(Value::as_str) {
        oauth.insert(
            "scopes".into(),
            scope.split_whitespace().collect::<Vec<_>>().into(),
        );
    }
    serde_json::to_string(&root).ok()
}

fn write_atomic(path: &Path, text: &str) -> std::io::Result<()> {
    let tmp = path.with_extension("json.ai-usage.tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path)
}

async fn read_credentials(dir: &Path, is_custom: bool) -> Option<Credentials> {
    if cfg!(target_os = "macos")
        && let Some(c) = read_keychain(dir, is_custom).await
    {
        return Some(c);
    }
    let text = std::fs::read_to_string(dir.join(CREDENTIALS_FILE)).ok()?;
    parse_credentials(&text, Source::File)
}

async fn read_keychain(dir: &Path, is_custom: bool) -> Option<Credentials> {
    let service = if is_custom {
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(dir.to_string_lossy().as_bytes());
        let hex: String = hash.iter().take(4).map(|b| format!("{b:02x}")).collect();
        format!("Claude Code-credentials-{hex}")
    } else {
        "Claude Code-credentials".to_string()
    };
    let out = super::command("/usr/bin/security")
        .args(["find-generic-password", "-s", &service, "-w"])
        .output()
        .await
        .ok()?;
    if !out.status.success() {
        return None;
    }
    parse_credentials(
        String::from_utf8_lossy(&out.stdout).trim(),
        Source::Keychain,
    )
}

fn parse_credentials(text: &str, source: Source) -> Option<Credentials> {
    let v: Value = serde_json::from_str(text).ok()?;
    let oauth = v.get("claudeAiOauth")?;
    let string = |key: &str| oauth.get(key).and_then(Value::as_str).map(str::to_string);
    Some(Credentials {
        access_token: string("accessToken")?,
        refresh_token: string("refreshToken"),
        expires_at: oauth.get("expiresAt").and_then(Value::as_i64),
        scopes: oauth
            .get("scopes")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
        subscription: string("subscriptionType"),
        source,
    })
}

fn read_identity(dir: &Path, is_custom: bool) -> Option<String> {
    let candidates = if is_custom {
        vec![dir.join(".claude.json")]
    } else {
        vec![home().join(".claude.json"), dir.join(".claude.json")]
    };
    candidates.iter().find_map(|p| {
        let v: Value = serde_json::from_str(&std::fs::read_to_string(p).ok()?).ok()?;
        v.pointer("/oauthAccount/emailAddress")?
            .as_str()
            .map(str::to_string)
    })
}

pub fn parse_usage(body: &Value) -> Vec<UsageWindow> {
    let mut windows: Vec<UsageWindow> = WINDOWS
        .iter()
        .filter_map(|(key, name)| {
            let w = body.get(*key)?;
            Some(UsageWindow {
                name: (*name).into(),
                used_percent: w.get("utilization")?.as_f64()?,
                resets_at: parse_time(w.get("resets_at")),
                ..Default::default()
            })
        })
        .collect();

    if let Some(extra) = body.get("extra_usage")
        && extra.get("is_enabled").and_then(Value::as_bool) == Some(true)
        && let Some(util) = extra.get("utilization").and_then(Value::as_f64)
    {
        windows.push(UsageWindow {
            name: "Uso extra (mensal)".into(),
            used_percent: util,
            ..Default::default()
        });
    }
    windows
}

fn parse_time(v: Option<&Value>) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(v?.as_str()?)
        .ok()
        .map(|d| d.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_usage_windows() {
        let body = json!({
            "five_hour": { "utilization": 42.0, "resets_at": "2026-10-26T18:00:00.000+00:00" },
            "seven_day": { "utilization": 13.5, "resets_at": "2026-10-30T10:00:00+00:00" },
            "seven_day_opus": null,
            "extra_usage": { "is_enabled": false, "utilization": null }
        });
        let w = parse_usage(&body);
        assert_eq!(w.len(), 2);
        assert_eq!(w[0].name, "Sessão (5h)");
        assert_eq!(w[0].used_percent, 42.0);
        assert_eq!(
            w[0].resets_at.unwrap().to_rfc3339(),
            "2026-10-26T18:00:00+00:00"
        );
        assert_eq!(w[1].used_percent, 13.5);
    }

    #[test]
    fn parses_credentials_file() {
        let text = r#"{"claudeAiOauth":{"accessToken":"abc","refreshToken":"r","expiresAt":1700000000000,"scopes":["user:profile","user:inference"],"subscriptionType":"max"}}"#;
        let c = parse_credentials(text, Source::File).unwrap();
        assert_eq!(c.access_token, "abc");
        assert_eq!(c.refresh_token.as_deref(), Some("r"));
        assert_eq!(c.expires_at, Some(1_700_000_000_000));
        assert_eq!(c.scopes, vec!["user:profile", "user:inference"]);
        assert_eq!(c.subscription.as_deref(), Some("max"));
        assert!(c.is_expired(1_700_000_000_000));
        assert!(!c.is_expired(1_699_000_000_000));
        assert!(parse_credentials("{}", Source::File).is_none());
    }

    #[test]
    fn applies_refreshed_tokens_preserving_fields() {
        let text = r#"{"claudeAiOauth":{"accessToken":"old","refreshToken":"r1","expiresAt":1,"refreshTokenExpiresAt":99,"scopes":["user:profile"],"subscriptionType":"team","rateLimitTier":"t"},"other":true}"#;
        let tokens = json!({ "access_token": "new", "refresh_token": "r2", "expires_in": 28800 });
        let updated: Value =
            serde_json::from_str(&apply_refresh(text, &tokens, 1_000).unwrap()).unwrap();
        let o = &updated["claudeAiOauth"];
        assert_eq!(o["accessToken"], "new");
        assert_eq!(o["refreshToken"], "r2");
        assert_eq!(o["expiresAt"], 1_000 + 28_800_000);
        assert_eq!(o["refreshTokenExpiresAt"], 99);
        assert_eq!(o["subscriptionType"], "team");
        assert_eq!(o["rateLimitTier"], "t");
        assert_eq!(o["scopes"], json!(["user:profile"]));
        assert_eq!(updated["other"], true);
        assert!(apply_refresh(text, &json!({ "error": "invalid_grant" }), 0).is_none());
    }
}
