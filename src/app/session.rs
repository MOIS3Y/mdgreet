use crate::GreeterWindow;
use crate::utils::systems;
use slint::{ComponentHandle, Image, SharedString, VecModel};
use std::path::Path;
use std::rc::Rc;

pub struct Session;

impl Session {
    pub fn init(ui: &GreeterWindow) {
        let system_sessions = systems::get_sessions();
        let compositors = if system_sessions.is_empty() {
            println!("systems: WARNING: No sessions discovered in the system!");
            Vec::new()
        } else {
            println!(
                "systems: discovered {} real sessions:",
                system_sessions.len()
            );
            for s in &system_sessions {
                println!("  - {} ({})", s.name, s.exec);
            }
            Self::convert_system_sessions(system_sessions)
        };

        let (comp_model, comp_menu_model, comp_icon) = Self::prepare_ui_models(&compositors);
        ui.set_compositors(comp_model.into());
        ui.set_compositor_menu_items(comp_menu_model.into());
        ui.set_selected_compositor_index(0);
        ui.set_composer_icon(comp_icon);

        let ui_handle = ui.as_weak();
        ui.on_compositor_selected(move |idx| {
            if let Some(_ui) = ui_handle.upgrade() {
                println!("Compositor selected at index {}", idx);
            }
        });
    }

    fn convert_system_sessions(
        system_sessions: Vec<systems::SystemSession>,
    ) -> Vec<crate::Compositor> {
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
    ) -> (
        Rc<VecModel<crate::Compositor>>,
        Rc<VecModel<crate::MenuItem>>,
        Image,
    ) {
        let comp_icon = Image::load_from_path(Path::new("ui/icons/auto_awesome_mosaic.svg"))
            .unwrap_or_default();

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
            comp_icon,
        )
    }
}
