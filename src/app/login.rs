use crate::GreeterWindow;
use crate::utils::cache::Cache;
use crate::utils::client::{ClientError, GreetdClient};
use gettextrs::gettext;
use slint::{ComponentHandle, Model};
use std::sync::{Arc, Mutex};

/// Handles the authentication flow and session startup.
pub struct Login;

impl Login {
    /// Registers the `on_login` callback for the Slint UI.
    ///
    /// This method captures the necessary UI handle and shared cache state,
    /// launching an asynchronous task to communicate with the `greetd` daemon
    /// when the user attempts to log in.
    pub fn init(ui: &GreeterWindow, cache: Arc<Mutex<Cache>>, is_demo: bool) {
        let ui_weak = ui.as_weak();

        ui.on_login(move |username, password| {
            // Upgrade the UI handle to access its current state
            let ui = match ui_weak.upgrade() {
                Some(ui) => ui,
                None => return,
            };

            let selected_compositor_idx = ui.get_selected_compositor_index();
            let compositors = ui.get_compositors();

            let exec_cmd =
                if let Some(comp) = compositors.row_data(selected_compositor_idx as usize) {
                    comp.exec.to_string()
                } else {
                    String::new()
                };

            if exec_cmd.is_empty() {
                let msg = gettext("No compositor selected");
                ui.set_auth_error(slint::SharedString::from(msg));
                return;
            }

            let username_str = username.to_string();
            let password_str = password.to_string();
            let exec_name =
                if let Some(comp) = compositors.row_data(selected_compositor_idx as usize) {
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
                    let mut cache_lock = cache.lock().unwrap();
                    cache_lock.set_last_user(username_str.clone());
                    cache_lock.set_last_session(username_str.clone(), exec_name.clone());
                    cache_lock.save();
                }
                ui.set_is_authenticating(false);
                std::process::exit(0);
            }

            // Clone references for the async task
            let ui_async = ui_weak.clone();
            let cache_async = cache.clone();

            tokio::spawn(async move {
                match GreetdClient::new().await {
                    Ok(mut client) => {
                        if let Err(e) = client.authenticate(&username_str, &password_str).await {
                            let _ = ui_async.upgrade_in_event_loop(move |ui| {
                                match e {
                                    ClientError::Auth(msg) => {
                                        ui.set_auth_error(slint::SharedString::from(msg));
                                    }
                                    ClientError::System(msg) => {
                                        ui.invoke_show_system_error(slint::SharedString::from(msg));
                                    }
                                }
                                ui.set_is_authenticating(false);
                            });
                            return;
                        }

                        // Save to cache after successful authentication
                        {
                            let mut cache_lock = cache_async.lock().unwrap();
                            cache_lock.set_last_user(username_str.clone());
                            cache_lock.set_last_session(username_str.clone(), exec_name.clone());
                            cache_lock.save();
                        }

                        let cmd = shlex::split(&exec_cmd).unwrap_or_else(|| vec![exec_cmd]);
                        let env = vec![];

                        if let Err(e) = client.start_session(cmd, env).await {
                            let _ = ui_async.upgrade_in_event_loop(move |ui| {
                                match e {
                                    ClientError::Auth(msg) => {
                                        ui.set_auth_error(slint::SharedString::from(msg));
                                    }
                                    ClientError::System(msg) => {
                                        ui.invoke_show_system_error(slint::SharedString::from(msg));
                                    }
                                }
                                ui.set_is_authenticating(false);
                            });
                        } else {
                            let _ = ui_async.upgrade_in_event_loop(move |ui| {
                                ui.set_is_authenticating(false);
                            });
                            std::process::exit(0); // Session started successfully
                        }
                    }
                    Err(e) => {
                        // Connection or initialization error
                        let err_msg = e.to_string();
                        let _ = ui_async.upgrade_in_event_loop(move |ui| {
                            ui.invoke_show_system_error(slint::SharedString::from(err_msg));
                            ui.set_is_authenticating(false);
                        });
                    }
                }
            });
        });
    }
}
