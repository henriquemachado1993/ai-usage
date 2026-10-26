use std::path::PathBuf;

use chrono::{DateTime, NaiveDate, Utc};
use reqwest::{Client, StatusCode};
use serde_json::Value;

use super::{Fetched, ProviderError, ProviderResult, capitalize, home};
use crate::model::UsageWindow;

const USER_URL: &str = "https://api.github.com/copilot_internal/user";
const QUOTAS: &[(&str, &str)] = &[
    ("premium_interactions", "Premium requests"),
    ("chat", "Chat"),
    ("completions", "Completions"),
];
const GH_CANDIDATES: &[&str] = &[
    "gh",
    "/opt/homebrew/bin/gh",
    "/usr/local/bin/gh",
    "/usr/bin/gh",
];

pub async fn detect() -> bool {
    gh_token().await.is_some()
}

pub async fn fetch(client: &Client, secret: Option<String>) -> ProviderResult {
    let token = match secret {
        Some(t) => t,
        None => gh_token().await.ok_or_else(|| {
            ProviderError::Other(
                "Token do GitHub não encontrado — rode `gh auth login` ou adicione um token".into(),
            )
        })?,
    };

    let resp = client
        .get(USER_URL)
        .header("Authorization", format!("token {token}"))
        .header("Accept", "application/json")
        .header("X-GitHub-Api-Version", "2025-04-01")
        .send()
        .await?;

    match resp.status() {
        StatusCode::UNAUTHORIZED => {
            return Err(ProviderError::Expired(
                "Token do GitHub inválido ou expirado".into(),
            ));
        }
        StatusCode::NOT_FOUND | StatusCode::FORBIDDEN => {
            return Err(ProviderError::Other(
                "Esta conta do GitHub não tem Copilot ativo".into(),
            ));
        }
        s if !s.is_success() => return Err(ProviderError::Other(format!("API respondeu {s}"))),
        _ => {}
    }

    let body: Value = resp.json().await?;
    Ok(parse_user(&body))
}

pub fn parse_user(body: &Value) -> Fetched {
    let identity = body
        .get("login")
        .and_then(Value::as_str)
        .map(str::to_string);
    let plan = body
        .get("copilot_plan")
        .and_then(Value::as_str)
        .map(|p| capitalize(&p.replace('_', " ")));
    let reset = parse_reset(
        body.get("quota_reset_date_utc")
            .or(body.get("quota_reset_date")),
    )
    .or_else(|| parse_reset(body.get("limited_user_reset_date")));

    let mut windows: Vec<UsageWindow> = Vec::new();
    if let Some(snapshots) = body.get("quota_snapshots").filter(|s| s.is_object()) {
        for (key, name) in QUOTAS {
            let Some(q) = snapshots.get(*key).filter(|q| q.is_object()) else {
                continue;
            };
            let unlimited = q.get("unlimited").and_then(Value::as_bool).unwrap_or(false);
            let limit = q.get("entitlement").and_then(Value::as_f64);
            let remaining = q.get("remaining").and_then(Value::as_f64);
            let percent_remaining = q.get("percent_remaining").and_then(Value::as_f64);
            if unlimited && *key != "premium_interactions" {
                continue;
            }
            let used_percent = if unlimited {
                0.0
            } else {
                percent_remaining
                    .map(|p| 100.0 - p)
                    .or_else(|| match (limit, remaining) {
                        (Some(l), Some(r)) if l > 0.0 => Some((l - r) / l * 100.0),
                        _ => None,
                    })
                    .unwrap_or(0.0)
            };
            windows.push(UsageWindow {
                name: (*name).into(),
                used_percent: used_percent.clamp(0.0, 100.0),
                resets_at: reset,
                used: match (limit, remaining) {
                    _ if unlimited => q.get("credits_used").and_then(Value::as_f64),
                    (Some(l), Some(r)) => Some((l - r).max(0.0)),
                    _ => None,
                },
                limit: if unlimited { None } else { limit },
                unlimited,
            });
        }
    } else if let (Some(remaining), Some(monthly)) =
        (body.get("limited_user_quotas"), body.get("monthly_quotas"))
    {
        for (key, name) in QUOTAS {
            let (Some(r), Some(l)) = (
                remaining.get(*key).and_then(Value::as_f64),
                monthly.get(*key).and_then(Value::as_f64),
            ) else {
                continue;
            };
            if l <= 0.0 {
                continue;
            }
            windows.push(UsageWindow {
                name: (*name).into(),
                used_percent: ((l - r) / l * 100.0).clamp(0.0, 100.0),
                resets_at: reset,
                used: Some((l - r).max(0.0)),
                limit: Some(l),
                unlimited: false,
            });
        }
    }

    Fetched {
        identity,
        plan,
        windows,
        stale_reason: None,
    }
}

fn parse_reset(v: Option<&Value>) -> Option<DateTime<Utc>> {
    let s = v?.as_str()?;
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .ok()
        .or_else(|| {
            NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .ok()?
                .and_hms_opt(0, 0, 0)
                .map(|d| d.and_utc())
        })
}

async fn gh_token() -> Option<String> {
    for program in GH_CANDIDATES {
        let Ok(out) = super::command(program)
            .args(["auth", "token", "--hostname", "github.com"])
            .output()
            .await
        else {
            continue;
        };
        let token = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if out.status.success() && !token.is_empty() {
            return Some(token);
        }
    }
    hosts_file_token()
}

fn hosts_file_token() -> Option<String> {
    let dir = std::env::var("GH_CONFIG_DIR")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            if cfg!(windows) {
                dirs::config_dir().unwrap_or_else(home).join("GitHub CLI")
            } else {
                home().join(".config").join("gh")
            }
        });
    let text = std::fs::read_to_string(dir.join("hosts.yml")).ok()?;
    text.lines()
        .find_map(|l| l.trim().strip_prefix("oauth_token:"))
        .map(|t| t.trim().trim_matches('"').to_string())
        .filter(|t| !t.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_paid_plan_snapshots() {
        let body = json!({
            "login": "octocat",
            "copilot_plan": "individual_pro",
            "quota_reset_date": "2026-11-01",
            "quota_snapshots": {
                "chat": { "entitlement": 0, "remaining": 0, "percent_remaining": 100, "unlimited": true },
                "completions": { "entitlement": 0, "remaining": 0, "percent_remaining": 100, "unlimited": true },
                "premium_interactions": { "entitlement": 300, "remaining": 225, "percent_remaining": 75.0, "unlimited": false }
            }
        });
        let f = parse_user(&body);
        assert_eq!(f.identity.as_deref(), Some("octocat"));
        assert_eq!(f.plan.as_deref(), Some("Individual pro"));
        assert_eq!(f.windows.len(), 1);
        let w = &f.windows[0];
        assert_eq!(w.name, "Premium requests");
        assert_eq!(w.used_percent, 25.0);
        assert_eq!(w.used, Some(75.0));
        assert_eq!(w.limit, Some(300.0));
        assert_eq!(
            w.resets_at.unwrap().to_rfc3339(),
            "2026-11-01T00:00:00+00:00"
        );
    }

    #[test]
    fn parses_token_based_billing() {
        let body = json!({
            "copilot_plan": "business",
            "quota_snapshots": {
                "premium_interactions": { "entitlement": 0, "remaining": 0, "percent_remaining": 100, "unlimited": true, "credits_used": 1119 }
            }
        });
        let w = &parse_user(&body).windows[0];
        assert!(w.unlimited);
        assert_eq!(w.used_percent, 0.0);
        assert_eq!(w.used, Some(1119.0));
        assert_eq!(w.limit, None);
    }

    #[test]
    fn parses_free_plan_quotas() {
        let body = json!({
            "login": "free-user",
            "limited_user_quotas": { "chat": 40, "completions": 1500 },
            "monthly_quotas": { "chat": 50, "completions": 2000 },
            "limited_user_reset_date": "2026-11-10"
        });
        let f = parse_user(&body);
        assert_eq!(f.windows.len(), 2);
        assert_eq!(f.windows[0].name, "Chat");
        assert_eq!(f.windows[0].used_percent, 20.0);
        assert_eq!(f.windows[1].used, Some(500.0));
    }
}
