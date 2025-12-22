//! # Error Types for fernctl
//!
//! Provides structured error types for all fernctl operations.

use miette::Diagnostic;
use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for fernctl operations.
pub type Result<T> = std::result::Result<T, FernctlError>;

/// Error type for fernctl operations.
#[derive(Debug, Error, Diagnostic)]
pub enum FernctlError {
    /// Error reading or writing state files.
    #[error("State file error: {message}")]
    #[diagnostic(code(fernctl::state))]
    State {
        /// Description of what went wrong.
        message: String,
        /// Path to the state file, if applicable.
        path: Option<PathBuf>,
        /// Underlying IO error.
        #[source]
        source: Option<std::io::Error>,
    },

    /// Error parsing state file content.
    #[error("Failed to parse {file_type}: {message}")]
    #[diagnostic(code(fernctl::parse))]
    Parse {
        /// Type of file being parsed (e.g., "obs-state.json").
        file_type: String,
        /// Description of the parse error.
        message: String,
    },

    /// Error controlling a service.
    #[error("Service control error: {message}")]
    #[diagnostic(code(fernctl::service))]
    Service {
        /// The service name.
        service: String,
        /// Description of what went wrong.
        message: String,
        /// Underlying error.
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Error with the TUI.
    #[error("TUI error: {message}")]
    #[diagnostic(code(fernctl::tui))]
    Tui {
        /// Description of what went wrong.
        message: String,
        /// Underlying IO error.
        #[source]
        source: Option<std::io::Error>,
    },

    /// Error with file watching.
    #[error("Watch error: {message}")]
    #[diagnostic(code(fernctl::watch))]
    Watch {
        /// Description of what went wrong.
        message: String,
        /// Underlying notify error.
        #[source]
        source: Option<notify::Error>,
    },

    /// Error finding or communicating with a process.
    #[error("Process error: {message}")]
    #[diagnostic(code(fernctl::process))]
    Process {
        /// Description of what went wrong.
        message: String,
        /// Underlying error.
        #[source]
        source: Option<std::io::Error>,
    },

    /// Configuration error.
    #[error("Configuration error: {message}")]
    #[diagnostic(code(fernctl::config))]
    Config {
        /// Description of what went wrong.
        message: String,
    },
}

impl FernctlError {
    /// Creates a state file error.
    pub fn state(message: impl Into<String>) -> Self {
        Self::State {
            message: message.into(),
            path: None,
            source: None,
        }
    }

    /// Creates a state file error with path.
    pub fn state_path(message: impl Into<String>, path: PathBuf) -> Self {
        Self::State {
            message: message.into(),
            path: Some(path),
            source: None,
        }
    }

    /// Creates a state file error with IO source.
    pub fn state_io(message: impl Into<String>, source: std::io::Error) -> Self {
        Self::State {
            message: message.into(),
            path: None,
            source: Some(source),
        }
    }

    /// Creates a parse error.
    pub fn parse(file_type: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Parse {
            file_type: file_type.into(),
            message: message.into(),
        }
    }

    /// Creates a service error.
    pub fn service(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Service {
            service: service.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Creates a TUI error.
    pub fn tui(message: impl Into<String>) -> Self {
        Self::Tui {
            message: message.into(),
            source: None,
        }
    }

    /// Creates a TUI error with IO source.
    pub fn tui_io(message: impl Into<String>, source: std::io::Error) -> Self {
        Self::Tui {
            message: message.into(),
            source: Some(source),
        }
    }

    /// Creates a watch error.
    pub fn watch(message: impl Into<String>) -> Self {
        Self::Watch {
            message: message.into(),
            source: None,
        }
    }

    /// Creates a watch error with notify source.
    pub fn watch_notify(message: impl Into<String>, source: notify::Error) -> Self {
        Self::Watch {
            message: message.into(),
            source: Some(source),
        }
    }

    /// Creates a process error.
    pub fn process(message: impl Into<String>) -> Self {
        Self::Process {
            message: message.into(),
            source: None,
        }
    }

    /// Creates a process error with IO source.
    pub fn process_io(message: impl Into<String>, source: std::io::Error) -> Self {
        Self::Process {
            message: message.into(),
            source: Some(source),
        }
    }

    /// Creates a configuration error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Creates a generic IO error.
    pub fn io(message: impl Into<String>, source: std::io::Error) -> Self {
        Self::State {
            message: message.into(),
            path: None,
            source: Some(source),
        }
    }
}

impl From<fern_theme::error::FernError> for FernctlError {
    fn from(err: fern_theme::error::FernError) -> Self {
        FernctlError::Config {
            message: err.to_string(),
        }
    }
}
