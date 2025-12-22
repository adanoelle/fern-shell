//! # Adapters
//!
//! Concrete implementations for service control, file watching, and shell IPC.
//!
//! These adapters implement the ports defined in the domain layer,
//! providing the actual integration with the system.
//!
//! ## Modules
//!
//! - [`service_control`] - Process start/stop/restart
//! - [`state_watcher`] - File watching with notify
//! - [`shell_ipc`] - QuickShell reload via SIGHUP

pub mod service_control;
pub mod shell_ipc;
pub mod state_watcher;

pub use service_control::ServiceController;
pub use shell_ipc::{find_quickshell_pid, is_shell_running, reload_shell, shell_uptime};
pub use state_watcher::{StateChange, StateWatcher};
