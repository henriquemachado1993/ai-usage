use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    Claude,
    Codex,
    Copilot,
}

impl ProviderKind {
    pub fn display_name(self) -> &'static str {
        match self {
            ProviderKind::Claude => "Claude",
            ProviderKind::Codex => "Codex",
            ProviderKind::Copilot => "Copilot",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum AccountSource {
    Auto,
    Manual,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub provider: ProviderKind,
    pub source: AccountSource,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub config_dir: Option<String>,
    #[serde(default)]
    pub has_token: bool,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Loading,
    Ok,
    Stale,
    Expired,
    Error,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UsageWindow {
    pub name: String,
    pub used_percent: f64,
    pub resets_at: Option<DateTime<Utc>>,
    pub used: Option<f64>,
    pub limit: Option<f64>,
    pub unlimited: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UsageSnapshot {
    pub account_id: String,
    pub provider: ProviderKind,
    pub source: AccountSource,
    pub label: String,
    pub identity: Option<String>,
    pub plan: Option<String>,
    pub windows: Vec<UsageWindow>,
    pub status: Status,
    pub message: Option<String>,
    pub fetched_at: DateTime<Utc>,
}

impl UsageSnapshot {
    pub fn provider_name(&self) -> &'static str {
        self.provider.display_name()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub refresh_minutes: u64,
    pub opacity: f64,
    pub always_on_top: bool,
    pub autostart: bool,
    pub dismissed: Vec<String>,
    pub collapsed: bool,
    pub expanded_width: f64,
    pub expanded_height: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            refresh_minutes: 5,
            opacity: 0.92,
            always_on_top: true,
            autostart: false,
            dismissed: Vec::new(),
            collapsed: false,
            expanded_width: 340.0,
            expanded_height: 420.0,
        }
    }
}
