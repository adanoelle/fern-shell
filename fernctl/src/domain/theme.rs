//! # Complete Theme Definition
//!
//! A [`Theme`] is a complete, coherent collection of all design tokens needed
//! to render the Fern Shell UI. It combines colors, typography, spacing, and
//! radius into a single validated structure.
//!
//! ## Design Rationale
//!
//! Rather than scattering token values throughout components, a `Theme` provides
//! a single source of truth. This ensures:
//!
//! 1. **Completeness** — All required tokens are present
//! 2. **Coherence** — Tokens are designed to work together
//! 3. **Configurability** — Users can customize via config files
//!
//! ## Theme Variants
//!
//! Fern supports multiple theme variants:
//!
//! - **Dark** — Dark backgrounds with light text (default)
//! - **Light** — Light backgrounds with dark text
//! - **Auto** — Follows system preference (future)
//!
//! ## Example
//!
//! ```rust,ignore
//! use fernctl::domain::theme::{Theme, ThemeVariant};
//!
//! // Load default dark theme
//! let theme = Theme::dark();
//!
//! // Access tokens
//! let bg = theme.colors.background;
//! let font = &theme.typography.family;
//! let radius = theme.radius.button();
//! ```

use super::tokens::{
    color::*,
    radius::RadiusScale,
    spacing::SpacingValue,
    typography::Typography,
};
use serde::{Deserialize, Serialize};

/// Theme variant selection.
///
/// Determines the base color palette for the theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeVariant {
    /// Dark theme with dark backgrounds and light text.
    #[default]
    Dark,
    /// Light theme with light backgrounds and dark text.
    Light,
    /// Automatically follow system preference.
    Auto,
}

impl ThemeVariant {
    /// Returns the variant name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
            Self::Auto => "auto",
        }
    }

    /// Creates a variant from its name.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "dark" => Some(Self::Dark),
            "light" => Some(Self::Light),
            "auto" => Some(Self::Auto),
            _ => None,
        }
    }
}

/// Color palette for a theme.
///
/// Contains all semantic colors used throughout the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorPalette {
    /// Primary background color.
    pub background: ColorToken<Background>,
    /// Elevated surface color.
    pub surface: ColorToken<Surface>,
    /// Hover state for surfaces.
    pub surface_hover: ColorToken<SurfaceHover>,
    /// Primary text color.
    pub foreground: ColorToken<Foreground>,
    /// Secondary/dimmed text color.
    pub foreground_dim: ColorToken<ForegroundDim>,
    /// Accent color for highlights.
    pub accent: ColorToken<Accent>,
    /// Error state color.
    pub error: ColorToken<Error>,
    /// Warning state color.
    pub warning: ColorToken<Warning>,
    /// Success state color.
    pub success: ColorToken<Success>,
    /// Info state color.
    pub info: ColorToken<Info>,
}

impl ColorPalette {
    /// Creates the default dark color palette (Catppuccin Mocha).
    #[must_use]
    pub fn dark() -> Self {
        Self {
            background: ColorToken::from_hex("#1e1e2e").expect("valid hex"),
            surface: ColorToken::from_hex("#313244").expect("valid hex"),
            surface_hover: ColorToken::from_hex("#45475a").expect("valid hex"),
            foreground: ColorToken::from_hex("#cdd6f4").expect("valid hex"),
            foreground_dim: ColorToken::from_hex("#a6adc8").expect("valid hex"),
            accent: ColorToken::from_hex("#89b4fa").expect("valid hex"),
            error: ColorToken::from_hex("#f38ba8").expect("valid hex"),
            warning: ColorToken::from_hex("#f9e2af").expect("valid hex"),
            success: ColorToken::from_hex("#a6e3a1").expect("valid hex"),
            info: ColorToken::from_hex("#89dceb").expect("valid hex"),
        }
    }

    /// Creates the default light color palette (Catppuccin Latte).
    #[must_use]
    pub fn light() -> Self {
        Self {
            background: ColorToken::from_hex("#eff1f5").expect("valid hex"),
            surface: ColorToken::from_hex("#e6e9ef").expect("valid hex"),
            surface_hover: ColorToken::from_hex("#dce0e8").expect("valid hex"),
            foreground: ColorToken::from_hex("#4c4f69").expect("valid hex"),
            foreground_dim: ColorToken::from_hex("#6c6f85").expect("valid hex"),
            accent: ColorToken::from_hex("#1e66f5").expect("valid hex"),
            error: ColorToken::from_hex("#d20f39").expect("valid hex"),
            warning: ColorToken::from_hex("#df8e1d").expect("valid hex"),
            success: ColorToken::from_hex("#40a02b").expect("valid hex"),
            info: ColorToken::from_hex("#04a5e5").expect("valid hex"),
        }
    }
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self::dark()
    }
}

/// Bar (panel) configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BarConfig {
    /// Bar height in pixels.
    #[serde(default = "default_bar_height")]
    pub height: u16,
    /// Bar position (top or bottom).
    #[serde(default)]
    pub position: BarPosition,
    /// Margin from screen edge.
    #[serde(default)]
    pub margin: u16,
}

fn default_bar_height() -> u16 {
    40
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            height: 40,
            position: BarPosition::Top,
            margin: 0,
        }
    }
}

/// Bar position on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BarPosition {
    /// Bar at top of screen.
    #[default]
    Top,
    /// Bar at bottom of screen.
    Bottom,
}

impl BarPosition {
    /// Returns the position name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }
}

/// A complete theme combining all design tokens.
///
/// `Theme` is the central configuration object for the Fern Shell UI.
/// It contains all colors, typography, spacing, and other visual settings
/// needed to render the interface consistently.
///
/// # Creating a Theme
///
/// ```rust
/// use fernctl::domain::theme::Theme;
///
/// // Default dark theme
/// let theme = Theme::dark();
///
/// // Default light theme
/// let theme = Theme::light();
///
/// // From configuration (typically via adapters)
/// // let theme = Theme::from_config(config)?;
/// ```
///
/// # Accessing Tokens
///
/// ```rust
/// use fernctl::domain::theme::Theme;
///
/// let theme = Theme::dark();
///
/// // Colors
/// let bg_color = &theme.colors.background;
/// let accent = &theme.colors.accent;
///
/// // Typography
/// let font = &theme.typography.family;
/// let mono = &theme.typography.mono;
///
/// // Radius
/// let button_radius = theme.radius.button();
/// let module_radius = theme.radius.module();
///
/// // Bar
/// let bar_height = theme.bar.height;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Theme {
    /// The theme variant (dark, light, auto).
    #[serde(default)]
    pub variant: ThemeVariant,
    /// Color palette.
    #[serde(default)]
    pub colors: ColorPalette,
    /// Typography settings.
    #[serde(default)]
    pub typography: Typography,
    /// Radius scale.
    #[serde(default)]
    pub radius: RadiusScale,
    /// Bar configuration.
    #[serde(default)]
    pub bar: BarConfig,
}

impl Theme {
    /// Creates a new dark theme with default values.
    ///
    /// This is the recommended default theme for Fern Shell.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::domain::theme::Theme;
    ///
    /// let theme = Theme::dark();
    /// assert_eq!(theme.variant.name(), "dark");
    /// ```
    #[must_use]
    pub fn dark() -> Self {
        Self {
            variant: ThemeVariant::Dark,
            colors: ColorPalette::dark(),
            typography: Typography::default(),
            radius: RadiusScale::default(),
            bar: BarConfig::default(),
        }
    }

    /// Creates a new light theme with default values.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::domain::theme::Theme;
    ///
    /// let theme = Theme::light();
    /// assert_eq!(theme.variant.name(), "light");
    /// ```
    #[must_use]
    pub fn light() -> Self {
        Self {
            variant: ThemeVariant::Light,
            colors: ColorPalette::light(),
            typography: Typography::default(),
            radius: RadiusScale::default(),
            bar: BarConfig::default(),
        }
    }

    /// Returns the default spacing value for modules.
    ///
    /// This is a convenience method for common usage.
    #[must_use]
    pub const fn module_spacing(&self) -> u16 {
        SpacingValue::Lg.pixels()
    }

    /// Returns the default spacing value for buttons.
    #[must_use]
    pub const fn button_spacing(&self) -> u16 {
        SpacingValue::Sm.pixels()
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_dark_default() {
        let theme = Theme::dark();
        assert_eq!(theme.variant, ThemeVariant::Dark);
        assert_eq!(theme.bar.height, 40);
    }

    #[test]
    fn theme_light() {
        let theme = Theme::light();
        assert_eq!(theme.variant, ThemeVariant::Light);
    }

    #[test]
    fn theme_serialization_roundtrip() {
        let original = Theme::dark();
        let json = serde_json::to_string_pretty(&original).unwrap();
        let restored: Theme = serde_json::from_str(&json).unwrap();
        assert_eq!(original.variant, restored.variant);
        assert_eq!(original.bar.height, restored.bar.height);
    }

    #[test]
    fn theme_variant_from_name() {
        assert_eq!(ThemeVariant::from_name("dark"), Some(ThemeVariant::Dark));
        assert_eq!(ThemeVariant::from_name("LIGHT"), Some(ThemeVariant::Light));
        assert_eq!(ThemeVariant::from_name("invalid"), None);
    }
}
