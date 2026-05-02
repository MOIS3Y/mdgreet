use crate::utils::systems::SystemUser;
use slint::{Image, SharedString, VecModel};
use std::path::Path;
use std::rc::Rc;

pub struct Auth;

impl Auth {
    pub fn get_mock_users() -> Vec<crate::UserData> {
        vec![
            crate::UserData {
                login: SharedString::from("stepan"),
                pretty_name: SharedString::from("Stepan Yankevych"),
                password: SharedString::from("1234"),
            },
            crate::UserData {
                login: SharedString::from("guest"),
                pretty_name: SharedString::from("Guest User"),
                password: SharedString::from(""),
            },
            crate::UserData {
                login: SharedString::from("linux_pro"),
                pretty_name: SharedString::from(""),
                password: SharedString::from("linux"),
            },
            crate::UserData {
                login: SharedString::from("jdoe"),
                pretty_name: SharedString::from("John Doe"),
                password: SharedString::from("admin"),
            },
        ]
    }

    pub fn convert_system_users(system_users: Vec<SystemUser>) -> Vec<crate::UserData> {
        system_users
            .into_iter()
            .map(|u| crate::UserData {
                login: SharedString::from(u.login),
                pretty_name: SharedString::from(u.pretty_name),
                password: SharedString::from(""), // Real password validation later
            })
            .collect()
    }

    pub fn prepare_ui_models(
        users_data: &[crate::UserData],
    ) -> (Rc<VecModel<crate::User>>, Rc<VecModel<crate::MenuItem>>) {
        let users_vec: Vec<crate::User> = users_data
            .iter()
            .map(|u| {
                let display_name = if u.pretty_name.is_empty() {
                    u.login.clone()
                } else {
                    u.pretty_name.clone()
                };

                let initials = display_name
                    .split_whitespace()
                    .map(|s: &str| s.chars().next().unwrap_or(' '))
                    .collect::<String>()
                    .to_uppercase();

                let final_initials = if initials.len() == 1 && display_name.len() > 1 {
                    display_name[..2].to_uppercase()
                } else {
                    initials
                };

                crate::User {
                    login: u.login.clone(),
                    pretty_name: display_name,
                    initials: SharedString::from(final_initials),
                    avatar: Image::default(),
                }
            })
            .collect();

        let person_icon =
            Image::load_from_path(Path::new("ui/icons/person.svg")).unwrap_or_default();
        let menu_items: Vec<crate::MenuItem> = users_vec
            .iter()
            .map(|u| crate::MenuItem {
                text: u.pretty_name.clone(),
                icon: person_icon.clone(),
                trailing_text: SharedString::default(),
                enabled: true,
            })
            .collect();

        (
            Rc::new(VecModel::from(users_vec)),
            Rc::new(VecModel::from(menu_items)),
        )
    }
}
