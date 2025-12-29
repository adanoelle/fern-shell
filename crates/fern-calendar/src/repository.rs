//! Repository trait for event persistence.

use chrono::NaiveDate;
use uuid::Uuid;

use crate::error::Result;
use crate::event::Event;

/// Repository trait for event CRUD operations.
///
/// This trait defines the interface for event persistence.
/// Implementations can use SQLite, in-memory storage, or other backends.
pub trait EventRepository {
    /// Create a new event.
    fn create(&self, event: &Event) -> Result<()>;

    /// Get an event by ID.
    fn get(&self, id: Uuid) -> Result<Option<Event>>;

    /// Get all events for a specific date.
    fn get_by_date(&self, date: NaiveDate) -> Result<Vec<Event>>;

    /// Get all events for a month (year, month).
    fn get_by_month(&self, year: i32, month: u32) -> Result<Vec<Event>>;

    /// Update an existing event.
    fn update(&self, event: &Event) -> Result<()>;

    /// Delete an event by ID.
    fn delete(&self, id: Uuid) -> Result<()>;

    /// Get all events (use sparingly).
    fn get_all(&self) -> Result<Vec<Event>>;
}
