use super::background::BackgroundConfig;
use super::theme::ThemeConfig;
use serde::{Deserialize, Serialize};

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
