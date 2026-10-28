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
                message: "Config illisible ou non enregistrée".to_string(),
                detail: inner.to_string(),
            },
            AppError::Logging(inner) => ErrorPayload {
                message: "Impossible de mettre à jour les logs".to_string(),
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
                message: "État interne incohérent — redémarre l’app".to_string(),
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
        HdrError::NoImages => "Pas de JPEG dans ce dossier".to_string(),
        HdrError::TooFewImages(_) => "Il faut au moins deux JPEG".to_string(),
        HdrError::Cancelled => "Fusion annulée".to_string(),
        HdrError::Inconsistent(_) => "Images incompatibles (taille ou canaux)".to_string(),
        HdrError::OpenCv(_) => "Échec de la fusion OpenCV".to_string(),
        HdrError::Io { .. } => "Lecture ou écriture de fichier impossible".to_string(),
        HdrError::Write { .. } => "Impossible d’écrire le fichier de sortie".to_string(),
        HdrError::Exif(_) => "EXIF non recopié".to_string(),
    }
}

fn capture_message(error: &CaptureError) -> String {
    match error {
        CaptureError::InvalidPlan(detail) => format!("Plan invalide : {detail}"),
        CaptureError::Cancelled => "Prise de vue annulée".to_string(),
        CaptureError::Failed { step, source } => {
            format!("Planté pendant {step:?} : {source}")
        }
        CaptureError::Camera(inner) => format!("Erreur boîtier : {inner}"),
        CaptureError::Hdr(inner) => hdr_message(inner),
        CaptureError::Io(_) => "Impossible d’écrire la séance".to_string(),
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
