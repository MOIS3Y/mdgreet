slint::include_modules!();

mod app;
mod cli;
mod config;
mod utils;

use cli::Args;
use config::GreeterConfig;
use slint::{ComponentHandle, Model};
use std::sync::{Arc, Mutex};
use tracing::info;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = GreeterConfig::load(&args.config);

    // Initialize logging first so early errors can be captured
    let _log_guard = utils::logging::init(&config.logging);

    // Initialize translations
    utils::i18n::init();

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

    let ui_login_weak = ui.as_weak();
    let cache_login = cache.clone();
    let is_demo = args.demo;

    ui.on_login(move |username, password| {
        let ui = ui_login_weak.unwrap();
        let selected_compositor_idx = ui.get_selected_compositor_index();
        let compositors = ui.get_compositors();

        let exec_cmd = if let Some(comp) = compositors.row_data(selected_compositor_idx as usize) {
            comp.exec.to_string()
        } else {
            String::new()
        };

        if exec_cmd.is_empty() {
            ui.set_auth_error(slint::SharedString::from(gettextrs::gettext(
                "No compositor selected",
            )));
            return;
        }

        let username_str = username.to_string();
        let password_str = password.to_string();
        let exec_name = if let Some(comp) = compositors.row_data(selected_compositor_idx as usize) {
            comp.name.to_string()
        } else {
            String::new()
        };

        ui.set_is_authenticating(true);

        if is_demo {
            tracing::info!(
                "Demo mode: simulated login for user {}, compositor exec: {}",
                username_str,
                exec_cmd
            );
            // Save to cache
            {
                let mut cache = cache_login.lock().unwrap();
                cache.set_last_user(username_str.clone());
                cache.set_last_session(username_str.clone(), exec_name.clone());
                cache.save();
            }
            ui.set_is_authenticating(false);
            std::process::exit(0);
        }

        let ui_weak_async = ui_login_weak.clone();
        let cache_async = cache_login.clone();

        tokio::spawn(async move {
            match crate::utils::client::GreetdClient::new().await {
                Ok(mut client) => {
                    if let Err(e) = client.authenticate(&username_str, &password_str).await {
                        let err_msg = e.to_string();
                        let _ = ui_weak_async.upgrade_in_event_loop(move |ui| {
                            ui.set_auth_error(slint::SharedString::from(err_msg));
                            ui.set_is_authenticating(false);
                        });
                        return;
                    }

                    // Save to cache
                    {
                        let mut cache = cache_async.lock().unwrap();
                        cache.set_last_user(username_str.clone());
                        cache.set_last_session(username_str.clone(), exec_name.clone());
                        cache.save();
                    }

                    let cmd = shlex::split(&exec_cmd).unwrap_or_else(|| vec![exec_cmd]);
                    let env = vec![];

                    if let Err(e) = client.start_session(cmd, env).await {
                        let err_msg = e.to_string();
                        let _ = ui_weak_async.upgrade_in_event_loop(move |ui| {
                            ui.set_auth_error(slint::SharedString::from(err_msg));
                            ui.set_is_authenticating(false);
                        });
                    } else {
                        let _ = ui_weak_async.upgrade_in_event_loop(move |ui| {
                            ui.set_is_authenticating(false);
                        });
                        std::process::exit(0);
                    }
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    let _ = ui_weak_async.upgrade_in_event_loop(move |ui| {
                        ui.set_auth_error(slint::SharedString::from(err_msg));
                        ui.set_is_authenticating(false);
                    });
                }
            }
        });
    });

    ui.run().unwrap();
}
