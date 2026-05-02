pub mod background;
pub mod power;
pub mod theme;

use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use background::BackgroundConfig;
pub use power::PowerConfig;
pub use theme::ThemeConfig;

/// The name for this greeter
pub const GREETER_NAME: &str = "mdgreet";

/// Cache directory for processed images
pub const CACHE_DIR: &str = "/var/cache/mdgreet";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GreeterConfig {
    pub theme: ThemeConfig,
    pub background: BackgroundConfig,
    pub power: PowerConfig,
}

impl GreeterConfig {
    pub fn is_dark_mode(&self) -> bool {
        match self.theme.mode.as_deref() {
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
                eprintln!("config: failed to load (path: {:?}): {}", path, e);
                eprintln!("config: using defaults");
                GreeterConfig::default()
            })
    }
}

pub fn resolve_config_path(cli_path: &Option<String>) -> PathBuf {
    cli_path
        .as_ref()
        .map(PathBuf::from)
        .or_else(|| std::env::var("MDGREET_CONFIG").ok().map(PathBuf::from))
        .or_else(|| {
            std::env::var("XDG_CONFIG_HOME")
                .ok()
                .map(PathBuf::from)
                .map(|d| d.join(format!("{}/{}.toml", GREETER_NAME, GREETER_NAME)))
        })
        .or_else(|| {
            dirs::home_dir()
                .map(|d| d.join(format!(".config/{}/{}.toml", GREETER_NAME, GREETER_NAME)))
        })
        .unwrap_or_else(|| PathBuf::from(format!("/etc/greetd/{}.toml", GREETER_NAME)))
}
