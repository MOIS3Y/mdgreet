use crate::GreeterWindow;
use crate::app::auth::UserData;
use crate::utils::cache::Cache;
use slint::{ComponentHandle, Model};
use std::sync::{Arc, Mutex};

/// Orchestrates the synchronization between the persistent cache and the UI state.
pub struct State;

impl State {
    /// Restores the initial UI state from the cache and registers state-syncing callbacks.
    ///
    /// This method performs two main tasks:
    /// 1. It reads the last selected user and their preferred compositor from the cache
    ///    and updates the UI dropdowns to match.
    /// 2. It registers the `on_user_selected` callback, which dynamically updates the
    ///    compositor dropdown whenever the user selects a different account.
    pub fn init(ui: &GreeterWindow, cache: Arc<Mutex<Cache>>, users_data: &[UserData]) {
        // 1. Restore Initial State from Cache (LRU)
        {
            let mut cache_lock = cache.lock().unwrap();
            if let Some(last_user) = cache_lock.last_user.clone() {
                if let Some(pos) = users_data.iter().position(|u| u.user_name == last_user) {
                    ui.set_selected_user_index(pos as i32);

                    if let Some(last_sess) = cache_lock.get_last_session(&last_user) {
                        Self::set_compositor_by_name(ui, last_sess);
                    }
                }
            }
        }

        // 2. Persistence Callbacks
        let cache_ui = cache.clone();
        let users_data_persistence = users_data.to_vec();
        let ui_weak = ui.as_weak();

        ui.on_user_selected(move |idx| {
            let Ok(idx) = usize::try_from(idx) else {
                return;
            };

            if let Some(user) = users_data_persistence.get(idx) {
                let mut cache_lock = cache_ui.lock().unwrap();
                if let Some(last_sess) = cache_lock.get_last_session(&user.user_name) {
                    if let Some(ui) = ui_weak.upgrade() {
                        Self::set_compositor_by_name(&ui, last_sess);
                    }
                }
            }
        });
    }

    /// Selects the compositor in the UI dropdown that matches the given name.
    fn set_compositor_by_name(ui: &GreeterWindow, target_session: &str) {
        let compositors = ui.get_compositors();
        if let Some(index) = (0..compositors.row_count()).find(|&i| {
            compositors
                .row_data(i)
                .is_some_and(|c| c.name == target_session)
        }) {
            ui.set_selected_compositor_index(index as i32);
        }
    }
}
