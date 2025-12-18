//! # Type-Safe Color Tokens
//!
//! This module provides color tokens with semantic roles encoded in the type
//! system. The compiler ensures colors are used appropriately — you cannot
//! accidentally use a text color as a background.
//!
//! ## Design Rationale
//!
//! In a traditional design system, colors are just values:
//!
//! ```rust,ignore
//! let surface = "#313244";
//! let text = "#cdd6f4";
//!
//! // Oops! Using text color as background — compiles but looks wrong
//! button.background = text;
//! ```
//!
//! With type-safe colors, the compiler catches this mistake:
//!
//! ```rust,ignore
//! let surface: ColorToken<Surface> = ColorToken::from_hex("#313244")?;
//! let text: ColorToken<Foreground> = ColorToken::from_hex("#cdd6f4")?;
//!
//! // This function only accepts background colors
//! fn set_background(color: impl BackgroundRole) { /* ... */ }
//!
//! set_background(surface);  // ✓ Compiles — Surface is a BackgroundRole
//! set_background(text);     // ✗ Error! — Foreground is not a BackgroundRole
//! ```
//!
//! ## Color Roles
//!
//! Colors are categorized by their semantic role:
//!
//! | Role | Trait | Example Uses |
//! |------|-------|--------------|
//! | Background | [`BackgroundRole`] | Panel backgrounds, button fills |
//! | Foreground | [`ForegroundRole`] | Text, icons |
//! | Accent | [`AccentRole`] | Focus rings, highlights |
//! | Status | [`StatusRole`] | Error, warning, success |
//!
//! ## Creating Colors
//!
//! ```rust
//! use fernctl::domain::tokens::color::*;
//!
//! // From hex string
//! let color = ColorToken::<Surface>::from_hex("#313244").unwrap();
//!
//! // From RGB values
//! let color = ColorToken::<Surface>::from_rgb(0x31, 0x32, 0x44);
//!
//! // With alpha
//! let color = ColorToken::<Surface>::from_rgba(0x31, 0x32, 0x44, 0x80);
//! ```
//!
//! ## Zero-Cost Abstraction
//!
//! The type parameter uses [`PhantomData`] and adds no runtime overhead.
//! A `ColorToken<Surface>` has the exact same memory layout as four `u8`s.

use crate::error::ConfigError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::marker::PhantomData;

// ============================================================================
// Sealed Trait Pattern
// ============================================================================

/// Private module to seal the role traits.
///
/// This prevents external crates from implementing role traits, ensuring
/// the type system guarantees remain sound.
mod private {
    /// Sealed trait that cannot be implemented outside this crate.
    pub trait Sealed {}
}

// ============================================================================
// Role Marker Traits
// ============================================================================

/// Marker trait for all color roles.
///
/// This is the base trait that all specific role traits extend. It is sealed,
/// meaning only types defined in this crate can implement it.
///
/// # Implementors
///
/// - [`Background`], [`Surface`], [`SurfaceHover`] — Background roles
/// - [`Foreground`], [`ForegroundDim`] — Foreground (text) roles
/// - [`Accent`] — Accent color role
/// - [`Error`], [`Warning`], [`Success`], [`Info`] — Status roles
pub trait ColorRole: private::Sealed + fmt::Debug + Clone + Copy + 'static {
    /// Returns the name of this role for debugging and serialization.
    fn role_name() -> &'static str;
}

/// Marker trait for colors that can be used as backgrounds.
///
/// Background colors are used for:
/// - Panel and window backgrounds
/// - Button fills
/// - Card surfaces
/// - Input field backgrounds
///
/// # Implementors
///
/// - [`Background`] — Primary background color
/// - [`Surface`] — Elevated surface color
/// - [`SurfaceHover`] — Hover state for surfaces
pub trait BackgroundRole: ColorRole {}

/// Marker trait for colors that can be used as foreground (text/icons).
///
/// Foreground colors are used for:
/// - Body text
/// - Icons
/// - Labels
/// - Any content that appears on top of a background
///
/// # Implementors
///
/// - [`Foreground`] — Primary text color
/// - [`ForegroundDim`] — Secondary/muted text color
pub trait ForegroundRole: ColorRole {}

/// Marker trait for accent colors.
///
/// Accent colors are used for:
/// - Focus indicators
/// - Active states
/// - Links
/// - Highlighted elements
///
/// # Implementors
///
/// - [`Accent`] — The primary accent color
pub trait AccentRole: ColorRole {}

/// Marker trait for status indication colors.
///
/// Status colors communicate state to the user:
/// - Errors and failures
/// - Warnings and cautions
/// - Success and confirmations
/// - Informational messages
///
/// # Implementors
///
/// - [`Error`] — Error/failure states
/// - [`Warning`] — Warning/caution states
/// - [`Success`] — Success/confirmation states
/// - [`Info`] — Informational states
pub trait StatusRole: ColorRole {}

// ============================================================================
// Role Marker Types
// ============================================================================

/// Primary background color role.
///
/// Used for the main background of panels, windows, and the shell itself.
/// This is typically the darkest color in dark themes or lightest in light themes.
///
/// # Example
///
/// ```rust
/// use fernctl::domain::tokens::color::*;
///
/// let bg: ColorToken<Background> = ColorToken::from_hex("#1e1e2e").unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Background;

impl private::Sealed for Background {}
impl ColorRole for Background {
    fn role_name() -> &'static str {
        "background"
    }
}
impl BackgroundRole for Background {}

/// Elevated surface color role.
///
/// Used for elements that appear elevated above the background, such as:
/// - Cards
/// - Modals
/// - Dropdown menus
/// - Tooltips
///
/// # Example
///
/// ```rust
/// use fernctl::domain::tokens::color::*;
///
/// let surface: ColorToken<Surface> = ColorToken::from_hex("#313244").unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Surface;

impl private::Sealed for Surface {}
impl ColorRole for Surface {
    fn role_name() -> &'static str {
        "surface"
    }
}
impl BackgroundRole for Surface {}

/// Hover state for surfaces.
///
/// Used when the user hovers over interactive surface elements.
/// Should be visually distinct from [`Surface`] but not jarring.
///
/// # Example
///
/// ```rust
/// use fernctl::domain::tokens::color::*;
///
/// let hover: ColorToken<SurfaceHover> = ColorToken::from_hex("#45475a").unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SurfaceHover;

impl private::Sealed for SurfaceHover {}
impl ColorRole for SurfaceHover {
    fn role_name() -> &'static str {
        "surface_hover"
    }
}
impl BackgroundRole for SurfaceHover {}

/// Primary foreground (text) color role.
///
/// Used for primary text content and icons. Should have sufficient contrast
/// against background colors for readability.
///
/// # Example
///
/// ```rust
/// use fernctl::domain::tokens::color::*;
///
/// let text: ColorToken<Foreground> = ColorToken::from_hex("#cdd6f4").unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Foreground;

impl private::Sealed for Foreground {}
impl ColorRole for Foreground {
    fn role_name() -> &'static str {
        "foreground"
    }
}
impl ForegroundRole for Foreground {}

/// Dimmed/secondary foreground color role.
///
/// Used for secondary text, placeholders, and less important content.
/// Should be readable but visually subordinate to primary foreground.
///
/// # Example
///
/// ```rust
/// use fernctl::domain::tokens::color::*;
///
/// let dim: ColorToken<ForegroundDim> = ColorToken::from_hex("#a6adc8").unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ForegroundDim;

impl private::Sealed for ForegroundDim {}
impl ColorRole for ForegroundDim {
    fn role_name() -> &'static str {
        "foreground_dim"
    }
}
impl ForegroundRole for ForegroundDim {}

/// Accent color role.
///
/// Used for focus indicators, active states, links, and branded elements.
/// This is typically the most vibrant color in the palette.
///
/// # Example
///
/// ```rust
/// use fernctl::domain::tokens::color::*;
///
/// let accent: ColorToken<Accent> = ColorToken::from_hex("#89b4fa").unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Accent;

impl private::Sealed for Accent {}
impl ColorRole for Accent {
    fn role_name() -> &'static str {
        "accent"
    }
}
impl AccentRole for Accent {}

/// Error status color role.
///
/// Used to indicate errors, failures, and destructive actions.
///
/// # Example
///
/// ```rust
/// use fernctl::domain::tokens::color::*;
///
/// let error: ColorToken<Error> = ColorToken::from_hex("#f38ba8").unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Error;

impl private::Sealed for Error {}
impl ColorRole for Error {
    fn role_name() -> &'static str {
        "error"
    }
}
impl StatusRole for Error {}

/// Warning status color role.
///
/// Used to indicate warnings, cautions, and potentially dangerous actions.
///
/// # Example
///
/// ```rust
/// use fernctl::domain::tokens::color::*;
///
/// let warning: ColorToken<Warning> = ColorToken::from_hex("#f9e2af").unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Warning;

impl private::Sealed for Warning {}
impl ColorRole for Warning {
    fn role_name() -> &'static str {
        "warning"
    }
}
impl StatusRole for Warning {}

/// Success status color role.
///
/// Used to indicate success, completion, and positive actions.
///
/// # Example
///
/// ```rust
/// use fernctl::domain::tokens::color::*;
///
/// let success: ColorToken<Success> = ColorToken::from_hex("#a6e3a1").unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Success;

impl private::Sealed for Success {}
impl ColorRole for Success {
    fn role_name() -> &'static str {
        "success"
    }
}
impl StatusRole for Success {}

/// Info status color role.
///
/// Used for informational messages and neutral highlights.
///
/// # Example
///
/// ```rust
/// use fernctl::domain::tokens::color::*;
///
/// let info: ColorToken<Info> = ColorToken::from_hex("#89dceb").unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Info;

impl private::Sealed for Info {}
impl ColorRole for Info {
    fn role_name() -> &'static str {
        "info"
    }
}
impl StatusRole for Info {}

// ============================================================================
// ColorToken Implementation
// ============================================================================

/// A color token with its semantic role encoded in the type.
///
/// `ColorToken<Role>` represents an RGBA color value along with its semantic
/// meaning in the design system. The role is tracked at compile time via the
/// type parameter, enabling the compiler to prevent misuse.
///
/// # Type Parameter
///
/// The `Role` parameter must implement [`ColorRole`] and determines how the
/// color can be used:
///
/// - `ColorToken<Surface>` — Can be used where [`BackgroundRole`] is expected
/// - `ColorToken<Foreground>` — Can be used where [`ForegroundRole`] is expected
///
/// # Memory Layout
///
/// Despite the type parameter, `ColorToken` has the same size as four bytes:
///
/// ```rust
/// use fernctl::domain::tokens::color::*;
/// use std::mem::size_of;
///
/// assert_eq!(size_of::<ColorToken<Surface>>(), 4);
/// ```
///
/// This is because `PhantomData<Role>` is a zero-sized type.
///
/// # Creating Colors
///
/// ```rust
/// use fernctl::domain::tokens::color::*;
///
/// // From hex string (most common)
/// let color = ColorToken::<Surface>::from_hex("#313244").unwrap();
///
/// // From RGB values
/// let color = ColorToken::<Surface>::from_rgb(0x31, 0x32, 0x44);
///
/// // From RGBA values
/// let color = ColorToken::<Surface>::from_rgba(0x31, 0x32, 0x44, 0xFF);
/// ```
///
/// # Converting to Output Formats
///
/// ```rust
/// use fernctl::domain::tokens::color::*;
///
/// let color = ColorToken::<Surface>::from_hex("#313244").unwrap();
///
/// assert_eq!(color.to_hex(), "#313244");
/// assert_eq!(color.to_rgb_tuple(), (0x31, 0x32, 0x44));
/// assert_eq!(color.to_css_rgb(), "rgb(49, 50, 68)");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorToken<Role: ColorRole> {
    /// Red component (0-255).
    r: u8,
    /// Green component (0-255).
    g: u8,
    /// Blue component (0-255).
    b: u8,
    /// Alpha component (0-255, where 255 is fully opaque).
    a: u8,
    /// Marker for the color's semantic role.
    _role: PhantomData<Role>,
}

impl<Role: ColorRole> ColorToken<Role> {
    /// Creates a new color token from RGB values (fully opaque).
    ///
    /// # Arguments
    ///
    /// * `r` — Red component (0-255)
    /// * `g` — Green component (0-255)
    /// * `b` — Blue component (0-255)
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::domain::tokens::color::*;
    ///
    /// let color = ColorToken::<Surface>::from_rgb(0x31, 0x32, 0x44);
    /// assert_eq!(color.alpha(), 255);
    /// ```
    #[must_use]
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,
            g,
            b,
            a: 255,
            _role: PhantomData,
        }
    }

    /// Creates a new color token from RGBA values.
    ///
    /// # Arguments
    ///
    /// * `r` — Red component (0-255)
    /// * `g` — Green component (0-255)
    /// * `b` — Blue component (0-255)
    /// * `a` — Alpha component (0-255, where 255 is fully opaque)
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::domain::tokens::color::*;
    ///
    /// let color = ColorToken::<Surface>::from_rgba(0x31, 0x32, 0x44, 0x80);
    /// assert_eq!(color.alpha(), 0x80);
    /// ```
    #[must_use]
    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r,
            g,
            b,
            a,
            _role: PhantomData,
        }
    }

    /// Creates a color token from a hex string.
    ///
    /// Supports the following formats:
    /// - `#RGB` — 3-digit shorthand (expands to 6-digit)
    /// - `#RRGGBB` — 6-digit hex (opaque)
    /// - `#RRGGBBAA` — 8-digit hex with alpha
    ///
    /// The `#` prefix is required.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::InvalidColor`] if:
    /// - The string doesn't start with `#`
    /// - The length is not 4, 7, or 9 characters
    /// - Any character is not a valid hex digit
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::domain::tokens::color::*;
    ///
    /// // 6-digit hex
    /// let color = ColorToken::<Surface>::from_hex("#313244").unwrap();
    ///
    /// // 3-digit shorthand
    /// let white = ColorToken::<Foreground>::from_hex("#fff").unwrap();
    ///
    /// // 8-digit with alpha
    /// let transparent = ColorToken::<Surface>::from_hex("#31324480").unwrap();
    ///
    /// // Invalid
    /// assert!(ColorToken::<Surface>::from_hex("313244").is_err());  // Missing #
    /// assert!(ColorToken::<Surface>::from_hex("#gg0000").is_err()); // Invalid hex
    /// ```
    pub fn from_hex(hex: &str) -> Result<Self, ConfigError> {
        parse_hex_color(hex).ok_or_else(|| ConfigError::InvalidColor {
            value: hex.to_string(),
            span: None,
            source_code: None,
        })
    }

    /// Returns the red component (0-255).
    #[must_use]
    pub const fn red(&self) -> u8 {
        self.r
    }

    /// Returns the green component (0-255).
    #[must_use]
    pub const fn green(&self) -> u8 {
        self.g
    }

    /// Returns the blue component (0-255).
    #[must_use]
    pub const fn blue(&self) -> u8 {
        self.b
    }

    /// Returns the alpha component (0-255, where 255 is fully opaque).
    #[must_use]
    pub const fn alpha(&self) -> u8 {
        self.a
    }

    /// Returns `true` if the color is fully opaque (alpha = 255).
    #[must_use]
    pub const fn is_opaque(&self) -> bool {
        self.a == 255
    }

    /// Returns the color as an RGB tuple.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::domain::tokens::color::*;
    ///
    /// let color = ColorToken::<Surface>::from_hex("#313244").unwrap();
    /// assert_eq!(color.to_rgb_tuple(), (0x31, 0x32, 0x44));
    /// ```
    #[must_use]
    pub const fn to_rgb_tuple(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    /// Returns the color as an RGBA tuple.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::domain::tokens::color::*;
    ///
    /// let color = ColorToken::<Surface>::from_rgba(0x31, 0x32, 0x44, 0x80);
    /// assert_eq!(color.to_rgba_tuple(), (0x31, 0x32, 0x44, 0x80));
    /// ```
    #[must_use]
    pub const fn to_rgba_tuple(&self) -> (u8, u8, u8, u8) {
        (self.r, self.g, self.b, self.a)
    }

    /// Returns the color as a hex string.
    ///
    /// Returns 6-digit format for opaque colors, 8-digit for transparent.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::domain::tokens::color::*;
    ///
    /// let opaque = ColorToken::<Surface>::from_hex("#313244").unwrap();
    /// assert_eq!(opaque.to_hex(), "#313244");
    ///
    /// let transparent = ColorToken::<Surface>::from_rgba(0x31, 0x32, 0x44, 0x80);
    /// assert_eq!(transparent.to_hex(), "#31324480");
    /// ```
    #[must_use]
    pub fn to_hex(&self) -> String {
        if self.is_opaque() {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }

    /// Returns the color as a CSS `rgb()` or `rgba()` function.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::domain::tokens::color::*;
    ///
    /// let opaque = ColorToken::<Surface>::from_rgb(49, 50, 68);
    /// assert_eq!(opaque.to_css_rgb(), "rgb(49, 50, 68)");
    ///
    /// let transparent = ColorToken::<Surface>::from_rgba(49, 50, 68, 128);
    /// assert_eq!(transparent.to_css_rgb(), "rgba(49, 50, 68, 0.50)");
    /// ```
    #[must_use]
    pub fn to_css_rgb(&self) -> String {
        if self.is_opaque() {
            format!("rgb({}, {}, {})", self.r, self.g, self.b)
        } else {
            let alpha = f32::from(self.a) / 255.0;
            format!("rgba({}, {}, {}, {:.2})", self.r, self.g, self.b, alpha)
        }
    }

    /// Returns the semantic role name of this color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::domain::tokens::color::*;
    ///
    /// let color = ColorToken::<Surface>::from_rgb(0, 0, 0);
    /// assert_eq!(color.role_name(), "surface");
    /// ```
    #[must_use]
    pub fn role_name(&self) -> &'static str {
        Role::role_name()
    }

    /// Creates a new color with a different alpha value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::domain::tokens::color::*;
    ///
    /// let opaque = ColorToken::<Surface>::from_hex("#313244").unwrap();
    /// let transparent = opaque.with_alpha(128);
    /// assert_eq!(transparent.alpha(), 128);
    /// ```
    #[must_use]
    pub const fn with_alpha(&self, alpha: u8) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: alpha,
            _role: PhantomData,
        }
    }
}

impl<Role: ColorRole> fmt::Debug for ColorToken<Role> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ColorToken")
            .field("role", &Role::role_name())
            .field("hex", &self.to_hex())
            .finish()
    }
}

impl<Role: ColorRole> fmt::Display for ColorToken<Role> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl<Role: ColorRole> Default for ColorToken<Role> {
    /// Returns a default color (black, fully opaque).
    fn default() -> Self {
        Self::from_rgb(0, 0, 0)
    }
}

// Serialization: Colors serialize to hex strings
impl<Role: ColorRole> Serialize for ColorToken<Role> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_hex().serialize(serializer)
    }
}

impl<'de, Role: ColorRole> Deserialize<'de> for ColorToken<Role> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hex = String::deserialize(deserializer)?;
        Self::from_hex(&hex).map_err(serde::de::Error::custom)
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parses a hex color string into RGBA components.
///
/// Supports `#RGB`, `#RRGGBB`, and `#RRGGBBAA` formats.
fn parse_hex_color<Role: ColorRole>(hex: &str) -> Option<ColorToken<Role>> {
    let hex = hex.strip_prefix('#')?;

    match hex.len() {
        // #RGB shorthand
        3 => {
            let r = parse_hex_digit(hex.as_bytes().first()?)?;
            let g = parse_hex_digit(hex.as_bytes().get(1)?)?;
            let b = parse_hex_digit(hex.as_bytes().get(2)?)?;
            // Expand: F -> FF
            Some(ColorToken::from_rgb(r | (r << 4), g | (g << 4), b | (b << 4)))
        }
        // #RRGGBB
        6 => {
            let r = parse_hex_byte(&hex[0..2])?;
            let g = parse_hex_byte(&hex[2..4])?;
            let b = parse_hex_byte(&hex[4..6])?;
            Some(ColorToken::from_rgb(r, g, b))
        }
        // #RRGGBBAA
        8 => {
            let r = parse_hex_byte(&hex[0..2])?;
            let g = parse_hex_byte(&hex[2..4])?;
            let b = parse_hex_byte(&hex[4..6])?;
            let a = parse_hex_byte(&hex[6..8])?;
            Some(ColorToken::from_rgba(r, g, b, a))
        }
        _ => None,
    }
}

/// Parses a single hex digit.
fn parse_hex_digit(byte: &u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Parses a two-character hex byte.
fn parse_hex_byte(s: &str) -> Option<u8> {
    u8::from_str_radix(s, 16).ok()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_token_size() {
        assert_eq!(std::mem::size_of::<ColorToken<Surface>>(), 4);
    }

    #[test]
    fn from_rgb() {
        let color = ColorToken::<Surface>::from_rgb(0x31, 0x32, 0x44);
        assert_eq!(color.red(), 0x31);
        assert_eq!(color.green(), 0x32);
        assert_eq!(color.blue(), 0x44);
        assert_eq!(color.alpha(), 255);
        assert!(color.is_opaque());
    }

    #[test]
    fn from_rgba() {
        let color = ColorToken::<Surface>::from_rgba(0x31, 0x32, 0x44, 0x80);
        assert_eq!(color.alpha(), 0x80);
        assert!(!color.is_opaque());
    }

    #[test]
    fn from_hex_6_digit() {
        let color = ColorToken::<Surface>::from_hex("#313244").unwrap();
        assert_eq!(color.red(), 0x31);
        assert_eq!(color.green(), 0x32);
        assert_eq!(color.blue(), 0x44);
    }

    #[test]
    fn from_hex_3_digit() {
        let color = ColorToken::<Foreground>::from_hex("#fff").unwrap();
        assert_eq!(color.red(), 255);
        assert_eq!(color.green(), 255);
        assert_eq!(color.blue(), 255);
    }

    #[test]
    fn from_hex_8_digit() {
        let color = ColorToken::<Surface>::from_hex("#31324480").unwrap();
        assert_eq!(color.alpha(), 0x80);
    }

    #[test]
    fn from_hex_invalid() {
        assert!(ColorToken::<Surface>::from_hex("313244").is_err()); // Missing #
        assert!(ColorToken::<Surface>::from_hex("#gg0000").is_err()); // Invalid hex
        assert!(ColorToken::<Surface>::from_hex("#12345").is_err()); // Wrong length
    }

    #[test]
    fn to_hex_roundtrip() {
        let original = "#89b4fa";
        let color = ColorToken::<Accent>::from_hex(original).unwrap();
        assert_eq!(color.to_hex(), original);
    }

    #[test]
    fn serialization_roundtrip() {
        let color = ColorToken::<Surface>::from_hex("#313244").unwrap();
        let json = serde_json::to_string(&color).unwrap();
        assert_eq!(json, "\"#313244\"");

        let restored: ColorToken<Surface> = serde_json::from_str(&json).unwrap();
        assert_eq!(color, restored);
    }

    #[test]
    fn role_name() {
        let surface = ColorToken::<Surface>::from_rgb(0, 0, 0);
        assert_eq!(surface.role_name(), "surface");

        let foreground = ColorToken::<Foreground>::from_rgb(0, 0, 0);
        assert_eq!(foreground.role_name(), "foreground");
    }
}
