use const_format::concatcp;

/// Get an environment variable at compile time, else return a default.
macro_rules! env_or {
    ($name:expr, $default:expr) => {
        if let Some(value) = option_env!($name) {
            value
        } else {
            $default
        }
    };
}

/// The name for this greeter
pub const GREETER_NAME: &str = "mdgreet";

/// The greetd config directory
const GREETD_CONFIG_DIR: &str = env_or!("GREETD_CONFIG_DIR", "/etc/greetd");

/// Default path to the config file
pub const CONFIG_PATH: &str = concatcp!(GREETD_CONFIG_DIR, "/", GREETER_NAME, ".toml");

/// Default theme name
pub const DEFAULT_THEME: &str = env_or!("MDGREET_DEFAULT_THEME", "purple");

/// Default color mode
pub const DEFAULT_MODE: &str = env_or!("MDGREET_DEFAULT_MODE", "dark");

/// Cache directory for processed images
pub const CACHE_DIR: &str = "/var/cache/mdgreet";

/// Default background image path
pub const DEFAULT_BACKGROUND: &str = "ui/images/background.png";

/// Default power commands
pub const DEFAULT_CMD_SHUTDOWN: &str = "systemctl poweroff";
pub const DEFAULT_CMD_REBOOT: &str = "systemctl reboot";
pub const DEFAULT_CMD_SLEEP: &str = "systemctl suspend";
pub const DEFAULT_CMD_HIBERNATE: &str = "systemctl hibernate";
