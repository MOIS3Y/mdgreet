use anyhow::{Context, Result};
use greetd_ipc::{AuthMessageType, ErrorType, Request, Response, codec::TokioCodec};
use tokio::net::UnixStream;
use tracing::{debug, info, warn};

const GREETD_SOCK_ENV_VAR: &str = "GREETD_SOCK";

#[derive(Clone, PartialEq, Debug)]
pub enum AuthStatus {
    NotStarted,
    InProgress,
    Done,
}

pub struct GreetdClient {
    socket: Option<UnixStream>,
    auth_status: AuthStatus,
    demo: bool,
}

impl GreetdClient {
    pub async fn new(demo: bool) -> Result<Self> {
        let socket = if demo {
            warn!("client: Running in DEMO mode (no greetd connection)");
            None
        } else {
            let sock_path = std::env::var(GREETD_SOCK_ENV_VAR).with_context(|| {
                format!(
                    "Missing environment variable '{}'. Is greetd running?",
                    GREETD_SOCK_ENV_VAR
                )
            })?;
            info!("client: Connecting to greetd socket at {}", sock_path);
            Some(UnixStream::connect(sock_path).await?)
        };

        Ok(Self {
            socket,
            auth_status: AuthStatus::NotStarted,
            demo,
        })
    }

    pub async fn create_session(&mut self, username: &str) -> Result<Response> {
        info!("client: Creating session for user: {}", username);

        let resp = if let Some(socket) = &mut self.socket {
            let msg = Request::CreateSession {
                username: username.to_string(),
            };
            msg.write_to(socket).await?;
            Response::read_from(socket).await?
        } else {
            // Demo mode simulation
            Response::AuthMessage {
                auth_message_type: AuthMessageType::Secret,
                auth_message: "Password:".to_string(),
            }
        };

        self.update_status(&resp);
        Ok(resp)
    }

    pub async fn send_auth_response(&mut self, input: Option<String>) -> Result<Response> {
        debug!("client: Sending auth response to greetd");

        let resp = if let Some(socket) = &mut self.socket {
            let msg = Request::PostAuthMessageResponse {
                response: input.clone(),
            };
            msg.write_to(socket).await?;
            Response::read_from(socket).await?
        } else {
            // Demo mode simulation: accept 'greet' as password
            match input.as_deref() {
                Some("greet") => Response::Success,
                _ => Response::Error {
                    error_type: ErrorType::AuthError,
                    description: "Invalid password (use 'greet' for demo)".to_string(),
                },
            }
        };

        self.update_status(&resp);
        Ok(resp)
    }

    pub async fn start_session(
        &mut self,
        command: Vec<String>,
        env: Vec<String>,
    ) -> Result<Response> {
        info!("client: Starting session with command: {:?}", command);

        if self.demo || self.socket.is_none() {
            info!("client: Demo mode - session start simulated.");
            return Ok(Response::Success);
        }

        let socket = self.socket.as_mut().unwrap();
        let msg = Request::StartSession { cmd: command, env };
        msg.write_to(socket).await?;

        let resp = Response::read_from(socket).await?;
        Ok(resp)
    }

    // pub async fn cancel_session(&mut self) -> Result<Response> {
    //     info!("client: Cancelling greetd session");
    //     self.auth_status = AuthStatus::NotStarted;

    //     if self.demo || self.socket.is_none() {
    //         return Ok(Response::Success);
    //     }

    //     let socket = self.socket.as_mut().unwrap();
    //     let msg = Request::CancelSession;
    //     msg.write_to(socket).await?;

    //     let resp = Response::read_from(socket).await?;
    //     Ok(resp)
    // }

    pub fn get_auth_status(&self) -> &AuthStatus {
        &self.auth_status
    }

    fn update_status(&mut self, response: &Response) {
        match response {
            Response::Success => self.auth_status = AuthStatus::Done,
            Response::AuthMessage { .. } => self.auth_status = AuthStatus::InProgress,
            Response::Error { .. } => self.auth_status = AuthStatus::NotStarted,
        }
    }
}
