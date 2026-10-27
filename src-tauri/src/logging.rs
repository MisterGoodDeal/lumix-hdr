use std::path::Path;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::reload;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Registry;

use crate::config::LogLevel;

pub type FilterHandle = reload::Handle<EnvFilter, Registry>;

pub struct Logging {
    pub guard: WorkerGuard,
    pub filter: FilterHandle,
}

#[derive(Debug, thiserror::Error)]
pub enum LoggingError {
    #[error("impossible d'initialiser les journaux: {0}")]
    Init(String),
    #[error("impossible de modifier le niveau de journalisation: {0}")]
    Reload(String),
}

pub fn init(log_dir: &Path, level: LogLevel) -> Result<Logging, LoggingError> {
    std::fs::create_dir_all(log_dir).map_err(|error| LoggingError::Init(error.to_string()))?;

    let file_appender = rolling::daily(log_dir, "lumix-hdr.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let (filter, handle) = reload::Layer::new(EnvFilter::new(level.as_filter()));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(fmt::layer().with_ansi(false).with_writer(non_blocking))
        .try_init()
        .map_err(|error| LoggingError::Init(error.to_string()))?;

    Ok(Logging {
        guard,
        filter: handle,
    })
}

pub fn reload_level(handle: &FilterHandle, level: LogLevel) -> Result<(), LoggingError> {
    handle
        .modify(|filter| {
            *filter = EnvFilter::new(level.as_filter());
        })
        .map_err(|error| LoggingError::Reload(error.to_string()))
}
