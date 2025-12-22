//! # Service Types
//!
//! Defines the known services that fernctl can manage.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Known services that fernctl can monitor and control.
///
/// Each service has associated metadata about how to start, stop,
/// and check its status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnownService {
    /// OBS Studio WebSocket bridge daemon (`fern-obs`).
    Obs,

    /// QuickShell process running the Fern shell.
    Shell,

    /// Theme file watcher (`fern-theme watch`).
    ThemeWatcher,
}

impl KnownService {
    /// Returns all known services.
    #[must_use]
    pub const fn all() -> &'static [KnownService] {
        &[Self::Obs, Self::Shell, Self::ThemeWatcher]
    }

    /// Returns the binary name for this service.
    #[must_use]
    pub const fn binary(&self) -> &'static str {
        match self {
            Self::Obs => "fern-obs",
            Self::Shell => "quickshell",
            Self::ThemeWatcher => "fern-theme",
        }
    }

    /// Returns the arguments to start this service.
    #[must_use]
    pub const fn start_args(&self) -> &'static [&'static str] {
        match self {
            Self::Obs => &["daemon"],
            Self::Shell => &["-p", "/run/current-system/sw/share/fern/shell.qml"],
            Self::ThemeWatcher => &["watch"],
        }
    }

    /// Returns the state file name for this service.
    #[must_use]
    pub const fn state_file(&self) -> &'static str {
        match self {
            Self::Obs => "obs-state.json",
            Self::Shell => "shell-log.json",
            Self::ThemeWatcher => "theme-state.json",
        }
    }

    /// Returns the service name as a string.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Obs => "obs",
            Self::Shell => "shell",
            Self::ThemeWatcher => "theme-watcher",
        }
    }

    /// Returns a human-readable display name.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Obs => "OBS Bridge",
            Self::Shell => "Fern Shell",
            Self::ThemeWatcher => "Theme Watcher",
        }
    }

    /// Returns a short description of what this service does.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Obs => "WebSocket bridge to OBS Studio",
            Self::Shell => "QuickShell-based desktop shell",
            Self::ThemeWatcher => "Watches config and applies themes",
        }
    }

    /// Parses a service name string into a `KnownService`.
    ///
    /// # Errors
    ///
    /// Returns `None` if the name doesn't match a known service.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "obs" | "fern-obs" => Some(Self::Obs),
            "shell" | "quickshell" => Some(Self::Shell),
            "theme" | "theme-watcher" | "fern-theme" => Some(Self::ThemeWatcher),
            _ => None,
        }
    }

    /// Returns whether this service should be automatically restarted.
    #[must_use]
    pub const fn auto_restart(&self) -> bool {
        match self {
            Self::Obs => true,
            Self::Shell => true,
            Self::ThemeWatcher => true,
        }
    }
}

impl fmt::Display for KnownService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_from_name() {
        assert_eq!(KnownService::from_name("obs"), Some(KnownService::Obs));
        assert_eq!(
            KnownService::from_name("fern-obs"),
            Some(KnownService::Obs)
        );
        assert_eq!(
            KnownService::from_name("shell"),
            Some(KnownService::Shell)
        );
        assert_eq!(
            KnownService::from_name("theme"),
            Some(KnownService::ThemeWatcher)
        );
        assert_eq!(KnownService::from_name("unknown"), None);
    }

    #[test]
    fn service_display() {
        assert_eq!(KnownService::Obs.to_string(), "obs");
        assert_eq!(KnownService::Shell.to_string(), "shell");
        assert_eq!(KnownService::ThemeWatcher.to_string(), "theme-watcher");
    }

    #[test]
    fn all_services_have_binaries() {
        for service in KnownService::all() {
            assert!(!service.binary().is_empty());
            assert!(!service.state_file().is_empty());
        }
    }
}
