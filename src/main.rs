slint::include_modules!();

mod app;
mod cli;
mod config;
mod utils;

use cli::Args;
use config::GreeterConfig;
use slint::{ComponentHandle, Model, SharedString};
use std::sync::{Arc, Mutex};
use tracing::info;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = GreeterConfig::load(&args.config);

    // Initialize logging
    utils::logging::init(&config.logging);

    let ui = GreeterWindow::new().unwrap();
    let is_dark = config.is_dark_mode();

    if args.demo {
        info!("*** RUNNING IN DEMO MODE ***");
    }

    let cache = Arc::new(Mutex::new(utils::cache::Cache::load()));

    // Initialize modules
    let users_data = app::Auth::init(&ui, args.demo).await;
    app::Session::init(&ui, args.demo);
    app::Power::init(&ui, &config.power);
    app::Appearance::init(&ui, &config.appearance);
    let _clock_timer = app::Clock::init(&ui);

    // Restore Initial State from Cache (LRU)
    {
        let mut cache_lock = cache.lock().unwrap();
        if let Some(last_user) = cache_lock.last_user.clone() {
            if let Some(pos) = users_data
                .iter()
                .position(|u| u.user_name.as_str() == last_user)
            {
                ui.set_selected_user_index(pos as i32);

                if let Some(last_sess) = cache_lock.get_last_session(&last_user).cloned() {
                    let compositors = ui.get_compositors();
                    for i in 0..compositors.row_count() {
                        if let Some(c) = compositors.row_data(i) {
                            if c.name.as_str() == last_sess {
                                ui.set_selected_compositor_index(i as i32);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    ui.invoke_set_color_scheme(is_dark);

    // Persistence Callbacks
    let cache_ui = cache.clone();
    let users_data_persistence = users_data.clone();
    let ui_weak = ui.as_weak();

    ui.on_user_selected(move |idx| {
        if idx < 0 {
            return;
        }
        if let Some(user) = users_data_persistence.get(idx as usize) {
            let mut cache = cache_ui.lock().unwrap();
            let username = user.user_name.to_string();

            if let Some(last_sess) = cache.get_last_session(&username).cloned() {
                if let Some(ui) = ui_weak.upgrade() {
                    let compositors = ui.get_compositors();
                    for i in 0..compositors.row_count() {
                        if let Some(c) = compositors.row_data(i) {
                            if c.name.as_str() == last_sess {
                                ui.set_selected_compositor_index(i as i32);
                                break;
                            }
                        }
                    }
                }
            }
        }
    });

    // Authentication Logic
    let ui_handle = ui.as_weak();
    let users_data_login = users_data.clone();
    let cache_login = cache.clone();

    ui.on_login(move |username_or_login, _password| {
        let ui = ui_handle.unwrap();
        if username_or_login.is_empty() {
            ui.set_auth_error(SharedString::from("Please select a user"));
            return;
        }

        let user_data = users_data_login
            .iter()
            .find(|u| u.user_name == username_or_login || u.real_name == username_or_login);

        match user_data {
            Some(data) => {
                info!("Login attempt for '{}'!", data.user_name);

                let current_comp_idx = ui.get_selected_compositor_index();
                if let Some(comp) = ui.get_compositors().row_data(current_comp_idx as usize) {
                    let mut cache = cache_login.lock().unwrap();
                    cache.set_last_user(data.user_name.to_string());
                    cache.set_last_session(data.user_name.to_string(), comp.name.to_string());
                    cache.save();
                }

                slint::quit_event_loop().unwrap();
            }
            None => {
                info!("Manual login attempt for '{}'!", username_or_login);
                slint::quit_event_loop().unwrap();
            }
        }
    });

    ui.run().unwrap();
}
