slint::include_modules!();

mod app;
mod cli;
mod config;
mod utils;

use chrono::Local;
use cli::Args;
use config::GreeterConfig;
use slint::{ComponentHandle, SharedString, Timer, TimerMode};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let ui = GreeterWindow::new().unwrap();

    let config = GreeterConfig::load(&args.config);
    let is_dark = config.is_dark_mode();

    // Initialize modules
    app::Background::init(&ui, &config.background);
    let users_data = app::Auth::init(&ui).await;
    app::Session::init(&ui);
    app::Power::init(&ui, &config.power);
    app::theme::load_and_apply(&ui, &config.theme.name);

    // Apply color scheme
    ui.invoke_set_color_scheme(is_dark);

    // Clock and Date timer
    let ui_weak = ui.as_weak();
    let update_time = move || {
        if let Some(ui) = ui_weak.upgrade() {
            let now = Local::now();
            ui.set_hours(SharedString::from(now.format("%H").to_string()));
            ui.set_minutes(SharedString::from(now.format("%M").to_string()));
            ui.set_date(SharedString::from(now.format("%A, %B %-d").to_string()));
        }
    };
    update_time();
    let timer = Timer::default();
    timer.start(TimerMode::Repeated, Duration::from_secs(1), update_time);

    // Authentication Logic
    let ui_handle = ui.as_weak();
    ui.on_login(move |username_or_login, password| {
        let ui = ui_handle.unwrap();
        if username_or_login.is_empty() {
            ui.set_auth_error(SharedString::from("Please select a user"));
            return;
        }

        let user_data = users_data
            .iter()
            .find(|u| u.login == username_or_login || u.pretty_name == username_or_login);
        match user_data {
            Some(data) => {
                if !data.password.is_empty() && data.password == password {
                    println!("Login successful for '{}'!", data.login);
                    slint::quit_event_loop().unwrap();
                } else if data.password.is_empty() {
                    if password == "greet" {
                        println!("Demo login successful for '{}'!", data.login);
                        slint::quit_event_loop().unwrap();
                    } else {
                        ui.set_auth_error(SharedString::from(
                            "Invalid password (use 'greet' for demo)",
                        ));
                    }
                } else {
                    ui.set_auth_error(SharedString::from("Invalid password"));
                }
            }
            None => {
                if password == "greet" {
                    println!("Manual login successful for '{}'!", username_or_login);
                    slint::quit_event_loop().unwrap();
                } else {
                    ui.set_auth_error(SharedString::from("User not found or invalid password"));
                }
            }
        }
    });

    ui.run().unwrap();
}
