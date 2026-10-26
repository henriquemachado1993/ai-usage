use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::model::{Account, Settings};

const KEYRING_SERVICE: &str = "com.aiusage.app";

pub struct Store {
    dir: PathBuf,
    pub accounts: Vec<Account>,
    pub settings: Settings,
}

impl Store {
    pub fn load(dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&dir);
        let accounts = read_json(&dir.join("accounts.json")).unwrap_or_default();
        let settings = read_json(&dir.join("settings.json")).unwrap_or_default();
        Self {
            dir,
            accounts,
            settings,
        }
    }

    pub fn save_accounts(&self) -> Result<(), String> {
        write_json(&self.dir.join("accounts.json"), &self.accounts)
    }

    pub fn save_settings(&self) -> Result<(), String> {
        write_json(&self.dir.join("settings.json"), &self.settings)
    }
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Option<T> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}

pub fn set_secret(account_id: &str, secret: &str) -> Result<(), String> {
    keyring::Entry::new(KEYRING_SERVICE, account_id)
        .and_then(|e| e.set_password(secret))
        .map_err(|e| format!("Falha ao salvar token no cofre do sistema: {e}"))
}

pub fn get_secret(account_id: &str) -> Option<String> {
    keyring::Entry::new(KEYRING_SERVICE, account_id)
        .and_then(|e| e.get_password())
        .ok()
}

pub fn delete_secret(account_id: &str) {
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, account_id) {
        let _ = entry.delete_credential();
    }
}
