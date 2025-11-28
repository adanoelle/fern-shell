//! # Scale-Based Spacing Tokens
//!
//! This module provides spacing tokens that follow a defined scale, preventing
//! arbitrary pixel values and ensuring visual consistency.
//!
//! ## Design Rationale
//!
//! Arbitrary spacing values create visual inconsistency:
//!
//! ```rust,ignore
//! // ❌ Bad: Arbitrary values
//! button.padding = 7;   // Why 7? Looks off.
//! card.margin = 13;     // Not on any scale.
//! ```
//!
//! Scale-based spacing ensures everything aligns:
//!
//! ```rust,ignore
//! // ✅ Good: Scale-based values
//! button.padding = SpacingSm::pixels();   // 8px, on 4px scale
//! card.margin = SpacingMd::pixels();      // 12px, on 4px scale
//! ```
//!
//! ## The 4px Base Unit
//!
//! All spacing in Fern is based on a 4px unit. This creates a consistent
//! rhythm and ensures elements align to a pixel grid.
//!
//! | Token | Multiplier | Pixels |
//! |-------|------------|--------|
//! | `SpacingXs` | 1× | 4px |
//! | `SpacingSm` | 2× | 8px |
//! | `SpacingMd` | 3× | 12px |
//! | `SpacingLg` | 4× | 16px |
//! | `SpacingXl` | 6× | 24px |
//!
//! ## Const Generics
//!
//! Spacing uses const generics to encode the multiplier in the type:
//!
//! ```rust
//! use fernctl::domain::tokens::spacing::*;
//!
//! // The multiplier is part of the type, not a runtime value
//! let _sm: Spacing<2> = Spacing::new();
//! assert_eq!(Spacing::<2>::pixels(), 8);
//!
//! // Type aliases provide semantic names
//! assert_eq!(SpacingSm::pixels(), 8);
//! ```
//!
//! ## Compile-Time Validation
//!
//! The type system ensures spacing values are always on the scale:
//!
//! ```rust
//! use fernctl::domain::tokens::spacing::*;
//!
//! // These are the only valid spacing values
//! let xs = SpacingXs::pixels();  // 4
//! let sm = SpacingSm::pixels();  // 8
//! let md = SpacingMd::pixels();  // 12
//! let lg = SpacingLg::pixels();  // 16
//! let xl = SpacingXl::pixels();  // 24
//!
//! // There's no way to express "7px" — it's not a type!
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use std::marker::PhantomData;

/// The base unit for spacing in pixels.
///
/// All spacing values are multiples of this base unit, creating a consistent
/// visual rhythm. The value 4 is chosen because:
///
/// - It's divisible by 2 (for half-spacing needs)
/// - It aligns well with common screen densities
/// - It's small enough for fine-grained control
/// - It's large enough to be visually meaningful
pub const SPACING_BASE: u16 = 4;

/// A spacing value that is a multiple of the base unit.
///
/// The multiplier `N` is encoded in the type, ensuring spacing values are
/// always on the scale. The actual pixel value is `N * SPACING_BASE`.
///
/// # Type Parameter
///
/// - `N` — The multiplier (e.g., 2 for 8px spacing)
///
/// # Memory Layout
///
/// `Spacing<N>` is a zero-sized type. It exists only at compile time to
/// encode the spacing value in the type system.
///
/// ```rust
/// use fernctl::domain::tokens::spacing::*;
/// use std::mem::size_of;
///
/// assert_eq!(size_of::<Spacing<2>>(), 0);
/// ```
///
/// # Example
///
/// ```rust
/// use fernctl::domain::tokens::spacing::*;
///
/// // Using const generics directly
/// let _spacing: Spacing<3> = Spacing::new();
/// assert_eq!(Spacing::<3>::pixels(), 12);
///
/// // Using type aliases
/// assert_eq!(SpacingMd::pixels(), 12);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Spacing<const N: u8> {
    _marker: PhantomData<()>,
}

impl<const N: u8> Spacing<N> {
    /// Creates a new spacing value.
    ///
    /// Since `Spacing` is a zero-sized type, this is a no-op at runtime.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::domain::tokens::spacing::Spacing;
    ///
    /// let spacing: Spacing<2> = Spacing::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    /// Returns the multiplier for this spacing value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::domain::tokens::spacing::*;
    ///
    /// assert_eq!(SpacingSm::multiplier(), 2);
    /// assert_eq!(SpacingLg::multiplier(), 4);
    /// ```
    #[must_use]
    pub const fn multiplier() -> u8 {
        N
    }

    /// Returns the spacing value in pixels.
    ///
    /// This is computed as `N * SPACING_BASE`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::domain::tokens::spacing::*;
    ///
    /// assert_eq!(SpacingXs::pixels(), 4);   // 1 * 4
    /// assert_eq!(SpacingSm::pixels(), 8);   // 2 * 4
    /// assert_eq!(SpacingMd::pixels(), 12);  // 3 * 4
    /// assert_eq!(SpacingLg::pixels(), 16);  // 4 * 4
    /// assert_eq!(SpacingXl::pixels(), 24);  // 6 * 4
    /// ```
    #[must_use]
    pub const fn pixels() -> u16 {
        SPACING_BASE * N as u16
    }

    /// Returns the semantic name for this spacing level, if it has one.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::domain::tokens::spacing::*;
    ///
    /// assert_eq!(SpacingSm::name(), Some("sm"));
    /// assert_eq!(Spacing::<7>::name(), None);  // Not a standard level
    /// ```
    #[must_use]
    pub const fn name() -> Option<&'static str> {
        match N {
            1 => Some("xs"),
            2 => Some("sm"),
            3 => Some("md"),
            4 => Some("lg"),
            6 => Some("xl"),
            _ => None,
        }
    }
}

impl<const N: u8> Default for Spacing<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: u8> fmt::Debug for Spacing<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match Self::name() {
            Some(name) => write!(f, "Spacing::{} ({}px)", name, Self::pixels()),
            None => write!(f, "Spacing<{}> ({}px)", N, Self::pixels()),
        }
    }
}

impl<const N: u8> fmt::Display for Spacing<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}px", Self::pixels())
    }
}

// Serialization: Spacing serializes to pixel value
impl<const N: u8> Serialize for Spacing<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Self::pixels().serialize(serializer)
    }
}

// Note: Deserialize is not implemented because spacing should be constructed
// from known const values, not arbitrary runtime values.

/// Extra-small spacing (4px).
///
/// Use for tight spaces where elements should feel closely related:
/// - Icon padding
/// - Inline element gaps
/// - Dense list items
///
/// # Example
///
/// ```rust
/// use fernctl::domain::tokens::spacing::SpacingXs;
///
/// assert_eq!(SpacingXs::pixels(), 4);
/// ```
pub type SpacingXs = Spacing<1>;

/// Small spacing (8px).
///
/// Use for standard small gaps:
/// - Button padding
/// - Input padding
/// - Small component margins
///
/// # Example
///
/// ```rust
/// use fernctl::domain::tokens::spacing::SpacingSm;
///
/// assert_eq!(SpacingSm::pixels(), 8);
/// ```
pub type SpacingSm = Spacing<2>;

/// Medium spacing (12px).
///
/// Use for standard component spacing:
/// - Card padding
/// - Section gaps
/// - Form field spacing
///
/// # Example
///
/// ```rust
/// use fernctl::domain::tokens::spacing::SpacingMd;
///
/// assert_eq!(SpacingMd::pixels(), 12);
/// ```
pub type SpacingMd = Spacing<3>;

/// Large spacing (16px).
///
/// Use for larger gaps between distinct elements:
/// - Panel margins
/// - Major section breaks
/// - Modal padding
///
/// # Example
///
/// ```rust
/// use fernctl::domain::tokens::spacing::SpacingLg;
///
/// assert_eq!(SpacingLg::pixels(), 16);
/// ```
pub type SpacingLg = Spacing<4>;

/// Extra-large spacing (24px).
///
/// Use for major layout spacing:
/// - Page margins
/// - Large section separations
/// - Header/footer padding
///
/// # Example
///
/// ```rust
/// use fernctl::domain::tokens::spacing::SpacingXl;
///
/// assert_eq!(SpacingXl::pixels(), 24);
/// ```
pub type SpacingXl = Spacing<6>;

/// A runtime spacing value for when const generics aren't suitable.
///
/// While `Spacing<N>` is preferred for compile-time safety, `SpacingValue`
/// is useful when the spacing level is determined at runtime (e.g., from
/// configuration).
///
/// # Example
///
/// ```rust
/// use fernctl::domain::tokens::spacing::SpacingValue;
///
/// let spacing = SpacingValue::from_name("md").unwrap();
/// assert_eq!(spacing.pixels(), 12);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpacingValue {
    /// Extra-small (4px)
    Xs,
    /// Small (8px)
    Sm,
    /// Medium (12px)
    Md,
    /// Large (16px)
    Lg,
    /// Extra-large (24px)
    Xl,
}

impl SpacingValue {
    /// Creates a spacing value from its name.
    ///
    /// # Arguments
    ///
    /// * `name` — One of "xs", "sm", "md", "lg", "xl"
    ///
    /// # Returns
    ///
    /// `Some(SpacingValue)` if the name is valid, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::domain::tokens::spacing::SpacingValue;
    ///
    /// assert_eq!(SpacingValue::from_name("sm"), Some(SpacingValue::Sm));
    /// assert_eq!(SpacingValue::from_name("invalid"), None);
    /// ```
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "xs" => Some(Self::Xs),
            "sm" => Some(Self::Sm),
            "md" => Some(Self::Md),
            "lg" => Some(Self::Lg),
            "xl" => Some(Self::Xl),
            _ => None,
        }
    }

    /// Returns the spacing value in pixels.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::domain::tokens::spacing::SpacingValue;
    ///
    /// assert_eq!(SpacingValue::Md.pixels(), 12);
    /// ```
    #[must_use]
    pub const fn pixels(&self) -> u16 {
        match self {
            Self::Xs => SpacingXs::pixels(),
            Self::Sm => SpacingSm::pixels(),
            Self::Md => SpacingMd::pixels(),
            Self::Lg => SpacingLg::pixels(),
            Self::Xl => SpacingXl::pixels(),
        }
    }

    /// Returns the name of this spacing level.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::domain::tokens::spacing::SpacingValue;
    ///
    /// assert_eq!(SpacingValue::Md.name(), "md");
    /// ```
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
        }
    }

    /// Returns all spacing values in order from smallest to largest.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::domain::tokens::spacing::SpacingValue;
    ///
    /// for spacing in SpacingValue::all() {
    ///     println!("{}: {}px", spacing.name(), spacing.pixels());
    /// }
    /// ```
    #[must_use]
    pub const fn all() -> &'static [SpacingValue] {
        &[Self::Xs, Self::Sm, Self::Md, Self::Lg, Self::Xl]
    }
}

impl Default for SpacingValue {
    fn default() -> Self {
        Self::Md
    }
}

impl fmt::Display for SpacingValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}px", self.pixels())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spacing_is_zero_sized() {
        assert_eq!(std::mem::size_of::<Spacing<2>>(), 0);
        assert_eq!(std::mem::size_of::<SpacingMd>(), 0);
    }

    #[test]
    fn spacing_values_correct() {
        assert_eq!(SpacingXs::pixels(), 4);
        assert_eq!(SpacingSm::pixels(), 8);
        assert_eq!(SpacingMd::pixels(), 12);
        assert_eq!(SpacingLg::pixels(), 16);
        assert_eq!(SpacingXl::pixels(), 24);
    }

    #[test]
    fn spacing_names() {
        assert_eq!(SpacingXs::name(), Some("xs"));
        assert_eq!(SpacingSm::name(), Some("sm"));
        assert_eq!(SpacingMd::name(), Some("md"));
        assert_eq!(SpacingLg::name(), Some("lg"));
        assert_eq!(SpacingXl::name(), Some("xl"));
        assert_eq!(Spacing::<7>::name(), None);
    }

    #[test]
    fn spacing_value_from_name() {
        assert_eq!(SpacingValue::from_name("xs"), Some(SpacingValue::Xs));
        assert_eq!(SpacingValue::from_name("SM"), Some(SpacingValue::Sm)); // Case insensitive
        assert_eq!(SpacingValue::from_name("invalid"), None);
    }

    #[test]
    fn spacing_serialization() {
        let sm = SpacingSm::new();
        let json = serde_json::to_string(&sm).unwrap();
        assert_eq!(json, "8");
    }

    #[test]
    fn spacing_value_serialization_roundtrip() {
        let original = SpacingValue::Md;
        let json = serde_json::to_string(&original).unwrap();
        let restored: SpacingValue = serde_json::from_str(&json).unwrap();
        assert_eq!(original, restored);
    }
}
