use crate::audio::monitor;
use crate::audio::types::{AudioSnapshot, ListenState};
use crate::error::{AppError, AppResult};
use crate::permissions;
use crate::state::{apply_custom_names, lock, read, write, AppState};
use crate::store::{self, Settings};
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

const MAX_CUSTOM_NAME_CHARS: usize = 64;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub identifier: String,
    pub data_path: String,
}

#[tauri::command]
pub fn get_devices(state: State<'_, Arc<AppState>>) -> AudioSnapshot {
    read(&state.snapshot).clone()
}

#[tauri::command]
pub fn refresh_devices(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> AppResult<AudioSnapshot> {
    monitor::refresh_snapshot(&app, &state)
}

/// Renomme un microphone uniquement dans l'application.
/// L'identifiant endpoint Windows et le nom systeme restent intacts.
#[tauri::command]
pub fn rename_device(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    device_id: String,
    custom_name: Option<String>,
) -> AppResult<AudioSnapshot> {
    store::validate_device_id(&device_id)?;

    let known = read(&state.snapshot)
        .devices
        .iter()
        .any(|device| device.id == device_id);
    if !known {
        return Err(AppError::device(
            "Ce microphone n'existe plus dans la liste des peripheriques.",
        ));
    }

    let cleaned = custom_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(MAX_CUSTOM_NAME_CHARS).collect::<String>());

    if let Some(name) = &cleaned {
        if name.chars().any(char::is_control) {
            return Err(AppError::invalid(
                "Le nom personnalise contient des caracteres interdits.",
            ));
        }
    }

    let names = {
        let mut store = lock(&state.store);
        store.set_custom_name(&device_id, cleaned.as_deref());
        store.save()?;
        store.custom_names.clone()
    };

    let mut snapshot = read(&state.snapshot).clone();
    apply_custom_names(&mut snapshot, &names);
    snapshot.captured_at = crate::audio::types::now_ms();
    *write(&state.snapshot) = snapshot.clone();
    let _ = app.emit("audio:devices", &snapshot);
    state.request_refresh();
    Ok(snapshot)
}

#[tauri::command]
pub fn reset_custom_names(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> AppResult<AudioSnapshot> {
    {
        let mut store = lock(&state.store);
        store.clear_custom_names();
        store.save()?;
    }
    let mut snapshot = read(&state.snapshot).clone();
    for device in &mut snapshot.devices {
        device.custom_name = None;
        device.display_name = device.name.clone();
    }
    snapshot.captured_at = crate::audio::types::now_ms();
    *write(&state.snapshot) = snapshot.clone();
    let _ = app.emit("audio:devices", &snapshot);
    state.request_refresh();
    Ok(snapshot)
}

#[tauri::command]
pub fn get_listen_state(state: State<'_, Arc<AppState>>) -> ListenState {
    state.listen.snapshot()
}

#[tauri::command]
pub fn start_listen(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    device_id: String,
) -> AppResult<ListenState> {
    store::validate_device_id(&device_id)?;

    let permission = permissions::microphone_permission();
    *write(&state.permission) = permission.clone();
    if permission == "denied" {
        return Err(permissions::permission_error());
    }

    let mut snapshot = read(&state.snapshot).clone();
    let mut device = snapshot
        .devices
        .iter()
        .find(|device| device.id == device_id)
        .cloned();

    if device.is_none() {
        // Le moniteur n'a peut-etre pas encore publie son premier instantane.
        snapshot = monitor::refresh_snapshot(&app, &state)?;
        device = snapshot
            .devices
            .iter()
            .find(|device| device.id == device_id)
            .cloned();
    }

    let device = device.ok_or_else(|| {
        AppError::device("Ce microphone n'est plus disponible. Actualisez la liste.")
    })?;

    if device.state != "active" {
        return Err(AppError::device(format!(
            "{} n'est pas connecte.",
            device.display_name
        )));
    }

    let output_is_speakers = matches!(
        snapshot.default_output_form_factor.as_str(),
        "speakers" | "hdmi" | "digital" | "spdif"
    );

    state.listen.start(
        &app,
        device.id,
        device.display_name,
        snapshot.default_output,
        output_is_speakers,
    )
}

#[tauri::command]
pub fn stop_listen(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> AppResult<ListenState> {
    Ok(state.listen.stop(&app))
}

#[tauri::command]
pub fn set_listen_volume(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    value: f32,
) -> AppResult<ListenState> {
    if !value.is_finite() {
        return Err(AppError::invalid("Volume d'ecoute invalide."));
    }
    let value = value.clamp(0.0, 100.0);
    let result = state.listen.set_volume(&app, value);
    {
        let mut store = lock(&state.store);
        store.settings.listen_volume = value;
        store.save()?;
    }
    Ok(result)
}

#[tauri::command]
pub fn set_listen_mute(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    muted: bool,
) -> AppResult<ListenState> {
    Ok(state.listen.set_muted(&app, muted))
}

#[tauri::command]
pub fn get_settings(state: State<'_, Arc<AppState>>) -> Settings {
    lock(&state.store).settings.clone()
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    mut settings: Settings,
) -> AppResult<Settings> {
    settings.sanitize();
    {
        let mut store = lock(&state.store);
        store.settings = settings.clone();
        store.save()?;
    }
    state.listen.set_volume(&app, settings.listen_volume);
    state.listen.set_guard(&app, settings.feedback_guard);
    Ok(settings)
}

#[tauri::command]
pub fn get_permission(state: State<'_, Arc<AppState>>) -> String {
    let permission = permissions::microphone_permission();
    *write(&state.permission) = permission.clone();
    permission
}

#[tauri::command]
pub fn open_microphone_settings() -> AppResult<()> {
    permissions::open_microphone_settings()
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        identifier: "com.audiomanager.desktop".to_string(),
        data_path: store::display_data_path(),
    }
}

#[tauri::command]
pub fn window_minimize(window: tauri::Window) -> AppResult<()> {
    window
        .minimize()
        .map_err(|error| AppError::internal(error.to_string()))
}

#[tauri::command]
pub fn window_toggle_maximize(window: tauri::Window) -> AppResult<bool> {
    let maximized = window
        .is_maximized()
        .map_err(|error| AppError::internal(error.to_string()))?;
    if maximized {
        window
            .unmaximize()
            .map_err(|error| AppError::internal(error.to_string()))?;
    } else {
        window
            .maximize()
            .map_err(|error| AppError::internal(error.to_string()))?;
    }
    Ok(!maximized)
}

#[tauri::command]
pub fn window_close(window: tauri::Window) -> AppResult<()> {
    window
        .close()
        .map_err(|error| AppError::internal(error.to_string()))
}
