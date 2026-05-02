use slint::{Image, SharedString, VecModel};
use std::path::Path;
use std::rc::Rc;

pub struct Session;

impl Session {
    pub fn get_mock_compositors() -> Vec<crate::Compositor> {
        vec![
            crate::Compositor {
                name: SharedString::from("Niri"),
                exec: SharedString::from("niri"),
            },
            crate::Compositor {
                name: SharedString::from("Hyprland"),
                exec: SharedString::from("Hyprland"),
            },
            crate::Compositor {
                name: SharedString::from("Sway"),
                exec: SharedString::from("sway"),
            },
        ]
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
        let comp_menu_items: Vec<crate::MenuItem> = compositors
            .iter()
            .map(|c| crate::MenuItem {
                text: c.name.clone(),
                icon: comp_icon.clone(),
                trailing_text: SharedString::default(),
                enabled: true,
            })
            .collect();

        (
            Rc::new(VecModel::from(compositors.to_vec())),
            Rc::new(VecModel::from(comp_menu_items)),
            comp_icon,
        )
    }
}
