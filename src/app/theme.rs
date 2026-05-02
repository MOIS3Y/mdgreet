use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use slint::ComponentHandle;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct MaterialTheme {
    pub schemes: MaterialSchemes,
}

fn string_to_color(color: String) -> slint::Color {
    let c = color.parse::<css_color_parser2::Color>().unwrap();
    slint::Color::from_argb_u8((c.a * 255.) as u8, c.r, c.g, c.b)
}

// Built-in themes
const SLINT_THEME: &str = include_str!("../../ui/themes/slint.json");
const PURPLE_THEME: &str = include_str!("../../ui/themes/purple.json");
const RED_THEME: &str = include_str!("../../ui/themes/red.json");
const GREEN_THEME: &str = include_str!("../../ui/themes/green.json");

pub struct Theme;

impl Theme {
    pub fn init(ui: &crate::GreeterWindow, theme_name: &str) {
        let theme = if theme_name == "custom" {
            let config_dir =
                std::env::var("MDGREET_CONFIG_DIR").unwrap_or_else(|_| ".".to_string());
            let theme_path = format!("{}/material-theme.json", config_dir);

            Self::load_custom_theme(&theme_path).unwrap_or_else(|e| {
                eprintln!("theme: failed to load custom: {}", e);
                eprintln!("theme: falling back to purple");
                Self::load_builtin_theme("purple")
                    .expect("theme: failed to load fallback purple theme")
            })
        } else {
            Self::load_builtin_theme(theme_name).unwrap_or_else(|| {
                eprintln!(
                    "theme: unknown theme '{}', falling back to purple",
                    theme_name
                );
                Self::load_builtin_theme("purple")
                    .expect("theme: failed to load fallback purple theme")
            })
        };

        Self::apply(ui, &theme);
    }

    pub fn load_builtin_theme(name: &str) -> Option<MaterialTheme> {
        let json = match name {
            "slint" => SLINT_THEME,
            "purple" => PURPLE_THEME,
            "red" => RED_THEME,
            "green" => GREEN_THEME,
            _ => return None,
        };
        serde_json::from_str(json).ok()
    }

    pub fn load_custom_theme(path: &str) -> Result<MaterialTheme> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("theme: failed to read: {}", path))?;
        serde_json::from_str(&content).context("theme: failed to parse JSON")
    }

    pub fn apply(ui: &crate::GreeterWindow, theme: &MaterialTheme) {
        let schemes: crate::MaterialSchemes = theme.schemes.clone().into();
        ui.global::<crate::MaterialPalette>().set_schemes(schemes);
    }
}
