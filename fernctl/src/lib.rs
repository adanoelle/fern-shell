//! # fernctl — Type-Safe Design Tokens for Fern Shell
//!
//! `fernctl` is the core library for Fern Shell's design system, providing
//! compile-time guarantees for design token correctness through Rust's type system.
//!
//! ## Overview
//!
//! This crate implements a **hexagonal architecture** (ports and adapters) where:
//!
//! - **Domain Layer** ([`domain`]): Pure business logic with zero external dependencies.
//!   Contains type-safe design tokens, theme definitions, and validation rules.
//!
//! - **Ports** ([`ports`]): Trait-based interfaces that define how the domain
//!   interacts with the outside world, without depending on specific implementations.
//!
//! - **Adapters** ([`adapters`]): Concrete implementations that connect ports to
//!   external systems (file I/O, D-Bus, notifications).
//!
//! - **Error Handling** ([`error`]): Rich, user-friendly error types with
//!   diagnostic information for debugging configuration issues.
//!
//! ## Design Philosophy
//!
//! Traditional design systems define tokens as stringly-typed values:
//!
//! ```text
//! button.background = "#1e1e2e"  // Is this a valid background color?
//! button.radius = 9999           // Is this valid for buttons?
//! ```
//!
//! These questions are answered at runtime (if at all), leading to subtle bugs.
//!
//! `fernctl` encodes design system rules in the type system:
//!
//! ```rust,ignore
//! // ColorToken<Surface> can only be used where BackgroundRole is expected
//! button.background: ColorToken<Surface>  // ✓ Compiles
//! button.background: ColorToken<Foreground>  // ✗ Type error!
//!
//! // ButtonRadius trait limits which radii are valid for buttons
//! button.radius: RadiusMd  // ✓ Compiles (Md implements ButtonRadius)
//! button.radius: RadiusFull  // ✗ Type error! (Full doesn't implement ButtonRadius)
//! ```
//!
//! ## Architecture Diagram
//!
//! ```text
//!                        ┌─────────────────────────────────┐
//!                        │      DRIVING ADAPTERS           │
//!                        │  ┌──────┐  ┌─────┐  ┌──────┐    │
//!                        │  │ TOML │  │ CLI │  │ JSON │    │
//!                        │  └──┬───┘  └──┬──┘  └──┬───┘    │
//!                        │     │         │        │        │
//!                        │  ┌──▼─────────▼────────▼───┐    │
//!                        │  │     INBOUND PORTS       │    │
//!                        │  │  ConfigPort, QueryPort  │    │
//!                        │  └───────────┬─────────────┘    │
//! ┌──────────────────────┼──────────────▼─────────────────┐│
//! │                      │        DOMAIN                  ││
//! │  ┌───────────────────┴──────────────────────────────┐ ││
//! │  │               DESIGN LANGUAGE                    │ ││
//! │  │  ┌──────────┐  ┌──────────┐  ┌──────────────┐    │ ││
//! │  │  │  Tokens  │  │  Scales  │  │  Components  │    │ ││
//! │  │  │  Color   │  │  Spacing │  │  ButtonSpec  │    │ ││
//! │  │  │  Radius  │  │  Radius  │  │  ModuleSpec  │    │ ││
//! │  │  └──────────┘  └──────────┘  └──────────────┘    │ ││
//! │  │                      │                           │ ││
//! │  │                 ┌────▼────┐                      │ ││
//! │  │                 │  Theme  │                      │ ││
//! │  │                 └─────────┘                      │ ││
//! │  └──────────────────────────────────────────────────┘ ││
//! │                         │                             ││
//! │  ┌──────────────────────▼───────────────────────────┐ ││
//! │  │              OUTBOUND PORTS                      │ ││
//! │  │  PersistPort, NotifyPort, IpcPort                │ ││
//! │  └──────────────────────┬───────────────────────────┘ ││
//! └─────────────────────────┼─────────────────────────────┘│
//!                        │  │                              │
//!                        │  ┌──▼─────────────────────────┐ │
//!                        │  │     DRIVEN ADAPTERS        │ │
//!                        │  │ ┌────────┐ ┌─────┐ ┌─────┐ │ │
//!                        │  │ │  File  │ │D-Bus│ │Notif│ │ │
//!                        │  │ │ System │ │ IPC │ │     │ │ │
//!                        │  │ └────────┘ └─────┘ └─────┘ │ │
//!                        │  └────────────────────────────┘ │
//!                        └─────────────────────────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ### Loading Configuration
//!
//! ```rust,ignore
//! use fernctl::prelude::*;
//! use fernctl::adapters::TomlConfigAdapter;
//!
//! // Load and validate a TOML configuration
//! let toml_content = std::fs::read_to_string("config.toml")?;
//! let adapter = TomlConfigAdapter::new();
//!
//! // Type-state ensures we can't use config before validation
//! let raw = adapter.parse(&toml_content)?;        // Config<Parsed>
//! let validated = raw.validate()?;                 // Config<Validated>
//! let theme = validated.into_theme();              // Theme
//! ```
//!
//! ### Using Design Tokens
//!
//! ```rust,ignore
//! use fernctl::domain::tokens::*;
//!
//! // Create type-safe colors
//! let surface: ColorToken<Surface> = ColorToken::from_hex("#313244")?;
//! let text: ColorToken<Foreground> = ColorToken::from_hex("#cdd6f4")?;
//!
//! // The type system prevents misuse:
//! fn set_background(color: impl BackgroundRole) { /* ... */ }
//! set_background(surface);  // ✓ Works - Surface is a BackgroundRole
//! // set_background(text);  // ✗ Won't compile - Foreground is not BackgroundRole
//! ```
//!
//! ## Feature Flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `cli` | ✓ | Enables the `fernctl` binary with clap |
//! | `fancy-errors` | ✓ | Enables colorful miette error output |
//! | `watch` | ✗ | Enables file watching for live reload |
//! | `dbus` | ✗ | Enables D-Bus IPC with QuickShell |
//!
//! ## Modules
//!
//! - [`domain`] — Core design language types (tokens, scales, themes)
//! - [`error`] — Error types with rich diagnostics
//! - [`ports`] — Interface traits for hexagonal architecture
//! - [`adapters`] — Implementations connecting to external systems
//!
//! ## Design Decisions
//!
//! This crate makes several opinionated choices:
//!
//! 1. **No unsafe code** — The entire crate is `#![forbid(unsafe_code)]`
//! 2. **No panics in library code** — All fallible operations return `Result`
//! 3. **Comprehensive documentation** — Every public item is documented
//! 4. **Type-level correctness** — Invalid states are unrepresentable
//!
//! For detailed rationale, see the module-level documentation in each submodule.

#![doc(html_root_url = "https://docs.rs/fernctl/0.1.0")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]

pub mod domain;
pub mod error;
pub mod ports;

#[cfg(feature = "cli")]
#[cfg_attr(docsrs, doc(cfg(feature = "cli")))]
pub mod adapters;

/// Convenient re-exports for common usage patterns.
///
/// Import this module to get started quickly:
///
/// ```rust,ignore
/// use fernctl::prelude::*;
/// ```
pub mod prelude {
    // Domain types
    pub use crate::domain::theme::Theme;
    pub use crate::domain::tokens::{
        ColorToken,
        BackgroundRole,
        ForegroundRole,
        AccentRole,
    };

    // Error types
    pub use crate::error::{
        FernError,
        Result,
        Severity,
    };

    // Port traits
    pub use crate::ports::inbound::ConfigPort;
    pub use crate::ports::outbound::NotifyPort;
}

/// Crate version from Cargo.toml.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
