use crate::utils::systems::SystemSession;
use slint::{Image, SharedString, VecModel};
use std::path::Path;
use std::rc::Rc;

pub struct Session;

impl Session {
    pub fn convert_system_sessions(system_sessions: Vec<SystemSession>) -> Vec<crate::Compositor> {
        system_sessions
            .into_iter()
            .map(|s| crate::Compositor {
                name: SharedString::from(s.name),
                exec: SharedString::from(s.exec),
            })
            .collect()
    }

    pub fn prepare_ui_models(
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

        // For the compositors list itself, if empty, we provide one dummy item to avoid crashes in Slint property bindings
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
