use crate::config::LoggingConfig;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub fn init(config: &LoggingConfig) {
    let log_level = config.level.as_deref().unwrap_or("info");

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(log_level))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let timer = tracing_subscriber::fmt::time::OffsetTime::local_rfc_3339()
        .expect("Couldn't get local time offset");

    FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_timer(timer)
        .with_target(true)
        .with_ansi(true)
        .init();
}
