//! # Domain Layer — The Core Design Language
//!
//! This module contains the pure business logic of the fernctl design system.
//! It has **zero external dependencies** beyond the standard library and
//! defines the fundamental types that make up the Fern design language.
//!
//! ## Module Organization
//!
//! ```text
//! domain/
//! ├── tokens/         — Primitive design tokens
//! │   ├── color       — Type-safe colors with semantic roles
//! │   ├── spacing     — Scale-based spacing values
//! │   ├── radius      — Semantic border radius
//! │   └── typography  — Font families and sizes
//! │
//! ├── components/     — Component specifications (future)
//! │   ├── button      — Button visual spec
//! │   └── module      — Bar module spec
//! │
//! └── theme           — Complete theme combining all tokens
//! ```
//!
//! ## Design Philosophy
//!
//! The domain layer embodies several key principles:
//!
//! ### 1. Type-Level Correctness
//!
//! Invalid states are **unrepresentable**. You cannot create a button with
//! an invalid radius or assign a foreground color to a background property
//! — the type system prevents it at compile time.
//!
//! ```rust,ignore
//! // This won't compile — RadiusFull doesn't implement ButtonRadius
//! let button: ButtonSpec<RadiusFull, ...> = ...;
//! ```
//!
//! ### 2. Zero Runtime Cost
//!
//! Marker traits and `PhantomData` exist only at compile time. The runtime
//! representation of `ColorToken<Surface>` is identical to raw RGB bytes.
//!
//! ### 3. Pure and Testable
//!
//! No I/O, no side effects, no global state. Every function is deterministic
//! and can be tested in isolation.
//!
//! ## Usage
//!
//! Import the prelude for common types:
//!
//! ```rust,ignore
//! use fern_theme::domain::prelude::*;
//!
//! let surface: ColorToken<Surface> = ColorToken::from_hex("#313244")?;
//! let spacing = SpacingMd::pixels(); // 12
//! ```

pub mod theme;
pub mod tokens;
pub mod user_config;

// Future modules
// pub mod components;

/// Convenient re-exports of domain types.
pub mod prelude {
    pub use super::theme::Theme;
    pub use super::tokens::{
        // Color types
        ColorToken,
        BackgroundRole,
        ForegroundRole,
        AccentRole,
        // Color role markers
        Background,
        Surface,
        SurfaceHover,
        Foreground,
        ForegroundDim,
        Accent,
        // Spacing
        Spacing,
        SpacingXs,
        SpacingSm,
        SpacingMd,
        SpacingLg,
        SpacingXl,
        // Radius
        RadiusSemantic,
        RadiusNone,
        RadiusSm,
        RadiusMd,
        RadiusLg,
        RadiusFull,
        // Typography
        FontFamily,
    };
}
