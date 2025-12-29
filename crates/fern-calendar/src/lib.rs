//! # Fern Calendar
//!
//! Calendar domain crate for Fern Shell. Provides event and schedule management
//! with SQLite persistence.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use fern_calendar::{Event, SqliteEventRepository, EventRepository};
//! use chrono::NaiveDate;
//!
//! // Open a database
//! let repo = SqliteEventRepository::open("calendar.db").unwrap();
//!
//! // Create an event
//! let event = Event::new("Team Meeting", NaiveDate::from_ymd_opt(2025, 6, 15).unwrap());
//! repo.create(&event).unwrap();
//!
//! // Query events
//! let events = repo.get_by_date(NaiveDate::from_ymd_opt(2025, 6, 15).unwrap()).unwrap();
//! ```

mod error;
mod event;
mod repository;
mod sqlite;
mod week_note;

pub use error::{CalendarError, Result};
pub use event::Event;
pub use repository::EventRepository;
pub use sqlite::SqliteEventRepository;
pub use week_note::WeekNote;
