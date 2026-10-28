use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use capture::{
    delete_session as remove_session_dir, list_session_summaries, load_session_detail,
    preview_plan, relocate_session, run_capture, BracketPlan, BracketRequest, CaptureProgress,
    CaptureSessionResult, SessionDetail, SessionSummary,
};
use hdr_core::{fuse_folder, jpeg_files_in_dir, FusionOptions, FusionOutputs};
use lumix::{
    format_shutter, Camera, CameraSettings, ExposureCapabilities, ExposureMode, MockCamera,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;

use crate::config::{
    demo_env_value, env_forces_demo, resolve_demo_mode, save_to_path, AppConfig, CameraDefaults,
    CaptureDefaults, LocalePreference, LogLevel, ThemePreference,
};
use crate::error::{AppError, ErrorPayload};
use crate::logging::{self, FilterHandle};

pub const FUSION_PROGRESS_EVENT: &str = "fusion-progress";
pub const CAPTURE_PROGRESS_EVENT: &str = "capture-progress";
pub const LIVEVIEW_FRAME_EVENT: &str = "liveview-frame";
pub const CAMERA_DISCONNECTED_EVENT: &str = "camera-disconnected";
pub const CAMERA_SETTINGS_CHANGED_EVENT: &str = "camera-settings-changed";

pub struct AppState {
    pub config_path: PathBuf,
    pub log_dir: PathBuf,
    /// Dossier sessions par défaut (`app_data/sessions`).
    pub default_sessions_dir: PathBuf,
    /// Dossier sessions effectif (défaut ou personnalisé).
    pub sessions_dir: Mutex<PathBuf>,
    pub config: Mutex<AppConfig>,
    pub filter: FilterHandle,
    /// Session caméra ouverte (réelle ou mock) — ne pas reconnecter à chaque commande.
    camera: Mutex<Option<Box<dyn Camera>>>,
    fusion_cancel: Mutex<Arc<AtomicBool>>,
    fusion_running: AtomicBool,
    capture_cancel: Mutex<Arc<AtomicBool>>,
    capture_running: AtomicBool,
    live_view_stop: Arc<AtomicBool>,
    live_view_handle: Mutex<Option<JoinHandle<()>>>,
    unplug_stop: Arc<AtomicBool>,
    unplug_handle: Mutex<Option<JoinHandle<()>>>,
    settings_watch_stop: Arc<AtomicBool>,
    settings_watch_handle: Mutex<Option<JoinHandle<()>>>,
    _guard: WorkerGuard,
}

impl AppState {
    pub fn new(
        config_path: PathBuf,
        log_dir: PathBuf,
        default_sessions_dir: PathBuf,
        sessions_dir: PathBuf,
        config: AppConfig,
        filter: FilterHandle,
        guard: WorkerGuard,
    ) -> Self {
        Self {
            config_path,
            log_dir,
            default_sessions_dir,
            sessions_dir: Mutex::new(sessions_dir),
            config: Mutex::new(config),
            filter,
            camera: Mutex::new(None),
            fusion_cancel: Mutex::new(Arc::new(AtomicBool::new(false))),
            fusion_running: AtomicBool::new(false),
            capture_cancel: Mutex::new(Arc::new(AtomicBool::new(false))),
            capture_running: AtomicBool::new(false),
            live_view_stop: Arc::new(AtomicBool::new(true)),
            live_view_handle: Mutex::new(None),
            unplug_stop: Arc::new(AtomicBool::new(true)),
            unplug_handle: Mutex::new(None),
            settings_watch_stop: Arc::new(AtomicBool::new(true)),
            settings_watch_handle: Mutex::new(None),
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
    pub locale: LocalePreference,
    pub theme: ThemePreference,
    pub log_dir: String,
    pub sessions_dir: String,
    pub sessions_dir_custom: bool,
    pub fusion: FusionOptions,
    pub capture: CaptureDefaults,
    pub camera: CameraDefaults,
}

#[derive(Debug, Clone, Serialize)]
pub struct LiveViewFramePayload {
    pub jpeg_base64: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CameraSettingsInput {
    pub iso: u32,
    pub aperture: f32,
    pub shutter_seconds: f64,
}

fn lock_config(state: &AppState) -> Result<std::sync::MutexGuard<'_, AppConfig>, AppError> {
    state.config.lock().map_err(|_| AppError::LockPoisoned)
}

fn lock_sessions_dir(state: &AppState) -> Result<std::sync::MutexGuard<'_, PathBuf>, AppError> {
    state
        .sessions_dir
        .lock()
        .map_err(|_| AppError::LockPoisoned)
}

fn sessions_path(state: &AppState) -> Result<PathBuf, AppError> {
    Ok(lock_sessions_dir(state)?.clone())
}

fn camera_available() -> bool {
    cfg!(windows)
}

fn sessions_dir_is_custom(config: &AppConfig) -> bool {
    config
        .sessions_dir
        .as_deref()
        .map(str::trim)
        .is_some_and(|path| !path.is_empty())
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
        locale: config.locale,
        theme: config.theme,
        log_dir: log_dir.display().to_string(),
        sessions_dir: sessions_dir.display().to_string(),
        sessions_dir_custom: sessions_dir_is_custom(config),
        fusion: config.fusion.clone(),
        capture: config.capture.clone(),
        camera: config.camera.clone(),
    }
}

fn status_of(state: &AppState) -> Result<AppStatus, AppError> {
    let config = lock_config(state)?;
    let sessions = sessions_path(state)?;
    Ok(build_status(&config, &state.log_dir, &sessions))
}

/// Résout le dossier sessions depuis la config (chemin personnalisé ou défaut).
pub fn resolve_sessions_dir(
    config: &AppConfig,
    default_sessions_dir: &Path,
) -> Result<PathBuf, AppError> {
    let path = match config
        .sessions_dir
        .as_deref()
        .map(str::trim)
        .filter(|path| !path.is_empty())
    {
        Some(custom) => PathBuf::from(custom),
        None => default_sessions_dir.to_path_buf(),
    };
    if !path.is_absolute() {
        return Err(AppError::Message(
            "Choisis un dossier avec un chemin complet".to_string(),
        ));
    }
    std::fs::create_dir_all(&path).map_err(|error| {
        AppError::Message(format!("Impossible de créer le dossier séances : {error}"))
    })?;
    Ok(path)
}

#[tauri::command]
pub fn get_app_status(state: State<'_, AppState>) -> Result<AppStatus, ErrorPayload> {
    Ok(status_of(&state)?)
}

#[tauri::command]
pub fn set_demo_mode(enabled: bool, state: State<'_, AppState>) -> Result<AppStatus, ErrorPayload> {
    clear_camera(&state)?;
    let mut config = lock_config(&state).map_err(ErrorPayload::from)?;
    config.demo_mode = enabled;
    save_to_path(&state.config_path, &config)?;
    info!(demo_mode = enabled, "mode démo mis à jour");
    let sessions = sessions_path(&state)?;
    Ok(build_status(&config, &state.log_dir, &sessions))
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
    let sessions = sessions_path(&state)?;
    Ok(build_status(&config, &state.log_dir, &sessions))
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
    let sessions = sessions_path(&state)?;
    Ok(build_status(&config, &state.log_dir, &sessions))
}

#[tauri::command]
pub fn set_locale(
    locale: LocalePreference,
    state: State<'_, AppState>,
) -> Result<AppStatus, ErrorPayload> {
    let mut config = lock_config(&state).map_err(ErrorPayload::from)?;
    config.locale = locale;
    save_to_path(&state.config_path, &config)?;
    info!(?locale, "langue UI mise à jour");
    let sessions = sessions_path(&state)?;
    Ok(build_status(&config, &state.log_dir, &sessions))
}

#[tauri::command]
pub fn set_theme(
    theme: ThemePreference,
    state: State<'_, AppState>,
) -> Result<AppStatus, ErrorPayload> {
    let mut config = lock_config(&state).map_err(ErrorPayload::from)?;
    config.theme = theme;
    save_to_path(&state.config_path, &config)?;
    info!(?theme, "thème UI mis à jour");
    let sessions = sessions_path(&state)?;
    Ok(build_status(&config, &state.log_dir, &sessions))
}

/// `path = null` réinitialise le dossier sessions au défaut de l'application.
#[tauri::command]
pub fn set_sessions_dir(
    path: Option<String>,
    state: State<'_, AppState>,
) -> Result<AppStatus, ErrorPayload> {
    let resolved = match path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(custom) => PathBuf::from(custom),
        None => state.default_sessions_dir.clone(),
    };
    if !resolved.is_absolute() {
        return Err(
            AppError::Message("Choisis un dossier avec un chemin complet".to_string()).into(),
        );
    }
    std::fs::create_dir_all(&resolved).map_err(|error| {
        AppError::Message(format!("Impossible de créer le dossier séances : {error}"))
    })?;

    let is_default = resolved == state.default_sessions_dir;
    let mut config = lock_config(&state).map_err(ErrorPayload::from)?;
    config.sessions_dir = if is_default {
        None
    } else {
        Some(resolved.display().to_string())
    };
    save_to_path(&state.config_path, &config)?;
    {
        let mut slot = lock_sessions_dir(&state)?;
        *slot = resolved.clone();
    }
    info!(sessions = %resolved.display(), "dossier des séances mis à jour");
    Ok(build_status(&config, &state.log_dir, &resolved))
}

#[tauri::command]
pub fn list_sessions(state: State<'_, AppState>) -> Result<Vec<SessionSummary>, ErrorPayload> {
    let parent = sessions_path(&state)?;
    Ok(list_session_summaries(&parent)?)
}

#[tauri::command]
pub fn get_session(
    session_dir: String,
    state: State<'_, AppState>,
) -> Result<SessionDetail, ErrorPayload> {
    let parent = sessions_path(&state)?;
    Ok(load_session_detail(&parent, Path::new(session_dir.trim()))?)
}

#[tauri::command]
pub fn delete_session(session_dir: String, state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    let parent = sessions_path(&state)?;
    let trimmed = session_dir.trim();
    remove_session_dir(&parent, Path::new(trimmed))?;
    info!(session = %trimmed, "séance supprimée");
    Ok(())
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
        return Err(
            AppError::Message("Prise de vue en cours — fusion plus tard".to_string()).into(),
        );
    }
    if state
        .fusion_running
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err(AppError::Message("Fusion déjà en cours".to_string()).into());
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
    .map_err(|_| AppError::Message("Fusion interrompue".to_string()))?;

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

#[derive(Debug, Clone, Serialize)]
pub struct CameraDeviceInfo {
    pub index: u32,
    pub model: String,
}

#[tauri::command]
pub fn list_cameras(state: State<'_, AppState>) -> Result<Vec<CameraDeviceInfo>, ErrorPayload> {
    let config = lock_config(&state).map_err(ErrorPayload::from)?;
    let env_value = demo_env_value();
    if resolve_demo_mode(&config, env_value.as_deref()) {
        return Ok(vec![CameraDeviceInfo {
            index: 0,
            model: "MockCamera (démo)".to_string(),
        }]);
    }
    list_real_cameras()
}

#[tauri::command]
pub fn connect_camera(
    app: AppHandle,
    device_index: Option<u32>,
    state: State<'_, AppState>,
) -> Result<CameraDeviceInfo, ErrorPayload> {
    let demo = is_demo(&state)?;
    let index = device_index.unwrap_or(0);

    // Remplace toute session précédente (Close_Device avant nouvelle Select).
    clear_camera(&state)?;

    let camera_defaults = {
        let config = lock_config(&state).map_err(ErrorPayload::from)?;
        config.camera.clone()
    };

    let (mut camera, model) = open_camera_session(demo, index)?;
    if let Err(error) = apply_camera_defaults(camera.as_mut(), &camera_defaults) {
        tracing::warn!(%error, "application des réglages sauvegardés impossible");
    }
    store_camera(&state, camera)?;
    start_unplug_watch(&app, &state, demo)?;
    start_settings_watch(&app, &state)?;
    info!(%model, index, "caméra en session");
    Ok(CameraDeviceInfo { index, model })
}

#[tauri::command]
pub fn disconnect_camera(state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    clear_camera(&state)?;
    info!("caméra déconnectée (commande UI)");
    Ok(())
}

#[tauri::command]
pub fn get_camera_settings(
    state: State<'_, AppState>,
) -> Result<ExposureCapabilities, ErrorPayload> {
    ensure_capture_allowed(&state)?;
    ensure_camera_idle(&state)?;
    let mut guard = lock_camera(&state)?;
    let camera = guard
        .as_mut()
        .ok_or_else(|| AppError::Message("Pas d’appareil connecté — va dans Studio".to_string()))?;
    Ok(camera
        .exposure_capabilities()
        .map_err(|error| AppError::Message(error.to_string()))?)
}

#[tauri::command]
pub fn set_camera_settings(
    settings: CameraSettingsInput,
    state: State<'_, AppState>,
) -> Result<ExposureCapabilities, ErrorPayload> {
    ensure_capture_allowed(&state)?;
    ensure_camera_idle(&state)?;
    {
        let mut guard = lock_camera(&state)?;
        let camera = guard.as_mut().ok_or_else(|| {
            AppError::Message("Pas d’appareil connecté — va dans Studio".to_string())
        })?;
        let applied = CameraSettings {
            mode: ExposureMode::M,
            aperture: settings.aperture,
            iso: settings.iso,
            shutter_seconds: settings.shutter_seconds,
            shutter_label: format_shutter(settings.shutter_seconds),
        };
        camera
            .apply_settings(&applied)
            .map_err(|error| AppError::Message(error.to_string()))?;
        lumix::suppress_settings_notify_briefly();
    }
    {
        let mut config = lock_config(&state).map_err(ErrorPayload::from)?;
        config.camera = CameraDefaults {
            iso: settings.iso,
            aperture: settings.aperture,
            shutter_seconds: settings.shutter_seconds,
        };
        save_to_path(&state.config_path, &config)?;
    }
    let mut guard = lock_camera(&state)?;
    let camera = guard
        .as_mut()
        .ok_or_else(|| AppError::Message("Pas d’appareil connecté — va dans Studio".to_string()))?;
    Ok(camera
        .exposure_capabilities()
        .map_err(|error| AppError::Message(error.to_string()))?)
}

#[tauri::command]
pub fn set_capture_defaults(
    defaults: CaptureDefaults,
    state: State<'_, AppState>,
) -> Result<AppStatus, ErrorPayload> {
    let mut config = lock_config(&state).map_err(ErrorPayload::from)?;
    config.capture = defaults;
    save_to_path(&state.config_path, &config)?;
    let sessions = sessions_path(&state)?;
    Ok(build_status(&config, &state.log_dir, &sessions))
}

#[tauri::command]
pub fn start_live_view(app: AppHandle, state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    ensure_capture_allowed(&state)?;
    stop_live_view_internal(&state)?;
    {
        let mut guard = lock_camera(&state)?;
        let camera = guard.as_mut().ok_or_else(|| {
            AppError::Message("Pas d’appareil connecté — va dans Studio".to_string())
        })?;
        camera
            .start_live_view()
            .map_err(|error| AppError::Message(error.to_string()))?;
    }

    state.live_view_stop.store(false, Ordering::SeqCst);
    let stop = state.live_view_stop.clone();
    let app_for_frames = app.clone();
    let handle = std::thread::Builder::new()
        .name("lumix-liveview".into())
        .spawn(move || {
            live_view_loop(app_for_frames, stop);
        })
        .map_err(|error| {
            AppError::Message(format!(
                "impossible de démarrer le thread Live View: {error}"
            ))
        })?;
    let mut slot = state
        .live_view_handle
        .lock()
        .map_err(|_| AppError::LockPoisoned)?;
    *slot = Some(handle);
    info!("Live View UI démarré");
    Ok(())
}

#[tauri::command]
pub fn stop_live_view(state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    stop_live_view_internal(&state)
}

#[tauri::command]
pub fn preview_bracket_plan(
    request: BracketRequest,
    state: State<'_, AppState>,
) -> Result<BracketPlan, ErrorPayload> {
    ensure_capture_allowed(&state)?;
    ensure_camera_idle(&state)?;
    let mut guard = lock_camera(&state)?;
    let camera = guard
        .as_mut()
        .ok_or_else(|| AppError::Message("Pas d’appareil connecté — va dans Studio".to_string()))?;
    Ok(preview_plan(camera.as_mut(), &request)?)
}

#[tauri::command]
pub async fn start_capture(
    app: AppHandle,
    request: BracketRequest,
    state: State<'_, AppState>,
) -> Result<CaptureSessionResult, ErrorPayload> {
    ensure_capture_allowed(&state)?;
    if state.fusion_running.load(Ordering::SeqCst) {
        return Err(AppError::Message("Fusion en cours — attends un peu".to_string()).into());
    }
    if state
        .capture_running
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err(AppError::Message("Prise de vue déjà en cours".to_string()).into());
    }
    let capture_result = {
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
        // PTP / fusion toujours en staging (dossier app) — évite Pictures/OneDrive
        // et les extensions shell Windows pendant le transfert USB.
        let staging_parent = state.default_sessions_dir.clone();
        let final_parent = sessions_path(&state)?;
        let app_version = env!("CARGO_PKG_VERSION").to_string();
        let app_for_progress = app.clone();

        // Pause LV / watches pendant le bracket (évite heartbeat + DATA_RCV pendant PTP).
        let _ = stop_live_view_internal(&state);
        stop_settings_watch(&state);
        stop_unplug_watch(&state);
        lumix::begin_capture_wait();

        // Emprunte la session ouverte (pas de nouveau Select_PnPDevice).
        let mut camera = take_camera(&state)?;

        let blocked = tauri::async_runtime::spawn_blocking(
            move || -> (Result<CaptureSessionResult, ErrorPayload>, Box<dyn Camera>) {
                let result = run_capture(
                    camera.as_mut(),
                    &request,
                    &staging_parent,
                    &fusion,
                    &cancel,
                    &app_version,
                    &mut |update: CaptureProgress| {
                        let _ = app_for_progress.emit(CAPTURE_PROGRESS_EVENT, update);
                    },
                )
                .map_err(ErrorPayload::from);
                (result, camera)
            },
        )
        .await
        .map_err(|_| AppError::Message("Prise de vue interrompue".to_string()))?;

        let (capture_result, camera) = blocked;
        let demo = is_demo(&state).unwrap_or(false);
        if let Err(error) = store_camera(&state, camera) {
            tracing::warn!(
                message = %error.message,
                "impossible de remettre la caméra en session"
            );
        } else {
            if let Err(error) = start_settings_watch(&app, &state) {
                tracing::warn!(
                    message = %error.message,
                    "impossible de relancer la surveillance des réglages"
                );
            }
            if let Err(error) = start_unplug_watch(&app, &state, demo) {
                tracing::warn!(
                    message = %error.message,
                    "impossible de relancer la surveillance PnP"
                );
            }
        }

        match capture_result {
            Ok(session) => {
                let relocated = if final_parent != state.default_sessions_dir {
                    relocate_session(session, &final_parent).map_err(ErrorPayload::from)
                } else {
                    Ok(session)
                };
                match &relocated {
                    Ok(session) => info!(
                        dir = %session.session_dir.display(),
                        fused = ?session.fused_jpeg,
                        "prise de vue terminée"
                    ),
                    Err(error) => tracing::warn!(
                        message = %error.message,
                        detail = %error.detail,
                        "déplacement vers le dossier séances custom échoué"
                    ),
                }
                relocated
            }
            Err(error) => {
                tracing::warn!(
                    message = %error.message,
                    detail = %error.detail,
                    "prise de vue échouée"
                );
                Err(error)
            }
        }
    };

    // Reprise Live View (best-effort) après libération du flag capture.
    let _ = start_live_view(app, state);

    capture_result
}

#[tauri::command]
pub fn cancel_capture(state: State<'_, AppState>) -> Result<(), ErrorPayload> {
    let cancel = state
        .capture_cancel
        .lock()
        .map_err(|_| AppError::LockPoisoned)?;
    cancel.store(true, Ordering::SeqCst);
    lumix::request_capture_cancel();
    info!("annulation de prise de vue demandée");
    Ok(())
}

fn is_demo(state: &AppState) -> Result<bool, ErrorPayload> {
    let config = lock_config(state).map_err(ErrorPayload::from)?;
    let env_value = demo_env_value();
    Ok(resolve_demo_mode(&config, env_value.as_deref()))
}

fn ensure_capture_allowed(state: &AppState) -> Result<(), ErrorPayload> {
    if is_demo(state)? {
        return Ok(());
    }
    if !camera_available() {
        return Err(AppError::Message("Caméra réelle : Windows seulement".to_string()).into());
    }
    Ok(())
}

fn ensure_camera_idle(state: &AppState) -> Result<(), ErrorPayload> {
    if state.capture_running.load(Ordering::SeqCst) {
        return Err(AppError::Message("Prise de vue en cours".to_string()).into());
    }
    Ok(())
}

fn open_camera_session(
    demo: bool,
    device_index: u32,
) -> Result<(Box<dyn Camera>, String), ErrorPayload> {
    if demo {
        let mut camera = MockCamera::new();
        camera
            .connect()
            .map_err(|error| AppError::Message(error.to_string()))?;
        return Ok((Box::new(camera), "MockCamera (démo)".to_string()));
    }
    #[cfg(windows)]
    {
        use lumix::LumixCamera;
        let mut camera = LumixCamera::with_device_index(device_index);
        camera
            .connect()
            .map_err(|error| AppError::Message(error.to_string()))?;
        let model = camera.model_name().to_string();
        Ok((Box::new(camera), model))
    }
    #[cfg(not(windows))]
    {
        let _ = device_index;
        Err(AppError::Message("SDK LUMIX pas dispo ici — passe en mode démo".to_string()).into())
    }
}

fn lock_camera(
    state: &AppState,
) -> Result<std::sync::MutexGuard<'_, Option<Box<dyn Camera>>>, ErrorPayload> {
    state
        .camera
        .lock()
        .map_err(|_| AppError::LockPoisoned.into())
}

fn store_camera(state: &AppState, camera: Box<dyn Camera>) -> Result<(), ErrorPayload> {
    let mut slot = lock_camera(state)?;
    *slot = Some(camera);
    Ok(())
}

fn take_camera(state: &AppState) -> Result<Box<dyn Camera>, ErrorPayload> {
    let mut slot = lock_camera(state)?;
    slot.take().ok_or_else(|| {
        AppError::Message("Pas d’appareil connecté — va dans Studio".to_string()).into()
    })
}

fn clear_camera(state: &AppState) -> Result<(), ErrorPayload> {
    stop_settings_watch(state);
    stop_unplug_watch(state);
    let _ = stop_live_view_internal(state);
    let mut slot = lock_camera(state)?;
    if let Some(mut camera) = slot.take() {
        if let Err(error) = camera.disconnect() {
            tracing::warn!(%error, "déconnexion caméra lors du clear");
        }
    }
    Ok(())
}

fn apply_camera_defaults(
    camera: &mut dyn Camera,
    defaults: &CameraDefaults,
) -> Result<(), AppError> {
    let applied = CameraSettings {
        mode: ExposureMode::M,
        aperture: defaults.aperture,
        iso: defaults.iso,
        shutter_seconds: defaults.shutter_seconds,
        shutter_label: format_shutter(defaults.shutter_seconds),
    };
    camera
        .apply_settings(&applied)
        .map_err(|error| AppError::Message(error.to_string()))
}

fn stop_live_view_internal(state: &AppState) -> Result<(), ErrorPayload> {
    // 1) Stop SDK d'abord (sample LiveView01) pour libérer le flux.
    {
        let mut guard = lock_camera(state)?;
        if let Some(camera) = guard.as_mut() {
            if let Err(error) = camera.stop_live_view() {
                tracing::warn!(%error, "arrêt Live View caméra");
            }
        }
    }
    // 2) Puis arrêter le thread poll.
    state.live_view_stop.store(true, Ordering::SeqCst);
    let handle = {
        let mut slot = state
            .live_view_handle
            .lock()
            .map_err(|_| AppError::LockPoisoned)?;
        slot.take()
    };
    if let Some(handle) = handle {
        let _ = handle.join();
    }
    Ok(())
}

fn live_view_loop(app: AppHandle, stop: Arc<AtomicBool>) {
    let mut consecutive_hard_errors = 0u32;
    while !stop.load(Ordering::SeqCst) {
        let jpeg = {
            let state = app.state::<AppState>();
            let mut guard = match state.camera.lock() {
                Ok(guard) => guard,
                Err(_) => break,
            };
            match guard.as_mut() {
                Some(camera) => match camera.poll_live_view_jpeg() {
                    Ok(frame) => {
                        consecutive_hard_errors = 0;
                        frame
                    }
                    Err(error) => {
                        consecutive_hard_errors = consecutive_hard_errors.saturating_add(1);
                        if consecutive_hard_errors == 1
                            || consecutive_hard_errors.is_multiple_of(30)
                        {
                            tracing::warn!(%error, consecutive_hard_errors, "poll Live View échoué");
                        }
                        None
                    }
                },
                None => break,
            }
        };
        if consecutive_hard_errors >= 45 {
            // ~3 s d'échecs durs → appareil probablement parti / session morte.
            tracing::warn!("Live View : trop d'erreurs — déconnexion");
            let state = app.state::<AppState>();
            let _ = clear_camera(&state);
            let _ = app.emit(CAMERA_DISCONNECTED_EVENT, ());
            break;
        }
        if let Some(bytes) = jpeg {
            let payload = LiveViewFramePayload {
                jpeg_base64: BASE64.encode(bytes),
            };
            let _ = app.emit(LIVEVIEW_FRAME_EVENT, payload);
        }
        std::thread::sleep(Duration::from_millis(66));
    }
}

fn start_settings_watch(app: &AppHandle, state: &AppState) -> Result<(), ErrorPayload> {
    stop_settings_watch(state);
    state.settings_watch_stop.store(false, Ordering::SeqCst);
    let stop = state.settings_watch_stop.clone();
    let app = app.clone();
    let handle = std::thread::Builder::new()
        .name("lumix-settings".into())
        .spawn(move || {
            settings_watch_loop(app, stop);
        })
        .map_err(|error| {
            AppError::Message(format!(
                "impossible de démarrer la surveillance des réglages: {error}"
            ))
        })?;
    let mut slot = state
        .settings_watch_handle
        .lock()
        .map_err(|_| AppError::LockPoisoned)?;
    *slot = Some(handle);
    Ok(())
}

fn stop_settings_watch(state: &AppState) {
    state.settings_watch_stop.store(true, Ordering::SeqCst);
    if let Ok(mut slot) = state.settings_watch_handle.lock() {
        if let Some(handle) = slot.take() {
            let _ = handle.join();
        }
    }
}

fn settings_watch_loop(app: AppHandle, stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(120));
        if stop.load(Ordering::SeqCst) {
            break;
        }
        if !lumix::take_settings_changed() {
            continue;
        }
        let settings = {
            let state = app.state::<AppState>();
            let mut guard = match state.camera.lock() {
                Ok(guard) => guard,
                Err(_) => break,
            };
            match guard.as_mut() {
                Some(camera) => match camera.read_settings() {
                    Ok(s) => s,
                    Err(error) => {
                        tracing::debug!(%error, "relecture réglages après notif échouée");
                        continue;
                    }
                },
                None => break,
            }
        };
        // Persistance best-effort des valeurs lues sur l'appareil.
        {
            let state = app.state::<AppState>();
            if let Ok(mut config) = state.config.lock() {
                config.camera = CameraDefaults {
                    iso: settings.iso,
                    aperture: settings.aperture,
                    shutter_seconds: settings.shutter_seconds,
                };
                let _ = save_to_path(&state.config_path, &config);
            };
        }
        let _ = app.emit(CAMERA_SETTINGS_CHANGED_EVENT, settings);
    }
}

fn start_unplug_watch(app: &AppHandle, state: &AppState, demo: bool) -> Result<(), ErrorPayload> {
    stop_unplug_watch(state);
    if demo {
        return Ok(());
    }
    state.unplug_stop.store(false, Ordering::SeqCst);
    let stop = state.unplug_stop.clone();
    let app = app.clone();
    let handle = std::thread::Builder::new()
        .name("lumix-unplug".into())
        .spawn(move || {
            unplug_watch_loop(app, stop);
        })
        .map_err(|error| {
            AppError::Message(format!(
                "impossible de démarrer la surveillance PnP: {error}"
            ))
        })?;
    let mut slot = state
        .unplug_handle
        .lock()
        .map_err(|_| AppError::LockPoisoned)?;
    *slot = Some(handle);
    Ok(())
}

fn stop_unplug_watch(state: &AppState) {
    state.unplug_stop.store(true, Ordering::SeqCst);
    if let Ok(mut slot) = state.unplug_handle.lock() {
        if let Some(handle) = slot.take() {
            let _ = handle.join();
        }
    }
}

fn unplug_watch_loop(app: AppHandle, stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_secs(2));
        if stop.load(Ordering::SeqCst) {
            break;
        }
        // Ne pas appeler Get_PnPDeviceInfo pendant une session / Live View :
        // ça provoque DATA_RCV en boucle et laisse le device « coincé » au Select suivant.
        // Pendant le LV, le thread liveview gère la détection ; ici heartbeat hors LV.
        let alive = {
            let state = app.state::<AppState>();
            let mut guard = match state.camera.lock() {
                Ok(guard) => guard,
                Err(_) => break,
            };
            match guard.as_mut() {
                Some(camera) if camera.is_live_view_running() => true,
                Some(camera) => camera.heartbeat().is_ok(),
                None => break,
            }
        };
        if alive {
            continue;
        }
        info!("heartbeat caméra échoué — déconnexion automatique");
        let state = app.state::<AppState>();
        let _ = clear_camera(&state);
        let _ = app.emit(CAMERA_DISCONNECTED_EVENT, ());
        break;
    }
}

fn list_real_cameras() -> Result<Vec<CameraDeviceInfo>, ErrorPayload> {
    #[cfg(windows)]
    {
        use std::ffi::CStr;
        use std::sync::Once;

        use lumix_sys::{
            lmx_sys_init, lmx_sys_list_pnp_models, LMX_BOOL_TRUE, LMX_MODEL_UTF8_STRIDE,
            LMX_PTP_ARRAY_MAX,
        };

        static INIT: Once = Once::new();
        INIT.call_once(|| unsafe {
            lmx_sys_init();
        });

        let mut models = vec![0u8; LMX_PTP_ARRAY_MAX * LMX_MODEL_UTF8_STRIDE];
        let mut count = 0u32;
        let mut ret_error = 0u32;
        let ok = unsafe {
            lmx_sys_list_pnp_models(
                models.as_mut_ptr().cast(),
                LMX_MODEL_UTF8_STRIDE as u32,
                LMX_PTP_ARRAY_MAX as u32,
                &mut count,
                &mut ret_error,
            )
        };
        if ok != LMX_BOOL_TRUE {
            return Err(AppError::Message(format!(
                "Impossible de lister les appareils (retError=0x{ret_error:08X})"
            ))
            .into());
        }
        let mut out = Vec::with_capacity(count as usize);
        for i in 0..count as usize {
            let start = i * LMX_MODEL_UTF8_STRIDE;
            let end = start + LMX_MODEL_UTF8_STRIDE;
            let name = CStr::from_bytes_until_nul(&models[start..end])
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|_| format!("Appareil {i}"));
            out.push(CameraDeviceInfo {
                index: i as u32,
                model: name,
            });
        }
        Ok(out)
    }
    #[cfg(not(windows))]
    {
        Err(AppError::Message("Liste d’appareils : Windows + SDK LUMIX".to_string()).into())
    }
}

/// Retourne `(config_path, log_dir, default_sessions_dir)`.
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
    let default_sessions_dir = data_dir.join("sessions");
    std::fs::create_dir_all(&default_sessions_dir).map_err(|error| {
        AppError::Message(format!("Impossible de créer le dossier séances : {error}"))
    })?;
    Ok((
        config_dir.join(crate::config::CONFIG_FILE_NAME),
        log_dir,
        default_sessions_dir,
    ))
}
