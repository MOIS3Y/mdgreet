use serde::{Deserialize, Serialize};

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
