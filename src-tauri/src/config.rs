use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use hdr_core::FusionOptions;
use serde::{Deserialize, Serialize};

pub const CONFIG_VERSION: u32 = 1;
pub const DEMO_ENV_VAR: &str = "LUMIX_HDR_DEMO";
pub const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn as_filter(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }
}

/// Derniers paramètres de bracketing (écran Studio).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureDefaults {
    #[serde(default = "default_session_name")]
    pub session_name: String,
    #[serde(default = "default_frame_count")]
    pub frame_count: u32,
    #[serde(default = "default_step_ev")]
    pub step_ev: f64,
    #[serde(default)]
    pub center_ev: f64,
}

fn default_session_name() -> String {
    "hdr".to_string()
}

fn default_frame_count() -> u32 {
    5
}

fn default_step_ev() -> f64 {
    1.0
}

impl Default for CaptureDefaults {
    fn default() -> Self {
        Self {
            session_name: default_session_name(),
            frame_count: default_frame_count(),
            step_ev: default_step_ev(),
            center_ev: 0.0,
        }
    }
}

/// Derniers réglages manuels appliqués à l'appareil.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraDefaults {
    #[serde(default = "default_iso")]
    pub iso: u32,
    #[serde(default = "default_aperture")]
    pub aperture: f32,
    #[serde(default = "default_shutter_seconds")]
    pub shutter_seconds: f64,
}

fn default_iso() -> u32 {
    200
}

fn default_aperture() -> f32 {
    2.8
}

fn default_shutter_seconds() -> f64 {
    1.0 / 60.0
}

impl Default for CameraDefaults {
    fn default() -> Self {
        Self {
            iso: default_iso(),
            aperture: default_aperture(),
            shutter_seconds: default_shutter_seconds(),
        }
    }
}

/// Préférence de langue UI : système (défaut) ou forcée.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LocalePreference {
    #[default]
    System,
    En,
    Fr,
}

/// Préférence de thème UI : système (défaut), sombre ou clair.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    #[default]
    System,
    Dark,
    Light,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: u32,
    #[serde(default)]
    pub demo_mode: bool,
    #[serde(default)]
    pub log_level: LogLevel,
    #[serde(default)]
    pub locale: LocalePreference,
    #[serde(default)]
    pub theme: ThemePreference,
    /// Dossier des séances HDR. `None` = défaut (`app_data/sessions`).
    #[serde(default)]
    pub sessions_dir: Option<String>,
    #[serde(default)]
    pub fusion: FusionOptions,
    #[serde(default)]
    pub capture: CaptureDefaults,
    #[serde(default)]
    pub camera: CameraDefaults,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            demo_mode: false,
            log_level: LogLevel::Info,
            locale: LocalePreference::System,
            theme: ThemePreference::System,
            sessions_dir: None,
            fusion: FusionOptions::default(),
            capture: CaptureDefaults::default(),
            camera: CameraDefaults::default(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("impossible de lire ou d'écrire la configuration: {0}")]
    Io(#[from] std::io::Error),
    #[error("configuration JSON invalide: {0}")]
    Json(#[from] serde_json::Error),
    #[error(
        "version de configuration non prise en charge: {found} (prise en charge: {supported})"
    )]
    UnsupportedVersion { found: u32, supported: u32 },
    #[error("chemin de configuration invalide")]
    InvalidPath,
}

pub fn load_from_path(path: &Path) -> Result<AppConfig, ConfigError> {
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let raw = fs::read_to_string(path)?;
    parse_config(&raw)
}

pub fn parse_config(raw: &str) -> Result<AppConfig, ConfigError> {
    let value: serde_json::Value = serde_json::from_str(raw)?;
    let found = value
        .get("version")
        .and_then(serde_json::Value::as_u64)
        .and_then(|version| u32::try_from(version).ok())
        .unwrap_or(0);

    if found != CONFIG_VERSION {
        return Err(ConfigError::UnsupportedVersion {
            found,
            supported: CONFIG_VERSION,
        });
    }

    Ok(serde_json::from_value(value)?)
}

pub fn save_to_path(path: &Path, config: &AppConfig) -> Result<(), ConfigError> {
    let parent = path.parent().ok_or(ConfigError::InvalidPath)?;
    fs::create_dir_all(parent)?;

    let file_name = path
        .file_name()
        .ok_or(ConfigError::InvalidPath)?
        .to_string_lossy();
    let tmp_path = parent.join(format!(".{file_name}.tmp"));
    let serialized = serde_json::to_string_pretty(config)?;

    {
        let mut file = File::create(&tmp_path)?;
        file.write_all(serialized.as_bytes())?;
        file.write_all(b"\n")?;
        file.sync_all()?;
    }

    replace_file(&tmp_path, path)?;
    Ok(())
}

fn replace_file(tmp: &Path, dest: &Path) -> std::io::Result<()> {
    match fs::rename(tmp, dest) {
        Ok(()) => Ok(()),
        Err(_) if dest.exists() => {
            fs::remove_file(dest)?;
            fs::rename(tmp, dest)
        }
        Err(error) => Err(error),
    }
}

pub fn env_forces_demo(value: Option<&str>) -> bool {
    matches!(
        value.map(str::trim),
        Some("1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON")
    )
}

pub fn resolve_demo_mode(config: &AppConfig, env_value: Option<&str>) -> bool {
    env_forces_demo(env_value) || config.demo_mode
}

pub fn demo_env_value() -> Option<String> {
    std::env::var(DEMO_ENV_VAR).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_config_dir() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("horloge système")
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("lumix-hdr-config-{}-{}", std::process::id(), nanos));
        fs::create_dir_all(&dir).expect("créer le dossier de test");
        dir
    }

    #[test]
    fn missing_file_returns_default() {
        let dir = temp_config_dir();
        let path = dir.join(CONFIG_FILE_NAME);
        let config = load_from_path(&path).expect("chargement");
        assert_eq!(config, AppConfig::default());
    }

    #[test]
    fn roundtrip_atomic_write() {
        let dir = temp_config_dir();
        let path = dir.join(CONFIG_FILE_NAME);
        let config = AppConfig {
            version: CONFIG_VERSION,
            demo_mode: true,
            log_level: LogLevel::Debug,
            locale: LocalePreference::Fr,
            theme: ThemePreference::Dark,
            sessions_dir: None,
            fusion: FusionOptions::default(),
            capture: CaptureDefaults::default(),
            camera: CameraDefaults::default(),
        };
        save_to_path(&path, &config).expect("écriture");
        assert!(path.exists());
        let leftover = dir.join(format!(".{CONFIG_FILE_NAME}.tmp"));
        assert!(!leftover.exists());
        let loaded = load_from_path(&path).expect("relecture");
        assert_eq!(loaded, config);
    }

    #[test]
    fn replace_existing_file() {
        let dir = temp_config_dir();
        let path = dir.join(CONFIG_FILE_NAME);
        save_to_path(&path, &AppConfig::default()).expect("écriture initiale");
        let updated = AppConfig {
            version: CONFIG_VERSION,
            demo_mode: true,
            log_level: LogLevel::Warn,
            locale: LocalePreference::En,
            theme: ThemePreference::Light,
            sessions_dir: Some(dir.join("custom-sessions").display().to_string()),
            fusion: FusionOptions::default(),
            capture: CaptureDefaults {
                session_name: "test".to_string(),
                frame_count: 7,
                step_ev: 1.0,
                center_ev: 0.0,
            },
            camera: CameraDefaults {
                iso: 800,
                aperture: 8.0,
                shutter_seconds: 1.0 / 40.0,
            },
        };
        save_to_path(&path, &updated).expect("remplacement");
        let loaded = load_from_path(&path).expect("relecture");
        assert_eq!(loaded, updated);
    }

    #[test]
    fn future_version_is_rejected() {
        let error = parse_config(r#"{"version": 2, "demo_mode": false}"#).expect_err("version 2");
        match error {
            ConfigError::UnsupportedVersion {
                found: 2,
                supported: 1,
            } => {}
            other => panic!("erreur inattendue: {other:?}"),
        }
    }

    #[test]
    fn missing_version_is_rejected() {
        let error = parse_config(r#"{"demo_mode": true}"#).expect_err("sans version");
        match error {
            ConfigError::UnsupportedVersion { found: 0, .. } => {}
            other => panic!("erreur inattendue: {other:?}"),
        }
    }

    #[test]
    fn invalid_json_is_rejected() {
        let error = parse_config("{").expect_err("json invalide");
        assert!(matches!(error, ConfigError::Json(_)));
    }

    #[test]
    fn missing_fusion_uses_default() {
        let config =
            parse_config(r#"{"version": 1, "demo_mode": false}"#).expect("config minimale");
        assert_eq!(config.fusion, FusionOptions::default());
        assert_eq!(config.capture, CaptureDefaults::default());
        assert_eq!(config.camera, CameraDefaults::default());
    }

    #[test]
    fn env_one_forces_demo() {
        let config = AppConfig::default();
        assert!(resolve_demo_mode(&config, Some("1")));
        assert!(resolve_demo_mode(&config, Some("true")));
        assert!(!resolve_demo_mode(&config, Some("0")));
        assert!(!resolve_demo_mode(&config, None));
    }

    #[test]
    fn env_does_not_disable_config_demo() {
        let config = AppConfig {
            version: CONFIG_VERSION,
            demo_mode: true,
            log_level: LogLevel::Info,
            locale: LocalePreference::System,
            theme: ThemePreference::System,
            sessions_dir: None,
            fusion: FusionOptions::default(),
            capture: CaptureDefaults::default(),
            camera: CameraDefaults::default(),
        };
        assert!(resolve_demo_mode(&config, Some("0")));
        assert!(resolve_demo_mode(&config, None));
    }

    #[test]
    fn missing_sessions_dir_is_none() {
        let config =
            parse_config(r#"{"version": 1, "demo_mode": false}"#).expect("config minimale");
        assert_eq!(config.sessions_dir, None);
    }

    #[test]
    fn missing_locale_defaults_to_system() {
        let config =
            parse_config(r#"{"version": 1, "demo_mode": false}"#).expect("config minimale");
        assert_eq!(config.locale, LocalePreference::System);
    }

    #[test]
    fn missing_theme_defaults_to_system() {
        let config =
            parse_config(r#"{"version": 1, "demo_mode": false}"#).expect("config minimale");
        assert_eq!(config.theme, ThemePreference::System);
    }
}
