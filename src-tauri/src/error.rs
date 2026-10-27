use serde::Serialize;

use crate::config::ConfigError;
use crate::logging::LoggingError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Config(#[from] ConfigError),
    #[error("{0}")]
    Logging(#[from] LoggingError),
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
