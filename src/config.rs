use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::warn;

pub const GREETER_NAME: &str = "mdgreet";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    pub mode: Option<String>,
    pub seed_color: Option<String>,
    pub path: Option<PathBuf>,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            mode: Some("dark".to_string()),
            seed_color: None,
            path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundConfig {
    pub path: Option<String>,
    pub color: Option<String>,
    pub blur: Option<f32>,
}

impl Default for BackgroundConfig {
    fn default() -> Self {
        Self {
            path: None,
            color: None,
            blur: Some(10.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    pub label: Option<String>,
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub background: BackgroundConfig,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            label: Some("Welcome!".to_string()),
            theme: ThemeConfig::default(),
            background: BackgroundConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: Option<String>,
    pub path: Option<PathBuf>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: Some("info".to_string()),
            path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerConfig {
    pub shutdown: Option<String>,
    pub reboot: Option<String>,
    pub sleep: Option<String>,
    pub hibernate: Option<String>,
}

impl Default for PowerConfig {
    fn default() -> Self {
        Self {
            shutdown: Some("systemctl poweroff".to_string()),
            reboot: Some("systemctl reboot".to_string()),
            sleep: Some("systemctl suspend".to_string()),
            hibernate: Some("systemctl hibernate".to_string()),
        }
    }
}

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
