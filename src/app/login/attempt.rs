//! Background lifetime of a single greetd connection and session snapshot.

use super::view::AuthView;
use crate::Compositor;
use crate::utils::cache::Cache;
use crate::utils::client::{
    AuthAnswer, ClientError, DemoScenario, GreetdClient,
};
use gettextrs::gettext;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, watch};

/// Selects the real transport or an isolated, deterministic preview peer.
#[derive(Clone, Copy)]
pub(super) enum Backend {
    System,
    Demo(DemoScenario),
}

impl Backend {
    /// Preview results must never exit the process or persist a login.
    fn is_demo(self) -> bool {
        matches!(self, Self::Demo(_))
    }

    /// Connect without inspecting or changing system authentication policy.
    async fn connect(
        self,
        preview_fingerprint: bool,
    ) -> Result<GreetdClient, ClientError> {
        match self {
            Self::Demo(scenario) => {
                Ok(GreetdClient::demo(preview_fingerprint, scenario))
            }
            Self::System => GreetdClient::new()
                .await
                .map_err(|error| ClientError::System(error.to_string())),
        }
    }
}

/// UI-side controls; dropping them never detaches cleanup from the task.
pub(super) struct Attempt {
    /// Account captured before the task starts.
    pub(super) username: String,
    /// Fresh UI responses; never logged or cached.
    pub(super) input: mpsc::UnboundedSender<AuthAnswer>,
    /// Cooperative cancellation signal observed by the conversation.
    pub(super) cancel: watch::Sender<bool>,
    /// Cleanup outcome read only after the task releases its gate.
    pub(super) cleanup_error: Arc<Mutex<Option<String>>>,
}

/// Owns the connection until cancellation or session startup completes.
pub(super) struct LoginTask {
    view: AuthView,
    username: String,
    session: Compositor,
    cache: Arc<Mutex<Cache>>,
    gate: Arc<tokio::sync::Mutex<()>>,
    backend: Backend,
    preview_fingerprint: bool,
    input: mpsc::UnboundedReceiver<AuthAnswer>,
    cancel: watch::Receiver<bool>,
    cleanup_error: Arc<Mutex<Option<String>>>,
}

impl LoginTask {
    /// Create paired UI controls and task ownership for one attempt.
    pub(super) fn new(
        view: AuthView,
        username: String,
        session: Compositor,
        cache: Arc<Mutex<Cache>>,
        gate: Arc<tokio::sync::Mutex<()>>,
        backend: Backend,
        preview_fingerprint: bool,
    ) -> (Attempt, Self) {
        let (tx, input) = mpsc::unbounded_channel();
        let (cancel_tx, cancel) = watch::channel(false);
        let cleanup_error = Arc::new(Mutex::new(None));
        let attempt = Attempt {
            username: username.clone(),
            input: tx,
            cancel: cancel_tx,
            cleanup_error: cleanup_error.clone(),
        };
        let task = Self {
            view,
            username,
            session,
            cache,
            gate,
            backend,
            preview_fingerprint,
            input,
            cancel,
            cleanup_error,
        };
        (attempt, task)
    }

    /// Serialize attempts, retaining the gate through all connection cleanup.
    pub(super) async fn run(mut self) {
        let gate = self.gate.clone();
        let _guard = gate.lock().await;
        if *self.cancel.borrow() {
            return;
        }
        let result = self.authenticate_and_launch().await;
        if *self.cancel.borrow()
            && let Err(error) = &result
        {
            *self.cleanup_error.lock().unwrap() = Some(error.to_string());
        }
        self.view.finish(result, self.backend.is_demo());
    }

    /// Authenticate before requesting permission to start the selected session.
    async fn authenticate_and_launch(&mut self) -> Result<bool, ClientError> {
        let mut client =
            self.backend.connect(self.preview_fingerprint).await?;
        let view = self.view.clone();
        let authenticated = client
            .authenticate(
                &self.username,
                &mut self.input,
                &mut self.cancel,
                move |event| view.event(event),
            )
            .await?;
        if !authenticated {
            return Ok(false);
        }
        if *self.cancel.borrow() || !self.view.permit_launch().await {
            client.cancel().await?;
            return Ok(false);
        }
        self.launch(&mut client).await?;
        if !self.backend.is_demo() {
            self.save_success();
        }
        Ok(true)
    }

    /// Preserve shell argument parsing and clean up a rejected startup.
    async fn launch(
        &self,
        client: &mut GreetdClient,
    ) -> Result<(), ClientError> {
        let cmd = shlex::split(&self.session.exec).ok_or_else(|| {
            ClientError::System(gettext("Invalid session command"))
        })?;
        if let Err(error) = client.start_session(cmd, vec![]).await {
            let _ = client.cancel().await;
            return Err(error);
        }
        Ok(())
    }

    /// Persist account/session choices only after greetd accepts startup.
    fn save_success(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.set_last_user(self.username.clone());
        cache.set_last_session(
            self.username.clone(),
            self.session.name.to_string(),
        );
        cache.save();
    }
}
