//! # Domain Types
//!
//! Pure business logic types for fernctl following the Elm architecture pattern.
//!
//! ## Overview
//!
//! - [`app`] - Application state and update logic
//! - [`action`] - Action types (messages)
//! - [`service`] - Known services and their configuration
//! - [`log`] - Log entry types and buffer
//! - [`hardware`] - Hardware sensor state and sparkline history

pub mod action;
pub mod app;
pub mod hardware;
pub mod log;
pub mod service;

pub use action::Action;
pub use app::{AppState, PanelFocus};
pub use hardware::HardwareState;
pub use log::{LogBuffer, LogEntry, LogLevel};
pub use service::KnownService;
