//! Error types for fern-obs.

use thiserror::Error;

/// Result type for fern-obs operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in fern-obs.
#[derive(Debug, Error)]
pub enum Error {
    /// Failed to connect to OBS.
    #[error("failed to connect to OBS at {host}:{port}: {message}")]
    Connection {
        /// The host we tried to connect to.
        host: String,
        /// The port we tried to connect to.
        port: u16,
        /// Error message.
        message: String,
    },

    /// OBS WebSocket error.
    #[error("OBS WebSocket error: {0}")]
    WebSocket(String),

    /// OBS returned an error for a request.
    #[error("OBS request failed: {0}")]
    Request(String),

    /// Authentication failed.
    #[error("authentication failed: {0}")]
    Auth(String),

    /// OBS is not connected.
    #[error("not connected to OBS")]
    NotConnected,

    /// I/O error (file operations).
    #[error("{context}: {source}")]
    Io {
        /// Context for the error.
        context: String,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// JSON serialization error.
    #[error("JSON error: {0}")]
    Json(String),

    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),
}

impl Error {
    /// Creates a connection error.
    pub fn connection(host: impl Into<String>, port: u16, message: impl Into<String>) -> Self {
        Self::Connection {
            host: host.into(),
            port,
            message: message.into(),
        }
    }

    /// Creates an I/O error with context.
    pub fn io(context: impl Into<String>, source: std::io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }

    /// Returns true if this error indicates OBS is not running.
    #[must_use]
    pub fn is_connection_refused(&self) -> bool {
        matches!(self, Self::Connection { .. })
    }
}

impl From<obws::error::Error> for Error {
    fn from(err: obws::error::Error) -> Self {
        Self::WebSocket(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}
