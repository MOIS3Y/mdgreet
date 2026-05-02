slint::include_modules!();

mod config;
mod constants;
mod image_utils;
mod theme;

use chrono::Local;
use clap::Parser;
use config::GreeterConfig;
use serde::{Deserialize, Serialize};
use slint::{ComponentHandle, Image, SharedString, Timer, TimerMode, VecModel};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "mdgreet", about = "Material Design greeter")]
struct Args {
    /// Path to configuration file
    #[arg(short, long)]
    config: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct PersistentState {
    last_compositor: Option<String>,
}

impl PersistentState {
    fn load() -> Self {
        let path = get_state_path();
        if let Ok(content) = fs::read_to_string(path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    fn save(&self) {
        let path = get_state_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_string(self) {
            let _ = fs::write(path, content);
        }
    }
}

fn get_state_path() -> PathBuf {
    let uid = unsafe { libc::getuid() };
    let base = if uid == 0 {
        PathBuf::from(constants::CACHE_DIR)
    } else {
        PathBuf::from(".cache")
    };
    base.join("state.json")
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

        if let Ok(orig_img) = Image::load_from_path(Path::new(path_str)) {
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

    let person_icon = Image::load_from_path(Path::new("ui/icons/person.svg")).unwrap_or_default();
    let user_menu_items: Vec<MenuItem> = users_vec
        .iter()
        .map(|u| MenuItem {
            text: u.name.clone(),
            icon: person_icon.clone(),
            trailing_text: SharedString::default(),
            enabled: true,
        })
        .collect();

    ui.set_users(Rc::new(VecModel::from(users_vec)).into());
    ui.set_user_menu_items(Rc::new(VecModel::from(user_menu_items)).into());

    // Prepare Compositors
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

    // Persistence logic
    let state = Rc::new(std::cell::RefCell::new(PersistentState::load()));
    let default_index = state
        .borrow()
        .last_compositor
        .as_ref()
        .and_then(|last| compositors_vec.iter().position(|c| c.name.as_str() == last))
        .unwrap_or(0);

    ui.set_compositors(Rc::new(VecModel::from(compositors_vec.clone())).into());
    ui.set_compositor_menu_items(Rc::new(VecModel::from(comp_menu_items)).into());
    ui.set_selected_compositor_index(default_index as i32);
    ui.set_composer_icon(comp_icon.clone());

    let state_clone = state.clone();
    let compositors_clone = compositors_vec.clone();
    ui.on_compositor_selected(move |idx| {
        if let Some(comp) = compositors_clone.get(idx as usize) {
            let mut state = state_clone.borrow_mut();
            state.last_compositor = Some(comp.name.to_string());
            state.save();
        }
    });

    // Power Callbacks
    let power_config = config.power.clone().unwrap_or_default();

    let shutdown_cmd = power_config
        .shutdown
        .unwrap_or(constants::DEFAULT_CMD_SHUTDOWN.to_string());
    ui.on_shutdown(move || {
        println!("Power Action: Shutdown with command '{}'", shutdown_cmd);
        // let _ = Command::new("sh").arg("-c").arg(&shutdown_cmd).spawn();
    });

    let reboot_cmd = power_config
        .reboot
        .unwrap_or(constants::DEFAULT_CMD_REBOOT.to_string());
    ui.on_reboot(move || {
        println!("Power Action: Reboot with command '{}'", reboot_cmd);
        // let _ = Command::new("sh").arg("-c").arg(&reboot_cmd).spawn();
    });

    let sleep_cmd = power_config
        .sleep
        .unwrap_or(constants::DEFAULT_CMD_SLEEP.to_string());
    ui.on_sleep(move || {
        println!("Power Action: Sleep with command '{}'", sleep_cmd);
        // let _ = Command::new("sh").arg("-c").arg(&sleep_cmd).spawn();
    });

    let hibernate_cmd = power_config
        .hibernate
        .unwrap_or(constants::DEFAULT_CMD_HIBERNATE.to_string());
    ui.on_hibernate(move || {
        println!("Power Action: Hibernate with command '{}'", hibernate_cmd);
        // let _ = Command::new("sh").arg("-c").arg(&hibernate_cmd).spawn();
    });

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
