slint::include_modules!();

mod app;
mod cli;
mod config;
mod utils;

use cli::Args;
use config::GreeterConfig;
use slint::{ComponentHandle, Model};
use std::sync::{Arc, Mutex};
use tracing::info;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = GreeterConfig::load(&args.config);

    // Initialize logging first so early errors can be captured
    let _log_guard = utils::logging::init(&config.logging);

    // Initialize translations
    utils::i18n::init();

    let ui = GreeterWindow::new().unwrap();
    let is_dark = config.is_dark_mode();

    if args.demo {
        info!("*** RUNNING IN DEMO MODE ***");
    }

    let cache = Arc::new(Mutex::new(utils::cache::Cache::load(&config)));

    // Initialize modules
    let users_data = app::Auth::init(&ui).await;
    app::Session::init(&ui);
    app::Power::init(&ui, &config.power);
    app::Appearance::init(&ui, &config);
    let _clock_timer = app::Clock::init(&ui);

    // Restore Initial State from Cache (LRU)
    {
        let mut cache_lock = cache.lock().unwrap();
        if let Some(last_user) = cache_lock.last_user.clone() {
            if let Some(pos) = users_data
                .iter()
                .position(|u| u.user_name.as_str() == last_user)
            {
                ui.set_selected_user_index(pos as i32);

                if let Some(last_sess) = cache_lock.get_last_session(&last_user).cloned() {
                    let compositors = ui.get_compositors();
                    for i in 0..compositors.row_count() {
                        if let Some(c) = compositors.row_data(i) {
                            if c.name.as_str() == last_sess {
                                ui.set_selected_compositor_index(i as i32);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    ui.invoke_set_color_scheme(is_dark);

    // Persistence Callbacks
    let cache_ui = cache.clone();
    let users_data_persistence = users_data.clone();
    let ui_weak = ui.as_weak();

    ui.on_user_selected(move |idx| {
        if idx < 0 {
            return;
        }
        if let Some(user) = users_data_persistence.get(idx as usize) {
            let mut cache = cache_ui.lock().unwrap();
            let username = user.user_name.to_string();

            if let Some(last_sess) = cache.get_last_session(&username).cloned() {
                if let Some(ui) = ui_weak.upgrade() {
                    let compositors = ui.get_compositors();
                    for i in 0..compositors.row_count() {
                        if let Some(c) = compositors.row_data(i) {
                            if c.name.as_str() == last_sess {
                                ui.set_selected_compositor_index(i as i32);
                                break;
                            }
                        }
                    }
                }
            }
        }
    });

    app::Login::init(&ui, cache.clone(), args.demo);

    ui.run().unwrap();
}
