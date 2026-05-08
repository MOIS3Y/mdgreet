use std::path::PathBuf;

/// The name of the greeter used for configuration and cache paths.
pub const GREETER_NAME: &str = "mdgreet";

/// Path to the system-wide login definitions file.
pub const LOGIN_DEFS: &str = "/etc/login.defs";

/// Path to the greetd environments configuration file.
pub const GREETD_ENVIRONMENTS: &str = "/etc/greetd/environments";

/// The standard system path for localized messages (translations).
pub const SYSTEM_LOCALE_DIR: &str = "/usr/share/locale";

/// Returns the default path to the configuration file.
pub fn default_config_path() -> PathBuf {
    PathBuf::from("/etc/greetd").join(format!("{}.toml", GREETER_NAME))
}

/// Returns the default directory for cache files.
pub fn default_cache_dir() -> PathBuf {
    PathBuf::from(format!("/var/cache/{}", GREETER_NAME))
}

/// Returns the default path for the log file.
pub fn default_log_path() -> PathBuf {
    PathBuf::from(format!("/var/log/{}/{}.log", GREETER_NAME, GREETER_NAME))
}
