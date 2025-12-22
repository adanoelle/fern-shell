//! # fernctl - Fern Shell Control Plane
//!
//! A unified control plane for managing Fern Shell services, viewing logs,
//! and controlling configuration.
//!
//! ## Features
//!
//! - **CLI Interface**: Commands for status, logs, service control, and configuration
//! - **TUI Dashboard**: Interactive terminal UI with service status, logs, and config panels
//! - **Service Orchestration**: Start, stop, and restart managed services
//! - **File Watching**: Real-time updates from state files
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                      ~/.local/state/fern/                           │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  obs-state.json  │ OBS daemon state (recording, streaming)         │
//! │  shell-log.json  │ QML shell logs (ring buffer)                    │
//! │  services.json   │ Service registry (managed by fernctl)           │
//! └─────────────────────────────────────────────────────────────────────┘
//!           ▲                    ▲                    ▲
//!           │                    │                    │
//!     ┌─────┴──────┐      ┌─────┴──────┐      ┌─────┴──────┐
//!     │  fern-obs  │      │  QML Shell │      │ fern-theme │
//!     └────────────┘      └────────────┘      └────────────┘
//!                                │
//!                        ┌───────▼───────┐
//!                        │   fernctl     │
//!                        │  CLI + TUI    │
//!                        └───────────────┘
//! ```
//!
//! ## Usage
//!
//! ```bash
//! # Show service status
//! fernctl status
//!
//! # View logs
//! fernctl logs -f
//!
//! # Launch TUI dashboard
//! fernctl tui
//!
//! # Control OBS service
//! fernctl obs start
//! fernctl obs stop
//! ```
//!
//! ## Modules
//!
//! - [`domain`] - Core types (AppState, Actions, Services, Logs)
//! - [`adapters`] - Service control, file watching, shell IPC
//! - [`cli`] - Command implementations
//! - [`tui`] - Terminal UI components
//! - [`error`] - Error types

pub mod adapters;
pub mod domain;
pub mod error;

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "tui")]
pub mod tui;

pub use domain::{Action, AppState, KnownService, LogBuffer, LogEntry, LogLevel, PanelFocus};
pub use error::{FernctlError, Result};
