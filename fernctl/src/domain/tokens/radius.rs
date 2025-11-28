//! # Semantic Border Radius Tokens
//!
//! This module provides border radius tokens with semantic names and
//! component-specific validity constraints.
//!
//! ## Design Rationale
//!
//! Border radius affects the visual character of UI elements. Rather than
//! allowing arbitrary pixel values, we define semantic names that can be
//! customized while maintaining consistent meaning.
//!
//! ```rust,ignore
//! // ❌ Bad: Magic numbers
//! button.radius = 4;
//! card.radius = 8;
//! pill.radius = 9999;
//!
//! // ✅ Good: Semantic names
//! button.radius = RadiusSm;   // "Small" - for compact elements
//! card.radius = RadiusMd;     // "Medium" - for cards/modules
//! pill.radius = RadiusFull;   // "Full" - for pill shapes
//! ```
//!
//! ## Component Constraints
//!
//! Not all radii make sense for all components. A button with `RadiusFull`
//! looks like a pill — probably not intended. The type system prevents this:
//!
//! ```rust,ignore
//! // ButtonSpec requires R: ButtonRadius
//! struct ButtonSpec<R: ButtonRadius> { ... }
//!
//! // RadiusSm implements ButtonRadius
//! ButtonSpec::<RadiusSm>::new()  // ✓ Compiles
//!
//! // RadiusFull does NOT implement ButtonRadius
//! ButtonSpec::<RadiusFull>::new()  // ✗ Error!
//! ```
//!
//! ## Customizable Values
//!
//! While the semantic names are fixed, the actual pixel values can be
//! customized via configuration:
//!
//! ```toml
//! [appearance.radius]
//! sm = 4   # Could be changed to 2 for sharper corners
//! md = 8   # Could be changed to 12 for rounder corners
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Sealed Trait Pattern
// ============================================================================

mod private {
    pub trait Sealed {}
}

// ============================================================================
// Radius Semantic Traits
// ============================================================================

/// Marker trait for all radius semantics.
///
/// This is the base trait that all radius types implement. It is sealed,
/// meaning only types in this crate can implement it.
pub trait RadiusSemantic: private::Sealed + fmt::Debug + Clone + Copy + 'static {
    /// The default pixel value for this radius.
    const DEFAULT_PIXELS: u16;

    /// Returns the semantic name of this radius.
    fn name() -> &'static str;
}

/// Marker trait for radii valid on buttons.
///
/// Buttons should have subtle rounding — not too sharp, not pill-shaped.
/// This trait marks which radius semantics are appropriate for buttons.
///
/// # Implementors
///
/// - [`RadiusNone`] — Sharp corners (allowed but not recommended)
/// - [`RadiusSm`] — Small rounding (recommended)
/// - [`RadiusMd`] — Medium rounding (allowed)
///
/// [`RadiusLg`] and [`RadiusFull`] are NOT valid for buttons.
pub trait ButtonRadius: RadiusSemantic {}

/// Marker trait for radii valid on modules/cards.
///
/// Modules and cards benefit from more rounding than buttons.
///
/// # Implementors
///
/// - [`RadiusSm`] — Small (for dense layouts)
/// - [`RadiusMd`] — Medium (recommended)
/// - [`RadiusLg`] — Large (for softer appearance)
///
/// [`RadiusNone`] and [`RadiusFull`] are NOT valid for modules.
pub trait ModuleRadius: RadiusSemantic {}

/// Marker trait for radii valid on inputs.
///
/// Input fields should have consistent, subtle rounding.
///
/// # Implementors
///
/// - [`RadiusNone`] — Sharp (allowed)
/// - [`RadiusSm`] — Small (recommended)
/// - [`RadiusMd`] — Medium (allowed)
pub trait InputRadius: RadiusSemantic {}

// ============================================================================
// Radius Marker Types
// ============================================================================

/// No border radius (sharp corners).
///
/// Use when you need completely square corners. This is valid for most
/// components but rarely recommended — even 2px of rounding softens the UI.
///
/// # Default Value
///
/// Always 0px (cannot be customized).
///
/// # Example
///
/// ```rust
/// use fernctl::domain::tokens::radius::*;
///
/// assert_eq!(RadiusNone::DEFAULT_PIXELS, 0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct RadiusNone;

impl private::Sealed for RadiusNone {}
impl RadiusSemantic for RadiusNone {
    const DEFAULT_PIXELS: u16 = 0;
    fn name() -> &'static str {
        "none"
    }
}
impl ButtonRadius for RadiusNone {}
impl InputRadius for RadiusNone {}

/// Small border radius.
///
/// Use for compact elements that need subtle rounding:
/// - Buttons
/// - Input fields
/// - Small badges
/// - Inline elements
///
/// # Default Value
///
/// 4px (customizable via configuration).
///
/// # Example
///
/// ```rust
/// use fernctl::domain::tokens::radius::*;
///
/// assert_eq!(RadiusSm::DEFAULT_PIXELS, 4);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct RadiusSm;

impl private::Sealed for RadiusSm {}
impl RadiusSemantic for RadiusSm {
    const DEFAULT_PIXELS: u16 = 4;
    fn name() -> &'static str {
        "sm"
    }
}
impl ButtonRadius for RadiusSm {}
impl ModuleRadius for RadiusSm {}
impl InputRadius for RadiusSm {}

/// Medium border radius.
///
/// Use for standard components:
/// - Cards
/// - Modals
/// - Dropdown menus
/// - Tooltips
///
/// # Default Value
///
/// 8px (customizable via configuration).
///
/// # Example
///
/// ```rust
/// use fernctl::domain::tokens::radius::*;
///
/// assert_eq!(RadiusMd::DEFAULT_PIXELS, 8);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct RadiusMd;

impl private::Sealed for RadiusMd {}
impl RadiusSemantic for RadiusMd {
    const DEFAULT_PIXELS: u16 = 8;
    fn name() -> &'static str {
        "md"
    }
}
impl ButtonRadius for RadiusMd {}
impl ModuleRadius for RadiusMd {}
impl InputRadius for RadiusMd {}

/// Large border radius.
///
/// Use for prominent, standalone elements:
/// - Large cards
/// - Featured content
/// - Hero sections
///
/// # Default Value
///
/// 12px (customizable via configuration).
///
/// # Example
///
/// ```rust
/// use fernctl::domain::tokens::radius::*;
///
/// assert_eq!(RadiusLg::DEFAULT_PIXELS, 12);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct RadiusLg;

impl private::Sealed for RadiusLg {}
impl RadiusSemantic for RadiusLg {
    const DEFAULT_PIXELS: u16 = 12;
    fn name() -> &'static str {
        "lg"
    }
}
impl ModuleRadius for RadiusLg {}

/// Full border radius (pill shape).
///
/// Use when you want completely rounded ends:
/// - Pill badges
/// - Toggle switches
/// - Circular avatars (on square elements)
///
/// # Default Value
///
/// 9999px — A large value that ensures full rounding regardless of element size.
///
/// # Note
///
/// This radius is NOT valid for buttons, modules, or inputs because it creates
/// a pill shape that's usually unintended.
///
/// # Example
///
/// ```rust
/// use fernctl::domain::tokens::radius::*;
///
/// assert_eq!(RadiusFull::DEFAULT_PIXELS, 9999);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct RadiusFull;

impl private::Sealed for RadiusFull {}
impl RadiusSemantic for RadiusFull {
    const DEFAULT_PIXELS: u16 = 9999;
    fn name() -> &'static str {
        "full"
    }
}

// ============================================================================
// Runtime Radius Scale
// ============================================================================

/// A complete radius scale with customizable pixel values.
///
/// While the radius marker types ([`RadiusSm`], [`RadiusMd`], etc.) provide
/// compile-time type safety, `RadiusScale` provides runtime-configurable
/// values for the actual pixels.
///
/// # Default Values
///
/// ```rust
/// use fernctl::domain::tokens::radius::RadiusScale;
///
/// let scale = RadiusScale::default();
/// assert_eq!(scale.none, 0);
/// assert_eq!(scale.sm, 4);
/// assert_eq!(scale.md, 8);
/// assert_eq!(scale.lg, 12);
/// assert_eq!(scale.full, 9999);
/// ```
///
/// # Customization
///
/// Users can customize the pixel values via configuration:
///
/// ```toml
/// [appearance.radius]
/// sm = 2    # Sharper corners
/// md = 6
/// lg = 10
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RadiusScale {
    /// No radius (always 0).
    #[serde(default)]
    pub none: u16,
    /// Small radius (default 4px).
    #[serde(default = "default_sm")]
    pub sm: u16,
    /// Medium radius (default 8px).
    #[serde(default = "default_md")]
    pub md: u16,
    /// Large radius (default 12px).
    #[serde(default = "default_lg")]
    pub lg: u16,
    /// Full radius (always 9999).
    #[serde(default = "default_full")]
    pub full: u16,
}

fn default_sm() -> u16 {
    RadiusSm::DEFAULT_PIXELS
}
fn default_md() -> u16 {
    RadiusMd::DEFAULT_PIXELS
}
fn default_lg() -> u16 {
    RadiusLg::DEFAULT_PIXELS
}
fn default_full() -> u16 {
    RadiusFull::DEFAULT_PIXELS
}

impl RadiusScale {
    /// Creates a new radius scale with default values.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            none: RadiusNone::DEFAULT_PIXELS,
            sm: RadiusSm::DEFAULT_PIXELS,
            md: RadiusMd::DEFAULT_PIXELS,
            lg: RadiusLg::DEFAULT_PIXELS,
            full: RadiusFull::DEFAULT_PIXELS,
        }
    }

    /// Returns the pixel value for a given semantic radius.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::domain::tokens::radius::*;
    ///
    /// let scale = RadiusScale::default();
    /// assert_eq!(scale.get::<RadiusMd>(), 8);
    /// ```
    #[must_use]
    pub fn get<R: RadiusSemantic>(&self) -> u16 {
        match R::name() {
            "none" => self.none,
            "sm" => self.sm,
            "md" => self.md,
            "lg" => self.lg,
            "full" => self.full,
            _ => R::DEFAULT_PIXELS,
        }
    }

    /// Returns the pixel value for a button radius.
    ///
    /// This is a convenience method that defaults to [`RadiusSm`].
    #[must_use]
    pub const fn button(&self) -> u16 {
        self.sm
    }

    /// Returns the pixel value for a module/card radius.
    ///
    /// This is a convenience method that defaults to [`RadiusMd`].
    #[must_use]
    pub const fn module(&self) -> u16 {
        self.md
    }

    /// Returns the pixel value for an input radius.
    ///
    /// This is a convenience method that defaults to [`RadiusSm`].
    #[must_use]
    pub const fn input(&self) -> u16 {
        self.sm
    }
}

impl Default for RadiusScale {
    fn default() -> Self {
        Self::new()
    }
}

/// A runtime radius value for when const generics aren't suitable.
///
/// Use when the radius level is determined at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RadiusValue {
    /// No radius
    None,
    /// Small radius
    Sm,
    /// Medium radius
    Md,
    /// Large radius
    Lg,
    /// Full radius
    Full,
}

impl RadiusValue {
    /// Creates a radius value from its name.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "none" => Some(Self::None),
            "sm" => Some(Self::Sm),
            "md" => Some(Self::Md),
            "lg" => Some(Self::Lg),
            "full" => Some(Self::Full),
            _ => None,
        }
    }

    /// Returns the default pixel value for this radius.
    #[must_use]
    pub const fn default_pixels(&self) -> u16 {
        match self {
            Self::None => RadiusNone::DEFAULT_PIXELS,
            Self::Sm => RadiusSm::DEFAULT_PIXELS,
            Self::Md => RadiusMd::DEFAULT_PIXELS,
            Self::Lg => RadiusLg::DEFAULT_PIXELS,
            Self::Full => RadiusFull::DEFAULT_PIXELS,
        }
    }

    /// Returns the name of this radius level.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Full => "full",
        }
    }

    /// Returns all radius values.
    #[must_use]
    pub const fn all() -> &'static [RadiusValue] {
        &[Self::None, Self::Sm, Self::Md, Self::Lg, Self::Full]
    }
}

impl Default for RadiusValue {
    fn default() -> Self {
        Self::Md
    }
}

impl fmt::Display for RadiusValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radius_defaults() {
        assert_eq!(RadiusNone::DEFAULT_PIXELS, 0);
        assert_eq!(RadiusSm::DEFAULT_PIXELS, 4);
        assert_eq!(RadiusMd::DEFAULT_PIXELS, 8);
        assert_eq!(RadiusLg::DEFAULT_PIXELS, 12);
        assert_eq!(RadiusFull::DEFAULT_PIXELS, 9999);
    }

    #[test]
    fn radius_scale_get() {
        let scale = RadiusScale::default();
        assert_eq!(scale.get::<RadiusSm>(), 4);
        assert_eq!(scale.get::<RadiusMd>(), 8);
    }

    #[test]
    fn radius_scale_serialization() {
        let scale = RadiusScale::default();
        let json = serde_json::to_string(&scale).unwrap();
        let restored: RadiusScale = serde_json::from_str(&json).unwrap();
        assert_eq!(scale, restored);
    }

    #[test]
    fn radius_value_from_name() {
        assert_eq!(RadiusValue::from_name("sm"), Some(RadiusValue::Sm));
        assert_eq!(RadiusValue::from_name("MD"), Some(RadiusValue::Md));
        assert_eq!(RadiusValue::from_name("invalid"), None);
    }

    // Compile-time test: These should compile
    fn _button_radius_sm(_: impl ButtonRadius) {}
    fn _module_radius_md(_: impl ModuleRadius) {}

    #[test]
    fn trait_implementations() {
        // ButtonRadius: None, Sm, Md
        _button_radius_sm(RadiusNone);
        _button_radius_sm(RadiusSm);
        _button_radius_sm(RadiusMd);

        // ModuleRadius: Sm, Md, Lg
        _module_radius_md(RadiusSm);
        _module_radius_md(RadiusMd);
        _module_radius_md(RadiusLg);
    }

    // These would NOT compile (which is the point):
    // _button_radius_sm(RadiusLg);   // RadiusLg doesn't implement ButtonRadius
    // _button_radius_sm(RadiusFull); // RadiusFull doesn't implement ButtonRadius
    // _module_radius_md(RadiusNone); // RadiusNone doesn't implement ModuleRadius
}
