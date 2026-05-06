use anyhow::{Context, Result};
use greetd_ipc::{AuthMessageType, ErrorType, Request, Response, codec::TokioCodec};
use tokio::net::UnixStream;
use tracing::{debug, error, info};

pub struct GreetdClient {
    stream: UnixStream,
}

impl GreetdClient {
    /// Connects to the greetd socket
    pub async fn new() -> Result<Self> {
        let socket_path = std::env::var("GREETD_SOCK")
            .context("GREETD_SOCK is not set. Are you running under greetd?")?;
        let stream = UnixStream::connect(socket_path).await?;
        Ok(Self { stream })
    }

    /// Authenticate a user with a given password
    pub async fn authenticate(&mut self, username: &str, password: &str) -> Result<()> {
        let mut request = Request::CreateSession {
            username: username.to_string(),
        };

        let mut secret_prompted = false;

        loop {
            // Write the current request to greetd
            request.write_to(&mut self.stream).await?;

            // Read the response from greetd
            let response = Response::read_from(&mut self.stream).await?;

            match response {
                Response::AuthMessage {
                    auth_message,
                    auth_message_type,
                } => {
                    debug!("Received AuthMessage: {:?}", auth_message);
                    let reply = match auth_message_type {
                        AuthMessageType::Secret => {
                            if secret_prompted {
                                let _ = Request::CancelSession.write_to(&mut self.stream).await;
                                anyhow::bail!("Invalid username or password");
                            }
                            secret_prompted = true;
                            Some(password.to_string())
                        }
                        AuthMessageType::Visible => Some(username.to_string()),
                        AuthMessageType::Info => {
                            info!("greetd info: {}", auth_message);
                            None
                        }
                        AuthMessageType::Error => {
                            error!("greetd error: {}", auth_message);
                            None
                        }
                    };

                    request = Request::PostAuthMessageResponse { response: reply };
                }
                Response::Success => {
                    info!("Authentication successful!");
                    return Ok(());
                }
                Response::Error {
                    error_type,
                    description,
                } => {
                    let _ = Request::CancelSession.write_to(&mut self.stream).await;
                    if let ErrorType::AuthError = error_type {
                        anyhow::bail!("Invalid username or password");
                    } else {
                        anyhow::bail!("greetd error: {}", description);
                    }
                }
            }
        }
    }

    /// Starts a session with the given command and environment variables
    pub async fn start_session(&mut self, cmd: Vec<String>, env: Vec<String>) -> Result<()> {
        let request = Request::StartSession { cmd, env };
        request.write_to(&mut self.stream).await?;

        let response = Response::read_from(&mut self.stream).await?;

        match response {
            Response::Success => {
                info!("Session started successfully!");
                Ok(())
            }
            Response::Error { description, .. } => {
                anyhow::bail!("Failed to start session: {}", description);
            }
            Response::AuthMessage { .. } => {
                anyhow::bail!("Unexpected AuthMessage when trying to start session");
            }
        }
    }
}
