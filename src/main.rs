slint::include_modules!();

mod config;
mod constants;
mod image_utils;
mod theme;

use chrono::Local;
use clap::Parser;
use config::GreeterConfig;
use slint::{ComponentHandle, Image, SharedString, Timer, TimerMode, VecModel};
use std::rc::Rc;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "mdgreet", about = "Material Design greeter")]
struct Args {
    /// Path to configuration file
    #[arg(short, long)]
    config: Option<String>,
}

fn main() {
    let args = Args::parse();
    let ui = GreeterWindow::new().unwrap();

    let config = GreeterConfig::load_or_default(args.config.as_deref());
    let is_dark = config.is_dark_mode();

    // Handle background images
    if let Some(bg_config) = &config.background {
        let path_str = bg_config
            .path
            .as_deref()
            .unwrap_or(constants::DEFAULT_BACKGROUND);
        let blur = bg_config.blur.unwrap_or(10.0);

        if let Ok(orig_img) = Image::load_from_path(std::path::Path::new(path_str)) {
            ui.set_background_original(orig_img);
        }

        match image_utils::prepare_background(path_str, blur) {
            Ok(cached_path) => {
                if let Ok(blur_img) = Image::load_from_path(&cached_path) {
                    ui.set_background_blurred(blur_img);
                }
            }
            Err(e) => eprintln!("Failed to prepare blurred background: {}", e),
        }
    }

    // Prepare Users
    let users_vec = vec![
        User {
            name: SharedString::from("Stepan Yankevych"),
            initials: SharedString::from("SY"),
            avatar: Image::default(),
        },
        User {
            name: SharedString::from("Guest User"),
            initials: SharedString::from("GU"),
            avatar: Image::default(),
        },
    ];

    // Load person icon for menu
    let person_icon =
        Image::load_from_path(std::path::Path::new("ui/icons/person.svg")).unwrap_or_default();

    // Create menu items from users
    let menu_items: Vec<MenuItem> = users_vec
        .iter()
        .map(|u| MenuItem {
            text: u.name.clone(),
            icon: person_icon.clone(),
            trailing_text: SharedString::default(),
            enabled: true,
        })
        .collect();

    // Set models
    ui.set_users(Rc::new(VecModel::from(users_vec)).into());
    ui.set_user_menu_items(Rc::new(VecModel::from(menu_items)).into());

    // Mock Composer Icon
    if let Ok(icon) = Image::load_from_path(std::path::Path::new("ui/icons/niri.svg")) {
        ui.set_composer_icon(icon);
    }

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

    // Login Callback
    ui.on_login(|username, password| {
        println!(
            "Login attempt for user '{}': password length {}",
            username,
            password.len()
        );
    });

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
