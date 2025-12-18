//! # Shared Error Types
//!
//! Common error types used across the Fern ecosystem.
//!
//! Each `fern-*` crate may define its own specialized error types, but this
//! module provides base errors that are commonly needed.

use thiserror::Error;

/// Common result type for fern-core operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in fern-core operations.
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error (file operations, etc.)
    #[error("{context}: {source}")]
    Io {
        /// Context describing what operation failed.
        context: String,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// Parse error (JSON, TOML, etc.)
    #[error("{context}: {message}")]
    Parse {
        /// Context describing what was being parsed.
        context: String,
        /// Error message.
        message: String,
    },

    /// Service error (daemon operations)
    #[error("service '{service}': {message}")]
    Service {
        /// The service that encountered the error.
        service: String,
        /// Error message.
        message: String,
    },

    /// IPC error (inter-process communication)
    #[error("IPC error: {0}")]
    Ipc(String),
}

impl Error {
    /// Creates an I/O error with context.
    pub fn io(context: impl Into<String>, source: std::io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }

    /// Creates a parse error.
    pub fn parse(context: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Parse {
            context: context.into(),
            message: message.into(),
        }
    }

    /// Creates a service error.
    pub fn service(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Service {
            service: service.into(),
            message: message.into(),
        }
    }

    /// Creates an IPC error.
    pub fn ipc(message: impl Into<String>) -> Self {
        Self::Ipc(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_error_display() {
        let err = Error::io(
            "reading config",
            std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"),
        );

        let msg = err.to_string();
        assert!(msg.contains("reading config"));
        assert!(msg.contains("file not found"));
    }

    #[test]
    fn service_error_display() {
        let err = Error::service("obs", "connection refused");

        let msg = err.to_string();
        assert!(msg.contains("obs"));
        assert!(msg.contains("connection refused"));
    }
}
