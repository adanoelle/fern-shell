//! # Typography Tokens
//!
//! This module provides type-safe typography tokens for font families and
//! size scales used throughout the Fern design system.
//!
//! ## Font Categories
//!
//! Fern uses three font categories:
//!
//! | Category | Purpose | Default |
//! |----------|---------|---------|
//! | Primary | UI text, labels | Inter |
//! | Monospace | Code, terminal icons | JetBrainsMono Nerd Font |
//! | Icon | Material icons | Material Symbols Rounded |
//!
//! ## Font Size Scale
//!
//! Font sizes follow a semantic scale rather than arbitrary pixel values:
//!
//! | Token | Pixels | Use Case |
//! |-------|--------|----------|
//! | `Xs` | 10px | Captions, legal text |
//! | `Sm` | 12px | Labels, secondary text |
//! | `Md` | 14px | Body text (default) |
//! | `Lg` | 16px | Subheadings |
//! | `Xl` | 20px | Headings |
//! | `Xxl` | 24px | Large headings |

use serde::{Deserialize, Serialize};
use std::fmt;

/// A font family specification.
///
/// `FontFamily` represents a font family name that can be used in the UI.
/// It stores the family name as a string and provides validation.
///
/// # Example
///
/// ```rust
/// use fern_theme::domain::tokens::typography::FontFamily;
///
/// let font = FontFamily::new("Inter");
/// assert_eq!(font.name(), "Inter");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FontFamily {
    name: String,
}

impl FontFamily {
    /// Creates a new font family from a name.
    ///
    /// # Arguments
    ///
    /// * `name` — The font family name (e.g., "Inter", "JetBrainsMono Nerd Font")
    ///
    /// # Example
    ///
    /// ```rust
    /// use fern_theme::domain::tokens::typography::FontFamily;
    ///
    /// let font = FontFamily::new("Inter");
    /// ```
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Returns the font family name.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fern_theme::domain::tokens::typography::FontFamily;
    ///
    /// let font = FontFamily::new("Inter");
    /// assert_eq!(font.name(), "Inter");
    /// ```
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the default primary (UI) font family.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fern_theme::domain::tokens::typography::FontFamily;
    ///
    /// let font = FontFamily::default_primary();
    /// assert_eq!(font.name(), "Inter");
    /// ```
    #[must_use]
    pub fn default_primary() -> Self {
        Self::new("Inter")
    }

    /// Returns the default monospace font family.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fern_theme::domain::tokens::typography::FontFamily;
    ///
    /// let font = FontFamily::default_mono();
    /// assert_eq!(font.name(), "JetBrainsMono Nerd Font");
    /// ```
    #[must_use]
    pub fn default_mono() -> Self {
        Self::new("JetBrainsMono Nerd Font")
    }

    /// Returns the default icon font family.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fern_theme::domain::tokens::typography::FontFamily;
    ///
    /// let font = FontFamily::default_icon();
    /// assert_eq!(font.name(), "Material Symbols Rounded");
    /// ```
    #[must_use]
    pub fn default_icon() -> Self {
        Self::new("Material Symbols Rounded")
    }
}

impl Default for FontFamily {
    fn default() -> Self {
        Self::default_primary()
    }
}

impl fmt::Display for FontFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl From<&str> for FontFamily {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for FontFamily {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

/// Font size scale with semantic sizes.
///
/// This enum represents the available font sizes in the design system.
/// Using semantic names rather than pixel values ensures consistency
/// and makes it easy to adjust the entire scale.
///
/// # Example
///
/// ```rust
/// use fern_theme::domain::tokens::typography::FontSize;
///
/// let size = FontSize::Md;
/// assert_eq!(size.pixels(), 14);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FontSize {
    /// Extra-small (10px) — Captions, legal text
    Xs,
    /// Small (12px) — Labels, secondary text
    Sm,
    /// Medium (14px) — Body text (default)
    Md,
    /// Large (16px) — Subheadings
    Lg,
    /// Extra-large (20px) — Headings
    Xl,
    /// Double extra-large (24px) — Large headings
    Xxl,
}

impl FontSize {
    /// Returns the size in pixels.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fern_theme::domain::tokens::typography::FontSize;
    ///
    /// assert_eq!(FontSize::Xs.pixels(), 10);
    /// assert_eq!(FontSize::Sm.pixels(), 12);
    /// assert_eq!(FontSize::Md.pixels(), 14);
    /// assert_eq!(FontSize::Lg.pixels(), 16);
    /// assert_eq!(FontSize::Xl.pixels(), 20);
    /// assert_eq!(FontSize::Xxl.pixels(), 24);
    /// ```
    #[must_use]
    pub const fn pixels(&self) -> u16 {
        match self {
            Self::Xs => 10,
            Self::Sm => 12,
            Self::Md => 14,
            Self::Lg => 16,
            Self::Xl => 20,
            Self::Xxl => 24,
        }
    }

    /// Returns the semantic name of this size.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fern_theme::domain::tokens::typography::FontSize;
    ///
    /// assert_eq!(FontSize::Md.name(), "md");
    /// ```
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
            Self::Xxl => "xxl",
        }
    }

    /// Creates a font size from its name.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fern_theme::domain::tokens::typography::FontSize;
    ///
    /// assert_eq!(FontSize::from_name("md"), Some(FontSize::Md));
    /// assert_eq!(FontSize::from_name("invalid"), None);
    /// ```
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "xs" => Some(Self::Xs),
            "sm" => Some(Self::Sm),
            "md" => Some(Self::Md),
            "lg" => Some(Self::Lg),
            "xl" => Some(Self::Xl),
            "xxl" => Some(Self::Xxl),
            _ => None,
        }
    }

    /// Returns all font sizes in order from smallest to largest.
    #[must_use]
    pub const fn all() -> &'static [FontSize] {
        &[
            Self::Xs,
            Self::Sm,
            Self::Md,
            Self::Lg,
            Self::Xl,
            Self::Xxl,
        ]
    }
}

impl Default for FontSize {
    fn default() -> Self {
        Self::Md
    }
}

impl fmt::Display for FontSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}px", self.pixels())
    }
}

/// Complete font size scale with pixel values.
///
/// While [`FontSize`] provides semantic names, `FontSizeScale` allows
/// customizing the actual pixel values via configuration.
///
/// # Example
///
/// ```rust
/// use fern_theme::domain::tokens::typography::FontSizeScale;
///
/// let scale = FontSizeScale::default();
/// assert_eq!(scale.md, 14);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FontSizeScale {
    /// Extra-small size in pixels.
    #[serde(default = "default_xs")]
    pub xs: u16,
    /// Small size in pixels.
    #[serde(default = "default_sm")]
    pub sm: u16,
    /// Medium size in pixels.
    #[serde(default = "default_md")]
    pub md: u16,
    /// Large size in pixels.
    #[serde(default = "default_lg")]
    pub lg: u16,
    /// Extra-large size in pixels.
    #[serde(default = "default_xl")]
    pub xl: u16,
    /// Double extra-large size in pixels.
    #[serde(default = "default_xxl")]
    pub xxl: u16,
}

fn default_xs() -> u16 {
    FontSize::Xs.pixels()
}
fn default_sm() -> u16 {
    FontSize::Sm.pixels()
}
fn default_md() -> u16 {
    FontSize::Md.pixels()
}
fn default_lg() -> u16 {
    FontSize::Lg.pixels()
}
fn default_xl() -> u16 {
    FontSize::Xl.pixels()
}
fn default_xxl() -> u16 {
    FontSize::Xxl.pixels()
}

impl FontSizeScale {
    /// Creates a new font size scale with default values.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            xs: 10,
            sm: 12,
            md: 14,
            lg: 16,
            xl: 20,
            xxl: 24,
        }
    }

    /// Returns the pixel value for a given semantic size.
    #[must_use]
    pub const fn get(&self, size: FontSize) -> u16 {
        match size {
            FontSize::Xs => self.xs,
            FontSize::Sm => self.sm,
            FontSize::Md => self.md,
            FontSize::Lg => self.lg,
            FontSize::Xl => self.xl,
            FontSize::Xxl => self.xxl,
        }
    }
}

impl Default for FontSizeScale {
    fn default() -> Self {
        Self::new()
    }
}

/// Typography configuration combining fonts and sizes.
///
/// This struct contains all typography-related settings for the theme.
///
/// # Example
///
/// ```rust
/// use fern_theme::domain::tokens::typography::Typography;
///
/// let typography = Typography::default();
/// assert_eq!(typography.family.name(), "Inter");
/// assert_eq!(typography.size.md, 14);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Typography {
    /// Primary (UI) font family.
    #[serde(default = "FontFamily::default_primary")]
    pub family: FontFamily,
    /// Monospace font family.
    #[serde(default = "FontFamily::default_mono")]
    pub mono: FontFamily,
    /// Icon font family.
    #[serde(default = "FontFamily::default_icon")]
    pub icon: FontFamily,
    /// Font size scale.
    #[serde(default)]
    pub size: FontSizeScale,
}

impl Typography {
    /// Creates typography configuration with default values.
    #[must_use]
    pub fn new() -> Self {
        Self {
            family: FontFamily::default_primary(),
            mono: FontFamily::default_mono(),
            icon: FontFamily::default_icon(),
            size: FontSizeScale::default(),
        }
    }
}

impl Default for Typography {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_family_creation() {
        let font = FontFamily::new("Test Font");
        assert_eq!(font.name(), "Test Font");
    }

    #[test]
    fn font_family_defaults() {
        assert_eq!(FontFamily::default_primary().name(), "Inter");
        assert_eq!(
            FontFamily::default_mono().name(),
            "JetBrainsMono Nerd Font"
        );
        assert_eq!(FontFamily::default_icon().name(), "Material Symbols Rounded");
    }

    #[test]
    fn font_size_pixels() {
        assert_eq!(FontSize::Xs.pixels(), 10);
        assert_eq!(FontSize::Sm.pixels(), 12);
        assert_eq!(FontSize::Md.pixels(), 14);
        assert_eq!(FontSize::Lg.pixels(), 16);
        assert_eq!(FontSize::Xl.pixels(), 20);
        assert_eq!(FontSize::Xxl.pixels(), 24);
    }

    #[test]
    fn font_size_from_name() {
        assert_eq!(FontSize::from_name("md"), Some(FontSize::Md));
        assert_eq!(FontSize::from_name("XL"), Some(FontSize::Xl));
        assert_eq!(FontSize::from_name("invalid"), None);
    }

    #[test]
    fn font_size_scale_get() {
        let scale = FontSizeScale::default();
        assert_eq!(scale.get(FontSize::Md), 14);
        assert_eq!(scale.get(FontSize::Xl), 20);
    }

    #[test]
    fn typography_serialization() {
        let typography = Typography::default();
        let json = serde_json::to_string(&typography).unwrap();
        let restored: Typography = serde_json::from_str(&json).unwrap();
        assert_eq!(typography.family, restored.family);
    }
}
