use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundConfig {
    pub path: Option<String>,
    pub blur: Option<f32>,
}

impl Default for BackgroundConfig {
    fn default() -> Self {
        Self {
            path: Some("ui/images/background.png".to_string()),
            blur: Some(10.0),
        }
    }
}
