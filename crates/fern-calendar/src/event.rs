//! Event domain type.

use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A calendar event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    /// Unique identifier.
    pub id: Uuid,
    /// Event title.
    pub title: String,
    /// Optional description.
    pub description: Option<String>,
    /// Date of the event.
    pub date: NaiveDate,
    /// Start time (None = all-day event).
    pub start_time: Option<NaiveTime>,
    /// End time (None = no specific end time).
    pub end_time: Option<NaiveTime>,
}

impl Event {
    /// Create a new event with a generated UUID.
    #[must_use]
    pub fn new(title: impl Into<String>, date: NaiveDate) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            description: None,
            date,
            start_time: None,
            end_time: None,
        }
    }

    /// Create an all-day event.
    #[must_use]
    pub fn all_day(title: impl Into<String>, date: NaiveDate) -> Self {
        Self::new(title, date)
    }

    /// Create a timed event.
    #[must_use]
    pub fn timed(
        title: impl Into<String>,
        date: NaiveDate,
        start: NaiveTime,
        end: Option<NaiveTime>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            description: None,
            date,
            start_time: Some(start),
            end_time: end,
        }
    }

    /// Check if this is an all-day event.
    #[must_use]
    pub fn is_all_day(&self) -> bool {
        self.start_time.is_none()
    }

    /// Builder: set description.
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_event_has_uuid() {
        let event = Event::new("Test", NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        assert!(!event.id.is_nil());
    }

    #[test]
    fn all_day_event() {
        let event = Event::all_day("Holiday", NaiveDate::from_ymd_opt(2025, 12, 25).unwrap());
        assert!(event.is_all_day());
    }

    #[test]
    fn timed_event() {
        let event = Event::timed(
            "Meeting",
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            Some(NaiveTime::from_hms_opt(10, 0, 0).unwrap()),
        );
        assert!(!event.is_all_day());
        assert_eq!(event.start_time, NaiveTime::from_hms_opt(9, 0, 0));
    }
}
