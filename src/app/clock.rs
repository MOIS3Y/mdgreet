use crate::GreeterWindow;
use chrono::Local;
use slint::{ComponentHandle, SharedString, Timer, TimerMode};
use std::time::Duration;

/// Manages the real-time clock displayed on the greeter interface.
pub struct Clock;

impl Clock {
    /// Initializes the clock, performs an immediate update, and starts
    /// a background timer to refresh the UI every second.
    ///
    /// The timer updates the `hours`, `minutes`, and `date` properties
    /// of the provided Slint UI instance.
    ///
    /// # Returns
    ///
    /// Returns the active [`slint::Timer`]. This timer must be kept alive
    /// for the clock to continue updating.
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
