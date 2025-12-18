//! # Design Tokens
//!
//! Design tokens are the atomic building blocks of the Fern design system.
//! They encode visual design decisions (colors, spacing, radii) as type-safe
//! Rust values.
//!
//! ## What Are Design Tokens?
//!
//! Design tokens are named values that store visual design attributes. Instead
//! of scattering magic values throughout code:
//!
//! ```rust,ignore
//! // ❌ Bad: Magic values everywhere
//! button.background = "#313244";
//! button.padding = 8;
//! button.radius = 4;
//! ```
//!
//! We use semantic tokens:
//!
//! ```rust,ignore
//! // ✅ Good: Semantic tokens
//! button.background = theme.surface;      // ColorToken<Surface>
//! button.padding = SpacingSm::pixels();   // 8px from scale
//! button.radius = theme.radius.button();  // Semantic radius
//! ```
//!
//! ## Token Categories
//!
//! | Category | Module | Description |
//! |----------|--------|-------------|
//! | Color | [`color`] | RGBA colors with semantic roles |
//! | Spacing | [`spacing`] | Scale-based spacing values |
//! | Radius | [`radius`] | Border radius with semantic names |
//! | Typography | [`typography`] | Font families and size scales |
//!
//! ## Type Safety
//!
//! Tokens use Rust's type system to prevent misuse:
//!
//! - **Color roles**: Can't use a foreground color as a background
//! - **Spacing scale**: Can't use arbitrary pixel values
//! - **Radius semantics**: Components can only use valid radii
//!
//! See each submodule for detailed examples.

pub mod color;
pub mod radius;
pub mod spacing;
pub mod typography;

// Re-export all token types
pub use color::*;
pub use radius::*;
pub use spacing::*;
pub use typography::*;
