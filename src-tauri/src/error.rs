use serde::Serialize;

use capture::CaptureError;
use hdr_core::HdrError;

use crate::config::ConfigError;
use crate::logging::LoggingError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Config(#[from] ConfigError),
    #[error("{0}")]
    Logging(#[from] LoggingError),
    #[error("{0}")]
    Hdr(#[from] HdrError),
    #[error("{0}")]
    Capture(#[from] CaptureError),
    #[error("verrou de configuration saturé")]
    LockPoisoned,
    #[error("{0}")]
    Message(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorPayload {
    pub message: String,
    pub detail: String,
}

impl From<AppError> for ErrorPayload {
    fn from(error: AppError) -> Self {
        match &error {
            AppError::Config(inner) => ErrorPayload {
                message: "La configuration n'a pas pu être lue ou enregistrée.".to_string(),
                detail: inner.to_string(),
            },
            AppError::Logging(inner) => ErrorPayload {
                message: "Les journaux n'ont pas pu être mis à jour.".to_string(),
                detail: inner.to_string(),
            },
            AppError::Hdr(inner) => ErrorPayload {
                message: hdr_message(inner),
                detail: inner.to_string(),
            },
            AppError::Capture(inner) => ErrorPayload {
                message: capture_message(inner),
                detail: inner.to_string(),
            },
            AppError::LockPoisoned => ErrorPayload {
                message: "L'application a rencontré un état interne incohérent.".to_string(),
                detail: error.to_string(),
            },
            AppError::Message(message) => ErrorPayload {
                message: message.clone(),
                detail: message.clone(),
            },
        }
    }
}

fn hdr_message(error: &HdrError) -> String {
    match error {
        HdrError::NoImages => "Aucune image JPEG dans ce dossier.".to_string(),
        HdrError::TooFewImages(_) => "Il faut au moins deux JPEG pour fusionner.".to_string(),
        HdrError::Cancelled => "La fusion a été annulée.".to_string(),
        HdrError::Inconsistent(_) => {
            "Les images ne sont pas compatibles (taille ou canaux).".to_string()
        }
        HdrError::OpenCv(_) => "OpenCV n'a pas pu fusionner ces images.".to_string(),
        HdrError::Io { .. } => "Un fichier n'a pas pu être lu ou écrit.".to_string(),
        HdrError::Write { .. } => "Le fichier de sortie n'a pas pu être enregistré.".to_string(),
        HdrError::Exif(_) => "Les métadonnées EXIF n'ont pas pu être recopiées.".to_string(),
    }
}

fn capture_message(error: &CaptureError) -> String {
    match error {
        CaptureError::InvalidPlan(detail) => format!("Plan de bracketing invalide : {detail}"),
        CaptureError::Cancelled => "La prise de vue a été annulée.".to_string(),
        CaptureError::Failed { step, source } => {
            format!("Échec pendant {step:?} : {source}")
        }
        CaptureError::Camera(inner) => format!("Erreur appareil : {inner}"),
        CaptureError::Hdr(inner) => hdr_message(inner),
        CaptureError::Io(_) => "Un fichier de session n'a pas pu être écrit.".to_string(),
        CaptureError::Message(message) => message.clone(),
    }
}

impl From<ConfigError> for ErrorPayload {
    fn from(error: ConfigError) -> Self {
        AppError::from(error).into()
    }
}

impl From<LoggingError> for ErrorPayload {
    fn from(error: LoggingError) -> Self {
        AppError::from(error).into()
    }
}

impl From<HdrError> for ErrorPayload {
    fn from(error: HdrError) -> Self {
        AppError::from(error).into()
    }
}

impl From<CaptureError> for ErrorPayload {
    fn from(error: CaptureError) -> Self {
        AppError::from(error).into()
    }
}
