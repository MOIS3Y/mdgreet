use crate::GreeterWindow;
use chrono::Local;
use slint::{ComponentHandle, SharedString, Timer, TimerMode};
use std::time::Duration;

pub struct Clock;

impl Clock {
    pub fn init(ui: &GreeterWindow) -> Timer {
        let ui_weak = ui.as_weak();
        let update_time = move || {
            if let Some(ui) = ui_weak.upgrade() {
                let now = Local::now();
                ui.set_hours(SharedString::from(now.format("%H").to_string()));
                ui.set_minutes(SharedString::from(now.format("%M").to_string()));
                ui.set_date(SharedString::from(now.format("%A, %B %-d").to_string()));
            }
        };

        // Initial update
        update_time();

        let timer = Timer::default();
        timer.start(TimerMode::Repeated, Duration::from_secs(1), update_time);

        timer
    }
}
