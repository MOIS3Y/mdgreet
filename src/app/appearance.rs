use crate::utils;
use anyhow::{Context, Result};
use material_colors::color::Argb;
use material_colors::theme::ThemeBuilder;
use serde::{Deserialize, Serialize};
use slint::{ComponentHandle, Image};
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;
use tracing::{info, warn};

/// Internal representation of a Material Design color scheme.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct MaterialScheme {
    pub primary: String,
    #[serde(rename = "surfaceTint")]
    pub surface_tint: String,
    #[serde(rename = "onPrimary")]
    pub on_primary: String,
    #[serde(rename = "primaryContainer")]
    pub primary_container: String,
    #[serde(rename = "onPrimaryContainer")]
    pub on_primary_container: String,
    pub secondary: String,
    #[serde(rename = "onSecondary")]
    pub on_secondary: String,
    #[serde(rename = "secondaryContainer")]
    pub secondary_container: String,
    #[serde(rename = "onSecondaryContainer")]
    pub on_secondary_container: String,
    pub tertiary: String,
    #[serde(rename = "onTertiary")]
    pub on_tertiary: String,
    #[serde(rename = "tertiaryContainer")]
    pub tertiary_container: String,
    #[serde(rename = "onTertiaryContainer")]
    pub on_tertiary_container: String,
    pub error: String,
    #[serde(rename = "onError")]
    pub on_error: String,
    #[serde(rename = "errorContainer")]
    pub error_container: String,
    #[serde(rename = "onErrorContainer")]
    pub on_error_container: String,
    pub background: String,
    #[serde(rename = "onBackground")]
    pub on_background: String,
    pub surface: String,
    #[serde(rename = "onSurface")]
    pub on_surface: String,
    #[serde(rename = "surfaceVariant")]
    pub surface_variant: String,
    #[serde(rename = "onSurfaceVariant")]
    pub on_surface_variant: String,
    pub outline: String,
    #[serde(rename = "outlineVariant")]
    pub outline_variant: String,
    pub shadow: String,
    pub scrim: String,
    #[serde(rename = "inverseSurface")]
    pub inverse_surface: String,
    #[serde(rename = "inverseOnSurface")]
    pub inverse_on_surface: String,
    #[serde(rename = "inversePrimary")]
    pub inverse_primary: String,
    #[serde(rename = "primaryFixed")]
    pub primary_fixed: String,
    #[serde(rename = "onPrimaryFixed")]
    pub on_primary_fixed: String,
    #[serde(rename = "primaryFixedDim")]
    pub primary_fixed_dim: String,
    #[serde(rename = "onPrimaryFixedVariant")]
    pub on_primary_fixed_variant: String,
    #[serde(rename = "secondaryFixed")]
    pub secondary_fixed: String,
    #[serde(rename = "onSecondaryFixed")]
    pub on_secondary_fixed: String,
    #[serde(rename = "secondaryFixedDim")]
    pub secondary_fixed_dim: String,
    #[serde(rename = "onSecondaryFixedVariant")]
    pub on_secondary_fixed_variant: String,
    #[serde(rename = "tertiaryFixed")]
    pub tertiary_fixed: String,
    #[serde(rename = "onTertiaryFixed")]
    pub on_tertiary_fixed: String,
    #[serde(rename = "tertiaryFixedDim")]
    pub tertiary_fixed_dim: String,
    #[serde(rename = "onTertiaryFixedVariant")]
    pub on_tertiary_fixed_variant: String,
    #[serde(rename = "surfaceDim")]
    pub surface_dim: String,
    #[serde(rename = "surfaceBright")]
    pub surface_bright: String,
    #[serde(rename = "surfaceContainerLowest")]
    pub surface_container_lowest: String,
    #[serde(rename = "surfaceContainerLow")]
    pub surface_container_low: String,
    #[serde(rename = "surfaceContainer")]
    pub surface_container: String,
    #[serde(rename = "surfaceContainerHigh")]
    pub surface_container_high: String,
    #[serde(rename = "surfaceContainerHighest")]
    pub surface_container_highest: String,
}

impl From<MaterialScheme> for crate::MaterialScheme {
    fn from(val: MaterialScheme) -> Self {
        crate::MaterialScheme {
            primary: string_to_color(val.primary),
            surfaceTint: string_to_color(val.surface_tint),
            onPrimary: string_to_color(val.on_primary),
            primaryContainer: string_to_color(val.primary_container),
            onPrimaryContainer: string_to_color(val.on_primary_container),
            secondary: string_to_color(val.secondary),
            onSecondary: string_to_color(val.on_secondary),
            secondaryContainer: string_to_color(val.secondary_container),
            onSecondaryContainer: string_to_color(val.on_secondary_container),
            tertiary: string_to_color(val.tertiary),
            onTertiary: string_to_color(val.on_tertiary),
            tertiaryContainer: string_to_color(val.tertiary_container),
            onTertiaryContainer: string_to_color(val.on_tertiary_container),
            error: string_to_color(val.error),
            onError: string_to_color(val.on_error),
            errorContainer: string_to_color(val.error_container),
            onErrorContainer: string_to_color(val.on_error_container),
            background: string_to_color(val.background),
            onBackground: string_to_color(val.on_background),
            surface: string_to_color(val.surface),
            onSurface: string_to_color(val.on_surface),
            surfaceVariant: string_to_color(val.surface_variant),
            onSurfaceVariant: string_to_color(val.on_surface_variant),
            outline: string_to_color(val.outline),
            outlineVariant: string_to_color(val.outline_variant),
            shadow: string_to_color(val.shadow),
            scrim: string_to_color(val.scrim),
            inverseSurface: string_to_color(val.inverse_surface),
            inverseOnSurface: string_to_color(val.inverse_on_surface),
            inversePrimary: string_to_color(val.inverse_primary),
            primaryFixed: string_to_color(val.primary_fixed),
            onPrimaryFixed: string_to_color(val.on_primary_fixed),
            primaryFixedDim: string_to_color(val.primary_fixed_dim),
            onPrimaryFixedVariant: string_to_color(val.on_primary_fixed_variant),
            secondaryFixed: string_to_color(val.secondary_fixed),
            onSecondaryFixed: string_to_color(val.on_secondary_fixed),
            secondaryFixedDim: string_to_color(val.secondary_fixed_dim),
            onSecondaryFixedVariant: string_to_color(val.on_secondary_fixed_variant),
            tertiaryFixed: string_to_color(val.tertiary_fixed),
            onTertiaryFixed: string_to_color(val.on_tertiary_fixed),
            tertiaryFixedDim: string_to_color(val.tertiary_fixed_dim),
            onTertiaryFixedVariant: string_to_color(val.on_tertiary_fixed_variant),
            surfaceDim: string_to_color(val.surface_dim),
            surfaceBright: string_to_color(val.surface_bright),
            surfaceContainerLowest: string_to_color(val.surface_container_lowest),
            surfaceContainerLow: string_to_color(val.surface_container_low),
            surfaceContainer: string_to_color(val.surface_container),
            surfaceContainerHigh: string_to_color(val.surface_container_high),
            surfaceContainerHighest: string_to_color(val.surface_container_highest),
        }
    }
}

/// Container for both dark and light color schemes.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct MaterialSchemes {
    pub dark: MaterialScheme,
    pub light: MaterialScheme,
}

impl From<MaterialSchemes> for crate::MaterialSchemes {
    fn from(val: MaterialSchemes) -> Self {
        crate::MaterialSchemes {
            dark: val.dark.into(),
            light: val.light.into(),
        }
    }
}

/// Full Material Design theme structure.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct MaterialTheme {
    pub schemes: MaterialSchemes,
}

/// Metadata used to track and invalidate dynamic themes.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ThemeMetadata {
    mode: String,
    wallpaper_path: String,
    wallpaper_mtime: u64,
    seed_color: String,
}

/// Converts a HEX or CSS color string to a Slint Color.
fn string_to_color(color: String) -> slint::Color {
    let c = color
        .parse::<css_color_parser2::Color>()
        .unwrap_or_else(|_| css_color_parser2::Color {
            r: 0,
            g: 0,
            b: 0,
            a: 1.0,
        });
    slint::Color::from_argb_u8((c.a * 255.) as u8, c.r, c.g, c.b)
}

/// Converts an Argb color object to a HEX string.
fn argb_to_hex(argb: Argb) -> String {
    format!("#{:02x}{:02x}{:02x}", argb.red, argb.green, argb.blue)
}

/// The built-in default theme embedded into the binary.
const DEFAULT_THEME: &str = include_str!("../../ui/themes/default.json");

/// Controller for the visual appearance of the greeter.
pub struct Appearance;

impl Appearance {
    /// Initializes the UI appearance based on the configuration.
    pub fn init(ui: &crate::GreeterWindow, config: &crate::config::GreeterConfig) {
        let app_config = &config.appearance;
        // 1. Initialize Theme
        let theme_name = &app_config.theme.name;
        let theme = match theme_name.as_str() {
            "custom" => {
                if let Some(theme_path) = &app_config.theme.path {
                    Self::load_custom_theme(&theme_path.to_string_lossy())
                        .unwrap_or_else(|_| Self::load_builtin_theme("default").unwrap())
                } else {
                    warn!(
                        "Custom theme mode requires a valid theme path. Falling back to default."
                    );
                    Self::load_builtin_theme("default").unwrap()
                }
            }
            "auto" => {
                if let Some(path) = &app_config.background.path {
                    if Path::new(path).exists() {
                        Self::get_dynamic_theme(config).unwrap_or_else(|| {
                            warn!("Failed to generate auto theme. Falling back to default.");
                            Self::load_builtin_theme("default").unwrap()
                        })
                    } else {
                        warn!(
                            "Auto mode requires a valid background image. Falling back to default."
                        );
                        Self::load_builtin_theme("default").unwrap()
                    }
                } else {
                    warn!(
                        "Auto mode requires a valid background image path. Falling back to default."
                    );
                    Self::load_builtin_theme("default").unwrap()
                }
            }
            "seed" => {
                if app_config.theme.seed_color.is_some() {
                    Self::get_dynamic_theme(config).unwrap_or_else(|| {
                        warn!("Failed to generate seed theme. Falling back to default.");
                        Self::load_builtin_theme("default").unwrap()
                    })
                } else {
                    warn!("Seed mode requires seed_color to be set. Falling back to default.");
                    Self::load_builtin_theme("default").unwrap()
                }
            }
            name => Self::load_builtin_theme(name)
                .unwrap_or_else(|| Self::load_builtin_theme("default").unwrap()),
        };

        Self::apply(ui, &theme);
        ui.invoke_set_color_scheme(config.is_dark_mode());

        if let Some(label) = &app_config.label {
            ui.set_greeting_msg(slint::SharedString::from(label));
        }

        let app_style = ui.global::<crate::AppStyle>();

        if let Some(opacity) = app_config.opacity {
            app_style.set_window_opacity(opacity);
        }
        if let Some(font_family) = &app_config.font_family {
            app_style.set_default_font_family(slint::SharedString::from(font_family));
        }
        if let Some(clock_font) = &app_config.clock.font_family {
            app_style.set_clock_font_family(slint::SharedString::from(clock_font));
        }
        if let Some(clock_weight) = app_config.clock.font_weight {
            app_style.set_clock_font_weight(clock_weight);
        }
        if let Some(clock_size) = app_config.clock.font_size {
            app_style.set_clock_font_size(clock_size);
        }

        // 2. Initialize Background
        let bg_config = &app_config.background;

        let fallback_color = if let Some(hex) = &bg_config.color {
            string_to_color(hex.clone())
        } else {
            // Default to theme background
            let bg_hex = if app_config.theme.mode.as_deref() == Some("light") {
                &theme.schemes.light.background
            } else {
                &theme.schemes.dark.background
            };
            string_to_color(bg_hex.clone())
        };

        // Pass color to UI
        ui.set_background_fallback_color(fallback_color);

        if let Some(wallpaper_path) = &bg_config.path {
            let blur_sigma = bg_config.blur.unwrap_or(10.0);

            let original = Image::load_from_path(Path::new(wallpaper_path))
                .unwrap_or_else(|_| Image::default());

            let blurred = match utils::image::prepare_background(config, wallpaper_path, blur_sigma)
            {
                Ok(path) => Image::load_from_path(&path).unwrap_or_else(|_| Image::default()),
                Err(_) => Image::default(),
            };

            ui.set_background_original(original);
            ui.set_background_blurred(blurred);
        } else {
            ui.set_background_original(Image::default());
            ui.set_background_blurred(Image::default());
        }
    }

    /// Generates or retrieves a dynamic theme based on current configuration.
    fn get_dynamic_theme(config: &crate::config::GreeterConfig) -> Option<MaterialTheme> {
        let cache_dir = utils::cache::get_cache_dir(config);
        let theme_path = cache_dir.join("generated_theme.json");
        let meta_path = cache_dir.join("generated_theme.toml");

        let wallpaper_path = config.appearance.background.path.as_deref().unwrap_or("");

        let wallpaper_mtime = if !wallpaper_path.is_empty() {
            fs::metadata(wallpaper_path)
                .and_then(|m| m.modified())
                .unwrap_or(UNIX_EPOCH)
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        } else {
            0
        };

        let current_meta = ThemeMetadata {
            mode: config.appearance.theme.name.clone(),
            wallpaper_path: wallpaper_path.to_string(),
            wallpaper_mtime,
            seed_color: config
                .appearance
                .theme
                .seed_color
                .clone()
                .unwrap_or_default(),
        };

        let is_valid = if meta_path.exists() && theme_path.exists() {
            if let Ok(meta_content) = fs::read_to_string(&meta_path) {
                if let Ok(cached_meta) = toml::from_str::<ThemeMetadata>(&meta_content) {
                    cached_meta == current_meta
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if is_valid {
            if let Ok(theme_content) = fs::read_to_string(&theme_path) {
                if let Ok(theme) = serde_json::from_str(&theme_content) {
                    return Some(theme);
                }
            }
        }

        info!("Generating dynamic theme (mode: {})", current_meta.mode);

        let argb = if current_meta.mode == "seed" {
            if let Ok(c) = current_meta.seed_color.parse::<css_color_parser2::Color>() {
                Argb::new((c.a * 255.) as u8, c.r, c.g, c.b)
            } else {
                return None;
            }
        } else {
            if !wallpaper_path.is_empty() {
                match utils::image::extract_seed_color(wallpaper_path) {
                    Ok([r, g, b, a]) => Argb::new(a, r, g, b),
                    Err(_) => return None,
                }
            } else {
                return None;
            }
        };

        let theme = Self::generate_from_seed(argb);

        if let Ok(theme_json) = serde_json::to_string_pretty(&theme) {
            let _ = fs::create_dir_all(&cache_dir);
            let _ = fs::write(&theme_path, theme_json);
            if let Ok(meta_toml) = toml::to_string_pretty(&current_meta) {
                let _ = fs::write(&meta_path, meta_toml);
            }
        }

        Some(theme)
    }

    /// Creates a full Material Theme from a source ARGB seed color.
    fn generate_from_seed(seed: Argb) -> MaterialTheme {
        let m3_theme = ThemeBuilder::with_source(seed).build();
        let light = Self::map_m3_scheme(m3_theme.schemes.light);
        let dark = Self::map_m3_scheme(m3_theme.schemes.dark);
        MaterialTheme {
            schemes: MaterialSchemes { light, dark },
        }
    }

    /// Maps a raw material-colors Scheme to our internal MaterialScheme.
    fn map_m3_scheme(s: material_colors::scheme::Scheme) -> MaterialScheme {
        MaterialScheme {
            primary: argb_to_hex(s.primary),
            surface_tint: argb_to_hex(s.primary),
            on_primary: argb_to_hex(s.on_primary),
            primary_container: argb_to_hex(s.primary_container),
            on_primary_container: argb_to_hex(s.on_primary_container),
            secondary: argb_to_hex(s.secondary),
            on_secondary: argb_to_hex(s.on_secondary),
            secondary_container: argb_to_hex(s.secondary_container),
            on_secondary_container: argb_to_hex(s.on_secondary_container),
            tertiary: argb_to_hex(s.tertiary),
            on_tertiary: argb_to_hex(s.on_tertiary),
            tertiary_container: argb_to_hex(s.tertiary_container),
            on_tertiary_container: argb_to_hex(s.on_tertiary_container),
            error: argb_to_hex(s.error),
            on_error: argb_to_hex(s.on_error),
            error_container: argb_to_hex(s.error_container),
            on_error_container: argb_to_hex(s.on_error_container),
            background: argb_to_hex(s.background),
            on_background: argb_to_hex(s.on_background),
            surface: argb_to_hex(s.surface),
            on_surface: argb_to_hex(s.on_surface),
            surface_variant: argb_to_hex(s.surface_variant),
            on_surface_variant: argb_to_hex(s.on_surface_variant),
            outline: argb_to_hex(s.outline),
            outline_variant: argb_to_hex(s.outline_variant),
            shadow: argb_to_hex(s.shadow),
            scrim: argb_to_hex(s.scrim),
            inverse_surface: argb_to_hex(s.inverse_surface),
            inverse_on_surface: argb_to_hex(s.inverse_on_surface),
            inverse_primary: argb_to_hex(s.inverse_primary),
            primary_fixed: argb_to_hex(s.primary),
            on_primary_fixed: argb_to_hex(s.on_primary),
            primary_fixed_dim: argb_to_hex(s.primary),
            on_primary_fixed_variant: argb_to_hex(s.on_primary),
            secondary_fixed: argb_to_hex(s.secondary),
            on_secondary_fixed: argb_to_hex(s.on_secondary),
            secondary_fixed_dim: argb_to_hex(s.secondary),
            on_secondary_fixed_variant: argb_to_hex(s.on_secondary_fixed_variant),
            tertiary_fixed: argb_to_hex(s.tertiary),
            on_tertiary_fixed: argb_to_hex(s.on_tertiary),
            tertiary_fixed_dim: argb_to_hex(s.tertiary),
            on_tertiary_fixed_variant: argb_to_hex(s.on_tertiary_fixed_variant),
            surface_dim: argb_to_hex(s.surface),
            surface_bright: argb_to_hex(s.surface_bright),
            surface_container_lowest: argb_to_hex(s.surface_container_lowest),
            surface_container_low: argb_to_hex(s.surface_container_low),
            surface_container: argb_to_hex(s.surface_container),
            surface_container_high: argb_to_hex(s.surface_container_high),
            surface_container_highest: argb_to_hex(s.surface_container_highest),
        }
    }

    /// Loads a theme from the embedded resources or built-in list.
    pub fn load_builtin_theme(name: &str) -> Option<MaterialTheme> {
        let json = match name {
            "default" | "slint" => DEFAULT_THEME,
            _ => return None,
        };
        serde_json::from_str(json).ok()
    }

    /// Loads a custom theme from a JSON file on disk.
    pub fn load_custom_theme(path: &str) -> Result<MaterialTheme> {
        let content =
            fs::read_to_string(path).with_context(|| format!("theme: failed to read: {}", path))?;
        serde_json::from_str(&content).context("theme: failed to parse JSON")
    }

    /// Applies a MaterialTheme to the Slint UI instance.
    pub fn apply(ui: &crate::GreeterWindow, theme: &MaterialTheme) {
        let schemes: crate::MaterialSchemes = theme.schemes.clone().into();
        ui.global::<crate::MaterialPalette>().set_schemes(schemes);
    }
}
