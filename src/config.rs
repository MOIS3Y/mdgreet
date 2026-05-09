use crate::utils::paths;
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::warn;

/// Material Design theme configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Theme name ("default", "slint", "auto", "seed", "custom").
    pub name: String,
    /// Color mode ("dark" or "light").
    pub mode: Option<String>,
    /// Seed color for theme generation in HEX format.
    pub seed_color: Option<String>,
    /// Path to a custom JSON theme file.
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

/// Background visual configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundConfig {
    /// Path to the background image.
    pub path: Option<String>,
    /// Fallback background color in HEX format.
    pub color: Option<String>,
    /// Gaussian blur intensity.
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

/// Typography configuration for the large clock.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockConfig {
    /// Font family for the clock digits.
    pub font_family: Option<String>,
    /// Font weight (300, 500, 700, etc.).
    pub font_weight: Option<i32>,
    /// Font size in pixels.
    pub font_size: Option<i32>,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            font_family: Some("FlexRounded".to_string()),
            font_weight: Some(500),
            font_size: Some(200),
        }
    }
}

/// General appearance settings including labels, fonts, and opacity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    /// Greeting message displayed on the login card.
    pub greeting: Option<String>,
    /// Global UI font family.
    pub font_family: Option<String>,
    /// Transparency level (0.0 to 1.0).
    pub opacity: Option<f32>,
    /// Clock-specific typography settings.
    #[serde(default)]
    pub clock: ClockConfig,
    /// Material Design theme settings.
    #[serde(default)]
    pub theme: ThemeConfig,
    /// Background image and color settings.
    #[serde(default)]
    pub background: BackgroundConfig,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            greeting: Some("Welcome!".to_string()),
            font_family: None,
            opacity: Some(0.7),
            clock: ClockConfig::default(),
            theme: ThemeConfig::default(),
            background: BackgroundConfig::default(),
        }
    }
}

/// Logging configuration for the greeter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Tracing filter level ("info", "debug", etc.).
    pub level: Option<String>,
    /// Path to the log file.
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

/// Custom commands for system power management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerConfig {
    /// Command to shut down the system.
    pub shutdown: Option<String>,
    /// Command to reboot the system.
    pub reboot: Option<String>,
    /// Command to suspend the system.
    pub sleep: Option<String>,
    /// Command to hibernate the system.
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

/// Cache configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheConfig {
    /// Directory to store generated themes and state.
    pub path: Option<PathBuf>,
}

/// Root configuration object for mdgreet.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GreeterConfig {
    /// Appearance and typography settings.
    #[serde(default)]
    pub appearance: AppearanceConfig,
    /// Power management commands.
    #[serde(default)]
    pub power: PowerConfig,
    /// Logging settings.
    #[serde(default)]
    pub logging: LoggingConfig,
    /// Cache directory settings.
    #[serde(default)]
    pub cache: CacheConfig,
}

impl GreeterConfig {
    /// Returns true if the configured theme mode is "dark".
    pub fn is_dark_mode(&self) -> bool {
        match self.appearance.theme.mode.as_deref() {
            Some("dark") => true,
            Some("light") => false,
            _ => true,
        }
    }

    /// Loads the configuration from a file, environment variables,
    /// and defaults.
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

/// Resolves the final path to the configuration file.
pub fn resolve_config_path(cli_path: &Option<String>) -> PathBuf {
    cli_path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(paths::default_config_path)
}
