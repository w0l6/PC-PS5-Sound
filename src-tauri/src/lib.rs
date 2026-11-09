mod audio;
mod commands;
mod error;
mod permissions;
mod state;
mod store;

use crate::audio::listen::ListenManager;
use crate::state::AppState;
use crate::store::AppStore;
use std::sync::mpsc;
use std::sync::Arc;
use tauri::Manager;

pub fn run() {
    let loaded = AppStore::load();
    let settings = loaded.settings.clone();
    let (refresh_tx, refresh_rx) = mpsc::channel();
    let listen = Arc::new(ListenManager::new(
        settings.listen_volume,
        settings.feedback_guard,
    ));
    let state = Arc::new(AppState::new(loaded, listen, refresh_tx));

    tauri::Builder::default()
        .manage(state.clone())
        .setup(move |app| {
            audio::monitor::spawn(app.handle().clone(), state.clone(), refresh_rx);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let Some(state) = window.try_state::<Arc<AppState>>() {
                    state.mark_shutdown();
                    state.listen.stop(window.app_handle());
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_devices,
            commands::refresh_devices,
            commands::rename_device,
            commands::reset_custom_names,
            commands::get_listen_state,
            commands::start_listen,
            commands::stop_listen,
            commands::set_listen_volume,
            commands::set_listen_mute,
            commands::get_settings,
            commands::update_settings,
            commands::get_permission,
            commands::open_microphone_settings,
            commands::get_app_info,
            commands::window_minimize,
            commands::window_toggle_maximize,
            commands::window_close,
        ])
        .run(tauri::generate_context!())
        .expect("erreur au lancement d'Audio Manager");
}
