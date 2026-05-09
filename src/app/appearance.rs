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

macro_rules! define_material_scheme {
    (
        $(
            $rust_field:ident => $slint_field:ident from $m3_field:ident
        ),* $(,)?
    ) => {
        /// Internal representation of a Material Design color scheme.
        #[derive(Serialize, Deserialize, Debug, Clone)]
        #[serde(rename_all = "camelCase")]
        pub(crate) struct MaterialScheme {
            $(
                pub $rust_field: String,
            )*
        }

        impl From<MaterialScheme> for crate::MaterialScheme {
            fn from(val: MaterialScheme) -> Self {
                crate::MaterialScheme {
                    $(
                        $slint_field: string_to_color(val.$rust_field),
                    )*
                }
            }
        }

        impl MaterialScheme {
            fn from_m3(s: material_colors::scheme::Scheme) -> Self {
                Self {
                    $(
                        $rust_field: argb_to_hex(s.$m3_field),
                    )*
                }
            }
        }
    };
}

define_material_scheme! {
    primary => primary from primary,
    surface_tint => surfaceTint from primary,
    on_primary => onPrimary from on_primary,
    primary_container => primaryContainer from primary_container,
    on_primary_container => onPrimaryContainer from on_primary_container,
    secondary => secondary from secondary,
    on_secondary => onSecondary from on_secondary,
    secondary_container => secondaryContainer from secondary_container,
    on_secondary_container => onSecondaryContainer from on_secondary_container,
    tertiary => tertiary from tertiary,
    on_tertiary => onTertiary from on_tertiary,
    tertiary_container => tertiaryContainer from tertiary_container,
    on_tertiary_container => onTertiaryContainer from on_tertiary_container,
    error => error from error,
    on_error => onError from on_error,
    error_container => errorContainer from error_container,
    on_error_container => onErrorContainer from on_error_container,
    background => background from background,
    on_background => onBackground from on_background,
    surface => surface from surface,
    on_surface => onSurface from on_surface,
    surface_variant => surfaceVariant from surface_variant,
    on_surface_variant => onSurfaceVariant from on_surface_variant,
    outline => outline from outline,
    outline_variant => outlineVariant from outline_variant,
    shadow => shadow from shadow,
    scrim => scrim from scrim,
    inverse_surface => inverseSurface from inverse_surface,
    inverse_on_surface => inverseOnSurface from inverse_on_surface,
    inverse_primary => inversePrimary from inverse_primary,
    primary_fixed => primaryFixed from primary,
    on_primary_fixed => onPrimaryFixed from on_primary,
    primary_fixed_dim => primaryFixedDim from primary,
    on_primary_fixed_variant => onPrimaryFixedVariant from on_primary,
    secondary_fixed => secondaryFixed from secondary,
    on_secondary_fixed => onSecondaryFixed from on_secondary,
    secondary_fixed_dim => secondaryFixedDim from secondary,
    on_secondary_fixed_variant => onSecondaryFixedVariant from on_secondary_fixed_variant,
    tertiary_fixed => tertiaryFixed from tertiary,
    on_tertiary_fixed => onTertiaryFixed from on_tertiary,
    tertiary_fixed_dim => tertiaryFixedDim from tertiary,
    on_tertiary_fixed_variant => onTertiaryFixedVariant from on_tertiary_fixed_variant,
    surface_dim => surfaceDim from surface,
    surface_bright => surfaceBright from surface_bright,
    surface_container_lowest => surfaceContainerLowest from surface_container_lowest,
    surface_container_low => surfaceContainerLow from surface_container_low,
    surface_container => surfaceContainer from surface_container,
    surface_container_high => surfaceContainerHigh from surface_container_high,
    surface_container_highest => surfaceContainerHighest from surface_container_highest,
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
        let light = MaterialScheme::from_m3(m3_theme.schemes.light);
        let dark = MaterialScheme::from_m3(m3_theme.schemes.dark);
        MaterialTheme {
            schemes: MaterialSchemes { light, dark },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_color() {
        let color = string_to_color("#ff0000".to_string());
        assert_eq!(color.red(), 255);
        assert_eq!(color.green(), 0);
        assert_eq!(color.blue(), 0);
        assert_eq!(color.alpha(), 255);

        let color_with_alpha = string_to_color("rgba(255, 0, 0, 0.5)".to_string());
        assert_eq!(color_with_alpha.red(), 255);
        assert_eq!(color_with_alpha.green(), 0);
        assert_eq!(color_with_alpha.blue(), 0);
        assert_eq!(color_with_alpha.alpha(), 127); // 0.5 * 255 is 127

        let invalid = string_to_color("invalid".to_string());
        assert_eq!(invalid.red(), 0);
        assert_eq!(invalid.green(), 0);
        assert_eq!(invalid.blue(), 0);
        assert_eq!(invalid.alpha(), 255);
    }

    #[test]
    fn test_argb_to_hex() {
        let argb = Argb::new(255, 255, 128, 64);
        assert_eq!(argb_to_hex(argb), "#ff8040");
    }

    #[test]
    fn test_material_scheme_macro() {
        // Test parsing from material_colors scheme
        let mut m3_scheme = ThemeBuilder::with_source(Argb::new(255, 0, 0, 0))
            .build()
            .schemes
            .light;
        m3_scheme.primary = Argb::new(255, 10, 20, 30);
        m3_scheme.on_primary = Argb::new(255, 40, 50, 60);

        // Ensure primary was successfully overridden
        let scheme = MaterialScheme::from_m3(m3_scheme);
        assert_eq!(scheme.primary, "#0a141e");
        assert_eq!(scheme.on_primary, "#28323c");

        // Test From trait mapping to crate::MaterialScheme
        let crate_scheme: crate::MaterialScheme = scheme.into();
        assert_eq!(crate_scheme.primary.red(), 10);
        assert_eq!(crate_scheme.primary.green(), 20);
        assert_eq!(crate_scheme.primary.blue(), 30);
        assert_eq!(crate_scheme.primary.alpha(), 255);

        // Test serde serialization names (camelCase)
        let json_str = r##"{
            "primary": "#0a141e",
            "surfaceTint": "#0a141e",
            "onPrimary": "#28323c",
            "primaryContainer": "#000000",
            "onPrimaryContainer": "#000000",
            "secondary": "#000000",
            "onSecondary": "#000000",
            "secondaryContainer": "#000000",
            "onSecondaryContainer": "#000000",
            "tertiary": "#000000",
            "onTertiary": "#000000",
            "tertiaryContainer": "#000000",
            "onTertiaryContainer": "#000000",
            "error": "#000000",
            "onError": "#000000",
            "errorContainer": "#000000",
            "onErrorContainer": "#000000",
            "background": "#000000",
            "onBackground": "#000000",
            "surface": "#000000",
            "onSurface": "#000000",
            "surfaceVariant": "#000000",
            "onSurfaceVariant": "#000000",
            "outline": "#000000",
            "outlineVariant": "#000000",
            "shadow": "#000000",
            "scrim": "#000000",
            "inverseSurface": "#000000",
            "inverseOnSurface": "#000000",
            "inversePrimary": "#000000",
            "primaryFixed": "#000000",
            "onPrimaryFixed": "#000000",
            "primaryFixedDim": "#000000",
            "onPrimaryFixedVariant": "#000000",
            "secondaryFixed": "#000000",
            "onSecondaryFixed": "#000000",
            "secondaryFixedDim": "#000000",
            "onSecondaryFixedVariant": "#000000",
            "tertiaryFixed": "#000000",
            "onTertiaryFixed": "#000000",
            "tertiaryFixedDim": "#000000",
            "onTertiaryFixedVariant": "#000000",
            "surfaceDim": "#000000",
            "surfaceBright": "#000000",
            "surfaceContainerLowest": "#000000",
            "surfaceContainerLow": "#000000",
            "surfaceContainer": "#000000",
            "surfaceContainerHigh": "#000000",
            "surfaceContainerHighest": "#000000"
        }"##;

        let deserialized: MaterialScheme = serde_json::from_str(json_str).unwrap();
        assert_eq!(deserialized.surface_tint, "#0a141e");
        assert_eq!(deserialized.on_primary, "#28323c");
    }

    #[test]
    fn test_load_builtin_theme() {
        assert!(Appearance::load_builtin_theme("default").is_some());
        assert!(Appearance::load_builtin_theme("slint").is_some());
        assert!(Appearance::load_builtin_theme("nonexistent").is_none());
    }
}
