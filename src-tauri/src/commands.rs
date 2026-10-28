use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use capture::{
    preview_plan, run_capture, BracketPlan, BracketRequest, CaptureProgress, CaptureSessionResult,
};
use hdr_core::{fuse_folder, jpeg_files_in_dir, FusionOptions, FusionOutputs};
use lumix::MockCamera;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;

use crate::config::{
    demo_env_value, env_forces_demo, resolve_demo_mode, save_to_path, AppConfig, LogLevel,
};
use crate::error::{AppError, ErrorPayload};
use crate::logging::{self, FilterHandle};

pub const FUSION_PROGRESS_EVENT: &str = "fusion-progress";
pub const CAPTURE_PROGRESS_EVENT: &str = "capture-progress";

pub struct AppState {
    pub config_path: PathBuf,
    pub log_dir: PathBuf,
    pub sessions_dir: PathBuf,
    pub config: Mutex<AppConfig>,
    pub filter: FilterHandle,
    fusion_cancel: Mutex<Arc<AtomicBool>>,
    fusion_running: AtomicBool,
    capture_cancel: Mutex<Arc<AtomicBool>>,
    capture_running: AtomicBool,
    _guard: WorkerGuard,
}

impl AppState {
    pub fn new(
        config_path: PathBuf,
        log_dir: PathBuf,
        sessions_dir: PathBuf,
        config: AppConfig,
        filter: FilterHandle,
        guard: WorkerGuard,
    ) -> Self {
        Self {
            config_path,
            log_dir,
            sessions_dir,
            config: Mutex::new(config),
            filter,
            fusion_cancel: Mutex::new(Arc::new(AtomicBool::new(false))),
            fusion_running: AtomicBool::new(false),
            capture_cancel: Mutex::new(Arc::new(AtomicBool::new(false))),
            capture_running: AtomicBool::new(false),
            _guard: guard,
        }
    }
}

struct FlagGuard<'a> {
    flag: &'a AtomicBool,
}

impl Drop for FlagGuard<'_> {
    fn drop(&mut self) {
        self.flag.store(false, Ordering::SeqCst);
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
    pub sessions_dir: String,
    pub fusion: FusionOptions,
}

fn lock_config(state: &AppState) -> Result<std::sync::MutexGuard<'_, AppConfig>, AppError> {
    state.config.lock().map_err(|_| AppError::LockPoisoned)
}

fn camera_available() -> bool {
    cfg!(windows)
}

fn build_status(
    config: &AppConfig,
    log_dir: &std::path::Path,
    sessions_dir: &std::path::Path,
) -> AppStatus {
    let env_value = demo_env_value();
    AppStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
        os: std::env::consts::OS.to_string(),
        demo_mode: resolve_demo_mode(config, env_value.as_deref()),
        demo_forced_by_env: env_forces_demo(env_value.as_deref()),
        camera_available: camera_available(),
        log_level: config.log_level,
        log_dir: log_dir.display().to_string(),
        sessions_dir: sessions_dir.display().to_string(),
        fusion: config.fusion.clone(),
    }
}

#[tauri::command]
pub fn get_app_status(state: State<'_, AppState>) -> Result<AppStatus, ErrorPayload> {
    let config = lock_config(&state).map_err(ErrorPayload::from)?;
    Ok(build_status(&config, &state.log_dir, &state.sessions_dir))
}

#[tauri::command]
pub fn set_demo_mode(enabled: bool, state: State<'_, AppState>) -> Result<AppStatus, ErrorPayload> {
    let mut config = lock_config(&state).map_err(ErrorPayload::from)?;
    config.demo_mode = enabled;
    save_to_path(&state.config_path, &config)?;
    info!(demo_mode = enabled, "mode démo mis à jour");
    Ok(build_status(&config, &state.log_dir, &state.sessions_dir))
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
    Ok(build_status(&config, &state.log_dir, &state.sessions_dir))
}

#[tauri::command]
pub fn get_log_dir(state: State<'_, AppState>) -> Result<String, ErrorPayload> {
    Ok(state.log_dir.display().to_string())
}

#[tauri::command]
pub fn set_fusion_options(
    options: FusionOptions,
    state: State<'_, AppState>,
) -> Result<AppStatus, ErrorPayload> {
    let mut config = lock_config(&state).map_err(ErrorPayload::from)?;
    config.fusion = options;
    save_to_path(&state.config_path, &config)?;
    info!(
        method = ?config.fusion.method,
        align = config.fusion.align,
        "options de fusion mises à jour"
    );
    Ok(build_status(&config, &state.log_dir, &state.sessions_dir))
}

#[tauri::command]
pub fn list_jpeg_folder(dir: String) -> Result<Vec<String>, ErrorPayload> {
    let files = jpeg_files_in_dir(Path::new(&dir))?;
    Ok(files
        .into_iter()
        .map(|path| path.display().to_string())
        .collect())
}

#[tauri::command]
pub async fn merge_image_folder(
    app: AppHandle,
    dir: String,
    options: FusionOptions,
    state: State<'_, AppState>,
) -> Result<FusionOutputs, ErrorPayload> {
    if state.capture_running.load(Ordering::SeqCst) {
        return Err(AppError::Message(
            "Une prise de vue est en cours ; fusion impossible.".to_string(),
        )
        .into());
    }
    if state
        .fusion_running
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err(AppError::Message("Une fusion est déjà en cours.".to_string()).into());
    }
    let _guard = FlagGuard {
        flag: &state.fusion_running,
    };

    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut slot = state
            .fusion_cancel
            .lock()
            .map_err(|_| AppError::LockPoisoned)?;
        *slot = cancel.clone();
    }

    {
        let mut config = lock_config(&state)?;
        config.fusion = options.clone();
        save_to_path(&state.config_path, &config)?;
    }

    let folder = PathBuf::from(&dir);
    let app_for_progress = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        fuse_folder(&folder, &options, &cancel, &mut |progress| {
            let _ = app_for_progress.emit(FUSION_PROGRESS_EVENT, progress);
        })
    })
    .await
    .map_err(|_| AppError::Message("La fusion a été interrompue.".to_string()))?;

    match &result {
        Ok(outputs) => info!(
            jpeg = ?outputs.jpeg,
            tiff = ?outputs.tiff,
            count = outputs.image_count,
            "fusion terminée"
        ),
        Err(error) => tracing::warn!(%error, "fusion échouée"),
    }

    Ok(result?)
}

#[tauri::command]
pub fn cancel_fusion(state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    let cancel = state
        .fusion_cancel
        .lock()
        .map_err(|_| AppError::LockPoisoned)?;
    cancel.store(true, Ordering::SeqCst);
    info!("annulation de fusion demandée");
    Ok(())
}

#[tauri::command]
pub fn open_path(path: String) -> Result<(), ErrorPayload> {
    let target = PathBuf::from(&path);
    if !target.exists() {
        return Err(AppError::Message(format!("chemin introuvable: {path}")).into());
    }
    open_path_native(&target)
}

fn open_path_native(path: &Path) -> Result<(), ErrorPayload> {
    let mut command = {
        #[cfg(target_os = "windows")]
        {
            let mut command = std::process::Command::new("explorer");
            command.arg(path);
            command
        }
        #[cfg(target_os = "macos")]
        {
            let mut command = std::process::Command::new("open");
            command.arg(path);
            command
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            let mut command = std::process::Command::new("xdg-open");
            command.arg(path);
            command
        }
    };
    command.spawn().map_err(|error| {
        AppError::Message(format!("impossible d'ouvrir {}: {error}", path.display()))
    })?;
    Ok(())
}

#[tauri::command]
pub fn preview_bracket_plan(
    request: BracketRequest,
    state: State<'_, AppState>,
) -> Result<BracketPlan, ErrorPayload> {
    ensure_demo_for_capture(&state)?;
    let mut camera = MockCamera::new();
    Ok(preview_plan(&mut camera, &request)?)
}

#[tauri::command]
pub async fn start_capture(
    app: AppHandle,
    request: BracketRequest,
    state: State<'_, AppState>,
) -> Result<CaptureSessionResult, ErrorPayload> {
    ensure_demo_for_capture(&state)?;
    if state.fusion_running.load(Ordering::SeqCst) {
        return Err(AppError::Message(
            "Une fusion est en cours ; prise de vue impossible.".to_string(),
        )
        .into());
    }
    if state
        .capture_running
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err(AppError::Message("Une prise de vue est déjà en cours.".to_string()).into());
    }
    let _guard = FlagGuard {
        flag: &state.capture_running,
    };

    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut slot = state
            .capture_cancel
            .lock()
            .map_err(|_| AppError::LockPoisoned)?;
        *slot = cancel.clone();
    }

    let fusion = {
        let config = lock_config(&state)?;
        config.fusion.clone()
    };
    let sessions_dir = state.sessions_dir.clone();
    let app_version = env!("CARGO_PKG_VERSION").to_string();
    let app_for_progress = app.clone();

    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut camera = MockCamera::new();
        run_capture(
            &mut camera,
            &request,
            &sessions_dir,
            &fusion,
            &cancel,
            &app_version,
            &mut |update: CaptureProgress| {
                let _ = app_for_progress.emit(CAPTURE_PROGRESS_EVENT, update);
            },
        )
    })
    .await
    .map_err(|_| AppError::Message("La prise de vue a été interrompue.".to_string()))?;

    match &result {
        Ok(session) => info!(
            dir = %session.session_dir.display(),
            fused = ?session.fused_jpeg,
            "prise de vue terminée"
        ),
        Err(error) => tracing::warn!(%error, "prise de vue échouée"),
    }

    Ok(result?)
}

#[tauri::command]
pub fn cancel_capture(state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    let cancel = state
        .capture_cancel
        .lock()
        .map_err(|_| AppError::LockPoisoned)?;
    cancel.store(true, Ordering::SeqCst);
    info!("annulation de prise de vue demandée");
    Ok(())
}

fn ensure_demo_for_capture(state: &AppState) -> Result<(), ErrorPayload> {
    let config = lock_config(state)?;
    let env_value = demo_env_value();
    if !resolve_demo_mode(&config, env_value.as_deref()) {
        return Err(AppError::Message(
            "La prise de vue phase 3 utilise MockCamera : activez le mode démo.".to_string(),
        )
        .into());
    }
    Ok(())
}

pub fn resolve_paths(app: &AppHandle) -> Result<(PathBuf, PathBuf, PathBuf), AppError> {
    let config_dir = app.path().app_config_dir().map_err(|error| {
        AppError::Message(format!("dossier de configuration introuvable: {error}"))
    })?;
    let log_dir = app
        .path()
        .app_log_dir()
        .map_err(|error| AppError::Message(format!("dossier de journaux introuvable: {error}")))?;
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| AppError::Message(format!("dossier de données introuvable: {error}")))?;
    let sessions_dir = data_dir.join("sessions");
    std::fs::create_dir_all(&sessions_dir).map_err(|error| {
        AppError::Message(format!(
            "impossible de créer le dossier de sessions: {error}"
        ))
    })?;
    Ok((
        config_dir.join(crate::config::CONFIG_FILE_NAME),
        log_dir,
        sessions_dir,
    ))
}
