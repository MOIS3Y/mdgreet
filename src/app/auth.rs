use crate::GreeterWindow;
use crate::utils::systems::{self, SystemUser};
use slint::{Image, SharedString, VecModel};
use std::path::Path;
use std::rc::Rc;

// Structural mirror for Rust backend data
#[derive(Clone)]
pub struct UserData {
    pub login: SharedString,
    pub pretty_name: SharedString,
    pub password: SharedString,
}

pub struct Auth;

impl Auth {
    pub async fn init(ui: &GreeterWindow) -> Vec<UserData> {
        let system_users = systems::get_users().await.unwrap_or_else(|e| {
            eprintln!("systems: failed to fetch real users: {}", e);
            Vec::new()
        });

        let users_data = if system_users.is_empty() {
            println!("systems: falling back to mock users");
            Self::get_mock_users()
        } else {
            println!(
                "systems: loaded {} users from AccountsService",
                system_users.len()
            );
            Self::convert_system_users(system_users)
        };

        let (users_model, user_menu_model) = Self::prepare_ui_models(&users_data);
        ui.set_users(users_model.into());
        ui.set_user_menu_items(user_menu_model.into());
        ui.set_selected_user_index(-1);

        users_data
    }

    fn get_mock_users() -> Vec<UserData> {
        vec![
            UserData {
                login: SharedString::from("stepan"),
                pretty_name: SharedString::from("Stepan Yankevych"),
                password: SharedString::from("1234"),
            },
            UserData {
                login: SharedString::from("guest"),
                pretty_name: SharedString::from("Guest User"),
                password: SharedString::from(""),
            },
            UserData {
                login: SharedString::from("linux_pro"),
                pretty_name: SharedString::from(""),
                password: SharedString::from("linux"),
            },
            UserData {
                login: SharedString::from("jdoe"),
                pretty_name: SharedString::from("John Doe"),
                password: SharedString::from("admin"),
            },
        ]
    }

    fn convert_system_users(system_users: Vec<SystemUser>) -> Vec<UserData> {
        system_users
            .into_iter()
            .map(|u| UserData {
                login: SharedString::from(u.login),
                pretty_name: SharedString::from(u.pretty_name),
                password: SharedString::from(""),
            })
            .collect()
    }

    fn prepare_ui_models(
        users_data: &[UserData],
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
