use std::path::PathBuf;
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;

use crate::config::{
    demo_env_value, env_forces_demo, resolve_demo_mode, save_to_path, AppConfig, LogLevel,
};
use crate::error::{AppError, ErrorPayload};
use crate::logging::{self, FilterHandle};

pub struct AppState {
    pub config_path: PathBuf,
    pub log_dir: PathBuf,
    pub config: Mutex<AppConfig>,
    pub filter: FilterHandle,
    _guard: WorkerGuard,
}

impl AppState {
    pub fn new(
        config_path: PathBuf,
        log_dir: PathBuf,
        config: AppConfig,
        filter: FilterHandle,
        guard: WorkerGuard,
    ) -> Self {
        Self {
            config_path,
            log_dir,
            config: Mutex::new(config),
            filter,
            _guard: guard,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AppStatus {
    pub version: String,
    pub os: String,
    pub demo_mode: bool,
    pub demo_forced_by_env: bool,
    pub camera_available: bool,
    pub log_level: LogLevel,
    pub log_dir: String,
}

fn lock_config(state: &AppState) -> Result<std::sync::MutexGuard<'_, AppConfig>, AppError> {
    state.config.lock().map_err(|_| AppError::LockPoisoned)
}

fn camera_available() -> bool {
    cfg!(windows)
}

fn build_status(config: &AppConfig, log_dir: &std::path::Path) -> AppStatus {
    let env_value = demo_env_value();
    AppStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
        os: std::env::consts::OS.to_string(),
        demo_mode: resolve_demo_mode(config, env_value.as_deref()),
        demo_forced_by_env: env_forces_demo(env_value.as_deref()),
        camera_available: camera_available(),
        log_level: config.log_level,
        log_dir: log_dir.display().to_string(),
    }
}

#[tauri::command]
pub fn get_app_status(state: State<'_, AppState>) -> Result<AppStatus, ErrorPayload> {
    let config = lock_config(&state).map_err(ErrorPayload::from)?;
    Ok(build_status(&config, &state.log_dir))
}

#[tauri::command]
pub fn set_demo_mode(enabled: bool, state: State<'_, AppState>) -> Result<AppStatus, ErrorPayload> {
    let mut config = lock_config(&state).map_err(ErrorPayload::from)?;
    config.demo_mode = enabled;
    save_to_path(&state.config_path, &config)?;
    info!(demo_mode = enabled, "mode démo mis à jour");
    Ok(build_status(&config, &state.log_dir))
}

#[tauri::command]
pub fn set_log_level(
    level: LogLevel,
    state: State<'_, AppState>,
) -> Result<AppStatus, ErrorPayload> {
    let mut config = lock_config(&state).map_err(ErrorPayload::from)?;
    config.log_level = level;
    save_to_path(&state.config_path, &config)?;
    logging::reload_level(&state.filter, level).map_err(AppError::from)?;
    info!(?level, "niveau de journalisation mis à jour");
    Ok(build_status(&config, &state.log_dir))
}

#[tauri::command]
pub fn get_log_dir(state: State<'_, AppState>) -> Result<String, ErrorPayload> {
    Ok(state.log_dir.display().to_string())
}

pub fn resolve_paths(app: &AppHandle) -> Result<(PathBuf, PathBuf), AppError> {
    let config_dir = app.path().app_config_dir().map_err(|error| {
        AppError::Message(format!("dossier de configuration introuvable: {error}"))
    })?;
    let log_dir = app
        .path()
        .app_log_dir()
        .map_err(|error| AppError::Message(format!("dossier de journaux introuvable: {error}")))?;
    Ok((config_dir.join(crate::config::CONFIG_FILE_NAME), log_dir))
}
