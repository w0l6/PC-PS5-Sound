use crate::audio::com::wide;
use crate::error::{AppError, AppResult};
use std::ffi::c_void;
use std::process::Command;
use windows::core::PCWSTR;
use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_SZ};

const CONSENT_ROOT: &str =
    "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\CapabilityAccessManager\\ConsentStore\\microphone";

/// Lit la valeur « Value » (Allow / Deny) d'une cle de consentement.
/// Retourne `Some(true)` pour Allow, `Some(false)` pour Deny, `None` sinon.
fn read_consent(subkey: &str) -> Option<bool> {
    let key = wide(subkey);
    let value = wide("Value");
    let mut buffer = [0u16; 128];
    let mut size = (buffer.len() * std::mem::size_of::<u16>()) as u32;

    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(key.as_ptr()),
            PCWSTR(value.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            Some(buffer.as_mut_ptr() as *mut c_void),
            Some(&mut size),
        )
    };

    if status.0 != 0 {
        return None;
    }

    let chars = (size as usize / std::mem::size_of::<u16>())
        .saturating_sub(1)
        .min(buffer.len());
    let raw = String::from_utf16_lossy(&buffer[..chars]);
    match raw.trim().to_ascii_lowercase().as_str() {
        "allow" => Some(true),
        "deny" => Some(false),
        _ => None,
    }
}

/// Etat de l'autorisation microphone Windows.
/// - `granted` : acces autorise
/// - `denied`  : acces refuse (interrupteur global ou applications de bureau)
/// - `unknown` : valeur absente (Windows demandera l'autorisation a l'usage)
///
/// Windows distingue l'acces global au microphone et l'acces des
/// « applications de bureau » (`NonPackaged`). Les deux doivent etre
/// autorises pour qu'un flux WASAPI puisse etre ouvert.
pub fn microphone_permission() -> String {
    let global = read_consent(CONSENT_ROOT);
    let desktop = read_consent(&format!("{CONSENT_ROOT}\\NonPackaged"));

    if global == Some(false) || desktop == Some(false) {
        return "denied".to_string();
    }
    if desktop == Some(true) || global == Some(true) {
        return "granted".to_string();
    }
    "unknown".to_string()
}

/// Ouvre la page Windows « Confidentialite et securite > Microphone ».
pub fn open_microphone_settings() -> AppResult<()> {
    Command::new("explorer.exe")
        .arg("ms-settings:privacy-microphone")
        .spawn()
        .map_err(|error| {
            AppError::new(
                "settings_open",
                format!("Impossible d'ouvrir les parametres Windows : {error}"),
            )
        })?;
    Ok(())
}

/// Message utilisateur lorsque Windows refuse l'acces au microphone.
pub fn permission_error() -> AppError {
    AppError::new(
        "permission_denied",
        "Windows bloque l'acces au microphone. Ouvrez Parametres > Confidentialite > Microphone et autorisez l'application.",
    )
}
