slint::include_modules!();

mod app;
mod cli;
mod config;
mod utils;

use cli::Args;
use config::GreeterConfig;
use slint::{ComponentHandle, SharedString};

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let ui = GreeterWindow::new().unwrap();

    let config = GreeterConfig::load(&args.config);
    let is_dark = config.is_dark_mode();

    // 1. Initialize all modules
    app::Background::init(&ui, &config.background);
    let users_data = app::Auth::init(&ui).await;
    app::Session::init(&ui);
    app::Power::init(&ui, &config.power);
    app::Theme::init(&ui, &config.theme.name);
    let _clock_timer = app::Clock::init(&ui);

    // 2. Apply global settings
    ui.invoke_set_color_scheme(is_dark);

    // 3. Authentication Callback
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

    // 4. Run application
    ui.run().unwrap();
}
