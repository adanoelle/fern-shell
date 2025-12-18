//! # Severity Levels for Errors and Notifications
//!
//! This module defines [`Severity`], an enumeration of importance levels for
//! errors and notifications in Fern Shell.
//!
//! ## Design Rationale
//!
//! Not all problems are equal. A misspelled config key is worth warning about,
//! but shouldn't prevent the shell from starting. An invalid color format is
//! an error that needs fixing, but a single module can fail without crashing
//! the entire shell. Only truly unrecoverable situations are fatal.
//!
//! By encoding severity in the type system (rather than as a string or integer),
//! we get:
//!
//! - **Exhaustive matching** — The compiler ensures all severities are handled
//! - **Type safety** — Can't accidentally pass a string where severity is expected
//! - **Rich methods** — Severity carries behavior (icons, ordering, etc.)
//!
//! ## Ordering
//!
//! Severities are ordered from least to most severe:
//!
//! ```text
//! Info < Warning < Error < Fatal
//! ```
//!
//! This enables filtering:
//!
//! ```rust
//! use fernctl::error::Severity;
//!
//! let min_severity = Severity::Warning;
//! let current = Severity::Info;
//!
//! if current >= min_severity {
//!     // Won't run - Info is less severe than Warning
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

/// Severity level for errors, warnings, and notifications.
///
/// `Severity` represents how serious a problem is and how it should be handled.
/// It is used throughout the error system and integrates with the notification
/// system for user-facing alerts.
///
/// # Variants
///
/// | Level | Meaning | Shell Behavior |
/// |-------|---------|----------------|
/// | [`Info`](Severity::Info) | Informational, not a problem | Log only |
/// | [`Warning`](Severity::Warning) | Suboptimal but functional | Log + optional notification |
/// | [`Error`](Severity::Error) | Operation failed | Log + notification + retry possible |
/// | [`Fatal`](Severity::Fatal) | Unrecoverable | Log + notification + exit |
///
/// # Ordering
///
/// `Severity` implements [`Ord`], with [`Info`](Severity::Info) being the least
/// severe and [`Fatal`](Severity::Fatal) being the most:
///
/// ```rust
/// use fernctl::error::Severity;
///
/// assert!(Severity::Info < Severity::Warning);
/// assert!(Severity::Warning < Severity::Error);
/// assert!(Severity::Error < Severity::Fatal);
/// ```
///
/// # Display
///
/// Each severity has both a human-readable name and an icon for terminal output:
///
/// ```rust
/// use fernctl::error::Severity;
///
/// assert_eq!(Severity::Error.as_str(), "error");
/// assert_eq!(Severity::Error.icon(), "✗");
/// ```
///
/// # Serialization
///
/// `Severity` serializes to lowercase strings for JSON/TOML compatibility:
///
/// ```rust
/// use fernctl::error::Severity;
///
/// let json = serde_json::to_string(&Severity::Warning).unwrap();
/// assert_eq!(json, "\"warning\"");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Severity {
    /// Informational message, not an error.
    ///
    /// Use for status updates, progress information, or optional suggestions
    /// that don't indicate any problem.
    ///
    /// # Example Scenarios
    ///
    /// - "Configuration loaded successfully"
    /// - "Using default theme"
    /// - "3 modules active"
    Info,

    /// Something is suboptimal but will work.
    ///
    /// Use when the configuration or operation has issues that should be
    /// addressed, but the shell can continue functioning.
    ///
    /// # Example Scenarios
    ///
    /// - Unknown configuration key (will be ignored)
    /// - Deprecated option used
    /// - Fallback to default value
    Warning,

    /// Operation failed, but recovery is possible.
    ///
    /// Use when a specific operation or module fails, but the shell as a
    /// whole can continue. The user should be notified and may need to
    /// take action.
    ///
    /// # Example Scenarios
    ///
    /// - Invalid color format in config
    /// - Module failed to load
    /// - Network request timed out
    Error,

    /// Unrecoverable failure, shell cannot continue.
    ///
    /// Use only for catastrophic failures where the shell cannot possibly
    /// function. This should be rare — prefer [`Error`](Severity::Error)
    /// when recovery is possible.
    ///
    /// # Example Scenarios
    ///
    /// - Core configuration file is invalid TOML
    /// - Required system service unavailable
    /// - Out of memory
    Fatal,
}

impl Severity {
    /// Returns the string representation of this severity.
    ///
    /// The returned string is lowercase and suitable for logging or
    /// serialization.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fernctl::error::Severity;
    ///
    /// assert_eq!(Severity::Info.as_str(), "info");
    /// assert_eq!(Severity::Warning.as_str(), "warning");
    /// assert_eq!(Severity::Error.as_str(), "error");
    /// assert_eq!(Severity::Fatal.as_str(), "fatal");
    /// ```
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Fatal => "fatal",
        }
    }

    /// Returns a Unicode icon representing this severity.
    ///
    /// These icons are designed for terminal output and provide quick visual
    /// identification of severity levels.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fernctl::error::Severity;
    ///
    /// assert_eq!(Severity::Info.icon(), "ℹ");
    /// assert_eq!(Severity::Warning.icon(), "⚠");
    /// assert_eq!(Severity::Error.icon(), "✗");
    /// assert_eq!(Severity::Fatal.icon(), "💀");
    /// ```
    ///
    /// # Terminal Output
    ///
    /// ```text
    /// ℹ Configuration loaded
    /// ⚠ Unknown key 'colour' (did you mean 'color'?)
    /// ✗ Invalid color format: #gg0000
    /// 💀 Cannot start: config.toml is not valid TOML
    /// ```
    #[must_use]
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::Info => "ℹ",
            Self::Warning => "⚠",
            Self::Error => "✗",
            Self::Fatal => "💀",
        }
    }

    /// Returns an ANSI color code for terminal styling.
    ///
    /// These codes are suitable for colorizing terminal output. Use with
    /// `\x1b[0m` to reset styling.
    ///
    /// # Colors
    ///
    /// | Severity | Color |
    /// |----------|-------|
    /// | Info | Blue |
    /// | Warning | Yellow |
    /// | Error | Red |
    /// | Fatal | Magenta (bright) |
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::error::Severity;
    ///
    /// let sev = Severity::Error;
    /// let reset = "\x1b[0m";
    /// println!("{}{} Something went wrong{}", sev.ansi_color(), sev.icon(), reset);
    /// ```
    #[must_use]
    pub const fn ansi_color(&self) -> &'static str {
        match self {
            Self::Info => "\x1b[34m",    // Blue
            Self::Warning => "\x1b[33m", // Yellow
            Self::Error => "\x1b[31m",   // Red
            Self::Fatal => "\x1b[95m",   // Bright magenta
        }
    }

    /// Returns `true` if this severity indicates an error condition.
    ///
    /// An error condition is either [`Error`](Severity::Error) or
    /// [`Fatal`](Severity::Fatal). This is useful for deciding whether
    /// to fail an operation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fernctl::error::Severity;
    ///
    /// assert!(!Severity::Info.is_error());
    /// assert!(!Severity::Warning.is_error());
    /// assert!(Severity::Error.is_error());
    /// assert!(Severity::Fatal.is_error());
    /// ```
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self, Self::Error | Self::Fatal)
    }

    /// Returns `true` if this severity is fatal.
    ///
    /// Fatal severity indicates the shell cannot continue and should exit.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fernctl::error::Severity;
    ///
    /// assert!(!Severity::Error.is_fatal());
    /// assert!(Severity::Fatal.is_fatal());
    /// ```
    #[must_use]
    pub const fn is_fatal(&self) -> bool {
        matches!(self, Self::Fatal)
    }

    /// Returns all severity levels in order from least to most severe.
    ///
    /// Useful for iteration or building UI elements like filters.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::error::Severity;
    ///
    /// for sev in Severity::all() {
    ///     println!("{}: {}", sev.icon(), sev.as_str());
    /// }
    /// ```
    #[must_use]
    pub const fn all() -> &'static [Severity] {
        &[Self::Info, Self::Warning, Self::Error, Self::Fatal]
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for Severity {
    /// Returns [`Severity::Info`] as the default severity.
    fn default() -> Self {
        Self::Info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordering_is_correct() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Error < Severity::Fatal);
    }

    #[test]
    fn serialization_roundtrip() {
        for sev in Severity::all() {
            let json = serde_json::to_string(sev).expect("serialize");
            let back: Severity = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(*sev, back);
        }
    }

    #[test]
    fn display_matches_as_str() {
        for sev in Severity::all() {
            assert_eq!(sev.to_string(), sev.as_str());
        }
    }

    #[test]
    fn is_error_correctness() {
        assert!(!Severity::Info.is_error());
        assert!(!Severity::Warning.is_error());
        assert!(Severity::Error.is_error());
        assert!(Severity::Fatal.is_error());
    }
}
