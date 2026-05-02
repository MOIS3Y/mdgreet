use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::constants;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundConfig {
    pub path: Option<String>,
    pub blur: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreeterConfig {
    pub theme: ThemeConfig,
    pub background: Option<BackgroundConfig>,
}

impl Default for GreeterConfig {
    fn default() -> Self {
        Self {
            theme: ThemeConfig {
                name: constants::DEFAULT_THEME.to_string(),
                mode: Some(constants::DEFAULT_MODE.to_string()),
            },
            background: Some(BackgroundConfig {
                path: Some(constants::DEFAULT_BACKGROUND.to_string()),
                blur: Some(10.0),
            }),
        }
    }
}

impl GreeterConfig {
    pub fn is_dark_mode(&self) -> bool {
        match self.theme.mode.as_deref() {
            Some("dark") => true,
            Some("light") => false,
            _ => true,
        }
    }

    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("config: failed to read: {}", path))?;
        toml::from_str(&content).context("config: failed to parse TOML")
    }

    pub fn load_or_default(config_path: Option<&str>) -> Self {
        let path = config_path
            .map(String::from)
            .or_else(|| std::env::var("MDGREET_CONFIG").ok())
            .unwrap_or_else(|| constants::CONFIG_PATH.to_string());

        Self::load(&path).unwrap_or_else(|e| {
            eprintln!("config: failed to load: {}", e);
            eprintln!("config: using defaults");
            Self::default()
        })
    }
}
