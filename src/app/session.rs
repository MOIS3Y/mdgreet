use crate::GreeterWindow;
use crate::utils::system::SystemSession;
use slint::{ComponentHandle, Image, SharedString, VecModel};
use std::rc::Rc;
use tracing::{info, warn};

pub struct Session;

impl Session {
    pub fn init(ui: &GreeterWindow, _demo: bool) {
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

    fn convert_system_sessions(system_sessions: Vec<SystemSession>) -> Vec<crate::Compositor> {
        system_sessions
            .into_iter()
            .map(|s| crate::Compositor {
                name: SharedString::from(s.name),
                exec: SharedString::from(s.exec),
            })
            .collect()
    }

    fn prepare_ui_models(
        compositors: &[crate::Compositor],
        comp_icon: &Image,
    ) -> (
        Rc<VecModel<crate::Compositor>>,
        Rc<VecModel<crate::MenuItem>>,
    ) {
        let menu_items: Vec<crate::MenuItem> = if compositors.is_empty() {
            vec![crate::MenuItem {
                text: SharedString::from("No sessions found"),
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
                name: SharedString::from("None"),
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
