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
    pub fn init(ui: &GreeterWindow, cache: Arc<Mutex<Cache>>, is_demo: bool) {
        let ui_weak = ui.as_weak();

        ui.on_login(move |username, password| {
            let Some(ui) = ui_weak.upgrade() else { return };

            let (exec_cmd, exec_name) = {
                let idx = ui.get_selected_compositor_index() as usize;
                let model = ui.get_compositors();
                match model.row_data(idx) {
                    Some(c) if !c.exec.is_empty() => (c.exec.to_string(), c.name.to_string()),
                    _ => {
                        ui.set_auth_error(slint::SharedString::from(gettext(
                            "No compositor selected",
                        )));
                        return;
                    }
                }
            };

            ui.set_is_authenticating(true);

            if is_demo {
                Self::handle_demo_login(&ui, &cache, &username, &exec_cmd, &exec_name);
                return;
            }

            tokio::spawn(Self::execute_login_flow(
                ui_weak.clone(),
                cache.clone(),
                username.to_string(),
                password.to_string(),
                exec_cmd,
                exec_name,
            ));
        });
    }

    /// Handles the demo mode login simulation.
    fn handle_demo_login(
        ui: &GreeterWindow,
        cache: &Arc<Mutex<Cache>>,
        username: &str,
        exec_cmd: &str,
        exec_name: &str,
    ) {
        tracing::info!(
            "Demo mode: simulated login for user {}, compositor exec: {}",
            username,
            exec_cmd
        );
        {
            let mut cache_lock = cache.lock().unwrap();
            cache_lock.set_last_user(username.to_string());
            cache_lock.set_last_session(username.to_string(), exec_name.to_string());
            cache_lock.save();
        }
        ui.set_is_authenticating(false);
        std::process::exit(0);
    }

    /// The asynchronous login task that communicates with greetd.
    async fn execute_login_flow(
        ui_weak: slint::Weak<GreeterWindow>,
        cache: Arc<Mutex<Cache>>,
        username: String,
        password: String,
        exec_cmd: String,
        exec_name: String,
    ) {
        let result = Self::perform_auth(&cache, &username, &password, &exec_cmd, &exec_name).await;

        let _ = ui_weak.upgrade_in_event_loop(move |ui| {
            ui.set_is_authenticating(false);
            match result {
                Ok(_) => std::process::exit(0),
                Err(e) => Self::report_error(&ui, e),
            }
        });
    }

    /// Orchestrates the greetd IPC communication.
    async fn perform_auth(
        cache: &Arc<Mutex<Cache>>,
        username: &str,
        password: &str,
        exec_cmd: &str,
        exec_name: &str,
    ) -> std::result::Result<(), ClientError> {
        let mut client = GreetdClient::new()
            .await
            .map_err(|e| ClientError::System(e.to_string()))?;

        client.authenticate(username, password).await?;

        // Save to cache after successful authentication
        {
            let mut cache_lock = cache.lock().unwrap();
            cache_lock.set_last_user(username.to_string());
            cache_lock.set_last_session(username.to_string(), exec_name.to_string());
            cache_lock.save();
        }

        let cmd = shlex::split(exec_cmd).unwrap_or_else(|| vec![exec_cmd.to_string()]);
        client.start_session(cmd, vec![]).await?;

        Ok(())
    }

    /// Displays the appropriate error message in the UI.
    fn report_error(ui: &GreeterWindow, error: ClientError) {
        match error {
            ClientError::Auth(msg) => {
                ui.set_auth_error(slint::SharedString::from(msg));
            }
            ClientError::System(msg) => {
                ui.invoke_show_system_error(slint::SharedString::from(msg));
            }
        }
    }
}
