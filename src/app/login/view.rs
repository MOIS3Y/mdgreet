//! Generation-checked updates from asynchronous authentication to Slint.

use crate::GreeterWindow;
use crate::utils::client::{AuthEvent, ClientError};
use gettextrs::gettext;
use slint::ComponentHandle;
use tokio::sync::oneshot;

/// A UI destination that ignores callbacks from superseded attempts.
#[derive(Clone)]
pub(super) struct AuthView {
    ui: slint::Weak<GreeterWindow>,
    generation: i32,
}

impl AuthView {
    /// Initialize the visible state and allocate a new event generation.
    pub(super) fn begin(
        ui: &GreeterWindow,
        submitted: bool,
        fingerprint: bool,
    ) -> Self {
        let view = Self::next(ui);
        ui.set_auth_active(true);
        ui.set_auth_prompt_ready(false);
        let fingerprint_mode = fingerprint && !ui.get_password_stage();
        ui.set_password_requested(fingerprint && !fingerprint_mode);
        ui.set_fingerprint_mode(fingerprint_mode);
        ui.set_is_authenticating(submitted);
        ui.set_auth_error("".into());
        ui.set_auth_notice("".into());
        ui.set_auth_notice_error(false);
        ui.set_auth_prompt("".into());
        ui.set_auth_secret(true);
        ui.set_auth_status(
            if fingerprint_mode {
                String::new()
            } else if fingerprint {
                gettext("Waiting for password prompt")
            } else {
                gettext("Authenticating")
            }
            .into(),
        );
        ui.invoke_clear_auth_input();
        view
    }

    /// Invalidate pending callbacks before displaying cancellation progress.
    pub(super) fn cancel(ui: &GreeterWindow, active: bool) -> Self {
        let view = Self::next(ui);
        ui.set_auth_active(false);
        ui.set_auth_prompt_ready(false);
        ui.set_password_requested(false);
        ui.set_is_authenticating(false);
        ui.set_auth_prompt("".into());
        ui.set_auth_notice("".into());
        ui.set_auth_notice_error(false);
        ui.set_auth_status(
            if active {
                gettext("Cancelling authentication")
            } else {
                String::new()
            }
            .into(),
        );
        ui.invoke_clear_auth_input();
        view
    }

    /// Allocate generations only on the UI thread.
    fn next(ui: &GreeterWindow) -> Self {
        let generation = ui.get_auth_generation() + 1;
        ui.set_auth_generation(generation);
        Self {
            ui: ui.as_weak(),
            generation,
        }
    }

    /// Marshal an update and recheck identity at execution time.
    fn update(&self, update: impl FnOnce(GreeterWindow) + Send + 'static) {
        let generation = self.generation;
        let _ = self.ui.upgrade_in_event_loop(move |ui| {
            if ui.get_auth_generation() == generation {
                update(ui);
            }
        });
    }

    /// Display typed PAM events; message text never determines state changes.
    pub(super) fn event(&self, event: AuthEvent) {
        self.update(move |ui| match event {
            AuthEvent::Message { text, error } => {
                // A retry instruction must not erase the preceding failure.
                if !error
                    && ui.get_fingerprint_mode()
                    && ui.get_auth_notice_error()
                {
                    return;
                }
                if error && ui.get_password_stage() {
                    ui.set_auth_error(text.into());
                    ui.set_auth_notice("".into());
                    ui.set_auth_notice_error(false);
                } else {
                    ui.set_auth_notice(text.into());
                    ui.set_auth_notice_error(error);
                }
                if error && ui.get_fingerprint_mode() {
                    ui.set_is_authenticating(false);
                }
            }
            AuthEvent::Prompt { text, secret } => {
                ui.set_is_authenticating(false);
                ui.set_auth_prompt_ready(true);
                ui.set_fingerprint_mode(false);
                ui.set_password_stage(true);
                ui.set_password_requested(false);
                ui.set_auth_notice("".into());
                ui.set_auth_notice_error(false);
                ui.set_auth_prompt(text.into());
                ui.set_auth_secret(secret);
                ui.set_auth_status("".into());
            }
            AuthEvent::Verifying => {
                ui.set_is_authenticating(true);
                ui.set_auth_prompt("".into());
                ui.set_auth_status(gettext("Authenticating").into());
            }
        });
    }

    /// Commit session launch on the UI thread, after queued cancellations.
    pub(super) async fn permit_launch(&self) -> bool {
        let (tx, rx) = oneshot::channel();
        self.update(move |ui| {
            ui.set_auth_launching(true);
            ui.set_is_authenticating(true);
            ui.set_auth_status(gettext("Starting session").into());
            let _ = tx.send(true);
        });
        rx.await.unwrap_or(false)
    }

    /// Reset controls and report the outcome of the current attempt only.
    pub(super) fn finish(
        &self,
        result: Result<bool, ClientError>,
        demo: bool,
    ) {
        self.update(move |ui| {
            ui.set_auth_active(false);
            ui.set_auth_launching(false);
            ui.set_auth_prompt_ready(false);
            ui.set_password_requested(false);
            ui.set_is_authenticating(false);
            ui.set_auth_prompt("".into());
            ui.invoke_clear_auth_input();
            if result.is_ok() {
                ui.set_auth_notice("".into());
                ui.set_auth_notice_error(false);
            }
            match result {
                Ok(true) if !demo => std::process::exit(0),
                Ok(true) => ui.set_auth_status(
                    gettext("Demo: authentication successful").into(),
                ),
                Ok(false) => ui.set_auth_status(
                    gettext("Authentication cancelled").into(),
                ),
                Err(error) => {
                    ui.set_auth_status("".into());
                    let message = match error {
                        ClientError::Auth(_) if ui.get_password_stage() => {
                            gettext("Invalid username or password")
                        }
                        other => other.to_string(),
                    };
                    ui.set_auth_error(message.into());
                }
            }
        });
    }

    /// Surface cleanup errors instead of declaring unacknowledged cancellation.
    pub(super) fn cancel_finished(&self, error: Option<String>) {
        self.update(move |ui| match error {
            Some(error) => {
                ui.set_auth_status("".into());
                ui.set_auth_error(error.into());
            }
            None => {
                ui.set_auth_status(gettext("Authentication cancelled").into())
            }
        });
    }
}
