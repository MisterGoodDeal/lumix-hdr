#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

mod commands;
mod config;
mod error;
mod logging;

use std::sync::OnceLock;

use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tracing::error;

use crate::commands::AppState;
use crate::config::load_from_path;
use crate::error::AppError;

static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

pub fn run() {
    install_panic_hook();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| match setup_app(app.handle()) {
            Ok(state) => {
                app.manage(state);
                if APP_HANDLE.set(app.handle().clone()).is_err() {
                    tracing::warn!("handle d'application déjà enregistré");
                }
                Ok(())
            }
            Err(error) => {
                eprintln!("échec de l'initialisation: {error}");
                Err(Box::new(error))
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_status,
            commands::set_demo_mode,
            commands::set_log_level,
            commands::get_log_dir,
            commands::set_fusion_options,
            commands::list_jpeg_folder,
            commands::merge_image_folder,
            commands::cancel_fusion,
            commands::open_path
        ]);

    if let Err(error) = builder.run(tauri::generate_context!()) {
        error!(%error, "échec du lancement de LumixHDR");
        std::process::exit(1);
    }
}

fn setup_app(app: &tauri::AppHandle) -> Result<AppState, AppError> {
    let (config_path, log_dir) = commands::resolve_paths(app)?;
    let config = load_from_path(&config_path)?;
    let logging = logging::init(&log_dir, config.log_level)?;
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        os = std::env::consts::OS,
        demo_mode = config.demo_mode,
        "LumixHDR démarré"
    );
    Ok(AppState::new(
        config_path,
        log_dir,
        config,
        logging.filter,
        logging.guard,
    ))
}

fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        error!(panic = %info, "panic");
        if let Some(app) = APP_HANDLE.get() {
            let app = app.clone();
            let detail = info.to_string();
            std::thread::spawn(move || {
                app.dialog()
                    .message(
                        "LumixHDR a rencontré une erreur inattendue. Consultez les journaux pour le détail technique.",
                    )
                    .title("LumixHDR")
                    .kind(MessageDialogKind::Error)
                    .blocking_show();
                tracing::debug!(%detail, "dialogue de panic affiché");
            });
        }
        default_hook(info);
    }));
}
