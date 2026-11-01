//! A deterministic, local greetd peer for previews and controller tests.

use super::*;
use greetd_ipc::{AuthMessageType, ErrorType};
use std::time::Duration;

/// Simulated outcomes, never used by a connection to the real greetd daemon.
#[derive(Clone, Copy)]
pub(crate) enum DemoScenario {
    /// Fingerprint succeeds without an input prompt.
    Fingerprint,
    /// Fingerprint times out and PAM asks for a password.
    Fallback,
    /// Two unmatched scans lead to a password prompt.
    Mismatch,
    /// The fallback password is rejected.
    Error,
}

impl DemoScenario {
    /// Read the optional preview setting once, before creating a task.
    pub(crate) fn from_env() -> Self {
        match std::env::var("MDGREET_DEMO_AUTH").as_deref() {
            Ok("fallback") => Self::Fallback,
            Ok("mismatch") => Self::Mismatch,
            Ok("error") => Self::Error,
            Ok("fingerprint") => Self::Fingerprint,
            _ => Self::Fallback,
        }
    }
}

/// A fake conversation owns its stage independently of real PAM services.
struct DemoPeer {
    scenario: DemoScenario,
    fingerprint: bool,
    scans: u8,
    waiting_for_scan: bool,
}

impl GreetdClient {
    /// Create an in-process IPC peer; never contact PAM or start a process.
    pub(crate) fn demo(fingerprint: bool, scenario: DemoScenario) -> Self {
        let (stream, peer) = UnixStream::pair().expect("demo socket pair");
        tokio::spawn(run_peer(
            peer,
            DemoPeer {
                scenario,
                fingerprint,
                scans: 0,
                waiting_for_scan: false,
            },
        ));
        Self { stream }
    }
}

/// Apply the real framing codec to the simulated request-response exchange.
async fn run_peer(mut peer: UnixStream, mut state: DemoPeer) {
    while let Ok(request) = Request::read_from(&mut peer).await {
        let response = state.respond(request).await;
        if response.write_to(&mut peer).await.is_err() {
            break;
        }
    }
}

impl DemoPeer {
    /// Dispatch lifecycle requests without executing any session command.
    async fn respond(&mut self, request: Request) -> Response {
        match request {
            Request::CancelSession | Request::StartSession { .. } => {
                Response::Success
            }
            Request::CreateSession { .. } if self.fingerprint => {
                self.waiting_for_scan = true;
                notice(
                    AuthMessageType::Info,
                    gettext("Demo: waiting for fingerprint"),
                )
            }
            Request::CreateSession { .. } => password_prompt(),
            Request::PostAuthMessageResponse { .. }
                if self.waiting_for_scan =>
            {
                self.scan().await
            }
            Request::PostAuthMessageResponse { response } => {
                if matches!(self.scenario, DemoScenario::Error)
                    && response.as_deref() == Some("incorrect-demo")
                {
                    Response::Error {
                        error_type: ErrorType::AuthError,
                        description: "demo".into(),
                    }
                } else {
                    Response::Success
                }
            }
        }
    }

    /// Simulate a bounded fingerprint stage before success or password fallback.
    async fn scan(&mut self) -> Response {
        tokio::time::sleep(Duration::from_secs(1)).await;
        if matches!(self.scenario, DemoScenario::Mismatch) && self.scans < 2 {
            self.scans += 1;
            return notice(
                AuthMessageType::Error,
                gettext("Demo: fingerprint not recognized"),
            );
        }
        self.waiting_for_scan = false;
        match self.scenario {
            DemoScenario::Fingerprint => Response::Success,
            _ => password_prompt(),
        }
    }
}

fn password_prompt() -> Response {
    notice(AuthMessageType::Secret, gettext("Password"))
}

fn notice(
    auth_message_type: AuthMessageType,
    auth_message: String,
) -> Response {
    Response::AuthMessage {
        auth_message_type,
        auth_message,
    }
}
