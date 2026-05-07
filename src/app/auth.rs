use crate::GreeterWindow;
use crate::utils::system::SystemUser;
use slint::{Image, SharedString, VecModel};
use std::rc::Rc;
use tracing::{error, info, warn};

/// Structural mirror for Rust backend data representing a user.
#[derive(Clone)]
pub struct UserData {
    /// The system login username.
    pub user_name: SharedString,
    /// The user's display name or real name.
    pub real_name: SharedString,
    /// The user's password (handled dynamically by greetd, usually empty).
    #[allow(dead_code)]
    pub password: SharedString,
}

/// Handles user discovery and UI data preparation for authentication.
pub struct Auth;

impl Auth {
    /// Discovers system users and initializes the Slint UI properties.
    ///
    /// Returns the gathered user data to be used by the application logic.
    pub async fn init(ui: &GreeterWindow, _demo: bool) -> Vec<UserData> {
        let system_users = SystemUser::all().await.unwrap_or_else(|e| {
            error!("AccountsService not available ({:?})", e);
            Vec::new()
        });

        let users_data = Self::convert_system_users(system_users);

        if users_data.is_empty() {
            warn!("No users discovered in the system!");
        } else {
            info!("Loaded {} users", users_data.len());
        }

        let person_icon = ui.get_default_user_icon();
        let (users_model, user_menu_model) = Self::prepare_ui_models(&users_data, &person_icon);

        ui.set_users(users_model.into());
        ui.set_user_menu_items(user_menu_model.into());
        ui.set_selected_user_index(-1);

        users_data
    }

    /// Converts raw system users into the `UserData` format.
    fn convert_system_users(system_users: Vec<SystemUser>) -> Vec<UserData> {
        system_users
            .into_iter()
            .map(|u| UserData {
                user_name: SharedString::from(u.user_name),
                real_name: SharedString::from(u.real_name),
                password: SharedString::from(""), // Greetd handles this
            })
            .collect()
    }

    /// Prepares VecModels for the UI from a list of user data.
    ///
    /// Generates initials for avatars and builds the menu items.
    pub fn prepare_ui_models(
        users_data: &[UserData],
        person_icon: &Image,
    ) -> (Rc<VecModel<crate::User>>, Rc<VecModel<crate::MenuItem>>) {
        let users_vec: Vec<crate::User> = users_data
            .iter()
            .map(|u| {
                let display_name = if u.real_name.is_empty() {
                    u.user_name.clone()
                } else {
                    u.real_name.clone()
                };

                let initials = display_name
                    .split_whitespace()
                    .map(|s: &str| s.chars().next().unwrap_or(' '))
                    .collect::<String>()
                    .to_uppercase();

                let final_initials =
                    if (initials.len() == 1 || initials.is_empty()) && display_name.len() > 1 {
                        display_name[..2.min(display_name.len())].to_uppercase()
                    } else {
                        initials
                    };

                crate::User {
                    user_name: u.user_name.clone(),
                    real_name: display_name,
                    initials: SharedString::from(final_initials),
                    avatar: Image::default(),
                }
            })
            .collect();

        let menu_items: Vec<crate::MenuItem> = users_vec
            .iter()
            .map(|u| crate::MenuItem {
                text: u.real_name.clone(),
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
