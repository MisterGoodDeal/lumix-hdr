use serde::{Deserialize, Serialize};

/// Étapes atomiques (pour Failed { step }).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureStep {
    Idle,
    Preparing,
    SetExposure,
    VerifyExposure,
    Trigger,
    WaitCaptureDone,
    Download,
    VerifyFile,
    Restoring,
    Merging,
    Done,
    Aborted,
}

/// État exposé à l'UI / manifeste.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CaptureState {
    Idle,
    Preparing,
    SetExposure { frame: usize, total: usize },
    VerifyExposure { frame: usize, total: usize },
    Trigger { frame: usize, total: usize },
    WaitCaptureDone { frame: usize, total: usize },
    Download { frame: usize, total: usize },
    VerifyFile { frame: usize, total: usize },
    Restoring,
    Merging,
    Done,
    Aborted,
    Failed { step: CaptureStep, error: String },
}

impl CaptureState {
    pub fn step(&self) -> CaptureStep {
        match self {
            Self::Idle => CaptureStep::Idle,
            Self::Preparing => CaptureStep::Preparing,
            Self::SetExposure { .. } => CaptureStep::SetExposure,
            Self::VerifyExposure { .. } => CaptureStep::VerifyExposure,
            Self::Trigger { .. } => CaptureStep::Trigger,
            Self::WaitCaptureDone { .. } => CaptureStep::WaitCaptureDone,
            Self::Download { .. } => CaptureStep::Download,
            Self::VerifyFile { .. } => CaptureStep::VerifyFile,
            Self::Restoring => CaptureStep::Restoring,
            Self::Merging => CaptureStep::Merging,
            Self::Done => CaptureStep::Done,
            Self::Aborted => CaptureStep::Aborted,
            Self::Failed { step, .. } => *step,
        }
    }

    pub fn label_fr(&self) -> String {
        match self {
            Self::Idle => "En attente".to_string(),
            Self::Preparing => "Préparation".to_string(),
            Self::SetExposure { frame, total } => {
                format!("Réglage expo ({}/{})", frame + 1, total)
            }
            Self::VerifyExposure { frame, total } => {
                format!("Vérification expo ({}/{})", frame + 1, total)
            }
            Self::Trigger { frame, total } => format!("Déclenchement ({}/{})", frame + 1, total),
            Self::WaitCaptureDone { frame, total } => {
                format!("Attente capture ({}/{})", frame + 1, total)
            }
            Self::Download { frame, total } => format!("Téléchargement ({}/{})", frame + 1, total),
            Self::VerifyFile { frame, total } => {
                format!("Vérification fichier ({}/{})", frame + 1, total)
            }
            Self::Restoring => "Restauration des réglages".to_string(),
            Self::Merging => "Fusion HDR".to_string(),
            Self::Done => "Terminé".to_string(),
            Self::Aborted => "Annulé".to_string(),
            Self::Failed { step, error } => format!("Échec ({step:?}): {error}"),
        }
    }
}
