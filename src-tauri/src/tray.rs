use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

use crate::model::UsageSnapshot;
use crate::state::AppState;

const TRAY_ID: &str = "main";

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "toggle", "Mostrar/ocultar", true, None::<&str>)?,
            &MenuItem::with_id(app, "refresh", "Atualizar agora", true, None::<&str>)?,
            &MenuItem::with_id(app, "settings", "Configurações", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?,
        ],
    )?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("AI Usage")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "toggle" => toggle_window(app),
            "refresh" => app.state::<AppState>().wake.notify_one(),
            "settings" => {
                show_window(app);
                let _ = app.emit("open-settings", ());
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

pub fn show_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

fn toggle_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        if w.is_visible().unwrap_or(false) {
            let _ = w.hide();
        } else {
            show_window(app);
        }
    }
}

pub fn update_tooltip(app: &AppHandle, snapshots: &[UsageSnapshot]) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let lines: Vec<String> = snapshots
        .iter()
        .filter_map(|s| {
            let max = s
                .windows
                .iter()
                .filter(|w| !w.unlimited)
                .map(|w| w.used_percent)
                .fold(None, |acc: Option<f64>, v| {
                    Some(acc.map_or(v, |a| a.max(v)))
                })?;
            Some(format!("{} ({}): {:.0}%", s.provider_name(), s.label, max))
        })
        .collect();
    let text = if lines.is_empty() {
        "AI Usage".to_string()
    } else {
        lines.join("\n")
    };
    let _ = tray.set_tooltip(Some(text));
}
