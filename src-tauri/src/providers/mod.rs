pub mod claude;
pub mod codex;
pub mod copilot;

use std::path::PathBuf;

use base64::Engine;
use chrono::Utc;
use reqwest::Client;

use crate::model::{Account, ProviderKind, Status, UsageSnapshot, UsageWindow};

#[derive(Debug, Default)]
pub struct Fetched {
    pub identity: Option<String>,
    pub plan: Option<String>,
    pub windows: Vec<UsageWindow>,
    pub stale_reason: Option<String>,
}

#[derive(Debug)]
pub enum ProviderError {
    Expired(String),
    Other(String),
}

impl From<reqwest::Error> for ProviderError {
    fn from(e: reqwest::Error) -> Self {
        ProviderError::Other(format!("Falha de rede: {e}"))
    }
}

pub type ProviderResult = Result<Fetched, ProviderError>;

pub async fn fetch(client: &Client, account: &Account, secret: Option<String>) -> UsageSnapshot {
    let result = match account.provider {
        ProviderKind::Claude => claude::fetch(client, account).await,
        ProviderKind::Codex => codex::fetch(client, account).await,
        ProviderKind::Copilot => copilot::fetch(client, secret).await,
    };

    let mut snapshot = placeholder(account, Status::Ok);
    match result {
        Ok(f) => {
            snapshot.status = if f.stale_reason.is_some() {
                Status::Stale
            } else {
                Status::Ok
            };
            snapshot.message = f.stale_reason;
            snapshot.identity = f.identity;
            snapshot.plan = f.plan;
            snapshot.windows = f.windows;
        }
        Err(ProviderError::Expired(msg)) => {
            snapshot.status = Status::Expired;
            snapshot.message = Some(msg);
        }
        Err(ProviderError::Other(msg)) => {
            snapshot.status = Status::Error;
            snapshot.message = Some(msg);
        }
    }
    snapshot.label = display_label(account, snapshot.identity.as_deref());
    snapshot
}

pub fn placeholder(account: &Account, status: Status) -> UsageSnapshot {
    UsageSnapshot {
        account_id: account.id.clone(),
        provider: account.provider,
        source: account.source,
        label: display_label(account, None),
        identity: None,
        plan: None,
        windows: Vec::new(),
        status,
        message: None,
        fetched_at: Utc::now(),
    }
}

fn display_label(account: &Account, identity: Option<&str>) -> String {
    account
        .label
        .clone()
        .filter(|l| !l.trim().is_empty())
        .or_else(|| identity.map(str::to_string))
        .unwrap_or_else(|| account.provider.display_name().to_string())
}

pub async fn detect_defaults() -> Vec<ProviderKind> {
    let mut found = Vec::new();
    if claude::detect() {
        found.push(ProviderKind::Claude);
    }
    if codex::detect() {
        found.push(ProviderKind::Codex);
    }
    if copilot::detect().await {
        found.push(ProviderKind::Copilot);
    }
    found
}

pub(crate) fn home() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

pub(crate) fn resolve_dir(custom: Option<&str>, env_var: &str, default_name: &str) -> PathBuf {
    custom
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(expand_tilde)
        .or_else(|| {
            std::env::var(env_var)
                .ok()
                .filter(|s| !s.is_empty())
                .map(PathBuf::from)
        })
        .unwrap_or_else(|| home().join(default_name))
}

fn expand_tilde(path: &str) -> PathBuf {
    match path.strip_prefix("~/").or_else(|| path.strip_prefix("~\\")) {
        Some(rest) => home().join(rest),
        None if path == "~" => home(),
        None => PathBuf::from(path),
    }
}

pub(crate) fn jwt_claims(token: &str) -> Option<serde_json::Value> {
    let payload = token.split('.').nth(1)?;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload.trim_end_matches('='))
        .ok()?;
    serde_json::from_slice(&bytes).ok()
}

pub(crate) fn window_name_from_seconds(seconds: i64) -> String {
    match seconds {
        18_000 => "Sessão (5h)".into(),
        604_800 => "Semanal".into(),
        s if s % 86_400 == 0 => format!("{} dias", s / 86_400),
        s if s % 3_600 == 0 => format!("{}h", s / 3_600),
        s => format!("{} min", s / 60),
    }
}

pub(crate) fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod live {
    use super::*;
    use crate::model::AccountSource;

    #[test]
    #[ignore = "consulta as APIs reais com as credenciais locais"]
    fn probe_default_accounts() {
        tauri::async_runtime::block_on(async {
            let client = Client::builder()
                .user_agent("ai-usage/dev")
                .build()
                .unwrap();
            for provider in detect_defaults().await {
                let account = Account {
                    id: format!("{provider:?}"),
                    provider,
                    source: AccountSource::Auto,
                    label: None,
                    config_dir: None,
                    has_token: false,
                };
                let s = fetch(&client, &account, None).await;
                println!(
                    "{:?} [{:?}] plan={:?} msg={:?}",
                    s.provider, s.status, s.plan, s.message
                );
                for w in &s.windows {
                    println!(
                        "  {} {:.1}% resets={:?} used={:?}/{:?}",
                        w.name, w.used_percent, w.resets_at, w.used, w.limit
                    );
                }
            }
        });
    }
}

#[cfg(windows)]
pub(crate) fn command(program: &str) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new(program);
    cmd.creation_flags(0x0800_0000);
    cmd
}

#[cfg(not(windows))]
pub(crate) fn command(program: &str) -> tokio::process::Command {
    tokio::process::Command::new(program)
}
