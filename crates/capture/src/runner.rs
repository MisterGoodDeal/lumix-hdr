use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use hdr_core::{fuse_folder, FusionOptions, FusionOutputs};
use lumix::{Camera, CameraError, CameraSettings};
use tracing::{info, warn};

use crate::error::CaptureError;
use crate::plan::{build_plan, BracketPlan, BracketRequest};
use crate::session::{
    create_session_dir, sha256_bytes, sha256_file, write_manifest, write_original,
    CaptureSessionResult, FileEntry, SessionManifest, MANIFEST_VERSION,
};
use crate::state::{CaptureState, CaptureStep};

pub type CaptureProgressFn<'a> = dyn FnMut(CaptureProgress) + Send + 'a;

#[derive(Debug, Clone, serde::Serialize)]
pub struct CaptureProgress {
    pub state: CaptureState,
    pub progress: f32,
    pub message: String,
}

pub fn preview_plan(
    camera: &mut dyn Camera,
    request: &BracketRequest,
) -> Result<BracketPlan, CaptureError> {
    if !camera.is_connected() {
        camera.connect()?;
    }
    let settings = camera.read_settings()?;
    let limits = camera.shutter_limits();
    build_plan(request, &settings, limits)
}

pub fn run_capture(
    camera: &mut dyn Camera,
    request: &BracketRequest,
    output_parent: &Path,
    fusion: &FusionOptions,
    cancel: &AtomicBool,
    app_version: &str,
    progress: &mut CaptureProgressFn<'_>,
) -> Result<CaptureSessionResult, CaptureError> {
    emit(
        progress,
        CaptureState::Preparing,
        0.02,
        "Connexion et lecture des réglages",
    );

    if cancel.load(Ordering::SeqCst) {
        return Err(CaptureError::Cancelled);
    }

    if !camera.is_connected() {
        map_cam(CaptureStep::Preparing, camera.connect())?;
    }

    let saved = map_cam(CaptureStep::Preparing, camera.read_settings())?;
    let plan = build_plan(request, &saved, camera.shutter_limits())?;
    let session_dir = create_session_dir(output_parent, request.session_name.trim())?;

    let mut originals: Vec<FileEntry> = Vec::new();
    let mut original_paths: Vec<PathBuf> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let total = plan.slots.len();

    let capture_result = (|| -> Result<Option<FusionOutputs>, CaptureError> {
        for slot in &plan.slots {
            check_cancel(cancel)?;
            let frame = slot.index;
            let frac_base = frame as f32 / total as f32;
            let span = 0.75 / total as f32;

            emit(
                progress,
                CaptureState::SetExposure { frame, total },
                0.05 + frac_base * 0.75,
                &format!("Réglage {} ({})", slot.shutter_label, slot.relative_ev),
            );
            map_cam(
                CaptureStep::SetExposure,
                camera.set_shutter(slot.shutter_seconds),
            )?;

            emit(
                progress,
                CaptureState::VerifyExposure { frame, total },
                0.05 + frac_base * 0.75 + span * 0.15,
                "Vérification de l'exposition",
            );
            map_cam(
                CaptureStep::VerifyExposure,
                camera.verify_shutter(slot.shutter_seconds),
            )?;

            emit(
                progress,
                CaptureState::Trigger { frame, total },
                0.05 + frac_base * 0.75 + span * 0.35,
                "Déclenchement",
            );
            map_cam(CaptureStep::Trigger, camera.trigger())?;

            emit(
                progress,
                CaptureState::WaitCaptureDone { frame, total },
                0.05 + frac_base * 0.75 + span * 0.55,
                "Attente fin de capture",
            );
            map_cam(CaptureStep::WaitCaptureDone, camera.wait_capture_done())?;

            emit(
                progress,
                CaptureState::Download { frame, total },
                0.05 + frac_base * 0.75 + span * 0.75,
                "Téléchargement",
            );
            let jpeg = map_cam(CaptureStep::Download, camera.download_jpeg())?;

            emit(
                progress,
                CaptureState::VerifyFile { frame, total },
                0.05 + frac_base * 0.75 + span * 0.9,
                "Vérification du fichier",
            );
            if jpeg.len() < 100 {
                return Err(CaptureError::failed(
                    CaptureStep::VerifyFile,
                    CameraError::Message("JPEG trop petit ou corrompu".to_string()),
                ));
            }
            let path = write_original(&session_dir, frame, &jpeg)?;
            let hash = sha256_bytes(&jpeg);
            originals.push(FileEntry {
                name: path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                sha256: hash,
                relative_ev: slot.relative_ev,
                shutter_label: slot.shutter_label.clone(),
            });
            original_paths.push(path);
        }

        check_cancel(cancel)?;
        Ok(None)
    })();

    // Toujours restaurer, même après échec / annulation.
    emit(
        progress,
        CaptureState::Restoring,
        0.82,
        "Restauration des réglages",
    );
    if let Err(restore_err) = restore_settings(camera, &saved) {
        warn!(%restore_err, "restauration des réglages incomplète");
        errors.push(format!("restauration: {restore_err}"));
    }

    let mut fused_jpeg = None;
    let mut fused_hash = None;
    let final_state;

    match capture_result {
        Ok(_) => {
            if cancel.load(Ordering::SeqCst) {
                final_state = CaptureState::Aborted;
            } else {
                emit(progress, CaptureState::Merging, 0.88, "Fusion HDR");
                match fuse_folder(&session_dir, fusion, cancel, &mut |p| {
                    let mapped = 0.88 + p * 0.1;
                    emit(
                        progress,
                        CaptureState::Merging,
                        mapped,
                        "Fusion HDR en cours",
                    );
                }) {
                    Ok(outputs) => {
                        if let Some(ref jpeg) = outputs.jpeg {
                            fused_hash = sha256_file(jpeg).ok();
                            fused_jpeg = Some(
                                jpeg.file_name()
                                    .map(|n| n.to_string_lossy().into_owned())
                                    .unwrap_or_else(|| jpeg.display().to_string()),
                            );
                        }
                        final_state = CaptureState::Done;
                        info!(dir = %session_dir.display(), "séance de capture terminée");
                        let _ = outputs;
                    }
                    Err(hdr_core::HdrError::Cancelled) => {
                        final_state = CaptureState::Aborted;
                        errors.push("fusion annulée".to_string());
                    }
                    Err(error) => {
                        final_state = CaptureState::Failed {
                            step: CaptureStep::Merging,
                            error: error.to_string(),
                        };
                        errors.push(error.to_string());
                    }
                }
            }
        }
        Err(CaptureError::Cancelled) => {
            final_state = CaptureState::Aborted;
            errors.push("capture annulée".to_string());
        }
        Err(CaptureError::Failed { step, source }) => {
            final_state = CaptureState::Failed {
                step,
                error: source.to_string(),
            };
            errors.push(source.to_string());
        }
        Err(error) => {
            final_state = CaptureState::Failed {
                step: CaptureStep::Preparing,
                error: error.to_string(),
            };
            errors.push(error.to_string());
        }
    }

    let manifest = SessionManifest {
        version: MANIFEST_VERSION,
        created_at: chrono::Local::now().to_rfc3339(),
        session_dir: session_dir.display().to_string(),
        plan,
        originals,
        fused_jpeg: fused_jpeg.clone(),
        fused_jpeg_sha256: fused_hash,
        state: final_state.clone(),
        errors,
        app_version: app_version.to_string(),
    };
    let manifest_path = write_manifest(&session_dir, &manifest)?;

    let progress_value = match &final_state {
        CaptureState::Done => 1.0,
        CaptureState::Aborted | CaptureState::Failed { .. } => 1.0,
        _ => 0.95,
    };
    emit(
        progress,
        final_state.clone(),
        progress_value,
        &final_state.label_fr(),
    );

    let fused_path = fused_jpeg.as_ref().map(|name| session_dir.join(name));

    let result = CaptureSessionResult {
        session_dir,
        manifest_path,
        fused_jpeg: fused_path,
        state: final_state.clone(),
        originals: original_paths,
    };

    match final_state {
        CaptureState::Done => Ok(result),
        CaptureState::Aborted => Err(CaptureError::Cancelled),
        CaptureState::Failed { step, error } => {
            Err(CaptureError::Message(format!("échec à {step:?}: {error}")))
        }
        other => Err(CaptureError::Message(format!(
            "état final inattendu: {other:?}"
        ))),
    }
}

fn restore_settings(camera: &mut dyn Camera, saved: &CameraSettings) -> Result<(), CaptureError> {
    if !camera.is_connected() {
        // Tentative de reconnexion pour restaurer si possible.
        if let Err(error) = camera.connect() {
            return Err(CaptureError::Camera(error));
        }
    }
    camera.apply_settings(saved).map_err(CaptureError::Camera)
}

fn map_cam<T>(step: CaptureStep, result: Result<T, CameraError>) -> Result<T, CaptureError> {
    result.map_err(|source| CaptureError::failed(step, source))
}

fn check_cancel(cancel: &AtomicBool) -> Result<(), CaptureError> {
    if cancel.load(Ordering::SeqCst) {
        Err(CaptureError::Cancelled)
    } else {
        Ok(())
    }
}

fn emit(progress: &mut CaptureProgressFn<'_>, state: CaptureState, value: f32, message: &str) {
    progress(CaptureProgress {
        state,
        progress: value.clamp(0.0, 1.0),
        message: message.to_string(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    use lumix::{MockCamera, MockFailAt};

    fn request() -> BracketRequest {
        BracketRequest {
            frame_count: 3,
            step_ev: 1.0,
            center_ev: 0.0,
            session_name: "unit".into(),
        }
    }

    #[test]
    fn happy_path_restores_and_merges() {
        let mut camera = MockCamera::new();
        camera.connect().expect("connect");
        let before = camera.read_settings().expect("settings");
        let parent = std::env::temp_dir().join(format!("lumixhdr-cap-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&parent);
        std::fs::create_dir_all(&parent).expect("parent");
        let cancel = AtomicBool::new(false);
        let fusion = FusionOptions::default();
        let result = run_capture(
            &mut camera,
            &request(),
            &parent,
            &fusion,
            &cancel,
            "0.1.0-test",
            &mut |_| {},
        )
        .expect("capture");
        assert!(matches!(result.state, CaptureState::Done));
        assert!(result.fused_jpeg.is_some());
        assert_eq!(result.originals.len(), 3);
        let after = camera.read_settings().expect("after");
        assert_eq!(after.shutter_label, before.shutter_label);
        let _ = std::fs::remove_dir_all(&parent);
    }

    #[test]
    fn restores_after_failure() {
        let mut camera = MockCamera::new();
        camera.connect().expect("connect");
        let before = camera.read_settings().expect("settings");
        camera.inject_failure(MockFailAt::Trigger);
        let parent = std::env::temp_dir().join(format!("lumixhdr-fail-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&parent);
        std::fs::create_dir_all(&parent).expect("parent");
        let cancel = AtomicBool::new(false);
        let err = run_capture(
            &mut camera,
            &request(),
            &parent,
            &FusionOptions::default(),
            &cancel,
            "0.1.0-test",
            &mut |_| {},
        )
        .expect_err("doit échouer");
        assert!(matches!(err, CaptureError::Message(_)));
        let after = camera.read_settings().expect("after");
        assert_eq!(after.shutter_seconds, before.shutter_seconds);
        let _ = std::fs::remove_dir_all(&parent);
    }

    #[test]
    fn cancel_aborts() {
        let mut camera = MockCamera::new();
        camera.connect().expect("connect");
        let parent = std::env::temp_dir().join(format!("lumixhdr-cancel-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&parent);
        std::fs::create_dir_all(&parent).expect("parent");
        let cancel = AtomicBool::new(true);
        let err = run_capture(
            &mut camera,
            &request(),
            &parent,
            &FusionOptions::default(),
            &cancel,
            "0.1.0-test",
            &mut |_| {},
        )
        .expect_err("annulé");
        assert!(matches!(err, CaptureError::Cancelled));
        let _ = std::fs::remove_dir_all(&parent);
    }

    #[test]
    fn disconnect_mid_run() {
        let mut camera = MockCamera::new();
        camera.connect().expect("connect");
        camera.inject_failure(MockFailAt::DisconnectAfterTriggers(1));
        let parent = std::env::temp_dir().join(format!("lumixhdr-disco-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&parent);
        std::fs::create_dir_all(&parent).expect("parent");
        let cancel = AtomicBool::new(false);
        let err = run_capture(
            &mut camera,
            &request(),
            &parent,
            &FusionOptions::default(),
            &cancel,
            "0.1.0-test",
            &mut |_| {},
        )
        .expect_err("déco");
        assert!(matches!(err, CaptureError::Message(_)));
        let _ = std::fs::remove_dir_all(&parent);
    }
}
