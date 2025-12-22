//! # Action Types
//!
//! Actions are messages that describe state changes in the application.
//! This follows the Elm architecture pattern where all state changes
//! flow through a central update function.

use super::log::LogEntry;
use super::service::KnownService;
use fern_core::ServiceInfo;
use serde::{Deserialize, Serialize};

/// Actions that can be dispatched to update application state.
///
/// Following the Elm architecture, actions are the only way to modify state.
/// Each variant describes a specific intent or event that the application
/// should respond to.
#[derive(Debug, Clone)]
pub enum Action {
    // === Service Actions ===
    /// A service's state has changed (from file watcher).
    ServiceStateChanged {
        /// The service name.
        name: String,
        /// Updated service info.
        info: ServiceInfo,
    },

    /// Request to start a service.
    StartService(KnownService),

    /// Request to stop a service.
    StopService(KnownService),

    /// Request to restart a service.
    RestartService(KnownService),

    // === Log Actions ===
    /// A new log entry was received.
    LogReceived(LogEntry),

    /// Multiple log entries received (from file sync).
    LogsSync(Vec<LogEntry>),

    /// Scroll the logs view by the given offset.
    ScrollLogs(i32),

    /// Clear all logs.
    ClearLogs,

    /// Set the log filter string.
    SetLogFilter(String),

    // === Config Actions ===
    /// Configuration has changed (from file watcher).
    ConfigChanged(ConfigSummary),

    /// Request to reload the QuickShell configuration.
    ReloadShell,

    /// Request to apply a theme by name.
    ApplyTheme {
        /// Theme name to apply.
        name: String,
    },

    // === TUI Navigation Actions ===
    /// Move focus to the next panel.
    FocusNext,

    /// Move focus to the previous panel.
    FocusPrev,

    /// Toggle the help panel visibility.
    ToggleHelp,

    /// Select the next item in the current panel.
    SelectNext,

    /// Select the previous item in the current panel.
    SelectPrev,

    // === System Actions ===
    /// Periodic tick for updates.
    Tick,

    /// An error occurred.
    Error(String),

    /// Request to quit the application.
    Quit,
}

/// Summary of the current configuration for display.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigSummary {
    /// Theme name (e.g., "catppuccin", "nord").
    pub theme: Option<String>,

    /// Theme variant (e.g., "mocha", "dark").
    pub variant: Option<String>,

    /// Bar position (e.g., "top", "bottom").
    pub bar_position: Option<String>,

    /// Bar height in pixels.
    pub bar_height: Option<u32>,

    /// Accent color.
    pub accent_color: Option<String>,
}

impl ConfigSummary {
    /// Creates a new empty config summary.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses a config summary from a JSON value.
    ///
    /// Extracts relevant fields from the full config JSON.
    #[must_use]
    pub fn from_json(value: &serde_json::Value) -> Self {
        Self {
            theme: value
                .get("theme")
                .or_else(|| value.get("appearance").and_then(|a| a.get("theme")))
                .and_then(|v| v.as_str())
                .map(String::from),
            variant: value
                .get("variant")
                .or_else(|| value.get("appearance").and_then(|a| a.get("variant")))
                .and_then(|v| v.as_str())
                .map(String::from),
            bar_position: value
                .get("bar")
                .and_then(|b| b.get("position"))
                .and_then(|v| v.as_str())
                .map(String::from),
            bar_height: value
                .get("bar")
                .and_then(|b| b.get("height"))
                .and_then(|v| v.as_u64())
                .map(|h| h as u32),
            accent_color: value
                .get("colors")
                .and_then(|c| c.get("accent"))
                .or_else(|| value.get("appearance").and_then(|a| a.get("accent")))
                .and_then(|v| v.as_str())
                .map(String::from),
        }
    }
}
