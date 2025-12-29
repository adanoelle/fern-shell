//! Error types for frond.

use std::io;

/// Result type alias using frond's Error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in the frond runtime.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error (terminal, files, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Terminal setup/teardown error
    #[error("Terminal error: {0}")]
    Terminal(String),

    /// Command execution error
    #[error("Command error: {0}")]
    Command(String),

    /// Subscription error
    #[error("Subscription error: {0}")]
    Subscription(String),
}
