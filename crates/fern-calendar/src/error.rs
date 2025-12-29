//! Error types for fern-calendar.

use thiserror::Error;

/// Calendar errors.
#[derive(Debug, Error)]
pub enum CalendarError {
    /// Database error.
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// Event not found.
    #[error("event not found: {0}")]
    NotFound(uuid::Uuid),

    /// Invalid data.
    #[error("invalid data: {0}")]
    InvalidData(String),
}

/// Result type alias for calendar operations.
pub type Result<T> = std::result::Result<T, CalendarError>;
