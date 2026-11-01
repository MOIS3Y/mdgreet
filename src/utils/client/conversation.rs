//! Request-boundary handling for one sequential greetd/PAM conversation.

use super::{AuthAnswer, AuthEvent, ClientError, GreetdClient};
use gettextrs::gettext;
use greetd_ipc::codec::TokioCodec;
use greetd_ipc::{AuthMessageType, ErrorType, Request, Response};
use std::time::Duration;
use tokio::sync::{mpsc, watch};

/// Maximum cleanup wait after a user asks to cancel an outstanding request.
const DRAIN_TIMEOUT: Duration = Duration::from_secs(15);
/// Maximum wait for acknowledgement of CancelSession at a request boundary.
const CANCEL_TIMEOUT: Duration = Duration::from_secs(5);

impl GreetdClient {
    /// Run a conversation, consuming each submitted response at most once.
    ///
    /// Returns true only for greetd authentication Success, and false after
    /// acknowledged cancellation. PAM messages are never treated as success.
    ///
    /// # Errors
    /// Returns credential rejection, an IPC failure, or a cleanup timeout.
    pub async fn authenticate(
        &mut self,
        username: &str,
        input: &mut mpsc::UnboundedReceiver<AuthAnswer>,
        cancel: &mut watch::Receiver<bool>,
        mut event: impl FnMut(AuthEvent),
    ) -> Result<bool, ClientError> {
        let mut request = Request::CreateSession {
            username: username.into(),
        };
        loop {
            if *cancel.borrow() {
                return self.cancelled().await;
            }
            request.write_to(&mut self.stream).await?;
            let response = self.read_auth_response(cancel).await?;
            if *cancel.borrow() {
                return self.cancelled().await;
            }
            match self
                .handle_response(response, input, cancel, &mut event)
                .await?
            {
                ConversationStep::Authenticated => return Ok(true),
                ConversationStep::Cancelled => return self.cancelled().await,
                ConversationStep::Reply(response) => {
                    request = Request::PostAuthMessageResponse { response };
                }
            }
        }
    }

    /// Keep protocol classification separate from request scheduling.
    async fn handle_response(
        &mut self,
        response: Response,
        input: &mut mpsc::UnboundedReceiver<AuthAnswer>,
        cancel: &mut watch::Receiver<bool>,
        event: &mut impl FnMut(AuthEvent),
    ) -> Result<ConversationStep, ClientError> {
        match response {
            Response::Success => Ok(ConversationStep::Authenticated),
            Response::Error { error_type, .. } => {
                // greetd may report a second error while reaping the failed
                // PAM worker. Cleanup must not replace the original verdict.
                if self.cancel().await.is_err() {
                    tracing::warn!(
                        "Authentication cleanup failed after a terminal error"
                    );
                }
                Err(authentication_error(error_type))
            }
            Response::AuthMessage {
                auth_message,
                auth_message_type,
            } => Ok(answer_message(
                auth_message_type,
                auth_message,
                input,
                cancel,
                event,
            )
            .await),
        }
    }

    /// Keep a partial frame intact while waiting for a cancellation boundary.
    async fn read_auth_response(
        &mut self,
        cancel: &mut watch::Receiver<bool>,
    ) -> Result<Response, ClientError> {
        let read = Response::read_from(&mut self.stream);
        tokio::pin!(read);
        tokio::select! {
            result = &mut read => Ok(result?),
            _ = cancel.changed() => {
                Ok(tokio::time::timeout(DRAIN_TIMEOUT, &mut read)
                    .await.map_err(|_| cancellation_timeout())??)
            }
        }
    }

    /// Complete a cancelled conversation before releasing its connection.
    async fn cancelled(&mut self) -> Result<bool, ClientError> {
        self.cancel().await?;
        Ok(false)
    }

    /// Cancel at a request boundary and consume the acknowledgement.
    ///
    /// # Errors
    /// Returns a transport error, rejection, or cancellation timeout.
    pub async fn cancel(&mut self) -> Result<(), ClientError> {
        tokio::time::timeout(CANCEL_TIMEOUT, async {
            Request::CancelSession.write_to(&mut self.stream).await?;
            match Response::read_from(&mut self.stream).await? {
                Response::Success => Ok(()),
                _ => Err(ClientError::System(gettext(
                    "Failed to cancel authentication",
                ))),
            }
        })
        .await
        .map_err(|_| cancellation_timeout())?
    }
}

/// Outcomes depend on IPC message types, never on localized message text.
enum ConversationStep {
    Reply(Option<String>),
    Authenticated,
    Cancelled,
}

/// Acknowledge notices or obtain a fresh response of the requested kind.
async fn answer_message(
    kind: AuthMessageType,
    text: String,
    input: &mut mpsc::UnboundedReceiver<AuthAnswer>,
    cancel: &mut watch::Receiver<bool>,
    event: &mut impl FnMut(AuthEvent),
) -> ConversationStep {
    match kind {
        AuthMessageType::Info | AuthMessageType::Error => {
            event(AuthEvent::Message {
                text,
                error: matches!(kind, AuthMessageType::Error),
            });
            ConversationStep::Reply(None)
        }
        AuthMessageType::Secret | AuthMessageType::Visible => {
            let secret = matches!(kind, AuthMessageType::Secret);
            event(AuthEvent::Prompt { text, secret });
            match receive_answer(input, cancel, secret).await {
                Some(answer) => {
                    event(AuthEvent::Verifying);
                    ConversationStep::Reply(Some(answer))
                }
                None => ConversationStep::Cancelled,
            }
        }
    }
}

/// Discard mismatched queued answers, especially passwords before Visible.
async fn receive_answer(
    input: &mut mpsc::UnboundedReceiver<AuthAnswer>,
    cancel: &mut watch::Receiver<bool>,
    secret: bool,
) -> Option<String> {
    loop {
        if *cancel.borrow() {
            return None;
        }
        let answer = tokio::select! {
            answer = input.recv() => answer?,
            _ = cancel.changed() => return None,
        };
        if answer.secret == secret {
            return Some(answer.value);
        }
    }
}

/// Map typed greetd failure categories without interpreting localized text.
fn authentication_error(kind: ErrorType) -> ClientError {
    match kind {
        ErrorType::AuthError => ClientError::Auth(gettext(
            "Authentication failed. Try again or use your password.",
        )),
        _ => ClientError::System(gettext("Authentication service error")),
    }
}

fn cancellation_timeout() -> ClientError {
    ClientError::System(gettext("Authentication cancellation timed out"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use greetd_ipc::{AuthMessageType, ErrorType};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio::net::UnixStream;
    use tokio::sync::{mpsc, watch};

    fn message(kind: AuthMessageType) -> Response {
        Response::AuthMessage {
            auth_message_type: kind,
            auth_message: "Локализованное сообщение".into(),
        }
    }

    async fn receive(peer: &mut UnixStream) -> Request {
        tokio::time::timeout(Duration::from_secs(2), Request::read_from(peer))
            .await
            .unwrap()
            .unwrap()
    }

    #[tokio::test]
    async fn conversation_acknowledges_messages_and_consumes_each_answer_once()
    {
        let (stream, mut peer) = UnixStream::pair().unwrap();
        let server = tokio::spawn(async move {
            assert!(
                matches!(receive(&mut peer).await, Request::CreateSession { username } if username == "alice")
            );
            for kind in [AuthMessageType::Info, AuthMessageType::Error] {
                message(kind).write_to(&mut peer).await.unwrap();
                assert!(matches!(
                    receive(&mut peer).await,
                    Request::PostAuthMessageResponse { response: None }
                ));
            }
            for (kind, expected) in [
                (AuthMessageType::Secret, "one"),
                (AuthMessageType::Secret, "two"),
                (AuthMessageType::Visible, "mdgreet:password"),
            ] {
                message(kind).write_to(&mut peer).await.unwrap();
                let request = receive(&mut peer).await;
                assert!(
                    matches!(request, Request::PostAuthMessageResponse { response: Some(value) } if value == expected)
                );
            }
            Response::Success.write_to(&mut peer).await.unwrap();
            assert!(matches!(
                receive(&mut peer).await,
                Request::StartSession { .. }
            ));
            Response::Success.write_to(&mut peer).await.unwrap();
        });
        let mut client = GreetdClient { stream };
        let (tx, mut rx) = mpsc::unbounded_channel();
        for (value, secret) in
            [("one", true), ("two", true), ("mdgreet:password", false)]
        {
            tx.send(AuthAnswer {
                value: value.into(),
                secret,
            })
            .unwrap();
        }
        let (_cancel, mut cancelled) = watch::channel(false);
        let mut prompts = Vec::new();
        assert!(
            client
                .authenticate("alice", &mut rx, &mut cancelled, |event| {
                    if let AuthEvent::Prompt { secret, .. } = event {
                        prompts.push(secret);
                    }
                })
                .await
                .unwrap()
        );
        assert_eq!(prompts, [true, true, false]);
        client
            .start_session(vec!["session".into()], vec![])
            .await
            .unwrap();
        server.await.unwrap();
    }

    #[tokio::test]
    async fn repeated_secret_waits_for_fresh_input_and_can_be_cancelled() {
        let (stream, mut peer) = UnixStream::pair().unwrap();
        let (cancel, mut cancelled) = watch::channel(false);
        let server = tokio::spawn(async move {
            receive(&mut peer).await;
            message(AuthMessageType::Secret)
                .write_to(&mut peer)
                .await
                .unwrap();
            assert!(matches!(
                receive(&mut peer).await,
                Request::PostAuthMessageResponse { response: Some(_) }
            ));
            message(AuthMessageType::Secret)
                .write_to(&mut peer)
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_millis(20)).await;
            cancel.send(true).unwrap();
            assert!(matches!(
                receive(&mut peer).await,
                Request::CancelSession
            ));
            Response::Success.write_to(&mut peer).await.unwrap();
        });
        let (tx, mut rx) = mpsc::unbounded_channel();
        tx.send(AuthAnswer {
            value: "one".into(),
            secret: true,
        })
        .unwrap();
        assert!(
            !GreetdClient { stream }
                .authenticate("alice", &mut rx, &mut cancelled, |_| {})
                .await
                .unwrap()
        );
        server.await.unwrap();
    }

    #[tokio::test]
    async fn cancellation_drains_inflight_success_without_starting_session() {
        let (stream, mut peer) = UnixStream::pair().unwrap();
        let (cancel, mut cancelled) = watch::channel(false);
        let server = tokio::spawn(async move {
            receive(&mut peer).await;
            cancel.send(true).unwrap();
            tokio::time::sleep(Duration::from_millis(20)).await;
            Response::Success.write_to(&mut peer).await.unwrap();
            assert!(matches!(
                receive(&mut peer).await,
                Request::CancelSession
            ));
            Response::Success.write_to(&mut peer).await.unwrap();
        });
        let (_tx, mut rx) = mpsc::unbounded_channel();
        assert!(
            !GreetdClient { stream }
                .authenticate("alice", &mut rx, &mut cancelled, |_| {})
                .await
                .unwrap()
        );
        server.await.unwrap();
    }

    #[tokio::test]
    async fn errors_are_terminal_and_cancellation_is_acknowledged() {
        for kind in [ErrorType::AuthError, ErrorType::Error] {
            let auth_error = matches!(kind, ErrorType::AuthError);
            let (stream, mut peer) = UnixStream::pair().unwrap();
            let server = tokio::spawn(async move {
                receive(&mut peer).await;
                Response::Error {
                    error_type: kind,
                    description: "failure".into(),
                }
                .write_to(&mut peer)
                .await
                .unwrap();
                assert!(matches!(
                    receive(&mut peer).await,
                    Request::CancelSession
                ));
                Response::Success.write_to(&mut peer).await.unwrap();
            });
            let (_tx, mut rx) = mpsc::unbounded_channel();
            let (_cancel, mut cancelled) = watch::channel(false);
            let result = GreetdClient { stream }
                .authenticate("alice", &mut rx, &mut cancelled, |_| {})
                .await;
            assert!(if auth_error {
                matches!(result, Err(ClientError::Auth(_)))
            } else {
                matches!(result, Err(ClientError::System(_)))
            });
            server.await.unwrap();
        }
    }

    #[tokio::test]
    async fn cleanup_failure_preserves_credential_rejection() {
        let (stream, mut peer) = UnixStream::pair().unwrap();
        let server = tokio::spawn(async move {
            receive(&mut peer).await;
            Response::Error {
                error_type: ErrorType::AuthError,
                description: "credentials rejected".into(),
            }
            .write_to(&mut peer)
            .await
            .unwrap();
            assert!(matches!(
                receive(&mut peer).await,
                Request::CancelSession
            ));
            Response::Error {
                error_type: ErrorType::Error,
                description: "worker already failed".into(),
            }
            .write_to(&mut peer)
            .await
            .unwrap();
        });
        let (_tx, mut rx) = mpsc::unbounded_channel();
        let (_cancel, mut cancelled) = watch::channel(false);
        assert!(matches!(
            GreetdClient { stream }
                .authenticate("alice", &mut rx, &mut cancelled, |_| {})
                .await,
            Err(ClientError::Auth(_))
        ));
        server.await.unwrap();
    }

    #[tokio::test]
    async fn disconnection_is_not_authentication_success() {
        let (stream, peer) = UnixStream::pair().unwrap();
        drop(peer);
        let (_tx, mut rx) = mpsc::unbounded_channel();
        let (_cancel, mut cancelled) = watch::channel(false);
        assert!(matches!(
            GreetdClient { stream }
                .authenticate("alice", &mut rx, &mut cancelled, |_| {})
                .await,
            Err(ClientError::System(_))
        ));
    }

    #[tokio::test]
    async fn unavailable_fingerprint_and_timeout_fall_back_to_password() {
        // No device/enrollment, mismatch and timeout are PAM policy outcomes;
        // IPC carries their messages followed by a fresh password prompt.
        for text in [
            "No device",
            "No enrollment",
            "No match",
            "Timeout",
            "Service unavailable",
        ] {
            let (stream, mut peer) = UnixStream::pair().unwrap();
            let server = tokio::spawn(async move {
                receive(&mut peer).await;
                Response::AuthMessage {
                    auth_message_type: AuthMessageType::Error,
                    auth_message: text.into(),
                }
                .write_to(&mut peer)
                .await
                .unwrap();
                assert!(matches!(
                    receive(&mut peer).await,
                    Request::PostAuthMessageResponse { response: None }
                ));
                message(AuthMessageType::Secret)
                    .write_to(&mut peer)
                    .await
                    .unwrap();
                assert!(matches!(
                    receive(&mut peer).await,
                    Request::PostAuthMessageResponse { response: Some(_) }
                ));
                Response::Success.write_to(&mut peer).await.unwrap();
            });
            let (tx, mut rx) = mpsc::unbounded_channel();
            tx.send(AuthAnswer {
                value: "password".into(),
                secret: true,
            })
            .unwrap();
            let (_cancel, mut cancelled) = watch::channel(false);
            let seen = Arc::new(Mutex::new(false));
            let observed = seen.clone();
            assert!(
                GreetdClient { stream }
                    .authenticate(
                        "alice",
                        &mut rx,
                        &mut cancelled,
                        move |event| {
                            if let AuthEvent::Message { .. } = event {
                                *observed.lock().unwrap() = true;
                            }
                        }
                    )
                    .await
                    .unwrap()
            );
            assert!(*seen.lock().unwrap());
            server.await.unwrap();
        }
    }

    #[tokio::test]
    async fn cancellation_failure_is_reported() {
        let (stream, mut peer) = UnixStream::pair().unwrap();
        let server = tokio::spawn(async move {
            assert!(matches!(
                receive(&mut peer).await,
                Request::CancelSession
            ));
            Response::Error {
                error_type: ErrorType::Error,
                description: "cannot cancel".into(),
            }
            .write_to(&mut peer)
            .await
            .unwrap();
        });
        assert!(GreetdClient { stream }.cancel().await.is_err());
        server.await.unwrap();
    }
    #[tokio::test]
    async fn queued_password_is_never_sent_to_a_visible_prompt() {
        let (stream, mut peer) = UnixStream::pair().unwrap();
        let server = tokio::spawn(async move {
            receive(&mut peer).await;
            message(AuthMessageType::Visible)
                .write_to(&mut peer)
                .await
                .unwrap();
            assert!(
                matches!(receive(&mut peer).await, Request::PostAuthMessageResponse { response: Some(value) } if value == "alice")
            );
            Response::Success.write_to(&mut peer).await.unwrap();
        });
        let (tx, mut rx) = mpsc::unbounded_channel();
        tx.send(AuthAnswer {
            value: "must-not-be-visible".into(),
            secret: true,
        })
        .unwrap();
        tx.send(AuthAnswer {
            value: "alice".into(),
            secret: false,
        })
        .unwrap();
        let (_cancel, mut cancelled) = watch::channel(false);
        assert!(
            GreetdClient { stream }
                .authenticate("alice", &mut rx, &mut cancelled, |_| {})
                .await
                .unwrap()
        );
        server.await.unwrap();
    }

    #[tokio::test]
    async fn cancellation_preserves_a_partially_read_frame() {
        use tokio::io::AsyncWriteExt;
        let (stream, mut peer) = UnixStream::pair().unwrap();
        let (cancel, mut cancelled) = watch::channel(false);
        let server = tokio::spawn(async move {
            receive(&mut peer).await;
            let mut frame = Vec::new();
            Response::Success.write_to(&mut frame).await.unwrap();
            peer.write_all(&frame[..2]).await.unwrap();
            tokio::time::sleep(Duration::from_millis(10)).await;
            cancel.send(true).unwrap();
            tokio::time::sleep(Duration::from_millis(10)).await;
            peer.write_all(&frame[2..]).await.unwrap();
            assert!(matches!(
                receive(&mut peer).await,
                Request::CancelSession
            ));
            Response::Success.write_to(&mut peer).await.unwrap();
        });
        let (_tx, mut rx) = mpsc::unbounded_channel();
        assert!(
            !GreetdClient { stream }
                .authenticate("alice", &mut rx, &mut cancelled, |_| {})
                .await
                .unwrap()
        );
        server.await.unwrap();
    }

    #[tokio::test(start_paused = true)]
    async fn cancellation_of_an_unresponsive_peer_is_bounded() {
        let (stream, mut peer) = UnixStream::pair().unwrap();
        let (cancel, mut cancelled) = watch::channel(false);
        let server = tokio::spawn(async move {
            receive(&mut peer).await;
            cancel.send(true).unwrap();
            tokio::time::sleep(Duration::from_secs(30)).await;
        });
        let (_tx, mut rx) = mpsc::unbounded_channel();
        assert!(matches!(
            GreetdClient { stream }
                .authenticate("alice", &mut rx, &mut cancelled, |_| {})
                .await,
            Err(ClientError::System(_))
        ));
        server.abort();
    }

    /// While PAM waits for a scanner, only notices are acknowledged. The queued
    /// response is sent once, after a Secret, without creating another session.
    #[tokio::test]
    async fn queued_password_waits_for_the_same_conversations_secret_prompt() {
        let (stream, mut peer) = UnixStream::pair().unwrap();
        let server = tokio::spawn(async move {
            assert!(matches!(
                receive(&mut peer).await,
                Request::CreateSession { .. }
            ));
            message(AuthMessageType::Info)
                .write_to(&mut peer)
                .await
                .unwrap();
            assert!(matches!(
                receive(&mut peer).await,
                Request::PostAuthMessageResponse { response: None }
            ));
            assert!(
                tokio::time::timeout(
                    Duration::from_millis(40),
                    Request::read_from(&mut peer)
                )
                .await
                .is_err()
            );
            message(AuthMessageType::Secret)
                .write_to(&mut peer)
                .await
                .unwrap();
            assert!(matches!(receive(&mut peer).await,
                Request::PostAuthMessageResponse { response: Some(value) }
                if value == "queued-dummy"));
            Response::Success.write_to(&mut peer).await.unwrap();
        });
        let (tx, mut input) = mpsc::unbounded_channel();
        tx.send(AuthAnswer {
            value: "queued-dummy".into(),
            secret: true,
        })
        .unwrap();
        let (_cancel, mut cancelled) = watch::channel(false);
        assert!(
            GreetdClient { stream }
                .authenticate("alice", &mut input, &mut cancelled, |_| {},)
                .await
                .unwrap()
        );
        server.await.unwrap();
    }

    #[tokio::test]
    async fn fingerprint_success_does_not_send_a_queued_password() {
        let (stream, mut peer) = UnixStream::pair().unwrap();
        let server = tokio::spawn(async move {
            receive(&mut peer).await;
            message(AuthMessageType::Info)
                .write_to(&mut peer)
                .await
                .unwrap();
            assert!(matches!(
                receive(&mut peer).await,
                Request::PostAuthMessageResponse { response: None }
            ));
            Response::Success.write_to(&mut peer).await.unwrap();
        });
        let (tx, mut input) = mpsc::unbounded_channel();
        tx.send(AuthAnswer {
            value: "unused-dummy".into(),
            secret: true,
        })
        .unwrap();
        let (_cancel, mut cancelled) = watch::channel(false);
        assert!(
            GreetdClient { stream }
                .authenticate("alice", &mut input, &mut cancelled, |_| {},)
                .await
                .unwrap()
        );
        assert_eq!(input.try_recv().unwrap().value, "unused-dummy");
        server.await.unwrap();
    }
}
