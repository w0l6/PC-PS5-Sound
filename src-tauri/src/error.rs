use serde::Serialize;

/// Erreur applicative structuree renvoyee au frontend.
/// Le frontend ne recoit jamais de chaine brute : il recoit toujours
/// un objet `{ code, message }`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub message: String,
}

impl AppError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("internal", message)
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new("invalid_input", message)
    }

    pub fn device(message: impl Into<String>) -> Self {
        Self::new("device_error", message)
    }

    pub fn audio(message: impl Into<String>) -> Self {
        Self::new("audio_error", message)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::new("io_error", value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::new("json_error", value.to_string())
    }
}

impl From<windows::core::Error> for AppError {
    fn from(value: windows::core::Error) -> Self {
        let hr = value.code().0 as u32;
        let message = match hr {
            0x80070005 => "Acces refuse par Windows. Verifiez l'autorisation du microphone."
                .to_string(),
            0x88890008 => "Format audio non pris en charge par le peripherique.".to_string(),
            0x8889000A => "Le peripherique audio a ete invalide ou deconnecte.".to_string(),
            0x88890004 => "Le peripherique audio n'est pas disponible.".to_string(),
            _ => value.message(),
        };
        Self::new(format!("windows_0x{hr:08X}"), message)
    }
}

pub type AppResult<T> = Result<T, AppError>;
