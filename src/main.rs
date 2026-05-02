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

fn main() {
    let args = Args::parse();
    let ui = GreeterWindow::new().unwrap();

    let config = GreeterConfig::load(&args.config);
    let is_dark = config.is_dark_mode();

    // 1. Background
    let background = app::Background::load(&config.background);
    ui.set_background_original(background.original);
    ui.set_background_blurred(background.blurred);

    // 2. Auth & Users
    let users_data = app::Auth::get_mock_users();
    let (users_model, user_menu_model) = app::Auth::prepare_ui_models(&users_data);
    ui.set_users(users_model.into());
    ui.set_user_menu_items(user_menu_model.into());
    ui.set_selected_user_index(-1);

    // 3. Sessions & Compositors
    let compositors = app::Session::get_mock_compositors();
    let (comp_model, comp_menu_model, comp_icon) = app::Session::prepare_ui_models(&compositors);
    ui.set_compositors(comp_model.into());
    ui.set_compositor_menu_items(comp_menu_model.into());
    ui.set_selected_compositor_index(0);
    ui.set_composer_icon(comp_icon);

    // 4. Theme
    app::theme::load_and_apply(&ui, &config.theme.name);
    ui.invoke_set_color_scheme(is_dark);

    // 5. Timer (Clock & Date)
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

    // 6. Callbacks
    let ui_login = ui.as_weak();
    ui.on_login(move |username_or_login, password| {
        let ui = ui_login.unwrap();
        if username_or_login.is_empty() {
            ui.set_auth_error(SharedString::from("Please select a user"));
            return;
        }
        let user_data = users_data
            .iter()
            .find(|u| u.login == username_or_login || u.pretty_name == username_or_login);
        match user_data {
            Some(data) => {
                if data.password == password {
                    println!("Login successful for '{}'!", data.login);
                    slint::quit_event_loop().unwrap();
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

    ui.on_compositor_selected(|idx| {
        println!("Compositor selected at index {}", idx);
    });

    let power_config = config.power.clone();
    let shutdown_cmd = power_config
        .shutdown
        .unwrap_or_else(|| "systemctl poweroff".to_string());
    ui.on_shutdown(move || println!("Power Action: Shutdown with command '{}'", shutdown_cmd));

    let reboot_cmd = power_config
        .reboot
        .unwrap_or_else(|| "systemctl reboot".to_string());
    ui.on_reboot(move || println!("Power Action: Reboot with command '{}'", reboot_cmd));

    let sleep_cmd = power_config
        .sleep
        .unwrap_or_else(|| "systemctl suspend".to_string());
    ui.on_sleep(move || println!("Power Action: Sleep with command '{}'", sleep_cmd));

    let hibernate_cmd = power_config
        .hibernate
        .unwrap_or_else(|| "systemctl hibernate".to_string());
    ui.on_hibernate(move || println!("Power Action: Hibernate with command '{}'", hibernate_cmd));

    ui.run().unwrap();
}
