use crate::GreeterWindow;
use chrono::{Datelike, Local};
use gettextrs::gettext;
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

                // Localize the date
                let weekday = match now.weekday() {
                    chrono::Weekday::Mon => gettext("Monday"),
                    chrono::Weekday::Tue => gettext("Tuesday"),
                    chrono::Weekday::Wed => gettext("Wednesday"),
                    chrono::Weekday::Thu => gettext("Thursday"),
                    chrono::Weekday::Fri => gettext("Friday"),
                    chrono::Weekday::Sat => gettext("Saturday"),
                    chrono::Weekday::Sun => gettext("Sunday"),
                };

                let month = match now.month() {
                    1 => gettext("January"),
                    2 => gettext("February"),
                    3 => gettext("March"),
                    4 => gettext("April"),
                    5 => gettext("May"),
                    6 => gettext("June"),
                    7 => gettext("July"),
                    8 => gettext("August"),
                    9 => gettext("September"),
                    10 => gettext("October"),
                    11 => gettext("November"),
                    12 => gettext("December"),
                    _ => String::new(),
                };

                let day = now.format("%-d").to_string();

                // Translators: This is the format string for the date on the lock screen.
                // You can change the order of the words or add punctuation (e.g., commas).
                // Available variables: {weekday}, {month}, {day}
                // Example for Russian: "{weekday}, {day} {month}" -> "Пятница, 8 мая"
                let date_format = gettext("{weekday}, {month} {day}");
                let localized_date = date_format
                    .replace("{weekday}", &weekday)
                    .replace("{month}", &month)
                    .replace("{day}", &day);

                ui.set_date(SharedString::from(localized_date));
            }
        };

        // Initial update
        update_time();

        let timer = Timer::default();
        timer.start(TimerMode::Repeated, Duration::from_secs(1), update_time);

        timer
    }
}
