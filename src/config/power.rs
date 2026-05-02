use serde::{Deserialize, Serialize};

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
