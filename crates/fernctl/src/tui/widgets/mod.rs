//! # TUI Widgets
//!
//! Reusable UI components for the TUI dashboard.
//!
//! ## Widgets
//!
//! - [`ServicesPanel`] - Service status list
//! - [`LogsPanel`] - Scrollable log viewer
//! - [`ConfigPanel`] - Configuration overview
//! - [`HelpPanel`] - Keyboard shortcuts help

pub mod config_panel;
pub mod help_panel;
pub mod logs_panel;
pub mod services_panel;

pub use config_panel::ConfigPanel;
pub use help_panel::HelpPanel;
pub use logs_panel::LogsPanel;
pub use services_panel::ServicesPanel;
