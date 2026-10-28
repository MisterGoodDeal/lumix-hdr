use serde::Serialize;

use hdr_core::HdrError;
use lumix::CameraError;

use crate::state::CaptureStep;

#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("plan invalide: {0}")]
    InvalidPlan(String),
    #[error("annulé")]
    Cancelled,
    #[error("étape {step:?}: {source}")]
    Failed {
        step: CaptureStep,
        #[source]
        source: CameraError,
    },
    #[error("{0}")]
    Camera(#[from] CameraError),
    #[error("{0}")]
    Hdr(#[from] HdrError),
    #[error("E/S: {0}")]
    Io(String),
    #[error("{0}")]
    Message(String),
}

impl CaptureError {
    pub fn failed(step: CaptureStep, source: CameraError) -> Self {
        Self::Failed { step, source }
    }
}

impl Serialize for CaptureError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
