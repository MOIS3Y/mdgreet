slint::include_modules!();

mod cli;
mod config;
mod theme;
mod utils;

use chrono::Local;
use cli::Args;
use config::GreeterConfig;
use slint::{ComponentHandle, Image, SharedString, Timer, TimerMode, VecModel};
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

fn main() {
    let args = Args::parse();
    let ui = GreeterWindow::new().unwrap();

    let config = GreeterConfig::load(&args.config);
    let is_dark = config.is_dark_mode();

    // Handle background images
    let bg_config = &config.background;
    let path_str = bg_config
        .path
        .as_deref()
        .unwrap_or("ui/images/background.png");
    let blur = bg_config.blur.unwrap_or(10.0);

    if let Ok(orig_img) = Image::load_from_path(Path::new(path_str)) {
        ui.set_background_original(orig_img);
    }

    match utils::image::prepare_background(path_str, blur) {
        Ok(cached_path) => {
            if let Ok(blur_img) = Image::load_from_path(&cached_path) {
                ui.set_background_blurred(blur_img);
            }
        }
        Err(e) => eprintln!("Failed to prepare blurred background: {}", e),
    }

    // Mock Users data
    let users_data = vec![
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
    ];

    // Prepare users for UI with fallback logic
    let users_vec: Vec<User> = users_data
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

            User {
                login: u.login.clone(),
                pretty_name: display_name,
                initials: SharedString::from(final_initials),
                avatar: Image::default(),
            }
        })
        .collect();

    let person_icon = Image::load_from_path(Path::new("ui/icons/person.svg")).unwrap_or_default();
    let user_menu_items: Vec<MenuItem> = users_vec
        .iter()
        .map(|u| MenuItem {
            text: u.pretty_name.clone(),
            icon: person_icon.clone(),
            trailing_text: SharedString::default(),
            enabled: true,
        })
        .collect();

    // Prepare Compositors (Mocks)
    let compositors_vec = vec![
        Compositor {
            name: SharedString::from("Niri"),
            exec: SharedString::from("niri"),
        },
        Compositor {
            name: SharedString::from("Hyprland"),
            exec: SharedString::from("Hyprland"),
        },
        Compositor {
            name: SharedString::from("Sway"),
            exec: SharedString::from("sway"),
        },
    ];

    let comp_icon =
        Image::load_from_path(Path::new("ui/icons/auto_awesome_mosaic.svg")).unwrap_or_default();
    let comp_menu_items: Vec<MenuItem> = compositors_vec
        .iter()
        .map(|c| MenuItem {
            text: c.name.clone(),
            icon: comp_icon.clone(),
            trailing_text: SharedString::default(),
            enabled: true,
        })
        .collect();

    // Initial UI State
    ui.set_users(Rc::new(VecModel::from(users_vec)).into());
    ui.set_user_menu_items(Rc::new(VecModel::from(user_menu_items)).into());
    ui.set_selected_user_index(-1);

    ui.set_compositors(Rc::new(VecModel::from(compositors_vec)).into());
    ui.set_compositor_menu_items(Rc::new(VecModel::from(comp_menu_items)).into());
    ui.set_selected_compositor_index(0);
    ui.set_composer_icon(comp_icon.clone());

    load_and_apply_theme(&ui, &config.theme.name);
    ui.invoke_set_color_scheme(is_dark);

    // Clock and Date timer
    let ui_weak = ui.as_weak();
    let update_time = move || {
        if let Some(ui) = ui_weak.upgrade() {
            let now = Local::now();
            ui.set_hours(SharedString::from(now.format("%H").to_string()));
            ui.set_minutes(SharedString::from(now.format("%M").to_string()));
            ui.set_date(SharedString::from(now.format("%A, %B %-d").to_string()));
        }
    };

    update_time();
    let timer = Timer::default();
    timer.start(TimerMode::Repeated, Duration::from_secs(1), update_time);

    // Login Callback with Validation
    let ui_login = ui.as_weak();
    ui.on_login(move |username_or_login, password| {
        let ui = ui_login.unwrap();

        if username_or_login.is_empty() {
            ui.set_auth_error(SharedString::from("Please select a user"));
            return;
        }

        let user_data = users_data
            .iter()
            .find(|u| u.login == username_or_login || u.pretty_name == username_or_login);

        match user_data {
            Some(data) => {
                if data.password == password {
                    println!("Login successful for '{}'!", data.login);
                    slint::quit_event_loop().unwrap();
                } else {
                    println!("Login failed for '{}': invalid password", data.login);
                    ui.set_auth_error(SharedString::from("Invalid password"));
                }
            }
            None => {
                if password == "greet" {
                    println!("Manual login successful for '{}'!", username_or_login);
                    slint::quit_event_loop().unwrap();
                } else {
                    ui.set_auth_error(SharedString::from("User not found or invalid password"));
                }
            }
        }
    });

    ui.on_compositor_selected(|idx| {
        println!("Compositor selected at index {}", idx);
    });

    // Power Callbacks
    let power_config = &config.power;
    let shutdown_cmd = power_config
        .shutdown
        .clone()
        .unwrap_or_else(|| "systemctl poweroff".to_string());
    ui.on_shutdown(move || println!("Power Action: Shutdown with command '{}'", shutdown_cmd));

    let reboot_cmd = power_config
        .reboot
        .clone()
        .unwrap_or_else(|| "systemctl reboot".to_string());
    ui.on_reboot(move || println!("Power Action: Reboot with command '{}'", reboot_cmd));

    let sleep_cmd = power_config
        .sleep
        .clone()
        .unwrap_or_else(|| "systemctl suspend".to_string());
    ui.on_sleep(move || println!("Power Action: Sleep with command '{}'", sleep_cmd));

    let hibernate_cmd = power_config
        .hibernate
        .clone()
        .unwrap_or_else(|| "systemctl hibernate".to_string());
    ui.on_hibernate(move || println!("Power Action: Hibernate with command '{}'", hibernate_cmd));

    ui.run().unwrap();
}

fn load_and_apply_theme(ui: &GreeterWindow, theme_name: &str) {
    let theme = if theme_name == "custom" {
        let config_dir = std::env::var("MDGREET_CONFIG_DIR").unwrap_or_else(|_| ".".to_string());
        let theme_path = format!("{}/material-theme.json", config_dir);

        theme::load_custom_theme(&theme_path).unwrap_or_else(|e| {
            eprintln!("theme: failed to load custom: {}", e);
            eprintln!("theme: falling back to purple");
            theme::load_builtin_theme("purple")
                .expect("theme: failed to load fallback purple theme")
        })
    } else {
        theme::load_builtin_theme(theme_name).unwrap_or_else(|| {
            eprintln!(
                "theme: unknown theme '{}', falling back to purple",
                theme_name
            );
            theme::load_builtin_theme("purple")
                .expect("theme: failed to load fallback purple theme")
        })
    };

    theme::apply_theme(ui, &theme);
}
