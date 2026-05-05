pub mod appearance;
pub mod background;
pub mod logging;
pub mod power;
pub mod theme;

use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use tracing::warn;

pub use appearance::AppearanceConfig;
pub use logging::LoggingConfig;
pub use power::PowerConfig;

/// The name for this greeter
pub const GREETER_NAME: &str = "mdgreet";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheConfig {
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GreeterConfig {
    #[serde(default)]
    pub appearance: AppearanceConfig,
    #[serde(default)]
    pub power: PowerConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub cache: CacheConfig,
}

impl GreeterConfig {
    pub fn is_dark_mode(&self) -> bool {
        match self.appearance.theme.mode.as_deref() {
            Some("dark") => true,
            Some("light") => false,
            _ => true,
        }
    }

    pub fn load(cli_path: &Option<String>) -> Self {
        let path = resolve_config_path(cli_path);

        Figment::from(Serialized::defaults(GreeterConfig::default()))
            .merge(Toml::file(&path))
            .merge(Env::prefixed("MDGREET_"))
            .extract()
            .unwrap_or_else(|e| {
                warn!("Failed to load (path: {:?}): {}", path, e);
                warn!("Using defaults");
                GreeterConfig::default()
            })
    }
}

pub fn resolve_config_path(cli_path: &Option<String>) -> PathBuf {
    cli_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("/etc/greetd/{}.toml", GREETER_NAME)))
}
