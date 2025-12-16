//! # CLI Command Implementations
//!
//! This module contains the implementation logic for each `fernctl` subcommand.
//! By separating command implementations from the CLI definition in [`main`],
//! we achieve:
//!
//! - **Testability** — Commands can be unit tested without parsing CLI arguments
//! - **Reusability** — Command logic can be called programmatically
//! - **Separation of concerns** — CLI parsing vs. business logic
//!
//! ## Module Structure
//!
//! Each command has its own submodule with:
//!
//! | Module | Command | Description |
//! |--------|---------|-------------|
//! | [`validate`] | `fernctl validate` | Validate configuration syntax and semantics |
//! | [`convert`] | `fernctl convert` | Convert TOML to JSON |
//! | [`query`] | `fernctl query` | Query specific theme values |
//! | [`watch`] | `fernctl watch` | Watch config and auto-convert on changes |
//!
//! ## Architecture
//!
//! Commands follow the hexagonal architecture pattern:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      CLI (main.rs)                          │
//! │  Parses arguments, constructs dependencies, calls commands  │
//! └─────────────────────────────┬───────────────────────────────┘
//!                               │
//!                               ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                   Commands (this module)                    │
//! │  Pure logic that accepts ports as dependencies              │
//! │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐       │
//! │  │ validate │ │ convert  │ │  query   │ │  watch   │       │
//! │  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘       │
//! └───────┼────────────┼────────────┼────────────┼──────────────┘
//!         │            │            │            │
//!         ▼            ▼            ▼            ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Ports (traits)                           │
//! │  ConfigPort, PersistPort, NotifyPort, WatchPort             │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage Example
//!
//! Commands are called with their dependencies injected:
//!
//! ```rust,ignore
//! use fernctl::commands::watch;
//! use fernctl::adapters::{TomlConfigAdapter, FileSystemAdapter};
//!
//! let config_adapter = TomlConfigAdapter::new();
//! let persist_adapter = FileSystemAdapter::new();
//!
//! // Options configure command behavior
//! let options = watch::WatchOptions {
//!     debounce_ms: 100,
//!     notify_on_success: true,
//!     ..Default::default()
//! };
//!
//! watch::run(&config_path, &json_path, options, &config_adapter, &persist_adapter)?;
//! ```
//!
//! ## Error Handling
//!
//! All commands return [`Result<T, FernError>`](crate::error::FernError).
//! Errors are enriched with context about what operation failed:
//!
//! ```text
//! Error: fern::config::parse_error
//!
//!   × TOML syntax error
//!    ╭─[config.toml:12:5]
//! 11 │ [bar]
//! 12 │ height = "forty"
//!    ·          ──────── expected integer
//!    ╰────
//!   help: Bar height must be a number in pixels
//! ```

pub mod convert;
pub mod query;
pub mod validate;

#[cfg(feature = "watch")]
#[cfg_attr(docsrs, doc(cfg(feature = "watch")))]
pub mod watch;
