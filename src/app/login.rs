//! Login callback registration and ownership of the current attempt.

mod attempt;
mod view;

use self::attempt::{Attempt, Backend, LoginTask};
use self::view::AuthView;
use crate::GreeterWindow;
use crate::config::{AuthConfig, FingerprintConfig};
use crate::utils::cache::Cache;
use crate::utils::client::{AuthAnswer, DemoScenario};
use gettextrs::gettext;
use slint::{ComponentHandle, Model};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/// Registers login actions and initializes the configured sign-in mode.
pub struct Login;

/// UI-thread owner; background tasks receive only immutable attempt snapshots.
struct LoginController {
    ui: slint::Weak<GreeterWindow>,
    cache: Arc<Mutex<Cache>>,
    gate: Arc<tokio::sync::Mutex<()>>,
    active: RefCell<Option<Attempt>>,
    backend: Backend,
    fingerprint: bool,
}

impl Login {
    /// Register callbacks without contacting PAM until login is requested.
    pub fn init(
        ui: &GreeterWindow,
        cache: Arc<Mutex<Cache>>,
        is_demo: bool,
        config: &AuthConfig,
    ) {
        let backend = if is_demo {
            Backend::Demo(DemoScenario::from_env())
        } else {
            Backend::System
        };
        Self::register(ui, cache, backend, &config.fingerprint);
    }

    /// Inject a transport to exercise controller transitions without real PAM.
    fn register(
        ui: &GreeterWindow,
        cache: Arc<Mutex<Cache>>,
        backend: Backend,
        config: &FingerprintConfig,
    ) {
        let fingerprint = config.enable;
        let controller = Rc::new(LoginController {
            ui: ui.as_weak(),
            cache,
            gate: Arc::new(tokio::sync::Mutex::new(())),
            active: RefCell::new(None),
            backend,
            fingerprint,
        });
        let begin = controller.clone();
        ui.on_auth_begin(move |user, answer, submit| {
            begin.begin(&user, &answer, submit);
        });
        let password = controller.clone();
        ui.on_use_password(move |user| password.use_password(&user));
        let fingerprint_mode = controller.clone();
        ui.on_use_fingerprint(move |user| {
            fingerprint_mode.use_fingerprint(&user);
        });
        let hint = controller.clone();
        ui.on_fingerprint_hint(move || hint.fingerprint_hint());
        ui.on_cancel_auth(move || controller.cancel());
        ui.set_fingerprint_auto_start(config.auto_start);
        ui.set_fingerprint_enabled(fingerprint);
        ui.set_fingerprint_mode(fingerprint && !ui.get_password_stage());
        if fingerprint && ui.get_show_login() {
            ui.invoke_auth_context_changed();
        }
    }
}

impl LoginController {
    /// Explain the current action without starting or restarting PAM.
    fn fingerprint_hint(&self) {
        let Some(ui) = self.ui.upgrade() else { return };
        if ui.get_auth_launching() || !ui.get_fingerprint_mode() {
            return;
        }
        ui.set_auth_error("".into());
        ui.set_auth_notice("".into());
        ui.set_auth_notice_error(false);
        ui.set_auth_status(
            if ui.get_auth_active() {
                gettext("Touch the fingerprint reader")
            } else {
                gettext("Use password to continue")
            }
            .into(),
        );
    }

    /// Keep the existing PAM attempt so changing cards cannot reset its limit.
    fn use_password(&self, username: &str) {
        let Some(ui) = self.ui.upgrade() else { return };
        if ui.get_auth_launching() || username.is_empty() {
            return;
        }
        if !ui.get_auth_active() {
            self.begin(username, "", false);
        }
        ui.set_password_requested(true);
        // Keep this account in password mode, including after a rejected
        // password or a round trip through the clock screen.
        ui.set_password_stage(true);
        ui.set_fingerprint_mode(false);
        ui.set_is_authenticating(false);
        ui.set_auth_status(gettext("Waiting for password prompt").into());
        ui.invoke_clear_auth_input();
    }

    /// A fallback prompt ends fingerprint mode for this conversation.
    fn use_fingerprint(&self, username: &str) {
        let Some(ui) = self.ui.upgrade() else { return };
        if ui.get_auth_launching() || ui.get_password_stage() {
            return;
        }
        if !ui.get_auth_active() {
            self.begin(username, "", false);
        }
        ui.set_password_requested(false);
        ui.set_fingerprint_mode(true);
        ui.set_is_authenticating(false);
    }

    /// Submit to the current conversation or snapshot a new attempt.
    fn begin(&self, username: &str, answer: &str, submit: bool) {
        let Some(ui) = self.ui.upgrade() else { return };
        if ui.get_auth_launching()
            || (submit && ui.get_is_authenticating())
            || username.is_empty()
        {
            return;
        }
        if submit && self.fingerprint && !ui.get_password_stage() {
            self.use_password(username);
        }
        if submit && self.submit(&ui, username, answer) {
            return;
        }
        self.stop_attempt();
        let Some(session) = ui
            .get_compositors()
            .row_data(ui.get_selected_compositor_index() as usize)
            .filter(|session| !session.exec.is_empty())
        else {
            ui.set_auth_error(gettext("No compositor selected").into());
            return;
        };
        self.start_task(&ui, username, answer, submit, session);
    }

    /// Snapshot UI data before handing connection ownership to the worker.
    fn start_task(
        &self,
        ui: &GreeterWindow,
        username: &str,
        answer: &str,
        submit: bool,
        session: crate::Compositor,
    ) {
        let view = AuthView::begin(ui, submit, self.fingerprint);
        let (attempt, task) = LoginTask::new(
            view,
            username.into(),
            session,
            self.cache.clone(),
            self.gate.clone(),
            self.backend,
            self.fingerprint,
        );
        if submit {
            let _ = attempt.input.send(AuthAnswer {
                value: answer.into(),
                secret: true,
            });
        }
        *self.active.borrow_mut() = Some(attempt);
        tokio::spawn(task.run());
    }

    /// Deliver each answer once and clear its UI copy immediately.
    fn submit(&self, ui: &GreeterWindow, user: &str, answer: &str) -> bool {
        // A terminal UI update can arrive before the task drops its receiver.
        if !ui.get_auth_active() {
            return false;
        }
        let active = self.active.borrow();
        let Some(attempt) = active.as_ref() else {
            return false;
        };
        if attempt.username != user {
            return false;
        }
        let answer = AuthAnswer {
            value: answer.into(),
            secret: ui.get_auth_secret(),
        };
        if attempt.input.send(answer).is_err() {
            return false;
        }
        ui.set_is_authenticating(true);
        ui.invoke_clear_auth_input();
        true
    }

    /// Request cancellation; the task retains ownership until cleanup finishes.
    fn stop_attempt(&self) -> Option<Arc<Mutex<Option<String>>>> {
        self.active.borrow_mut().take().map(|old| {
            let _ = old.cancel.send(true);
            old.cleanup_error
        })
    }

    /// Invalidate stale UI events, then report acknowledged cleanup separately.
    fn cancel(&self) {
        let Some(ui) = self.ui.upgrade() else { return };
        if ui.get_auth_launching() {
            return;
        }
        let error = self.stop_attempt();
        let view = AuthView::cancel(&ui, error.is_some());
        let Some(error) = error else { return };
        let gate = self.gate.clone();
        tokio::spawn(async move {
            let _guard = gate.lock().await;
            view.cancel_finished(error.lock().unwrap().take());
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint::Rgb8Pixel;
    use slint::platform::software_renderer::{
        MinimalSoftwareWindow, RepaintBufferType,
    };
    use slint::platform::{
        EventLoopProxy, Platform, WindowAdapter, WindowEvent,
    };
    use std::time::{Duration, Instant};
    use tokio::sync::{mpsc, watch};

    thread_local! {
        static WINDOW: RefCell<Option<Rc<MinimalSoftwareWindow>>> =
            const { RefCell::new(None) };
    }

    type UiEvent = Box<dyn FnOnce() + Send>;
    type Events = std::sync::mpsc::Receiver<UiEvent>;

    /// Only forwards closures; it owns no real windowing or authentication service.
    struct Headless(std::sync::mpsc::Sender<UiEvent>);

    impl Platform for Headless {
        fn create_window_adapter(
            &self,
        ) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
            let window =
                MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
            WINDOW.with(|slot| *slot.borrow_mut() = Some(window.clone()));
            Ok(window)
        }

        fn new_event_loop_proxy(&self) -> Option<Box<dyn EventLoopProxy>> {
            Some(Box::new(Headless(self.0.clone())))
        }
    }

    impl EventLoopProxy for Headless {
        fn quit_event_loop(&self) -> Result<(), slint::EventLoopError> {
            Ok(())
        }

        fn invoke_from_event_loop(
            &self,
            event: UiEvent,
        ) -> Result<(), slint::EventLoopError> {
            self.0
                .send(event)
                .map_err(|_| slint::EventLoopError::EventLoopTerminated)
        }
    }

    /// Bound every asynchronous expectation, including deferred Slint trackers.
    async fn pump_until(events: &Events, condition: impl Fn() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            slint::platform::update_timers_and_animations();
            while let Ok(event) = events.try_recv() {
                event();
            }
            if condition() {
                return;
            }
            assert!(Instant::now() < deadline, "UI transition timed out");
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Build an isolated UI with dummy accounts, a non-executable session and cache.
    fn configured_clock_ui(
        scenario: DemoScenario,
        config: &FingerprintConfig,
    ) -> (GreeterWindow, Arc<Mutex<Cache>>) {
        let ui = GreeterWindow::new().unwrap();
        ui.set_compositors(
            Rc::new(slint::VecModel::from(vec![crate::Compositor {
                name: "Test session".into(),
                exec: "must-never-execute".into(),
            }]))
            .into(),
        );
        let users: Vec<crate::User> = ["alice", "bob"]
            .into_iter()
            .map(|name| crate::User {
                user_name: name.into(),
                real_name: name.into(),
                initials: name[..1].to_uppercase().into(),
                ..Default::default()
            })
            .collect();
        let menu = users
            .iter()
            .map(|user| crate::MenuItem {
                text: user.real_name.clone(),
                enabled: true,
                ..Default::default()
            })
            .collect::<Vec<_>>();
        ui.set_users(Rc::new(slint::VecModel::from(users)).into());
        ui.set_user_menu_items(Rc::new(slint::VecModel::from(menu)).into());
        let cache = Arc::new(Mutex::new(Cache::default()));
        ui.set_selected_user_index(0);
        Login::register(&ui, cache.clone(), Backend::Demo(scenario), config);
        ui.window().set_size(slint::PhysicalSize::new(800, 720));
        ui.show().unwrap();
        (ui, cache)
    }

    /// Existing scenarios use automatic fingerprint start, as before.
    fn clock_ui(
        scenario: DemoScenario,
        fingerprint: bool,
    ) -> (GreeterWindow, Arc<Mutex<Cache>>) {
        configured_clock_ui(
            scenario,
            &FingerprintConfig {
                enable: fingerprint,
                ..Default::default()
            },
        )
    }

    /// Open the login card explicitly for authentication and layout scenarios.
    fn demo_ui(
        scenario: DemoScenario,
        fingerprint: bool,
    ) -> (GreeterWindow, Arc<Mutex<Cache>>) {
        let (ui, cache) = clock_ui(scenario, fingerprint);
        ui.set_show_login(true);
        (ui, cache)
    }

    /// Render the actual card so geometry checks include Slint layout constraints.
    fn render(ui: &GreeterWindow) -> image::RgbImage {
        ui.window().request_redraw();
        slint::platform::update_timers_and_animations();
        WINDOW.with(|slot| {
            let window = slot.borrow().as_ref().unwrap().clone();
            let size = window.size();
            let mut pixels = vec![
                Rgb8Pixel::default();
                (size.width * size.height) as usize
            ];
            window.draw_if_needed(|renderer| {
                renderer.render(&mut pixels, size.width as usize);
            });
            let bytes = pixels.iter().flat_map(|p| [p.r, p.g, p.b]).collect();
            image::RgbImage::from_raw(size.width, size.height, bytes).unwrap()
        })
    }

    /// Export dummy screenshots only when explicitly requested by a developer.
    fn preview(ui: &GreeterWindow, name: &str) -> image::RgbImage {
        let image = render(ui);
        if let Ok(directory) = std::env::var("MDGREET_TEST_RENDER_DIR") {
            std::fs::create_dir_all(&directory).unwrap();
            image
                .save(std::path::Path::new(&directory).join(name))
                .unwrap();
        }
        image
    }

    /// Locate the filled primary control, excluding labels and avatar geometry.
    fn primary_rows(image: &image::RgbImage) -> Vec<u32> {
        let rows = (460..610)
            .filter(|&y| {
                (360..440)
                    .filter(|&x| {
                        let [red, green, blue] = image.get_pixel(x, y).0;
                        red < 140
                            && blue > red.saturating_add(20)
                            && blue > green.saturating_add(10)
                    })
                    .count()
                    > 24
            })
            .collect::<Vec<_>>();
        assert!(!rows.is_empty(), "primary control must be visible");
        rows
    }

    /// Count rendered content in the reserved fingerprint feedback area.
    fn feedback_pixels(image: &image::RgbImage) -> usize {
        let background = image.get_pixel(250, 410);
        (410..506)
            .flat_map(|y| (250..550).map(move |x| (x, y)))
            .filter(|&(x, y)| image.get_pixel(x, y) != background)
            .count()
    }

    async fn password_only(events: &Events) {
        let (ui, cache) = demo_ui(DemoScenario::Fingerprint, false);
        // Let startup trackers settle before explicitly submitting the form.
        pump_until(events, || !ui.get_auth_active()).await;
        assert!(!ui.get_fingerprint_mode());
        ui.invoke_auth_begin("alice".into(), "demo-only".into(), true);
        pump_until(events, || !ui.get_auth_active()).await;
        assert!(ui.get_auth_status().contains("successful"));
        assert!(cache.lock().unwrap().last_user.is_none());
    }

    async fn explicit_switch_keeps_conversation(events: &Events) -> Vec<u32> {
        let (ui, cache) = demo_ui(DemoScenario::Fallback, true);
        pump_until(events, || ui.get_auth_notice().contains("Demo: waiting"))
            .await;
        assert!(ui.get_fingerprint_mode());
        assert!(!ui.get_auth_prompt_ready());
        assert!(ui.get_auth_status().is_empty());
        let fingerprint = preview(&ui, "fingerprint.png");
        assert_eq!(feedback_pixels(&fingerprint), 0);
        let hint_generation = ui.get_auth_generation();
        ui.invoke_fingerprint_hint();
        assert_eq!(ui.get_auth_generation(), hint_generation);
        assert!(
            ui.get_auth_status()
                .contains("Touch the fingerprint reader")
        );
        let hint = preview(&ui, "fingerprint-hint.png");
        assert!(feedback_pixels(&hint) > 0);
        assert_eq!(primary_rows(&fingerprint), primary_rows(&hint));
        let generation = ui.get_auth_generation();
        ui.invoke_use_password("alice".into());
        assert!(!ui.get_fingerprint_mode());
        assert!(ui.get_password_requested());
        assert!(!ui.get_is_authenticating());
        assert_eq!(ui.get_auth_generation(), generation);
        pump_until(events, || ui.get_auth_prompt_ready()).await;
        assert!(ui.get_auth_active(), "no secret may have been sent yet");
        assert!(!ui.get_fingerprint_mode());
        assert!(ui.get_password_stage());
        ui.invoke_use_fingerprint("alice".into());
        assert!(
            !ui.get_fingerprint_mode(),
            "fallback must not restart scans"
        );
        assert_eq!(ui.get_auth_generation(), generation);
        let password = preview(&ui, "password.png");
        assert_eq!(primary_rows(&fingerprint), primary_rows(&password));
        ui.invoke_auth_begin("alice".into(), "demo-only".into(), true);
        pump_until(events, || !ui.get_auth_active()).await;
        assert!(ui.get_auth_status().contains("successful"));
        assert!(cache.lock().unwrap().last_user.is_none());
        primary_rows(&fingerprint)
    }

    /// The default successful-fingerprint demo must also allow password preview.
    async fn default_demo_password_switch(events: &Events, finish_scan: bool) {
        let (ui, cache) = demo_ui(DemoScenario::Fallback, true);
        if finish_scan {
            pump_until(events, || ui.get_auth_prompt_ready()).await;
            ui.invoke_auth_begin("alice".into(), "demo-only".into(), true);
            pump_until(events, || !ui.get_auth_active()).await;
        } else {
            pump_until(events, || {
                ui.get_auth_notice().contains("Demo: waiting")
            })
            .await;
        }
        ui.invoke_use_password("alice".into());
        pump_until(events, || ui.get_auth_prompt_ready()).await;
        assert!(ui.get_auth_active());
        assert!(!ui.get_fingerprint_mode());
        assert!(!ui.get_password_requested());
        assert!(!ui.get_is_authenticating());
        preview(
            &ui,
            if finish_scan {
                "demo-password-after-success.png"
            } else {
                "demo-password-during-scan.png"
            },
        );
        ui.invoke_auth_begin("alice".into(), "demo-only".into(), true);
        pump_until(events, || !ui.get_auth_active()).await;
        assert!(!ui.get_password_requested());
        assert!(ui.get_auth_error().is_empty());
        assert!(ui.get_auth_status().contains("successful"));
        assert!(cache.lock().unwrap().last_user.is_none());
    }

    async fn mismatches_fall_back_without_restart(
        events: &Events,
        expected_rows: &[u32],
    ) {
        let (ui, _) = demo_ui(DemoScenario::Mismatch, true);
        pump_until(events, || ui.get_auth_notice_error()).await;
        let generation = ui.get_auth_generation();
        assert!(ui.get_fingerprint_mode());
        assert!(!ui.get_auth_prompt_ready());
        assert!(!ui.get_is_authenticating());
        let failure = preview(&ui, "fingerprint-error.png");
        assert!(feedback_pixels(&failure) > 0);
        assert_eq!(primary_rows(&failure), expected_rows);
        pump_until(events, || ui.get_auth_prompt_ready()).await;
        assert_eq!(ui.get_auth_generation(), generation);
        assert!(ui.get_password_stage());
        assert!(!ui.get_fingerprint_mode());
        ui.invoke_cancel_auth();
        pump_until(events, || ui.get_auth_status().contains("cancelled"))
            .await;
    }

    async fn password_rejection(events: &Events, expected_rows: &[u32]) {
        let (ui, _) = demo_ui(DemoScenario::Error, true);
        pump_until(events, || ui.get_auth_prompt_ready()).await;
        ui.invoke_auth_begin("alice".into(), "incorrect-demo".into(), true);
        pump_until(events, || !ui.get_auth_active()).await;
        assert!(!ui.get_auth_error().is_empty());
        assert!(!ui.get_auth_launching());
        assert!(!ui.get_auth_prompt_ready());
        assert!(ui.get_password_stage());
        assert!(!ui.get_fingerprint_mode());
        let error = preview(&ui, "password-error.png");
        let rows = primary_rows(&error);
        assert_eq!(rows, expected_rows);
        settle(&ui, events).await;
        type_text(&ui, "x");
        pump_until(events, || ui.get_auth_error().is_empty()).await;
        assert_eq!(rows, primary_rows(&render(&ui)));
        ui.window()
            .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                text: slint::platform::Key::Escape.into(),
            });
        pump_until(events, || !ui.get_show_login()).await;
    }

    async fn cancellation_and_user_switch(events: &Events) {
        let (ui, cache) = demo_ui(DemoScenario::Fingerprint, true);
        pump_until(events, || ui.get_auth_notice().contains("Demo: waiting"))
            .await;
        let generation = ui.get_auth_generation();
        ui.set_selected_user_index(1);
        pump_until(events, || {
            ui.get_auth_generation() > generation && ui.get_auth_active()
        })
        .await;
        ui.invoke_cancel_auth();
        pump_until(events, || ui.get_auth_status().contains("cancelled"))
            .await;
        assert!(!ui.get_auth_active());
        assert!(!ui.get_auth_launching());
        let generation = ui.get_auth_generation();
        ui.set_auth_error("Simulated stopped attempt".into());
        ui.invoke_fingerprint_hint();
        assert!(ui.get_auth_error().is_empty());
        assert_eq!(ui.get_auth_generation(), generation);
        assert!(!ui.get_auth_active());
        assert!(ui.get_auth_status().contains("Use password to continue"));
        assert_eq!(ui.get_selected_user_index(), 1);
        assert!(cache.lock().unwrap().last_user.is_none());
    }

    /// Retrying a rejected password starts one new conversation, without a click
    /// on the scanner and without reusing the response in the failed conversation.
    async fn sequential_password_retry(events: &Events) {
        let (ui, _) = demo_ui(DemoScenario::Error, true);
        pump_until(events, || ui.get_auth_notice().contains("Demo: waiting"))
            .await;
        ui.invoke_use_password("alice".into());
        ui.invoke_auth_begin("alice".into(), "incorrect-demo".into(), true);
        pump_until(events, || !ui.get_auth_active()).await;
        assert!(ui.get_auth_error().contains("Invalid username or password"));
        let generation = ui.get_auth_generation();
        ui.invoke_auth_begin("alice".into(), "correct-demo".into(), true);
        pump_until(events, || !ui.get_auth_active()).await;
        assert!(ui.get_auth_generation() > generation);
        assert!(ui.get_auth_status().contains("successful"));
        assert!(!ui.get_fingerprint_mode());
    }

    /// A still-open channel belonging to a terminal attempt cannot swallow a retry.
    async fn terminal_attempt_cannot_consume_retry(events: &Events) {
        let (ui, cache) = clock_ui(DemoScenario::Fallback, false);
        pump_until(events, || true).await;
        let (input, mut old_input) = mpsc::unbounded_channel();
        let (cancel, _cancelled) = watch::channel(false);
        let controller = LoginController {
            ui: ui.as_weak(),
            cache,
            gate: Arc::new(tokio::sync::Mutex::new(())),
            active: RefCell::new(Some(Attempt {
                username: "alice".into(),
                input,
                cancel,
                cleanup_error: Arc::new(Mutex::new(None)),
            })),
            backend: Backend::Demo(DemoScenario::Fallback),
            fingerprint: false,
        };
        controller.begin("alice", "correct-demo", true);
        assert!(old_input.try_recv().is_err());
        pump_until(events, || !ui.get_auth_active()).await;
        assert!(ui.get_auth_status().contains("successful"));
    }

    /// Explicit mode never starts a conversation merely by displaying the card.
    async fn explicit_fingerprint_start(events: &Events) {
        let (ui, _) = configured_clock_ui(
            DemoScenario::Fallback,
            &FingerprintConfig {
                enable: true,
                auto_start: false,
            },
        );
        ui.set_show_login(true);
        render(&ui);
        let deadline = Instant::now() + Duration::from_millis(100);
        pump_until(events, || Instant::now() >= deadline).await;
        assert!(ui.get_fingerprint_mode());
        assert!(!ui.get_auth_active());
        assert!(ui.get_auth_notice().is_empty());
        ui.set_selected_user_index(1);
        let deadline = Instant::now() + Duration::from_millis(100);
        pump_until(events, || Instant::now() >= deadline).await;
        assert!(!ui.get_auth_active());
        ui.invoke_use_fingerprint("bob".into());
        pump_until(events, || ui.get_auth_notice().contains("Demo: waiting"))
            .await;
        let generation = ui.get_auth_generation();
        ui.invoke_fingerprint_hint();
        assert_eq!(ui.get_auth_generation(), generation);
        ui.invoke_return_to_clock();
        pump_until(events, || ui.get_auth_status().contains("cancelled"))
            .await;
        ui.set_show_login(true);
        let deadline = Instant::now() + Duration::from_millis(100);
        pump_until(events, || Instant::now() >= deadline).await;
        assert!(!ui.get_auth_active());
        ui.invoke_use_password("bob".into());
        assert!(ui.get_auth_active());
        pump_until(events, || ui.get_auth_prompt_ready()).await;
        ui.invoke_auth_begin("bob".into(), "dummy-password".into(), true);
        pump_until(events, || !ui.get_auth_active()).await;
        assert!(ui.get_auth_status().contains("successful"));
    }

    type Submissions = Rc<RefCell<Vec<(String, String)>>>;

    /// Observe what the form submits without sending the dummy input to a service.
    fn capture_submit(ui: &GreeterWindow) -> Submissions {
        let submissions = Rc::new(RefCell::new(Vec::new()));
        let captured = submissions.clone();
        ui.on_auth_begin(move |user, answer, submit| {
            if submit {
                captured
                    .borrow_mut()
                    .push((user.to_string(), answer.to_string()));
            }
        });
        submissions
    }

    /// Let conditional fields mount and their deferred focus requests complete.
    async fn settle(ui: &GreeterWindow, events: &Events) {
        render(ui);
        let deadline = Instant::now() + Duration::from_millis(50);
        pump_until(events, || Instant::now() >= deadline).await;
        render(ui);
    }

    fn key(ui: &GreeterWindow, text: slint::SharedString) {
        ui.window()
            .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
        ui.window()
            .dispatch_event(WindowEvent::KeyReleased { text });
    }

    fn type_text(ui: &GreeterWindow, text: &str) {
        for character in text.chars() {
            key(ui, character.to_string().into());
        }
    }

    /// Direct password entry and changing accounts need no mouse click in the field.
    async fn direct_password_and_user_change(events: &Events) {
        let (ui, _) = demo_ui(DemoScenario::Fingerprint, false);
        ui.set_show_login(true);
        settle(&ui, events).await;
        let submissions = capture_submit(&ui);
        type_text(&ui, "direct-demo");
        key(&ui, slint::platform::Key::Return.into());
        assert_eq!(
            *submissions.borrow(),
            [("alice".into(), "direct-demo".into())]
        );
        submissions.borrow_mut().clear();
        ui.set_selected_user_index(1);
        settle(&ui, events).await;
        type_text(&ui, "another-demo");
        key(&ui, slint::platform::Key::Return.into());
        assert_eq!(
            *submissions.borrow(),
            [("bob".into(), "another-demo".into())]
        );

        // Opening the account picker must keep its focus despite later UI frames.
        submissions.borrow_mut().clear();
        key(&ui, slint::platform::Key::Backtab.into());
        key(&ui, slint::platform::Key::Return.into());
        settle(&ui, events).await;
        key(&ui, slint::platform::Key::Return.into());
        assert!(submissions.borrow().is_empty());
        ui.hide().unwrap();
    }

    /// The clock wake-up key is consumed, then the recreated field receives focus.
    async fn return_from_clock(events: &Events) {
        let (ui, _) = demo_ui(DemoScenario::Fingerprint, false);
        ui.set_show_login(true);
        settle(&ui, events).await;
        let submissions = capture_submit(&ui);
        key(&ui, slint::platform::Key::Escape.into());
        settle(&ui, events).await;
        assert!(!ui.get_show_login());
        key(&ui, "w".into());
        settle(&ui, events).await;
        assert!(ui.get_show_login());
        type_text(&ui, "after-clock-demo");
        key(&ui, slint::platform::Key::Return.into());
        assert_eq!(
            *submissions.borrow(),
            [("alice".into(), "after-clock-demo".into())],
        );
    }

    /// Both modes start on the clock, without a background login attempt.
    async fn initial_clock(events: &Events, fingerprint: bool) {
        let (ui, _) = clock_ui(DemoScenario::Fingerprint, fingerprint);
        settle(&ui, events).await;
        assert!(!ui.get_show_login());
        assert!(!ui.get_auth_active());
        assert!(ui.get_auth_notice().is_empty());

        key(&ui, "w".into());
        assert!(ui.get_show_login());
        if fingerprint {
            pump_until(events, || {
                ui.get_auth_notice().contains("Demo: waiting")
            })
            .await;
            assert!(ui.get_fingerprint_mode());
            ui.invoke_return_to_clock();
            pump_until(events, || ui.get_auth_status().contains("cancelled"))
                .await;
            assert!(!ui.get_show_login());
            assert!(!ui.get_auth_active());
        } else {
            settle(&ui, events).await;
            let submissions = capture_submit(&ui);
            type_text(&ui, "from-clock-demo");
            key(&ui, slint::platform::Key::Return.into());
            assert_eq!(
                *submissions.borrow(),
                [("alice".into(), "from-clock-demo".into())],
            );
        }
    }

    /// Switching from fingerprint waits for the actual prompt before focusing.
    async fn after_fingerprint(events: &Events) {
        let (ui, _) = demo_ui(DemoScenario::Fallback, true);
        pump_until(events, || ui.get_auth_notice().contains("Demo: waiting"))
            .await;
        ui.invoke_use_password("alice".into());
        pump_until(events, || ui.get_auth_prompt_ready()).await;
        settle(&ui, events).await;
        let submissions = capture_submit(&ui);
        type_text(&ui, "switched-demo");
        key(&ui, slint::platform::Key::Return.into());
        assert_eq!(
            *submissions.borrow(),
            [("alice".into(), "switched-demo".into())],
        );
    }

    /// Typing a manual username must not redirect subsequent characters to password.
    async fn manual_username(events: &Events) {
        let (ui, _) = demo_ui(DemoScenario::Fingerprint, false);
        ui.set_show_login(true);
        settle(&ui, events).await;
        let submissions = capture_submit(&ui);
        // Skip the password visibility control, then activate "Not listed?".
        key(&ui, slint::platform::Key::Tab.into());
        key(&ui, slint::platform::Key::Tab.into());
        key(&ui, " ".into());
        settle(&ui, events).await;
        type_text(&ui, "guest");
        settle(&ui, events).await;
        key(&ui, slint::platform::Key::Return.into());
        pump_until(events, || ui.get_auth_prompt_ready()).await;
        settle(&ui, events).await;
        type_text(&ui, "manual-demo");
        key(&ui, slint::platform::Key::Return.into());
        assert_eq!(
            *submissions.borrow(),
            [("guest".into(), "manual-demo".into())],
        );
    }

    /// Password entry stays usable while the same PAM session waits for a scan.
    async fn password_during_scan(events: &Events) {
        let (ui, _) = demo_ui(DemoScenario::Error, true);
        pump_until(events, || ui.get_auth_notice().contains("Demo: waiting"))
            .await;
        let generation = ui.get_auth_generation();
        ui.invoke_use_password("alice".into());
        settle(&ui, events).await;
        assert!(!ui.get_auth_prompt_ready());
        assert!(!ui.get_is_authenticating());
        type_text(&ui, "correct-demo");
        key(&ui, slint::platform::Key::Return.into());
        assert!(ui.get_is_authenticating());
        // Another activation must not queue a second response.
        ui.invoke_auth_begin("alice".into(), "incorrect-demo".into(), true);
        pump_until(events, || !ui.get_auth_active()).await;
        assert_eq!(ui.get_auth_generation(), generation);
        assert!(ui.get_auth_status().contains("successful"));
        assert!(ui.get_auth_error().is_empty());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn compact_modes_preserve_login_and_cancellation() {
        let (tx, events) = std::sync::mpsc::channel();
        slint::platform::set_platform(Box::new(Headless(tx))).unwrap();
        password_only(&events).await;
        let primary_rows = explicit_switch_keeps_conversation(&events).await;
        default_demo_password_switch(&events, false).await;
        default_demo_password_switch(&events, true).await;
        mismatches_fall_back_without_restart(&events, &primary_rows).await;
        password_rejection(&events, &primary_rows).await;
        cancellation_and_user_switch(&events).await;
        initial_clock(&events, false).await;
        initial_clock(&events, true).await;
        manual_username(&events).await;
        direct_password_and_user_change(&events).await;
        return_from_clock(&events).await;
        after_fingerprint(&events).await;
        password_during_scan(&events).await;
        sequential_password_retry(&events).await;
        terminal_attempt_cannot_consume_retry(&events).await;
        explicit_fingerprint_start(&events).await;
    }
}
