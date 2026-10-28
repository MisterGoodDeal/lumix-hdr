use serde::Serialize;

#[derive(Debug, Clone, thiserror::Error, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CameraError {
    #[error("appareil déjà occupé")]
    Busy,
    #[error("délai d'attente dépassé")]
    Timeout,
    #[error("appareil déconnecté")]
    Disconnected,
    #[error("appareil non connecté")]
    NotConnected,
    #[error("réglage hors plage: {0}")]
    OutOfRange(String),
    #[error("échec de vérification: {0}")]
    VerifyFailed(String),
    #[error("{0}")]
    Message(String),
}
