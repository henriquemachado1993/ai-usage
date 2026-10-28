use serde::Deserialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt;

use crate::model::{Account, AccountSource, ProviderKind, Settings, UsageSnapshot};
use crate::state::AppState;
use crate::{poller, store};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAccount {
    provider: ProviderKind,
    label: Option<String>,
    config_dir: Option<String>,
    token: Option<String>,
}

fn non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn publish(app: &AppHandle, state: &AppState) {
    let _ = app.emit("usage-updated", state.ordered_snapshots());
}

#[tauri::command]
pub fn get_snapshots(state: State<'_, AppState>) -> Vec<UsageSnapshot> {
    state.ordered_snapshots()
}

#[tauri::command]
pub fn refresh_now(state: State<'_, AppState>) {
    state.wake.notify_one();
}

#[tauri::command]
pub fn list_accounts(state: State<'_, AppState>) -> Vec<Account> {
    state.store.lock().unwrap().accounts.clone()
}

#[tauri::command]
pub fn add_account(
    app: AppHandle,
    state: State<'_, AppState>,
    input: NewAccount,
) -> Result<Account, String> {
    let config_dir = non_empty(input.config_dir);
    let token = non_empty(input.token);

    match input.provider {
        ProviderKind::Copilot if token.is_none() => {
            return Err("Informe um token do GitHub com acesso ao Copilot".into());
        }
        ProviderKind::Claude | ProviderKind::Codex if config_dir.is_none() => {
            return Err("Informe o diretório de configuração da conta".into());
        }
        _ => {}
    }

    let account = Account {
        id: uuid::Uuid::new_v4().to_string(),
        provider: input.provider,
        source: AccountSource::Manual,
        label: non_empty(input.label),
        config_dir,
        has_token: token.is_some(),
    };
    if let Some(t) = &token {
        store::set_secret(&account.id, t)?;
    }

    {
        let mut s = state.store.lock().unwrap();
        s.accounts.push(account.clone());
        s.save_accounts()?;
    }
    publish(&app, &state);
    state.wake.notify_one();
    Ok(account)
}

#[tauri::command]
pub fn remove_account(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    {
        let mut s = state.store.lock().unwrap();
        let Some(pos) = s.accounts.iter().position(|a| a.id == id) else {
            return Ok(());
        };
        let account = s.accounts.remove(pos);
        if account.has_token {
            store::delete_secret(&account.id);
        }
        if account.source == AccountSource::Auto && !s.settings.dismissed.contains(&id) {
            s.settings.dismissed.push(id.clone());
            s.save_settings()?;
        }
        s.save_accounts()?;
    }
    state.snapshots.lock().unwrap().remove(&id);
    publish(&app, &state);
    Ok(())
}

#[tauri::command]
pub fn rename_account(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    label: Option<String>,
) -> Result<(), String> {
    let label = non_empty(label);
    {
        let mut s = state.store.lock().unwrap();
        if let Some(a) = s.accounts.iter_mut().find(|a| a.id == id) {
            a.label = label.clone();
        }
        s.save_accounts()?;
    }
    if let Some(snap) = state.snapshots.lock().unwrap().get_mut(&id) {
        snap.label = label
            .or_else(|| snap.identity.clone())
            .unwrap_or_else(|| snap.provider_name().to_string());
    }
    publish(&app, &state);
    Ok(())
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.store.lock().unwrap().settings.clone()
}

#[tauri::command]
pub fn set_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<Settings, String> {
    let previous_scale = state.store.lock().unwrap().settings.scale();
    let saved = {
        let mut s = state.store.lock().unwrap();
        let interval_changed = s.settings.refresh_minutes != settings.refresh_minutes;
        s.settings = Settings {
            refresh_minutes: settings.refresh_minutes.clamp(1, 120),
            opacity: settings.opacity.clamp(0.3, 1.0),
            dismissed: s.settings.dismissed.clone(),
            collapsed: s.settings.collapsed,
            expanded_width: s.settings.expanded_width,
            expanded_height: s.settings.expanded_height,
            ui_scale: Settings::normalize_scale(settings.ui_scale),
            ..settings
        };
        s.save_settings()?;
        if interval_changed {
            state.wake.notify_one();
        }
        s.settings.clone()
    };

    crate::dock::apply_scale(&app, previous_scale, saved.scale(), saved.collapsed)?;

    if let Some(w) = app.get_webview_window("main") {
        w.set_always_on_top(saved.always_on_top)
            .map_err(|e| e.to_string())?;
    }
    let autolaunch = app.autolaunch();
    if autolaunch.is_enabled().unwrap_or(false) != saved.autostart {
        let result = if saved.autostart {
            autolaunch.enable()
        } else {
            autolaunch.disable()
        };
        result.map_err(|e| format!("Falha ao configurar inicialização automática: {e}"))?;
    }
    Ok(saved)
}

#[tauri::command]
pub async fn redetect_accounts(app: AppHandle) -> Result<(), String> {
    {
        let state = app.state::<AppState>();
        let mut s = state.store.lock().unwrap();
        s.settings.dismissed.clear();
        s.save_settings()?;
    }
    poller::sync_auto_accounts(&app).await;
    let state = app.state::<AppState>();
    publish(&app, &state);
    state.wake.notify_one();
    Ok(())
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}
