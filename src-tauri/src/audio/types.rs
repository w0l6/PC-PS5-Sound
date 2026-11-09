use serde::Serialize;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Etat d'un peripherique d'enregistrement tel qu'expose au frontend.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevice {
    /// Identifiant endpoint Windows stable. Jamais modifie.
    pub id: String,
    /// Nom Windows d'origine.
    pub name: String,
    /// Nom affiche : nom personnalise s'il existe, sinon nom Windows.
    pub display_name: String,
    /// Nom personnalise local (null si aucun).
    pub custom_name: Option<String>,
    pub device_desc: String,
    pub interface_name: String,
    /// active | disabled | unplugged | notpresent | unknown
    pub state: String,
    /// Libelle francais de l'etat.
    pub state_label: String,
    pub is_default: bool,
    pub is_default_communication: bool,
    /// Volume de capture Windows, en lecture seule (0-100).
    pub volume_percent: f32,
    /// Mute de capture Windows, en lecture seule.
    pub muted: bool,
    pub form_factor: String,
    pub form_factor_label: String,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioSnapshot {
    pub devices: Vec<AudioDevice>,
    pub default_output: Option<String>,
    pub default_output_form_factor: String,
    /// granted | denied | unknown
    pub permission: String,
    pub monitor_status: String,
    pub captured_at: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeterPayload {
    pub levels: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListenState {
    pub active: bool,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    pub volume: f32,
    pub muted: bool,
    pub output_name: Option<String>,
    pub peak: f32,
    pub feedback_guard: bool,
    /// idle | starting | listening | stopping | error
    pub status: String,
    pub error: Option<String>,
    pub started_at: Option<u64>,
}

impl Default for ListenState {
    fn default() -> Self {
        Self {
            active: false,
            device_id: None,
            device_name: None,
            volume: 80.0,
            muted: false,
            output_name: None,
            peak: 0.0,
            feedback_guard: true,
            status: "idle".to_string(),
            error: None,
            started_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Notice {
    pub kind: String,
    pub message: String,
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
