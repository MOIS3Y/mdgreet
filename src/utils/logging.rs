use crate::config::LoggingConfig;
use crate::utils::paths;
use std::fs::OpenOptions;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

/// Initializes the global tracing subscriber for the application.
///
/// This function sets up logging based on the provided configuration.
/// It attempts to open a log file for non-blocking writes. If the file
/// cannot be opened, it falls back to logging to standard output.
///
/// # Returns
///
/// Returns a [`tracing_appender::non_blocking::WorkerGuard`] if file
/// logging is successfully initialized. This guard must be kept alive
/// for the duration of the program to ensure all logs are flushed.
pub fn init(config: &LoggingConfig) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let log_level = config.level.as_deref().unwrap_or("info");

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(log_level))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let timer = tracing_subscriber::fmt::time::OffsetTime::local_rfc_3339()
        .expect("Couldn't get local time offset");

    let log_path = config.path.clone().unwrap_or_else(paths::default_log_path);

    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    match OpenOptions::new().create(true).append(true).open(&log_path) {
        Ok(f) => {
            let (non_blocking, guard) = tracing_appender::non_blocking(f);
            FmtSubscriber::builder()
                .with_env_filter(filter)
                .with_timer(timer)
                .with_target(true)
                .with_ansi(false)
                .with_writer(non_blocking)
                .init();
            Some(guard)
        }
        Err(_) => {
            FmtSubscriber::builder()
                .with_env_filter(filter)
                .with_timer(timer)
                .with_target(true)
                .with_ansi(true)
                .init();
            None
        }
    }
}
