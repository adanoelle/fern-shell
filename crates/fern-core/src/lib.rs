//! # fern-core
//!
//! Shared types, traits, and utilities for the Fern Shell ecosystem.
//!
//! This crate provides common infrastructure used across all `fern-*` crates:
//!
//! - [`paths`] - XDG-compliant configuration and state directories
//! - [`state`] - Service state types for inter-process communication
//! - [`config`] - Common configuration loading patterns
//! - [`error`] - Shared error types
//!
//! ## Crate Ecosystem
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                        fern-core                            │
//! │              (shared types & utilities)                     │
//! └──────────┬──────────────┬──────────────┬───────────────────┘
//!            │              │              │
//!      ┌─────▼─────┐  ┌─────▼─────┐  ┌─────▼─────┐
//!      │fern-theme │  │ fern-obs  │  │ fernctl   │
//!      │           │  │           │  │           │
//!      │  Themes   │  │    OBS    │  │  Control  │
//!      │  Config   │  │  Bridge   │  │   Plane   │
//!      └───────────┘  └───────────┘  └───────────┘
//! ```
//!
//! ## Feature Flags
//!
//! - `async` - Enable async runtime support for service daemons
//! - `dbus` - Enable D-Bus IPC support

pub mod config;
pub mod error;
pub mod paths;
pub mod state;

pub use error::{Error, Result};
pub use paths::FernPaths;
pub use state::{HealthCheck, ServiceInfo, ServiceStatus};
