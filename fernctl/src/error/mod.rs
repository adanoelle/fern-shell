//! # Error Handling for Fern Shell
//!
//! This module provides a comprehensive error handling system designed for both
//! developers and end users. It follows these principles:
//!
//! 1. **Rich diagnostics** — Errors include source locations, suggestions, and
//!    documentation links to help users fix problems.
//!
//! 2. **Severity levels** — Not all problems are equal. Warnings allow the shell
//!    to continue while errors require user intervention.
//!
//! 3. **Notification-ready** — Errors implement [`Notifiable`] so they can be
//!    displayed as desktop notifications when the notification system is available.
//!
//! 4. **Type-safe error codes** — Each error variant has a unique code that can
//!    be looked up in documentation.
//!
//! ## Error Hierarchy
//!
//! ```text
//! FernError (top-level)
//! ├── ConfigError     — Configuration parsing and validation
//! ├── IoError         — File system operations
//! ├── WatchError      — File watching failures
//! └── IpcError        — D-Bus communication errors
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use fernctl::error::{Result, ConfigError, Severity};
//!
//! fn validate_color(value: &str) -> Result<Color> {
//!     parse_hex_color(value).map_err(|_| ConfigError::InvalidColor {
//!         value: value.to_string(),
//!         span: None,
//!     })
//! }
//! ```
//!
//! ## Integration with miette
//!
//! All error types implement [`miette::Diagnostic`], enabling rich terminal output:
//!
//! ```text
//! Error: fern::config::invalid_color
//!
//!   × Invalid color format: #gg0000
//!    ╭─[config.toml:5:10]
//!  4 │ [appearance]
//!  5 │ accent = "#gg0000"
//!    ·          ────────── this color value
//!    ╰────
//!   help: Colors must be hex format: #RRGGBB or #RRGGBBAA
//! ```

mod config;
mod notify;
mod severity;

pub use config::ConfigError;
pub use notify::{Notifiable, Notification};
pub use severity::Severity;

use thiserror::Error;

/// A specialized [`Result`] type for fernctl operations.
///
/// This type alias reduces boilerplate when returning errors from functions:
///
/// ```rust,ignore
/// use fernctl::error::Result;
///
/// fn load_config(path: &Path) -> Result<Config> {
///     // ...
/// }
/// ```
///
/// It is equivalent to `std::result::Result<T, FernError>`.
pub type Result<T> = std::result::Result<T, FernError>;

/// The top-level error type for all fernctl operations.
///
/// `FernError` is an enum that wraps all specific error types in the crate.
/// It implements [`miette::Diagnostic`] for rich error reporting and
/// [`Notifiable`] for desktop notification integration.
///
/// # Error Categories
///
/// | Variant | Description | Typical Cause |
/// |---------|-------------|---------------|
/// | [`Config`](FernError::Config) | Configuration errors | Invalid TOML, bad values |
/// | [`Io`](FernError::Io) | File system errors | Missing files, permissions |
/// | [`Watch`](FernError::Watch) | File watcher errors | System resource limits |
/// | [`Ipc`](FernError::Ipc) | D-Bus errors | Service not running |
///
/// # Example
///
/// ```rust,ignore
/// use fernctl::error::{FernError, Result};
///
/// fn process_config(path: &Path) -> Result<Theme> {
///     let content = std::fs::read_to_string(path)
///         .map_err(|e| FernError::Io(e.into()))?;
///
///     let config = parse_toml(&content)?;  // Returns ConfigError
///     Ok(config.into_theme())
/// }
/// ```
///
/// # Downcasting
///
/// You can match on specific error variants:
///
/// ```rust,ignore
/// match result {
///     Err(FernError::Config(ConfigError::InvalidColor { value, .. })) => {
///         eprintln!("Bad color: {}", value);
///     }
///     Err(e) => eprintln!("Other error: {}", e),
///     Ok(theme) => { /* use theme */ }
/// }
/// ```
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum FernError {
    /// Configuration parsing or validation error.
    ///
    /// This variant wraps [`ConfigError`], which provides detailed information
    /// about what went wrong in the configuration file.
    #[error(transparent)]
    Config(#[from] ConfigError),

    /// File system I/O error.
    ///
    /// Wraps standard I/O errors with additional context about what operation
    /// was being attempted.
    #[error("I/O error: {message}")]
    Io {
        /// Human-readable description of what operation failed.
        message: String,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// File watcher error.
    ///
    /// Occurs when the file watching system fails to initialize or encounters
    /// an error while monitoring files.
    #[error("File watch error: {message}")]
    Watch {
        /// Description of what went wrong.
        message: String,
    },

    /// Inter-process communication error.
    ///
    /// Occurs when communication with QuickShell via D-Bus fails.
    #[error("IPC error: {message}")]
    Ipc {
        /// Description of the communication failure.
        message: String,
    },
}

impl FernError {
    /// Creates an I/O error with context.
    ///
    /// # Arguments
    ///
    /// * `message` — A description of what operation was being attempted
    /// * `source` — The underlying I/O error
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let result = std::fs::read_to_string(path)
    ///     .map_err(|e| FernError::io("reading config file", e))?;
    /// ```
    #[must_use]
    pub fn io(message: impl Into<String>, source: std::io::Error) -> Self {
        Self::Io {
            message: message.into(),
            source,
        }
    }

    /// Creates a file watch error.
    ///
    /// # Arguments
    ///
    /// * `message` — A description of what went wrong
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// FernError::watch("failed to initialize file watcher")
    /// ```
    #[must_use]
    pub fn watch(message: impl Into<String>) -> Self {
        Self::Watch {
            message: message.into(),
        }
    }

    /// Creates an IPC error.
    ///
    /// # Arguments
    ///
    /// * `message` — A description of the communication failure
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// FernError::ipc("QuickShell did not respond to reload signal")
    /// ```
    #[must_use]
    pub fn ipc(message: impl Into<String>) -> Self {
        Self::Ipc {
            message: message.into(),
        }
    }

    /// Returns the severity level of this error.
    ///
    /// Severity determines how the error should be handled:
    ///
    /// - [`Severity::Warning`] — Log and continue
    /// - [`Severity::Error`] — Operation failed, but shell can continue
    /// - [`Severity::Fatal`] — Shell cannot continue
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if error.severity().is_fatal() {
    ///     std::process::exit(1);
    /// }
    /// ```
    #[must_use]
    pub fn severity(&self) -> Severity {
        match self {
            Self::Config(e) => e.severity(),
            Self::Io { .. } => Severity::Error,
            Self::Watch { .. } => Severity::Warning,
            Self::Ipc { .. } => Severity::Error,
        }
    }

    /// Returns the error code for documentation lookup.
    ///
    /// Error codes follow the pattern `fern::<category>::<specific>` and can
    /// be used to find detailed documentation about the error.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Prints something like "fern::config::invalid_color"
    /// println!("See docs for: {}", error.code());
    /// ```
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Config(e) => e.code(),
            Self::Io { .. } => "fern::io::error",
            Self::Watch { .. } => "fern::watch::error",
            Self::Ipc { .. } => "fern::ipc::error",
        }
    }
}

// Implement miette::Diagnostic manually to avoid derive macro conflicts
impl miette::Diagnostic for FernError {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new(FernError::code(self)))
    }

    fn severity(&self) -> Option<miette::Severity> {
        Some(match FernError::severity(self) {
            Severity::Info => miette::Severity::Advice,
            Severity::Warning => miette::Severity::Warning,
            Severity::Error => miette::Severity::Error,
            Severity::Fatal => miette::Severity::Error,
        })
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            Self::Config(e) => e.help(),
            Self::Io { .. } => Some(Box::new("Check file permissions and path")),
            Self::Watch { .. } => Some(Box::new("Check system resource limits (inotify watches on Linux)")),
            Self::Ipc { .. } => Some(Box::new("Ensure QuickShell is running and the D-Bus session is available")),
        }
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        match self {
            Self::Config(e) => e.source_code(),
            _ => None,
        }
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        match self {
            Self::Config(e) => e.labels(),
            _ => None,
        }
    }

    fn diagnostic_source(&self) -> Option<&dyn miette::Diagnostic> {
        match self {
            Self::Config(e) => Some(e),
            _ => None,
        }
    }
}

impl Notifiable for FernError {
    fn severity(&self) -> Severity {
        FernError::severity(self)
    }

    fn title(&self) -> String {
        match self {
            Self::Config(e) => e.title(),
            Self::Io { .. } => "File Error".to_string(),
            Self::Watch { .. } => "Watch Error".to_string(),
            Self::Ipc { .. } => "Communication Error".to_string(),
        }
    }

    fn body(&self) -> String {
        self.to_string()
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::Config(e) => e.suggestion(),
            Self::Io { .. } => Some("Check file permissions and path".to_string()),
            Self::Watch { .. } => Some("Check system resource limits".to_string()),
            Self::Ipc { .. } => Some("Ensure QuickShell is running".to_string()),
        }
    }

    fn code(&self) -> Option<&'static str> {
        Some(FernError::code(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<FernError>();
    }

    #[test]
    fn io_error_has_correct_code() {
        let err = FernError::io(
            "test",
            std::io::Error::new(std::io::ErrorKind::NotFound, "not found"),
        );
        assert_eq!(FernError::code(&err), "fern::io::error");
    }
}
