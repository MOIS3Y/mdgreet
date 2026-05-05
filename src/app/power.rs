use crate::GreeterWindow;
use crate::config::PowerConfig;
use std::process::Command;
use tracing::{error, info};

pub struct Power;

impl Power {
    fn execute_command(cmd_string: &str) {
        info!("Executing power action: {}", cmd_string);

        let parts = match shlex::split(cmd_string) {
            Some(p) if !p.is_empty() => p,
            _ => {
                error!("Invalid power command: {}", cmd_string);
                return;
            }
        };

        if let Err(e) = Command::new(&parts[0]).args(&parts[1..]).spawn() {
            error!("Failed to execute command '{}': {}", cmd_string, e);
        }
    }

    pub fn init(ui: &GreeterWindow, config: &PowerConfig) {
        let shutdown_cmd = config
            .shutdown
            .clone()
            .unwrap_or_else(|| "systemctl poweroff".to_string());
        ui.on_shutdown(move || {
            Self::execute_command(&shutdown_cmd);
        });

        let reboot_cmd = config
            .reboot
            .clone()
            .unwrap_or_else(|| "systemctl reboot".to_string());
        ui.on_reboot(move || {
            Self::execute_command(&reboot_cmd);
        });

        let sleep_cmd = config
            .sleep
            .clone()
            .unwrap_or_else(|| "systemctl suspend".to_string());
        ui.on_sleep(move || {
            Self::execute_command(&sleep_cmd);
        });

        let hibernate_cmd = config
            .hibernate
            .clone()
            .unwrap_or_else(|| "systemctl hibernate".to_string());
        ui.on_hibernate(move || {
            Self::execute_command(&hibernate_cmd);
        });
    }
}
