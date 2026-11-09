use crate::audio::com::ComGuard;
use crate::audio::enumerate::{create_enumerator, enumerate_capture_devices};
use crate::audio::types::{now_ms, AudioSnapshot, MeterPayload, Notice};
use crate::error::AppResult;
use crate::permissions;
use crate::state::{apply_custom_names, lock, write, AppState};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use windows::Win32::Media::Audio::Endpoints::IAudioMeterInformation;

const ENUMERATION_INTERVAL: Duration = Duration::from_millis(1000);
const METER_INTERVAL: Duration = Duration::from_millis(40);
const PERMISSION_INTERVAL: Duration = Duration::from_secs(5);

fn snapshot_signature(snapshot: &AudioSnapshot) -> String {
    let mut copy = snapshot.clone();
    copy.captured_at = 0;
    serde_json::to_string(&copy).unwrap_or_default()
}

/// Thread de surveillance : enumere les peripheriques une fois par seconde,
/// publie les niveaux a 25 Hz et detecte la disparition du micro ecoute.
pub fn spawn(app: AppHandle, state: Arc<AppState>, refresh: Receiver<()>) -> JoinHandle<()> {
    std::thread::Builder::new()
        .name("audio-monitor".to_string())
        .spawn(move || monitor_loop(app, state, refresh))
        .expect("impossible de demarrer le moniteur audio")
}

/// Enumeration synchrone demandee par une commande (rafraichissement manuel,
/// verification avant demarrage de l'ecoute).
pub fn refresh_snapshot(app: &AppHandle, state: &AppState) -> AppResult<AudioSnapshot> {
    let _com = ComGuard::mta();
    let enumerator = create_enumerator()?;
    let enumeration = enumerate_capture_devices(&enumerator)?;
    let names = {
        let store = lock(&state.store);
        store.custom_names.clone()
    };
    let permission = permissions::microphone_permission();
    let mut snapshot = AudioSnapshot {
        devices: enumeration
            .devices
            .iter()
            .map(|device| device.info.clone())
            .collect(),
        default_output: enumeration.default_output_name.clone(),
        default_output_form_factor: enumeration.default_output_form_factor.clone(),
        permission: permission.clone(),
        monitor_status: "running".to_string(),
        captured_at: now_ms(),
    };
    apply_custom_names(&mut snapshot, &names);
    *write(&state.snapshot) = snapshot.clone();
    *write(&state.permission) = permission;
    let _ = app.emit("audio:devices", &snapshot);
    Ok(snapshot)
}

fn monitor_loop(app: AppHandle, state: Arc<AppState>, refresh: Receiver<()>) {
    let _com = ComGuard::mta();
    let mut last_enumeration = Instant::now() - ENUMERATION_INTERVAL;
    let mut last_permission = Instant::now() - PERMISSION_INTERVAL;
    let mut meters: Vec<(String, IAudioMeterInformation)> = Vec::new();
    let mut last_signature = String::new();
    let mut had_meters = false;
    let mut error_active = false;
    let mut permission = permissions::microphone_permission();

    loop {
        if state.is_shutting_down() {
            break;
        }

        let forced = refresh.try_recv().is_ok();
        if forced || last_enumeration.elapsed() >= ENUMERATION_INTERVAL {
            last_enumeration = Instant::now();
            let names = {
                let store = lock(&state.store);
                store.custom_names.clone()
            };

            match create_enumerator().and_then(|enumerator| enumerate_capture_devices(&enumerator)) {
                Ok(enumeration) => {
                    if error_active {
                        error_active = false;
                    }

                    let mut snapshot = AudioSnapshot {
                        devices: enumeration
                            .devices
                            .iter()
                            .map(|device| device.info.clone())
                            .collect(),
                        default_output: enumeration.default_output_name.clone(),
                        default_output_form_factor: enumeration
                            .default_output_form_factor
                            .clone(),
                        permission: permission.clone(),
                        monitor_status: "running".to_string(),
                        captured_at: now_ms(),
                    };
                    apply_custom_names(&mut snapshot, &names);

                    let signature = snapshot_signature(&snapshot);
                    if signature != last_signature {
                        last_signature = signature;
                        *write(&state.snapshot) = snapshot.clone();
                        let _ = app.emit("audio:devices", &snapshot);
                    }

                    // Le peripherique ecoute a disparu ou n'est plus actif.
                    if let Some(active) = state.listen.active_device() {
                        let still_active = snapshot
                            .devices
                            .iter()
                            .any(|device| device.id == active && device.state == "active");
                        if !still_active {
                            let name = state
                                .listen
                                .snapshot()
                                .device_name
                                .unwrap_or_else(|| "Le microphone".to_string());
                            state.listen.stop(&app);
                            let _ = app.emit(
                                "audio:notice",
                                &Notice {
                                    kind: "warning".to_string(),
                                    message: format!(
                                        "{name} n'est plus disponible. L'ecoute a ete arretee."
                                    ),
                                },
                            );
                        }
                    }

                    meters = enumeration
                        .devices
                        .into_iter()
                        .filter(|device| device.info.state == "active")
                        .filter_map(|device| device.meter.map(|meter| (device.info.id, meter)))
                        .collect();
                }
                Err(error) => {
                    if !error_active {
                        error_active = true;
                        let _ = app.emit(
                            "audio:notice",
                            &Notice {
                                kind: "error".to_string(),
                                message: format!(
                                    "Impossible d'enumerer les microphones : {}",
                                    error.message
                                ),
                            },
                        );
                    }
                    let mut snapshot = write(&state.snapshot).clone();
                    snapshot.monitor_status = "error".to_string();
                    snapshot.permission = permission.clone();
                    snapshot.captured_at = now_ms();
                    let signature = snapshot_signature(&snapshot);
                    if signature != last_signature {
                        last_signature = signature;
                        *write(&state.snapshot) = snapshot.clone();
                        let _ = app.emit("audio:devices", &snapshot);
                    }
                }
            }
        }

        if last_permission.elapsed() >= PERMISSION_INTERVAL {
            last_permission = Instant::now();
            let current = permissions::microphone_permission();
            if current != permission {
                permission = current;
                *write(&state.permission) = permission.clone();
                let mut snapshot = write(&state.snapshot).clone();
                snapshot.permission = permission.clone();
                snapshot.captured_at = now_ms();
                last_signature = snapshot_signature(&snapshot);
                *write(&state.snapshot) = snapshot.clone();
                let _ = app.emit("audio:devices", &snapshot);
            }
        }

        let mut levels: HashMap<String, f32> = HashMap::with_capacity(meters.len());
        for (id, meter) in &meters {
            if let Ok(peak) = unsafe { meter.GetPeakValue() } {
                if peak.is_finite() {
                    levels.insert(id.clone(), peak.clamp(0.0, 1.0));
                }
            }
        }
        if !levels.is_empty() || had_meters {
            had_meters = !levels.is_empty();
            let _ = app.emit("audio:meter", &MeterPayload { levels });
        }

        std::thread::sleep(METER_INTERVAL);
    }

    *write(&state.permission) = permission;
}
