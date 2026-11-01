use anyhow::{Context, Result};
use gettextrs::gettext;
use greetd_ipc::{Request, Response, codec::TokioCodec};
use tokio::net::UnixStream;
use tracing::info;

mod conversation;
mod demo;
pub(crate) use demo::DemoScenario;

#[derive(Debug)]
/// Authentication rejection or transport/service failure.
pub enum ClientError {
    /// PAM rejected the credentials.
    Auth(String),
    /// IPC or session lifecycle failed.
    System(String),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Auth(msg) => write!(f, "{}", msg),
            ClientError::System(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ClientError {}

impl From<std::io::Error> for ClientError {
    fn from(e: std::io::Error) -> Self {
        ClientError::System(e.to_string())
    }
}

impl From<greetd_ipc::codec::Error> for ClientError {
    fn from(e: greetd_ipc::codec::Error) -> Self {
        ClientError::System(e.to_string())
    }
}

/// Answers retain their input mode so a queued password is never sent to a
/// visible PAM prompt (for example, a username or challenge question).
pub struct AuthAnswer {
    /// The response, deliberately excluded from Debug and logs.
    pub value: String,
    /// Whether the response belongs to a secret prompt.
    pub secret: bool,
}

/// Conversation updates delivered to the interface without parsing PAM text.
pub enum AuthEvent {
    /// Informational or error text from PAM.
    Message { text: String, error: bool },
    /// A fresh input request; earlier responses must not be replayed.
    Prompt { text: String, secret: bool },
    /// A response has been submitted and is being checked.
    Verifying,
}

/// A client for interacting with the greetd daemon over a Unix domain socket.
pub struct GreetdClient {
    /// The asynchronous stream connected to the greetd socket.
    stream: UnixStream,
}

impl GreetdClient {
    /// Creates a new client by connecting to the socket path defined in the
    /// `GREETD_SOCK` environment variable.
    ///
    /// # Errors
    ///
    /// Returns an error if `GREETD_SOCK` is not set or if the connection fails.
    pub async fn new() -> Result<Self> {
        let socket_path = std::env::var("GREETD_SOCK").context(gettext(
            "GREETD_SOCK is not set. Are you running under greetd?",
        ))?;
        let stream = UnixStream::connect(socket_path).await?;
        Ok(Self { stream })
    }

    /// Requests greetd to start a new session with the specified command
    /// and environment.
    ///
    /// This should only be called after a successful [`Self::authenticate`] call.
    ///
    /// # Errors
    ///
    /// Returns an error if the command cannot be started or if greetd
    /// returns an error response.
    pub async fn start_session(
        &mut self,
        cmd: Vec<String>,
        env: Vec<String>,
    ) -> std::result::Result<(), ClientError> {
        let request = Request::StartSession { cmd, env };
        request.write_to(&mut self.stream).await?;

        let response = Response::read_from(&mut self.stream).await?;

        match response {
            Response::Success => {
                info!("Session started successfully!");
                Ok(())
            }
            Response::Error { description, .. } => {
                let prefix = gettext("Failed to start session");
                Err(ClientError::System(format!(
                    "{}: {}",
                    prefix, description
                )))
            }
            Response::AuthMessage { .. } => {
                let msg = gettext(
                    "Unexpected AuthMessage when trying to start session",
                );
                Err(ClientError::System(msg))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use greetd_ipc::ErrorType;

    #[tokio::test]
    async fn session_start_failure_is_not_success() {
        let (stream, mut peer) = UnixStream::pair().unwrap();
        let server = tokio::spawn(async move {
            assert!(matches!(
                tokio::time::timeout(
                    std::time::Duration::from_secs(2),
                    Request::read_from(&mut peer),
                )
                .await
                .unwrap()
                .unwrap(),
                Request::StartSession { .. }
            ));
            Response::Error {
                error_type: ErrorType::Error,
                description: "cannot launch".into(),
            }
            .write_to(&mut peer)
            .await
            .unwrap();
        });
        assert!(
            GreetdClient { stream }
                .start_session(vec!["example".into()], vec![])
                .await
                .is_err()
        );
        server.await.unwrap();
    }
}
