use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
