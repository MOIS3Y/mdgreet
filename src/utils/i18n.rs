use gettextrs::{LocaleCategory, bind_textdomain_codeset, bindtextdomain, setlocale, textdomain};
use std::path::Path;
use tracing::debug;

/// Initializes the internationalization (i18n) framework.
///
/// This function sets up the translation domain using a reliable
/// lookup order:
/// - `MDGREET_LOCALES_DIR` environment variable (for Nix wrapper).
/// - Development locales directory (for `cargo run`).
/// - Standard system path `/usr/share/locale` (for FHS distros).
pub fn init() {
    setlocale(LocaleCategory::LcAll, "");

    let dev_locales = env!("LOCALES_DIR_DEV");
    let env_locales = std::env::var("MDGREET_LOCALES_DIR").ok();

    let locales_dir = if let Some(path) = env_locales {
        debug!("Using i18n locales from MDGREET_LOCALES_DIR: {}", path);
        path
    } else if Path::new(dev_locales).exists() {
        debug!("Using i18n locales from dev path: {}", dev_locales);
        dev_locales.to_string()
    } else {
        let sys_path = crate::utils::paths::SYSTEM_LOCALE_DIR;
        debug!("Using i18n locales from system path: {}", sys_path);
        sys_path.to_string()
    };

    // Explicitly bind the domain to our resolved path.
    // This tells C gettext exactly where to look for .mo files.
    let _ = bindtextdomain("mdgreet", &locales_dir);

    // Force UTF-8 encoding (critical for modern Linux and Slint)
    let _ = bind_textdomain_codeset("mdgreet", "UTF-8");

    // Set the global domain. This is strictly required so that
    // Rust's `gettext("...")` calls know which domain to use.
    let _ = textdomain("mdgreet");

    // Initialize Slint. It redundantly calls bindtextdomain again,
    // but with the exact same path, ensuring perfect synchronization.
    slint::init_translations!(&locales_dir);
}
