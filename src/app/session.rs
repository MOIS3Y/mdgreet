use crate::GreeterWindow;
use crate::utils::system::SystemSession;
use gettextrs::gettext;
use slint::{ComponentHandle, Image, SharedString, VecModel};
use std::rc::Rc;
use tracing::{info, warn};

/// Handles the discovery and UI binding of available desktop sessions.
pub struct Session;

impl Session {
    /// Discovers system sessions and initializes the Slint UI properties.
    ///
    /// This populates the session selector with available Wayland/X11
    /// compositors and registers the callback for session selection.
    pub fn init(ui: &GreeterWindow) {
        let system_sessions = SystemSession::all();

        if system_sessions.is_empty() {
            warn!("No sessions discovered in the system!");
        } else {
            info!("Discovered {} real sessions:", system_sessions.len());
            for s in &system_sessions {
                info!("  - {} ({})", s.name, s.exec);
            }
        }

        let compositors = Self::convert_system_sessions(system_sessions);
        let comp_icon = ui.get_default_session_icon();
        let (comp_model, comp_menu_model) = Self::prepare_ui_models(&compositors, &comp_icon);

        ui.set_compositors(comp_model.into());
        ui.set_compositor_menu_items(comp_menu_model.into());
        ui.set_selected_compositor_index(0);
        ui.set_composer_icon(comp_icon);

        let ui_handle = ui.as_weak();
        ui.on_compositor_selected(move |idx| {
            if let Some(_ui) = ui_handle.upgrade() {
                info!("Compositor selected at index {}", idx);
            }
        });
    }

    /// Converts raw system sessions into Slint-compatible Compositor structs.
    fn convert_system_sessions(system_sessions: Vec<SystemSession>) -> Vec<crate::Compositor> {
        system_sessions
            .into_iter()
            .map(|s| crate::Compositor {
                name: SharedString::from(s.name),
                exec: SharedString::from(s.exec),
            })
            .collect()
    }

    /// Prepares VecModels for the UI from a list of compositors.
    ///
    /// Generates both the data model and the visual menu items model.
    fn prepare_ui_models(
        compositors: &[crate::Compositor],
        comp_icon: &Image,
    ) -> (
        Rc<VecModel<crate::Compositor>>,
        Rc<VecModel<crate::MenuItem>>,
    ) {
        let menu_items: Vec<crate::MenuItem> = if compositors.is_empty() {
            vec![crate::MenuItem {
                text: SharedString::from(gettext("No sessions found")),
                icon: comp_icon.clone(),
                trailing_text: SharedString::default(),
                enabled: false,
            }]
        } else {
            compositors
                .iter()
                .map(|c| crate::MenuItem {
                    text: c.name.clone(),
                    icon: comp_icon.clone(),
                    trailing_text: SharedString::default(),
                    enabled: true,
                })
                .collect()
        };

        let comps_vec = if compositors.is_empty() {
            vec![crate::Compositor {
                name: SharedString::from(gettext("None")),
                exec: SharedString::from(""),
            }]
        } else {
            compositors.to_vec()
        };

        (
            Rc::new(VecModel::from(comps_vec)),
            Rc::new(VecModel::from(menu_items)),
        )
    }
}
