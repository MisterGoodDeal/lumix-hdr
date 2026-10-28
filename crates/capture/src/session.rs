use std::fs;
use std::path::{Path, PathBuf};

use chrono::Local;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::CaptureError;
use crate::plan::BracketPlan;
use crate::state::CaptureState;

pub const MANIFEST_VERSION: u32 = 1;
pub const MANIFEST_FILE: &str = "manifest.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub sha256: String,
    pub relative_ev: f32,
    pub shutter_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionManifest {
    pub version: u32,
    pub created_at: String,
    pub session_dir: String,
    pub plan: BracketPlan,
    pub originals: Vec<FileEntry>,
    pub fused_jpeg: Option<String>,
    pub fused_jpeg_sha256: Option<String>,
    pub state: CaptureState,
    pub errors: Vec<String>,
    pub app_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureSessionResult {
    pub session_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub fused_jpeg: Option<PathBuf>,
    pub state: CaptureState,
    pub originals: Vec<PathBuf>,
}

pub fn create_session_dir(parent: &Path, session_name: &str) -> Result<PathBuf, CaptureError> {
    fs::create_dir_all(parent).map_err(|e| CaptureError::Io(e.to_string()))?;
    let stamp = Local::now().format("%Y-%m-%d_%H%M%S");
    let safe = session_name.trim();
    let mut candidate = parent.join(format!("{stamp}_{safe}"));
    let mut suffix = 2u32;
    while candidate.exists() {
        candidate = parent.join(format!("{stamp}_{safe}_{suffix}"));
        suffix = suffix.saturating_add(1);
        if suffix > 10_000 {
            return Err(CaptureError::Io(
                "impossible de créer un dossier de session unique".to_string(),
            ));
        }
    }
    fs::create_dir(&candidate).map_err(|e| CaptureError::Io(e.to_string()))?;
    Ok(candidate)
}

pub fn sha256_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex_encode(&hasher.finalize())
}

pub fn sha256_file(path: &Path) -> Result<String, CaptureError> {
    let data = fs::read(path).map_err(|e| CaptureError::Io(e.to_string()))?;
    Ok(sha256_bytes(&data))
}

pub fn write_original(
    session_dir: &Path,
    index: usize,
    jpeg: &[u8],
) -> Result<PathBuf, CaptureError> {
    let name = format!("frame_{index:02}.jpg");
    let path = session_dir.join(&name);
    if path.exists() {
        return Err(CaptureError::Io(format!(
            "refus d'écraser {}",
            path.display()
        )));
    }
    fs::write(&path, jpeg).map_err(|e| CaptureError::Io(e.to_string()))?;
    Ok(path)
}

pub fn write_manifest(
    session_dir: &Path,
    manifest: &SessionManifest,
) -> Result<PathBuf, CaptureError> {
    let path = session_dir.join(MANIFEST_FILE);
    let json =
        serde_json::to_string_pretty(manifest).map_err(|e| CaptureError::Message(e.to_string()))?;
    let tmp = session_dir.join(format!("{MANIFEST_FILE}.tmp"));
    fs::write(&tmp, json.as_bytes()).map_err(|e| CaptureError::Io(e.to_string()))?;
    fs::rename(&tmp, &path).map_err(|e| CaptureError::Io(e.to_string()))?;
    Ok(path)
}

/// Déplace une séance staging vers `final_parent` (dossier custom).
///
/// La capture PTP écrit d’abord sous le dossier app (staging) pour éviter les
/// extensions shell / indexation Windows sur `Pictures` & co. pendant le transfert.
pub fn relocate_session(
    result: CaptureSessionResult,
    final_parent: &Path,
) -> Result<CaptureSessionResult, CaptureError> {
    if result.session_dir.parent() == Some(final_parent) {
        return Ok(result);
    }
    fs::create_dir_all(final_parent).map_err(|e| CaptureError::Io(e.to_string()))?;

    let name = result
        .session_dir
        .file_name()
        .ok_or_else(|| CaptureError::Io("dossier de séance sans nom".to_string()))?;
    let mut dest = final_parent.join(name);
    let mut suffix = 2u32;
    while dest.exists() {
        let stem = name.to_string_lossy();
        dest = final_parent.join(format!("{stem}_{suffix}"));
        suffix = suffix.saturating_add(1);
        if suffix > 10_000 {
            return Err(CaptureError::Io(
                "impossible de trouver un nom unique dans le dossier séances".to_string(),
            ));
        }
    }

    if let Err(rename_err) = fs::rename(&result.session_dir, &dest) {
        copy_dir_all(&result.session_dir, &dest)?;
        if let Err(clean_err) = fs::remove_dir_all(&result.session_dir) {
            return Err(CaptureError::Io(format!(
                "séance copiée vers {} mais nettoyage staging échoué ({clean_err}); \
                 rename avait échoué ({rename_err})",
                dest.display()
            )));
        }
    }

    let fused_name = result
        .fused_jpeg
        .as_ref()
        .and_then(|path| path.file_name().map(|n| n.to_string_lossy().into_owned()));
    let originals: Vec<PathBuf> = result
        .originals
        .iter()
        .filter_map(|path| path.file_name().map(|n| dest.join(n)))
        .collect();
    let fused_jpeg = fused_name.map(|name| dest.join(name));
    let manifest_path = dest.join(MANIFEST_FILE);

    // Réécrire le chemin absolu dans le manifeste (best-effort).
    if let Ok(raw) = fs::read_to_string(&manifest_path) {
        if let Ok(mut manifest) = serde_json::from_str::<SessionManifest>(&raw) {
            manifest.session_dir = dest.display().to_string();
            let _ = write_manifest(&dest, &manifest);
        }
    }

    Ok(CaptureSessionResult {
        session_dir: dest,
        manifest_path,
        fused_jpeg,
        state: result.state,
        originals,
    })
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), CaptureError> {
    fs::create_dir_all(dst).map_err(|e| CaptureError::Io(e.to_string()))?;
    for entry in fs::read_dir(src).map_err(|e| CaptureError::Io(e.to_string()))? {
        let entry = entry.map_err(|e| CaptureError::Io(e.to_string()))?;
        let ty = entry
            .file_type()
            .map_err(|e| CaptureError::Io(e.to_string()))?;
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &to)?;
        } else {
            fs::copy(entry.path(), &to).map_err(|e| CaptureError::Io(e.to_string()))?;
        }
    }
    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0xf) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn unique_session_dirs() {
        let parent = env::temp_dir().join(format!("lumixhdr-session-{}", std::process::id()));
        let _ = fs::remove_dir_all(&parent);
        fs::create_dir_all(&parent).expect("parent");
        let a = create_session_dir(&parent, "hdr").expect("a");
        // same second collision: force second call by creating sibling manually
        let b = create_session_dir(&parent, "hdr").expect("b");
        assert_ne!(a, b);
        let _ = fs::remove_dir_all(&parent);
    }

    #[test]
    fn relocate_moves_session_tree() {
        let root = env::temp_dir().join(format!("lumixhdr-reloc-{}", std::process::id()));
        let staging = root.join("staging");
        let custom = root.join("custom");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&staging).expect("staging");
        fs::create_dir_all(&custom).expect("custom");
        let session = create_session_dir(&staging, "hdr").expect("session");
        fs::write(session.join("frame_00.jpg"), b"jpeg").expect("frame");
        fs::write(session.join(MANIFEST_FILE), b"{}").expect("manifest");

        let moved = relocate_session(
            CaptureSessionResult {
                session_dir: session.clone(),
                manifest_path: session.join(MANIFEST_FILE),
                fused_jpeg: None,
                state: CaptureState::Done,
                originals: vec![session.join("frame_00.jpg")],
            },
            &custom,
        )
        .expect("relocate");

        assert!(!session.exists());
        assert!(moved.session_dir.starts_with(&custom));
        assert!(moved.session_dir.join("frame_00.jpg").is_file());
        let _ = fs::remove_dir_all(&root);
    }
}
