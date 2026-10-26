use std::time::Duration;

use futures::future::join_all;
use tauri::{AppHandle, Emitter, Manager};

use crate::model::{Account, AccountSource, UsageSnapshot};
use crate::state::AppState;
use crate::{providers, store, tray};

pub fn spawn(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        sync_auto_accounts(&app).await;
        loop {
            refresh_all(&app).await;
            let state = app.state::<AppState>();
            let minutes = state.store.lock().unwrap().settings.refresh_minutes.max(1);
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(minutes * 60)) => {}
                _ = state.wake.notified() => {}
            }
        }
    });
}

pub async fn sync_auto_accounts(app: &AppHandle) {
    let detected = providers::detect_defaults().await;
    let state = app.state::<AppState>();
    let mut store = state.store.lock().unwrap();
    let mut changed = false;
    for provider in detected {
        let id = format!("{}-auto", provider.display_name().to_lowercase());
        if store.accounts.iter().any(|a| a.id == id) || store.settings.dismissed.contains(&id) {
            continue;
        }
        store.accounts.push(Account {
            id,
            provider,
            source: AccountSource::Auto,
            label: None,
            config_dir: None,
            has_token: false,
        });
        changed = true;
    }
    if changed {
        let _ = store.save_accounts();
    }
}

async fn refresh_all(app: &AppHandle) {
    let state = app.state::<AppState>();
    let accounts = state.store.lock().unwrap().accounts.clone();
    let _ = app.emit("refreshing", true);

    let results: Vec<UsageSnapshot> = join_all(accounts.iter().map(|account| {
        let client = state.client.clone();
        async move {
            let secret = account
                .has_token
                .then(|| store::get_secret(&account.id))
                .flatten();
            providers::fetch(&client, account, secret).await
        }
    }))
    .await;

    {
        let mut snapshots = state.snapshots.lock().unwrap();
        for s in results {
            snapshots.insert(s.account_id.clone(), s);
        }
    }

    let ordered = state.ordered_snapshots();
    tray::update_tooltip(app, &ordered);
    let _ = app.emit("usage-updated", &ordered);
    let _ = app.emit("refreshing", false);
}
