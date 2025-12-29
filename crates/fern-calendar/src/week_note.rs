//! Weekly notes domain type.

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A note associated with a specific week.
///
/// Notes are tied to the Monday of their week for consistency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeekNote {
    /// Unique identifier.
    pub id: Uuid,
    /// The Monday of the week this note belongs to.
    pub week_start: NaiveDate,
    /// The note text.
    pub text: String,
    /// Position for ordering (0-indexed).
    pub position: u32,
}

impl WeekNote {
    /// Create a new week note.
    ///
    /// # Arguments
    ///
    /// * `week_start` - The Monday of the week (will be normalized if not a Monday)
    /// * `text` - The note text
    /// * `position` - The position in the list
    #[must_use]
    pub fn new(week_start: NaiveDate, text: impl Into<String>, position: u32) -> Self {
        // Normalize to Monday of the week
        let days_from_monday = week_start.weekday().num_days_from_monday();
        let monday = week_start - chrono::Duration::days(days_from_monday as i64);

        Self {
            id: Uuid::new_v4(),
            week_start: monday,
            text: text.into(),
            position,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_note_normalizes_to_monday() {
        // Create a note for a Wednesday
        let wednesday = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(); // Wednesday
        let note = WeekNote::new(wednesday, "Test note", 0);

        // Should normalize to Monday (Dec 29, 2025)
        assert_eq!(
            note.week_start,
            NaiveDate::from_ymd_opt(2025, 12, 29).unwrap()
        );
    }

    #[test]
    fn new_note_keeps_monday() {
        let monday = NaiveDate::from_ymd_opt(2025, 12, 29).unwrap(); // Monday
        let note = WeekNote::new(monday, "Test note", 0);

        assert_eq!(note.week_start, monday);
    }
}
