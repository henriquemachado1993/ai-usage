use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Duration, Utc};
use reqwest::{Client, StatusCode};
use serde_json::Value;

use super::{
    Fetched, ProviderError, ProviderResult, capitalize, jwt_claims, resolve_dir,
    window_name_from_seconds,
};
use crate::model::{Account, UsageWindow};

const USAGE_URL: &str = "https://chatgpt.com/backend-api/wham/usage";

fn codex_home(account: Option<&Account>) -> PathBuf {
    resolve_dir(
        account.and_then(|a| a.config_dir.as_deref()),
        "CODEX_HOME",
        ".codex",
    )
}

pub fn detect() -> bool {
    codex_home(None).join("auth.json").exists()
}

pub async fn fetch(client: &Client, account: &Account) -> ProviderResult {
    let home = codex_home(Some(account));
    let auth: Value = fs::read_to_string(home.join("auth.json"))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .ok_or_else(|| {
            ProviderError::Other(format!(
                "auth.json do Codex não encontrado em {}",
                home.display()
            ))
        })?;

    let Some(tokens) = auth.get("tokens").filter(|t| t.is_object()) else {
        return Err(ProviderError::Other(
            "Codex logado via API key — não há limites de plano para exibir".into(),
        ));
    };
    let access_token = tokens
        .get("access_token")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let account_id = tokens.get("account_id").and_then(Value::as_str);
    let claims = tokens
        .get("id_token")
        .and_then(Value::as_str)
        .and_then(jwt_claims);
    let identity = claims
        .as_ref()
        .and_then(|c| c.get("email"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let jwt_plan = claims
        .as_ref()
        .and_then(|c| c.pointer("/https:~1~1api.openai.com~1auth/chatgpt_plan_type"))
        .and_then(Value::as_str)
        .map(capitalize);

    let mut req = client
        .get(USAGE_URL)
        .bearer_auth(access_token)
        .header("Accept", "application/json");
    if let Some(id) = account_id {
        req = req.header("ChatGPT-Account-Id", id);
    }

    let api_failure = match req.send().await {
        Ok(resp) if resp.status().is_success() => {
            let body: Value = resp.json().await?;
            let (plan, windows) = parse_api_usage(&body);
            return Ok(Fetched {
                identity,
                plan: plan.or(jwt_plan),
                windows,
                stale_reason: None,
            });
        }
        Ok(resp)
            if matches!(
                resp.status(),
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
            ) =>
        {
            None
        }
        Ok(resp) => Some(format!("API respondeu {}", resp.status())),
        Err(e) => Some(format!("falha de rede: {e}")),
    };

    let local = latest_session_limits(&home.join("sessions"));
    match (local, api_failure) {
        (Some(windows), None) => Ok(Fetched {
            identity,
            plan: jwt_plan,
            windows,
            stale_reason: Some("Token expirado — exibindo o último uso registrado localmente. Use o Codex para renovar.".into()),
        }),
        (Some(windows), Some(err)) => Ok(Fetched {
            identity,
            plan: jwt_plan,
            windows,
            stale_reason: Some(format!("API indisponível ({err}) — exibindo o último uso registrado localmente.")),
        }),
        (None, None) => Err(ProviderError::Expired("Sessão expirada — rode o Codex para renovar o login".into())),
        (None, Some(err)) => Err(ProviderError::Other(err)),
    }
}

pub fn parse_api_usage(body: &Value) -> (Option<String>, Vec<UsageWindow>) {
    let plan = body
        .get("plan_type")
        .and_then(Value::as_str)
        .map(capitalize);
    let windows = ["primary_window", "secondary_window"]
        .iter()
        .filter_map(|key| {
            let w = body.get("rate_limit")?.get(*key)?;
            let used = w.get("used_percent")?.as_f64()?;
            let seconds = w.get("limit_window_seconds").and_then(Value::as_i64);
            let resets_at = w
                .get("reset_at")
                .and_then(Value::as_i64)
                .and_then(|t| DateTime::from_timestamp(t, 0))
                .or_else(|| {
                    w.get("reset_after_seconds")
                        .and_then(Value::as_i64)
                        .map(|s| Utc::now() + Duration::seconds(s))
                });
            Some(UsageWindow {
                name: seconds
                    .map(window_name_from_seconds)
                    .unwrap_or_else(|| default_name(key)),
                used_percent: used,
                resets_at,
                ..Default::default()
            })
        })
        .collect();
    (plan, windows)
}

fn default_name(key: &str) -> String {
    if key.starts_with("primary") {
        "Sessão (5h)".into()
    } else {
        "Semanal".into()
    }
}

fn latest_session_limits(dir: &Path) -> Option<Vec<UsageWindow>> {
    let mut files = Vec::new();
    collect_rollouts(dir, &mut files);
    files.sort_by_key(|(modified, _)| std::cmp::Reverse(*modified));
    files.iter().take(5).find_map(|(_, path)| {
        let text = fs::read_to_string(path).ok()?;
        text.lines()
            .rev()
            .find_map(|line| parse_session_line(line, Utc::now()))
    })
}

fn collect_rollouts(dir: &Path, out: &mut Vec<(SystemTime, PathBuf)>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rollouts(&path, out);
        } else if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("rollout-") && n.ends_with(".jsonl"))
            && let Ok(modified) = entry.metadata().and_then(|m| m.modified())
        {
            out.push((modified, path));
        }
    }
}

pub fn parse_session_line(line: &str, now: DateTime<Utc>) -> Option<Vec<UsageWindow>> {
    if !line.contains("rate_limits") {
        return None;
    }
    let v: Value = serde_json::from_str(line).ok()?;
    let payload = v.get("payload")?;
    if payload.get("type")?.as_str()? != "token_count" {
        return None;
    }
    let limits = payload.get("rate_limits").filter(|l| l.is_object())?;
    let logged_at = v
        .get("timestamp")
        .and_then(Value::as_str)
        .and_then(|t| DateTime::parse_from_rfc3339(t).ok())
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or(now);

    let windows: Vec<UsageWindow> = ["primary", "secondary"]
        .iter()
        .filter_map(|key| {
            let w = limits.get(*key).filter(|w| w.is_object())?;
            let mut used = w.get("used_percent")?.as_f64()?;
            let minutes = w.get("window_minutes").and_then(Value::as_i64);
            let mut resets_at = w
                .get("resets_at")
                .and_then(Value::as_i64)
                .and_then(|t| DateTime::from_timestamp(t, 0))
                .or_else(|| {
                    w.get("resets_in_seconds")
                        .and_then(Value::as_i64)
                        .map(|s| logged_at + Duration::seconds(s))
                });
            if resets_at.is_some_and(|r| r <= now) {
                used = 0.0;
                resets_at = None;
            }
            Some(UsageWindow {
                name: minutes
                    .map(|m| window_name_from_seconds(m * 60))
                    .unwrap_or_else(|| default_name(key)),
                used_percent: used,
                resets_at,
                ..Default::default()
            })
        })
        .collect();
    (!windows.is_empty()).then_some(windows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_api_payload() {
        let body = json!({
            "plan_type": "plus",
            "rate_limit": {
                "primary_window": { "used_percent": 37, "limit_window_seconds": 18000, "reset_at": 1793030400 },
                "secondary_window": { "used_percent": 8.5, "limit_window_seconds": 604800, "reset_after_seconds": 3600 }
            }
        });
        let (plan, w) = parse_api_usage(&body);
        assert_eq!(plan.as_deref(), Some("Plus"));
        assert_eq!(w.len(), 2);
        assert_eq!(w[0].name, "Sessão (5h)");
        assert_eq!(w[0].used_percent, 37.0);
        assert_eq!(w[0].resets_at.unwrap().timestamp(), 1793030400);
        assert_eq!(w[1].name, "Semanal");
        assert!(w[1].resets_at.is_some());
    }

    #[test]
    fn parses_session_token_count() {
        let now = DateTime::parse_from_rfc3339("2026-10-26T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let line = r#"{"timestamp":"2026-10-26T11:30:00Z","type":"event_msg","payload":{"type":"token_count","info":null,"rate_limits":{"primary":{"used_percent":55.0,"window_minutes":300,"resets_in_seconds":7200},"secondary":{"used_percent":20.0,"window_minutes":10080,"resets_in_seconds":60}}}}"#;
        let w = parse_session_line(line, now).unwrap();
        assert_eq!(w[0].name, "Sessão (5h)");
        assert_eq!(w[0].used_percent, 55.0);
        assert_eq!(w[1].name, "Semanal");
        assert_eq!(w[1].used_percent, 0.0, "janela já resetada deve zerar");
        assert!(parse_session_line(r#"{"payload":{"type":"agent_message"}}"#, now).is_none());
    }
}
