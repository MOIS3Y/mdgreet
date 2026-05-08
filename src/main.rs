slint::include_modules!();

mod app;
mod cli;
mod config;
mod utils;

use cli::Args;
use config::GreeterConfig;
use slint::ComponentHandle;
use std::sync::{Arc, Mutex};
use tracing::info;

/// The main entry point for the `mdgreet` greeter.
///
/// This function acts purely as an orchestrator. It handles:
/// - CLI argument parsing and configuration loading.
/// - Initialization of foundational services (logging, i18n).
/// - Creation of the shared UI instance and persistence cache.
/// - Delegation of business logic and UI bindings to dedicated `app::*`
///   modules.
#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = GreeterConfig::load(&args.config);

    // Initialize logging first so early errors can be captured
    let _log_guard = utils::logging::init(&config.logging);

    if args.demo {
        info!("*** RUNNING IN DEMO MODE ***");
    }

    // Initialize translations
    utils::i18n::init();

    // Create the main Slint UI instance
    let ui = GreeterWindow::new().unwrap();

    // Initialize the shared state cache
    let cache = Arc::new(Mutex::new(utils::cache::Cache::load(&config)));

    // Initialize UI and system integration modules
    let users_data = app::Auth::init(&ui).await;
    app::Session::init(&ui);
    app::Power::init(&ui, &config.power);
    app::Appearance::init(&ui, &config);
    let _clock_timer = app::Clock::init(&ui);

    ui.invoke_set_color_scheme(config.is_dark_mode());

    // Initialize State and Login orchestrators
    app::State::init(&ui, cache.clone(), &users_data);
    app::Login::init(&ui, cache.clone(), args.demo);

    // Start the Slint event loop
    ui.run().unwrap();
}
