use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Preferences locales de l'application.
/// Toutes les valeurs sont validees avant d'etre ecrites sur le disque.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// dark | light | system
    pub theme: String,
    /// Volume d'ecoute logiciel par defaut (0-100).
    pub listen_volume: f32,
    /// Protection anti-larsen (limiteur + attenuation en sortie haut-parleurs).
    pub feedback_guard: bool,
    /// Afficher les peripheriques deconnectes / desactives.
    pub show_inactive: bool,
    /// Reduire les animations de l'interface.
    pub reduce_motion: bool,
    /// Cartes compactes dans la liste des microphones.
    pub compact_cards: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            listen_volume: 80.0,
            feedback_guard: true,
            show_inactive: true,
            reduce_motion: false,
            compact_cards: false,
        }
    }
}

impl Settings {
    /// Ramene toutes les valeurs dans des bornes sures.
    pub fn sanitize(&mut self) {
        let theme = self.theme.trim().to_ascii_lowercase();
        self.theme = match theme.as_str() {
            "light" | "system" => theme,
            _ => "dark".to_string(),
        };
        if !self.listen_volume.is_finite() {
            self.listen_volume = 80.0;
        }
        self.listen_volume = self.listen_volume.clamp(0.0, 100.0);
    }
}

/// Donnees persistantes de l'application.
/// `custom_names` associe l'identifiant endpoint Windows (jamais modifie)
/// au nom choisi par l'utilisateur, uniquement dans cette application.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct AppStore {
    pub custom_names: HashMap<String, String>,
    pub settings: Settings,
}

impl AppStore {
    /// Charge le fichier de donnees. Un fichier corrompu est sauvegarde
    /// en `.bak` puis remplace par une configuration par defaut.
    pub fn load() -> Self {
        let path = data_file_path();
        let raw = match fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(_) => return Self::default(),
        };

        match serde_json::from_str::<AppStore>(&raw) {
            Ok(mut store) => {
                store.settings.sanitize();
                store.custom_names.retain(|key, value| {
                    !key.trim().is_empty() && !value.trim().is_empty()
                });
                store
            }
            Err(error) => {
                eprintln!("[store] fichier de donnees illisible ({error}), reinitialisation");
                let backup = path.with_extension("json.bak");
                let _ = fs::rename(&path, &backup);
                Self::default()
            }
        }
    }

    /// Ecriture atomique : fichier temporaire puis remplacement du fichier final.
    pub fn save(&self) -> AppResult<()> {
        let path = data_file_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                AppError::new(
                    "store_write",
                    format!("Impossible de creer le dossier de donnees : {error}"),
                )
            })?;
        }

        let payload = serde_json::to_vec_pretty(self)?;
        let temp = path.with_extension("json.tmp");

        {
            let mut file = fs::File::create(&temp).map_err(|error| {
                AppError::new(
                    "store_write",
                    format!("Impossible d'ecrire les preferences : {error}"),
                )
            })?;
            file.write_all(&payload).map_err(|error| {
                AppError::new(
                    "store_write",
                    format!("Impossible d'ecrire les preferences : {error}"),
                )
            })?;
            file.sync_all().ok();
        }

        // Windows ne remplace pas un fichier existant avec `rename`.
        if path.exists() {
            fs::remove_file(&path).map_err(|error| {
                AppError::new(
                    "store_write",
                    format!("Impossible de remplacer les preferences : {error}"),
                )
            })?;
        }
        fs::rename(&temp, &path).map_err(|error| {
            AppError::new(
                "store_write",
                format!("Impossible de finaliser les preferences : {error}"),
            )
        })?;

        Ok(())
    }

    pub fn set_custom_name(&mut self, device_id: &str, name: Option<&str>) {
        match name.map(str::trim).filter(|value| !value.is_empty()) {
            Some(value) => {
                self.custom_names
                    .insert(device_id.to_string(), value.to_string());
            }
            None => {
                self.custom_names.remove(device_id);
            }
        }
    }

    pub fn clear_custom_names(&mut self) {
        self.custom_names.clear();
    }
}

/// Dossier de donnees : `%APPDATA%\com.audiomanager.desktop`.
pub fn data_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("com.audiomanager.desktop")
}

pub fn data_file_path() -> PathBuf {
    data_dir().join("audio-manager.json")
}

pub fn display_data_path() -> String {
    data_file_path().to_string_lossy().to_string()
}

/// Verifie qu'un identifiant de peripherique n'est pas vide.
pub fn validate_device_id(device_id: &str) -> AppResult<()> {
    if device_id.trim().is_empty() {
        return Err(AppError::invalid("Identifiant de peripherique manquant."));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_clamps_values_and_normalises_theme() {
        let mut settings = Settings {
            theme: "  LIGHT ".to_string(),
            listen_volume: 250.0,
            ..Settings::default()
        };
        settings.sanitize();

        assert_eq!(settings.theme, "light");
        assert_eq!(settings.listen_volume, 100.0);

        let mut settings = Settings {
            theme: "neon".to_string(),
            listen_volume: -20.0,
            ..Settings::default()
        };
        settings.sanitize();

        assert_eq!(settings.theme, "dark");
        assert_eq!(settings.listen_volume, 0.0);
    }

    #[test]
    fn sanitize_replaces_not_a_number() {
        let mut settings = Settings {
            listen_volume: f32::NAN,
            ..Settings::default()
        };
        settings.sanitize();

        assert_eq!(settings.listen_volume, 80.0);
    }

    #[test]
    fn custom_names_are_trimmed_and_removable() {
        let mut store = AppStore::default();

        store.set_custom_name("endpoint", Some("  Studio  "));
        assert_eq!(
            store.custom_names.get("endpoint").map(String::as_str),
            Some("Studio")
        );

        store.set_custom_name("endpoint", Some("   "));
        assert!(!store.custom_names.contains_key("endpoint"));
    }

    #[test]
    fn empty_device_id_is_rejected() {
        assert!(validate_device_id("  ").is_err());
        assert!(validate_device_id("endpoint").is_ok());
    }
}

