//! # User Configuration Format
//!
//! This module defines the user-facing configuration format that matches the
//! TOML structure users write. It provides a bridge between the user's config
//! file and the internal [`Theme`] representation.
//!
//! ## Design Rationale
//!
//! Users write configuration in a hierarchical, intuitive format:
//!
//! ```toml
//! [appearance]
//! theme = "dark"
//! accent = "#ff6b6b"
//! font_family = "Inter"
//!
//! [bar]
//! height = 40
//! position = "top"
//! ```
//!
//! But the internal [`Theme`] struct has a different structure optimized for
//! the domain model. This module handles the transformation.
//!
//! ## Transformation Flow
//!
//! ```text
//! TOML file
//!     ↓
//! [parse to UserConfig]
//!     ↓
//! UserConfig::into_theme()
//!     ↓
//! Theme (validated, ready for use)
//! ```

use crate::domain::theme::{BarConfig, BarPosition, ColorPalette, Theme, ThemeVariant};
use crate::domain::tokens::color::{Accent, ColorToken};
use crate::domain::tokens::radius::RadiusScale;
use crate::domain::tokens::typography::{FontFamily, FontSizeScale, Typography};
use crate::error::{ConfigError, FernError, Result};
use serde::Deserialize;

// ============================================================================
// UserConfig — Top-level configuration
// ============================================================================

/// User-facing configuration format matching the TOML structure.
///
/// This struct deserializes directly from the user's TOML file and provides
/// transformation to the internal [`Theme`] representation.
///
/// # Example
///
/// ```rust,ignore
/// let toml_content = r#"
///     [appearance]
///     theme = "dark"
///     accent = "#ff6b6b"
///
///     [bar]
///     height = 32
/// "#;
///
/// let user_config: UserConfig = toml::from_str(toml_content)?;
/// let theme = user_config.into_theme()?;
/// ```
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UserConfig {
    /// Appearance settings (colors, fonts, etc.)
    #[serde(default)]
    pub appearance: AppearanceConfig,

    /// Bar/panel configuration
    #[serde(default)]
    pub bar: UserBarConfig,

    /// Module-specific settings (passed through to QuickShell, not validated)
    #[serde(default)]
    pub modules: serde_json::Value,
}

impl UserConfig {
    /// Transforms the user configuration into a validated [`Theme`].
    ///
    /// This method:
    /// 1. Determines the base theme variant (dark/light)
    /// 2. Applies any color overrides (e.g., accent)
    /// 3. Applies typography overrides
    /// 4. Applies radius overrides
    /// 5. Builds the bar configuration
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - An invalid color value is provided (e.g., "#gg0000")
    /// - An invalid theme variant is specified
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let user_config = UserConfig::default();
    /// let theme = user_config.into_theme()?;
    /// assert_eq!(theme.variant, ThemeVariant::Dark);
    /// ```
    pub fn into_theme(self) -> Result<Theme> {
        // Determine base theme variant
        let variant = ThemeVariant::from_name(&self.appearance.theme).unwrap_or(ThemeVariant::Dark);

        // Start with default colors for the variant
        let mut colors = match variant {
            ThemeVariant::Dark => ColorPalette::dark(),
            ThemeVariant::Light => ColorPalette::light(),
            ThemeVariant::Auto => ColorPalette::dark(), // Default to dark for auto
        };

        // Apply accent color override if provided
        if let Some(ref accent) = self.appearance.accent {
            colors.accent = ColorToken::<Accent>::from_hex(accent).map_err(|_| {
                FernError::Config(ConfigError::InvalidColor {
                    value: accent.clone(),
                    span: None,
                    source_code: None,
                })
            })?;
        }

        // Build typography with overrides
        let typography = Typography {
            family: self
                .appearance
                .font_family
                .map(FontFamily::new)
                .unwrap_or_else(FontFamily::default_primary),
            mono: self
                .appearance
                .font_mono
                .map(FontFamily::new)
                .unwrap_or_else(FontFamily::default_mono),
            icon: self
                .appearance
                .font_icon
                .map(FontFamily::new)
                .unwrap_or_else(FontFamily::default_icon),
            size: FontSizeScale::default(),
        };

        // Build radius scale with overrides
        let radius = self
            .appearance
            .radius
            .map(UserRadiusConfig::into_scale)
            .unwrap_or_default();

        // Build bar configuration
        let bar = BarConfig {
            height: self.bar.height,
            position: BarPosition::from_name(&self.bar.position).unwrap_or_default(),
            margin: self.bar.margin,
        };

        Ok(Theme {
            variant,
            colors,
            typography,
            radius,
            bar,
        })
    }
}

// ============================================================================
// AppearanceConfig — Visual styling options
// ============================================================================

/// Appearance configuration section.
///
/// Contains all visual styling options: theme variant, colors, fonts, and radii.
#[derive(Debug, Clone, Deserialize)]
pub struct AppearanceConfig {
    /// Theme variant: "dark", "light", or "auto"
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Accent color override (hex format, e.g., "#ff6b6b")
    #[serde(default)]
    pub accent: Option<String>,

    /// Primary font family name
    #[serde(default)]
    pub font_family: Option<String>,

    /// Monospace font family name
    #[serde(default)]
    pub font_mono: Option<String>,

    /// Icon font family name
    #[serde(default)]
    pub font_icon: Option<String>,

    /// Border radius overrides
    #[serde(default)]
    pub radius: Option<UserRadiusConfig>,
}

fn default_theme() -> String {
    "dark".to_string()
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            accent: None,
            font_family: None,
            font_mono: None,
            font_icon: None,
            radius: None,
        }
    }
}

// ============================================================================
// UserBarConfig — Bar/panel settings
// ============================================================================

/// Bar/panel configuration section.
///
/// Note: `modules_left`, `modules_center`, and `modules_right` are parsed
/// but not included in the Theme — they're passed through to QuickShell.
#[derive(Debug, Clone, Deserialize)]
pub struct UserBarConfig {
    /// Bar height in pixels
    #[serde(default = "default_bar_height")]
    pub height: u16,

    /// Bar position: "top" or "bottom"
    #[serde(default = "default_bar_position")]
    pub position: String,

    /// Margin from screen edge
    #[serde(default)]
    pub margin: u16,

    /// Modules on the left (passed through to QuickShell)
    #[serde(default)]
    pub modules_left: Vec<String>,

    /// Modules in the center (passed through to QuickShell)
    #[serde(default)]
    pub modules_center: Vec<String>,

    /// Modules on the right (passed through to QuickShell)
    #[serde(default)]
    pub modules_right: Vec<String>,
}

fn default_bar_height() -> u16 {
    40
}

fn default_bar_position() -> String {
    "top".to_string()
}

impl Default for UserBarConfig {
    fn default() -> Self {
        Self {
            height: default_bar_height(),
            position: default_bar_position(),
            margin: 0,
            modules_left: vec!["workspaces".to_string()],
            modules_center: vec!["clock".to_string()],
            modules_right: vec!["tray".to_string()],
        }
    }
}

// ============================================================================
// UserRadiusConfig — Radius overrides
// ============================================================================

/// Border radius configuration section.
///
/// Allows users to customize the radius scale values.
#[derive(Debug, Clone, Deserialize)]
pub struct UserRadiusConfig {
    /// No radius (typically 0)
    #[serde(default)]
    pub none: Option<u16>,

    /// Small radius (default 4px)
    #[serde(default)]
    pub sm: Option<u16>,

    /// Medium radius (default 8px)
    #[serde(default)]
    pub md: Option<u16>,

    /// Large radius (default 12px)
    #[serde(default)]
    pub lg: Option<u16>,

    /// Full radius (default 9999px)
    #[serde(default)]
    pub full: Option<u16>,
}

impl UserRadiusConfig {
    /// Converts user radius config into a [`RadiusScale`].
    fn into_scale(self) -> RadiusScale {
        let default = RadiusScale::default();
        RadiusScale {
            none: self.none.unwrap_or(default.none),
            sm: self.sm.unwrap_or(default.sm),
            md: self.md.unwrap_or(default.md),
            lg: self.lg.unwrap_or(default.lg),
            full: self.full.unwrap_or(default.full),
        }
    }
}

impl Default for UserRadiusConfig {
    fn default() -> Self {
        Self {
            none: None,
            sm: None,
            md: None,
            lg: None,
            full: None,
        }
    }
}

// ============================================================================
// BarPosition helper
// ============================================================================

impl BarPosition {
    /// Creates a bar position from its string name.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "top" => Some(Self::Top),
            "bottom" => Some(Self::Bottom),
            _ => None,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_user_config() {
        let config = UserConfig::default();
        assert_eq!(config.appearance.theme, "dark");
        assert_eq!(config.bar.height, 40);
    }

    #[test]
    fn user_config_into_theme_defaults() {
        let config = UserConfig::default();
        let theme = config.into_theme().unwrap();

        assert_eq!(theme.variant, ThemeVariant::Dark);
        assert_eq!(theme.bar.height, 40);
        assert_eq!(theme.bar.position, BarPosition::Top);
    }

    #[test]
    fn user_config_accent_override() {
        let config = UserConfig {
            appearance: AppearanceConfig {
                theme: "dark".to_string(),
                accent: Some("#ff6b6b".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let theme = config.into_theme().unwrap();
        assert_eq!(theme.colors.accent.to_hex(), "#ff6b6b");
    }

    #[test]
    fn user_config_invalid_accent() {
        let config = UserConfig {
            appearance: AppearanceConfig {
                accent: Some("#gg0000".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let result = config.into_theme();
        assert!(result.is_err());
    }

    #[test]
    fn user_config_light_theme() {
        let config = UserConfig {
            appearance: AppearanceConfig {
                theme: "light".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        let theme = config.into_theme().unwrap();
        assert_eq!(theme.variant, ThemeVariant::Light);
    }

    #[test]
    fn user_config_font_override() {
        let config = UserConfig {
            appearance: AppearanceConfig {
                font_family: Some("Fira Sans".to_string()),
                font_mono: Some("Fira Code".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let theme = config.into_theme().unwrap();
        assert_eq!(theme.typography.family.name(), "Fira Sans");
        assert_eq!(theme.typography.mono.name(), "Fira Code");
    }

    #[test]
    fn user_config_radius_override() {
        let config = UserConfig {
            appearance: AppearanceConfig {
                radius: Some(UserRadiusConfig {
                    sm: Some(2),
                    md: Some(6),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        let theme = config.into_theme().unwrap();
        assert_eq!(theme.radius.sm, 2);
        assert_eq!(theme.radius.md, 6);
        assert_eq!(theme.radius.lg, 12); // Default unchanged
    }

    #[test]
    fn user_config_bar_position() {
        let config = UserConfig {
            bar: UserBarConfig {
                position: "bottom".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };

        let theme = config.into_theme().unwrap();
        assert_eq!(theme.bar.position, BarPosition::Bottom);
    }

    #[test]
    fn bar_position_from_name() {
        assert_eq!(BarPosition::from_name("top"), Some(BarPosition::Top));
        assert_eq!(BarPosition::from_name("BOTTOM"), Some(BarPosition::Bottom));
        assert_eq!(BarPosition::from_name("invalid"), None);
    }

    #[test]
    fn deserialize_from_toml() {
        let toml_content = r##"
[appearance]
theme = "dark"
accent = "#89b4fa"
font_family = "Inter"

[bar]
height = 32
position = "top"
"##;

        let config: UserConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.appearance.theme, "dark");
        assert_eq!(config.appearance.accent, Some("#89b4fa".to_string()));
        assert_eq!(config.bar.height, 32);

        let theme = config.into_theme().unwrap();
        assert_eq!(theme.colors.accent.to_hex(), "#89b4fa");
        assert_eq!(theme.bar.height, 32);
    }
}
