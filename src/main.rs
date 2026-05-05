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
    let _log_guard = utils::logging::init(&config.logging);

    let ui = GreeterWindow::new().unwrap();
    let is_dark = config.is_dark_mode();

    if args.demo {
        info!("*** RUNNING IN DEMO MODE ***");
    }

    let cache = Arc::new(Mutex::new(utils::cache::Cache::load(&config)));

    // Initialize modules
    let users_data = app::Auth::init(&ui, args.demo).await;
    app::Session::init(&ui, args.demo);
    app::Power::init(&ui, &config.power);
    app::Appearance::init(&ui, &config);
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
    let demo_mode = args.demo;

    ui.on_login(move |username_or_login, password| {
        let ui_weak = ui_handle.clone();
        let users_data_login = users_data_login.clone();
        let cache_login = cache_login.clone();
        let username_or_login = username_or_login.to_string();
        let password = password.to_string();

        let _ = slint::spawn_local(async move {
            let ui = ui_weak.unwrap();
            if username_or_login.is_empty() {
                ui.set_auth_error(SharedString::from("Please select a user"));
                return;
            }

            let user_data = users_data_login
                .iter()
                .find(|u| u.user_name == username_or_login || u.real_name == username_or_login);

            let actual_username = match user_data {
                Some(data) => data.user_name.to_string(),
                None => username_or_login.clone(),
            };

            info!("Login attempt for '{}'!", actual_username);

            let current_comp_idx = ui.get_selected_compositor_index();
            let mut comp_name = String::new();
            let mut comp_exec = String::new();
            if let Some(comp) = ui.get_compositors().row_data(current_comp_idx as usize) {
                comp_name = comp.name.to_string();
                comp_exec = comp.exec.to_string();
            }

            if comp_exec.is_empty() {
                ui.set_auth_error(SharedString::from("Please select a session"));
                return;
            }

            // Greetd Interaction
            match crate::utils::client::GreetdClient::new(demo_mode).await {
                Ok(mut client) => match client.create_session(&actual_username).await {
                    Ok(_) => {
                        let status = client.get_auth_status().clone();
                        if status == crate::utils::client::AuthStatus::InProgress {
                            match client.send_auth_response(Some(password)).await {
                                Ok(_) => {
                                    if *client.get_auth_status()
                                        == crate::utils::client::AuthStatus::Done
                                    {
                                        let cmd: Vec<String> = shlex::split(&comp_exec)
                                            .unwrap_or_else(|| vec![comp_exec.clone()]);
                                        let env = vec![];
                                        match client.start_session(cmd, env).await {
                                            Ok(_) => {
                                                let mut cache = cache_login.lock().unwrap();
                                                cache.set_last_user(actual_username.clone());
                                                cache.set_last_session(actual_username, comp_name);
                                                cache.save();
                                                info!("Session started successfully. Exiting.");
                                                slint::quit_event_loop().unwrap();
                                            }
                                            Err(e) => ui.set_auth_error(SharedString::from(
                                                format!("Failed to start session: {}", e),
                                            )),
                                        }
                                    } else {
                                        ui.set_auth_error(SharedString::from(
                                            "Authentication failed",
                                        ));
                                    }
                                }
                                Err(e) => ui.set_auth_error(SharedString::from(format!(
                                    "Auth error: {}",
                                    e
                                ))),
                            }
                        } else if status == crate::utils::client::AuthStatus::Done {
                            let cmd: Vec<String> =
                                shlex::split(&comp_exec).unwrap_or_else(|| vec![comp_exec.clone()]);
                            let env = vec![];
                            match client.start_session(cmd, env).await {
                                Ok(_) => {
                                    let mut cache = cache_login.lock().unwrap();
                                    cache.set_last_user(actual_username.clone());
                                    cache.set_last_session(actual_username, comp_name);
                                    cache.save();
                                    info!("Session started successfully. Exiting.");
                                    slint::quit_event_loop().unwrap();
                                }
                                Err(e) => ui.set_auth_error(SharedString::from(format!(
                                    "Failed to start session: {}",
                                    e
                                ))),
                            }
                        } else {
                            ui.set_auth_error(SharedString::from(
                                "Failed to initialize authentication",
                            ));
                        }
                    }
                    Err(e) => {
                        ui.set_auth_error(SharedString::from(format!("Session error: {}", e)))
                    }
                },
                Err(e) => ui.set_auth_error(SharedString::from(format!("Client error: {}", e))),
            }
        });
    });

    ui.run().unwrap();
}
