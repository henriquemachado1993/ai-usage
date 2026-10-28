mod commands;
mod dock;
mod model;
mod poller;
mod providers;
mod state;
mod store;
mod tray;

use tauri::{Manager, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_window_state::StateFlags;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_window(app);
        }))
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(StateFlags::POSITION | StateFlags::SIZE)
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let store = store::Store::load(app.path().app_config_dir()?);
            let always_on_top = store.settings.always_on_top;
            app.manage(state::AppState::new(store));
            app.manage(dock::Dock::default());

            if let Some(window) = app.get_webview_window("main") {
                window.set_always_on_top(always_on_top)?;
            }
            dock::snap(app.handle());
            dock::initialize(app.handle()).map_err(std::io::Error::other)?;
            tray::create(app.handle())?;
            poller::spawn(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window.hide();
            }
            WindowEvent::Moved(_) => dock::on_moved(window.app_handle()),
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshots,
            commands::refresh_now,
            commands::list_accounts,
            commands::add_account,
            commands::remove_account,
            commands::rename_account,
            commands::get_settings,
            commands::set_settings,
            commands::redetect_accounts,
            commands::quit_app,
            dock::get_dock,
            dock::set_collapsed,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
