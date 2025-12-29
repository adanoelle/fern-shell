//! Data types for calendar events, goals, and intentions.

use std::collections::HashMap;

use chrono::{NaiveDate, NaiveTime};
use uuid::Uuid;

/// A calendar event.
#[derive(Debug, Clone)]
pub struct Event {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    /// Start time of the event. None = all-day event (sorted last).
    pub start_time: Option<NaiveTime>,
    /// End time of the event. None = no end time specified.
    pub end_time: Option<NaiveTime>,
    pub date: NaiveDate,
}

impl From<fern_calendar::Event> for Event {
    fn from(e: fern_calendar::Event) -> Self {
        Self {
            id: e.id,
            title: e.title,
            description: e.description,
            start_time: e.start_time,
            end_time: e.end_time,
            date: e.date,
        }
    }
}

impl From<&Event> for fern_calendar::Event {
    fn from(e: &Event) -> Self {
        Self {
            id: e.id,
            title: e.title.clone(),
            description: e.description.clone(),
            start_time: e.start_time,
            end_time: e.end_time,
            date: e.date,
        }
    }
}

/// A weekly note - a bullet point for the week.
#[derive(Debug, Clone)]
pub struct WeekNote {
    pub id: Uuid,
    pub week_start: NaiveDate,
    pub text: String,
    pub position: u32,
}

impl From<fern_calendar::WeekNote> for WeekNote {
    fn from(n: fern_calendar::WeekNote) -> Self {
        Self {
            id: n.id,
            week_start: n.week_start,
            text: n.text,
            position: n.position,
        }
    }
}

impl From<&WeekNote> for fern_calendar::WeekNote {
    fn from(n: &WeekNote) -> Self {
        Self {
            id: n.id,
            week_start: n.week_start,
            text: n.text.clone(),
            position: n.position,
        }
    }
}

/// A monthly goal.
#[derive(Debug, Clone)]
pub struct Goal {
    pub id: u64,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
}

/// A monthly intention - a guiding focus for the month.
#[derive(Debug, Clone)]
pub struct Intention {
    pub id: u64,
    pub text: String,
}

/// All data for a given month.
#[derive(Debug, Clone, Default)]
pub struct MonthData {
    /// Events indexed by day of month (1-31).
    pub events_by_day: HashMap<u32, Vec<Event>>,
    /// Monthly goals.
    pub goals: Vec<Goal>,
    /// Monthly intentions.
    pub intentions: Vec<Intention>,
}

impl MonthData {
    /// Get events for a specific day, sorted by start time (all-day events last).
    pub fn events_for_day(&self, day: u32) -> Vec<&Event> {
        let mut events: Vec<_> = self
            .events_by_day
            .get(&day)
            .map(|v| v.iter().collect())
            .unwrap_or_default();

        events.sort_by_key(|e| match e.start_time {
            Some(t) => (0, t),
            None => (1, NaiveTime::from_hms_opt(23, 59, 59).unwrap()),
        });

        events
    }

    /// Count events for a specific day.
    pub fn event_count(&self, day: u32) -> usize {
        self.events_by_day.get(&day).map(|v| v.len()).unwrap_or(0)
    }
}

/// Generate sample data for testing.
pub fn sample_data(year: i32, month: u32) -> MonthData {
    let mut events_by_day: HashMap<u32, Vec<Event>> = HashMap::new();

    // Sample events for the 15th
    let date_15 = NaiveDate::from_ymd_opt(year, month, 15).unwrap();
    events_by_day.insert(
        15,
        vec![
            Event {
                id: Uuid::new_v4(),
                title: "Lunch with Alex".to_string(),
                description: Some("Meet at the usual place".to_string()),
                start_time: NaiveTime::from_hms_opt(12, 0, 0),
                end_time: NaiveTime::from_hms_opt(13, 0, 0),
                date: date_15,
            },
            Event {
                id: Uuid::new_v4(),
                title: "Code review".to_string(),
                description: Some("Review PR #123".to_string()),
                start_time: NaiveTime::from_hms_opt(15, 0, 0),
                end_time: NaiveTime::from_hms_opt(16, 30, 0),
                date: date_15,
            },
            Event {
                id: Uuid::new_v4(),
                title: "Release day".to_string(),
                description: Some("Deploy v2.0 to production".to_string()),
                start_time: None, // All-day event
                end_time: None,
                date: date_15,
            },
            Event {
                id: Uuid::new_v4(),
                title: "Team standup".to_string(),
                description: None,
                start_time: NaiveTime::from_hms_opt(9, 30, 0),
                end_time: NaiveTime::from_hms_opt(9, 45, 0),
                date: date_15,
            },
        ],
    );

    // Sample events for the 17th
    if let Some(date_17) = NaiveDate::from_ymd_opt(year, month, 17) {
        events_by_day.insert(
            17,
            vec![Event {
                id: Uuid::new_v4(),
                title: "Dentist appointment".to_string(),
                description: Some("Annual checkup".to_string()),
                start_time: NaiveTime::from_hms_opt(10, 0, 0),
                end_time: NaiveTime::from_hms_opt(11, 0, 0),
                date: date_17,
            }],
        );
    }

    // Sample events for the 20th (today marker in many cases)
    if let Some(date_20) = NaiveDate::from_ymd_opt(year, month, 20) {
        events_by_day.insert(
            20,
            vec![
                Event {
                    id: Uuid::new_v4(),
                    title: "Morning jog".to_string(),
                    description: None,
                    start_time: NaiveTime::from_hms_opt(6, 30, 0),
                    end_time: NaiveTime::from_hms_opt(7, 15, 0),
                    date: date_20,
                },
                Event {
                    id: Uuid::new_v4(),
                    title: "Project deadline".to_string(),
                    description: Some("Submit final deliverables".to_string()),
                    start_time: NaiveTime::from_hms_opt(17, 0, 0),
                    end_time: None, // No specific end time
                    date: date_20,
                },
            ],
        );
    }

    // Sample goals
    let goals = vec![
        Goal {
            id: 1,
            title: "Ship calendar feature".to_string(),
            description: Some("Complete the month view implementation".to_string()),
            completed: true,
        },
        Goal {
            id: 2,
            title: "Read 2 books".to_string(),
            description: Some("Currently reading: The Pragmatic Programmer".to_string()),
            completed: false,
        },
        Goal {
            id: 3,
            title: "Exercise 3x per week".to_string(),
            description: None,
            completed: false,
        },
    ];

    // Sample intentions
    let intentions = vec![
        Intention {
            id: 1,
            text: "Focus on deep work and minimize distractions".to_string(),
        },
        Intention {
            id: 2,
            text: "Ship meaningful features this month".to_string(),
        },
        Intention {
            id: 3,
            text: "Practice gratitude daily".to_string(),
        },
    ];

    MonthData {
        events_by_day,
        goals,
        intentions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_events_sorted_by_time() {
        let data = sample_data(2025, 12);
        let events = data.events_for_day(15);

        // Should have 4 events
        assert_eq!(events.len(), 4);

        // First should be 9:30 standup
        assert_eq!(events[0].title, "Team standup");

        // Last should be the all-day event
        assert!(events.last().unwrap().start_time.is_none());
    }

    #[test]
    fn test_event_count() {
        let data = sample_data(2025, 12);
        assert_eq!(data.event_count(15), 4);
        assert_eq!(data.event_count(17), 1);
        assert_eq!(data.event_count(1), 0);
    }
}
