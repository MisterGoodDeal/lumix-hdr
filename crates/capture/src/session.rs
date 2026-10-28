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
}
