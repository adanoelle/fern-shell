//! # CLI Commands
//!
//! Command implementations for the fernctl CLI.
//!
//! ## Available Commands
//!
//! - `status` - Show service status
//! - `logs` - View aggregated logs
//! - `reload` - Reload QuickShell configuration
//! - `obs` - OBS daemon control
//! - `theme` - Theme management

pub mod logs;
pub mod obs;
pub mod reload;
pub mod status;
pub mod theme;
