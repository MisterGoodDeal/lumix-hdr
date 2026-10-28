use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::CaptureError;
use crate::session::{FileEntry, SessionManifest, MANIFEST_FILE};
use crate::state::CaptureState;

/// Résumé d’une séance pour la grille Historique.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_dir: String,
    pub created_at: String,
    pub session_name: String,
    pub frame_count: u8,
    pub step_ev: f32,
    pub center_ev: f32,
    pub state_kind: String,
    pub thumb_jpeg: Option<String>,
    pub has_fused: bool,
    pub original_count: usize,
    pub error_count: usize,
}

/// Détail complet pour l’écran avant/après.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDetail {
    pub session_dir: String,
    pub created_at: String,
    pub session_name: String,
    pub app_version: String,
    pub state: CaptureState,
    pub errors: Vec<String>,
    pub frame_count: u8,
    pub step_ev: f32,
    pub center_ev: f32,
    pub base_iso: u32,
    pub base_aperture: f32,
    pub base_shutter_label: String,
    pub originals: Vec<FileEntry>,
    pub original_paths: Vec<String>,
    pub fused_jpeg: Option<String>,
    pub before_jpeg: Option<String>,
    pub slots: Vec<crate::plan::ExposureSlot>,
}

/// Liste les séances (dossiers avec `manifest.json`), plus récentes d’abord.
pub fn list_session_summaries(parent: &Path) -> Result<Vec<SessionSummary>, CaptureError> {
    if !parent.exists() {
        return Ok(Vec::new());
    }
    let entries = fs::read_dir(parent).map_err(|e| CaptureError::Io(e.to_string()))?;
    let mut summaries = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| CaptureError::Io(e.to_string()))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let manifest_path = path.join(MANIFEST_FILE);
        if !manifest_path.is_file() {
            continue;
        }
        match load_summary_from_dir(&path) {
            Ok(summary) => summaries.push(summary),
            Err(error) => {
                tracing::debug!(
                    dir = %path.display(),
                    %error,
                    "séance ignorée (manifeste illisible)"
                );
            }
        }
    }
    summaries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(summaries)
}

/// Charge le détail d’une séance. `parent` borne le chemin (anti path-traversal).
pub fn load_session_detail(
    parent: &Path,
    session_dir: &Path,
) -> Result<SessionDetail, CaptureError> {
    let canonical_session = resolve_session_dir(parent, session_dir)?;
    let manifest = read_manifest(&canonical_session)?;
    Ok(detail_from_manifest(&canonical_session, manifest))
}

/// Supprime une séance (dossier entier). `parent` borne le chemin.
pub fn delete_session(parent: &Path, session_dir: &Path) -> Result<(), CaptureError> {
    let canonical_session = resolve_session_dir(parent, session_dir)?;
    // Exige un manifeste : évite d’effacer un dossier quelconque sous sessions/.
    let manifest_path = canonical_session.join(MANIFEST_FILE);
    if !manifest_path.is_file() {
        return Err(CaptureError::Message(
            "pas de manifeste — suppression refusée".to_string(),
        ));
    }
    fs::remove_dir_all(&canonical_session).map_err(|e| CaptureError::Io(e.to_string()))?;
    Ok(())
}

fn resolve_session_dir(parent: &Path, session_dir: &Path) -> Result<PathBuf, CaptureError> {
    let canonical_parent = canonicalize_dir(parent)?;
    let canonical_session = canonicalize_dir(session_dir)?;
    if !canonical_session.starts_with(&canonical_parent) {
        return Err(CaptureError::Message(
            "dossier de séance hors du répertoire sessions".to_string(),
        ));
    }
    if canonical_session == canonical_parent {
        return Err(CaptureError::Message(
            "refus de supprimer le dossier sessions racine".to_string(),
        ));
    }
    Ok(canonical_session)
}

fn load_summary_from_dir(dir: &Path) -> Result<SessionSummary, CaptureError> {
    let manifest = read_manifest(dir)?;
    let fused_path = manifest
        .fused_jpeg
        .as_ref()
        .map(|name| dir.join(name))
        .filter(|path| path.is_file());
    let before = pick_before_frame(dir, &manifest.originals);
    let thumb = fused_path
        .as_ref()
        .or(before.as_ref())
        .map(|path| path.display().to_string());
    Ok(SessionSummary {
        session_dir: dir.display().to_string(),
        created_at: manifest.created_at,
        session_name: manifest.plan.request.session_name,
        frame_count: manifest.plan.request.frame_count,
        step_ev: manifest.plan.request.step_ev,
        center_ev: manifest.plan.request.center_ev,
        state_kind: state_kind(&manifest.state),
        thumb_jpeg: thumb,
        has_fused: fused_path.is_some(),
        original_count: manifest.originals.len(),
        error_count: manifest.errors.len(),
    })
}

fn detail_from_manifest(dir: &Path, manifest: SessionManifest) -> SessionDetail {
    let fused_jpeg = manifest
        .fused_jpeg
        .as_ref()
        .map(|name| dir.join(name))
        .filter(|path| path.is_file())
        .map(|path| path.display().to_string());
    let before_jpeg = pick_before_frame(dir, &manifest.originals).map(|p| p.display().to_string());
    let original_paths: Vec<String> = manifest
        .originals
        .iter()
        .map(|entry| dir.join(&entry.name).display().to_string())
        .collect();
    SessionDetail {
        session_dir: dir.display().to_string(),
        created_at: manifest.created_at,
        session_name: manifest.plan.request.session_name,
        app_version: manifest.app_version,
        state: manifest.state,
        errors: manifest.errors,
        frame_count: manifest.plan.request.frame_count,
        step_ev: manifest.plan.request.step_ev,
        center_ev: manifest.plan.request.center_ev,
        base_iso: manifest.plan.base.iso,
        base_aperture: manifest.plan.base.aperture,
        base_shutter_label: manifest.plan.base.shutter_label,
        originals: manifest.originals,
        original_paths,
        fused_jpeg,
        before_jpeg,
        slots: manifest.plan.slots,
    }
}

fn pick_before_frame(dir: &Path, originals: &[FileEntry]) -> Option<PathBuf> {
    if originals.is_empty() {
        return None;
    }
    let best = originals.iter().min_by(|a, b| {
        a.relative_ev
            .abs()
            .partial_cmp(&b.relative_ev.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    })?;
    let path = dir.join(&best.name);
    path.is_file().then_some(path)
}

fn read_manifest(dir: &Path) -> Result<SessionManifest, CaptureError> {
    let path = dir.join(MANIFEST_FILE);
    let raw = fs::read_to_string(&path).map_err(|e| CaptureError::Io(e.to_string()))?;
    serde_json::from_str(&raw).map_err(|e| CaptureError::Message(format!("manifeste: {e}")))
}

fn state_kind(state: &CaptureState) -> String {
    match state {
        CaptureState::Done => "done".to_string(),
        CaptureState::Aborted => "aborted".to_string(),
        CaptureState::Failed { .. } => "failed".to_string(),
        other => format!("{:?}", other).to_ascii_lowercase(),
    }
}

fn canonicalize_dir(path: &Path) -> Result<PathBuf, CaptureError> {
    fs::canonicalize(path).map_err(|e| CaptureError::Io(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{BracketPlan, BracketRequest, ExposureSlot};
    use crate::session::{create_session_dir, write_manifest, FileEntry};
    use lumix::{CameraSettings, ExposureMode};

    fn sample_manifest(dir: &Path) -> SessionManifest {
        SessionManifest {
            version: 1,
            created_at: "2026-10-28T12:00:00+01:00".to_string(),
            session_dir: dir.display().to_string(),
            plan: BracketPlan {
                request: BracketRequest {
                    frame_count: 3,
                    step_ev: 1.0,
                    center_ev: 0.0,
                    session_name: "hdr".into(),
                },
                base: CameraSettings {
                    mode: ExposureMode::M,
                    aperture: 2.8,
                    iso: 200,
                    shutter_seconds: 0.01,
                    shutter_label: "1/100".into(),
                },
                slots: vec![ExposureSlot {
                    index: 0,
                    relative_ev: 0.0,
                    shutter_seconds: 0.01,
                    shutter_label: "1/100".into(),
                }],
            },
            originals: vec![FileEntry {
                name: "frame_00.jpg".into(),
                sha256: "abc".into(),
                relative_ev: 0.0,
                shutter_label: "1/100".into(),
            }],
            fused_jpeg: Some("hdr-fused.jpg".into()),
            fused_jpeg_sha256: None,
            state: CaptureState::Done,
            errors: vec![],
            app_version: "0.1.0".into(),
        }
    }

    #[test]
    fn lists_and_loads_session() {
        let root = std::env::temp_dir().join(format!("lumixhdr-hist-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("root");
        let session = create_session_dir(&root, "hdr").expect("session");
        fs::write(session.join("frame_00.jpg"), b"before").expect("frame");
        fs::write(session.join("hdr-fused.jpg"), b"after").expect("fused");
        write_manifest(&session, &sample_manifest(&session)).expect("manifest");

        let list = list_session_summaries(&root).expect("list");
        assert_eq!(list.len(), 1);
        assert!(list[0].has_fused);
        assert_eq!(list[0].session_name, "hdr");

        let detail = load_session_detail(&root, &session).expect("detail");
        assert!(detail.fused_jpeg.is_some());
        assert!(detail.before_jpeg.is_some());
        assert_eq!(detail.originals.len(), 1);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn rejects_path_outside_parent() {
        let root = std::env::temp_dir().join(format!("lumixhdr-hist-out-{}", std::process::id()));
        let other =
            std::env::temp_dir().join(format!("lumixhdr-hist-other-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&other);
        fs::create_dir_all(&root).expect("root");
        fs::create_dir_all(&other).expect("other");
        let err = load_session_detail(&root, &other).expect_err("outside");
        assert!(matches!(err, CaptureError::Message(_)));
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&other);
    }

    #[test]
    fn deletes_session_folder() {
        let root = std::env::temp_dir().join(format!("lumixhdr-hist-del-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("root");
        let session = create_session_dir(&root, "hdr").expect("session");
        fs::write(session.join("frame_00.jpg"), b"x").expect("frame");
        write_manifest(&session, &sample_manifest(&session)).expect("manifest");
        delete_session(&root, &session).expect("delete");
        assert!(!session.exists());
        assert!(list_session_summaries(&root).expect("list").is_empty());
        let _ = fs::remove_dir_all(&root);
    }
}
