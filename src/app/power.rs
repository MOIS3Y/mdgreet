use crate::GreeterWindow;
use crate::config::PowerConfig;

pub struct Power;

impl Power {
    pub fn init(ui: &GreeterWindow, config: &PowerConfig) {
        let shutdown_cmd = config
            .shutdown
            .clone()
            .unwrap_or_else(|| "systemctl poweroff".to_string());
        ui.on_shutdown(move || {
            println!("Power Action: Shutdown with command '{}'", shutdown_cmd);
        });

        let reboot_cmd = config
            .reboot
            .clone()
            .unwrap_or_else(|| "systemctl reboot".to_string());
        ui.on_reboot(move || {
            println!("Power Action: Reboot with command '{}'", reboot_cmd);
        });

        let sleep_cmd = config
            .sleep
            .clone()
            .unwrap_or_else(|| "systemctl suspend".to_string());
        ui.on_sleep(move || {
            println!("Power Action: Sleep with command '{}'", sleep_cmd);
        });

        let hibernate_cmd = config
            .hibernate
            .clone()
            .unwrap_or_else(|| "systemctl hibernate".to_string());
        ui.on_hibernate(move || {
            println!("Power Action: Hibernate with command '{}'", hibernate_cmd);
        });
    }
}
