//! SQLite implementation of the event repository.

use std::fmt;
use std::path::Path;
use std::sync::Mutex;

use chrono::{Datelike, NaiveDate, NaiveTime};
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{CalendarError, Result};
use crate::event::Event;
use crate::repository::EventRepository;

/// SQLite-backed event repository.
pub struct SqliteEventRepository {
    conn: Mutex<Connection>,
}

impl fmt::Debug for SqliteEventRepository {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteEventRepository")
            .field("conn", &"<Mutex<Connection>>")
            .finish()
    }
}

impl SqliteEventRepository {
    /// Open or create a database at the given path.
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be opened or initialized.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path)?;
        let repo = Self {
            conn: Mutex::new(conn),
        };
        repo.init_schema()?;
        Ok(repo)
    }

    /// Create an in-memory database (useful for testing).
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be created.
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let repo = Self {
            conn: Mutex::new(conn),
        };
        repo.init_schema()?;
        Ok(repo)
    }

    /// Initialize the database schema.
    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r"
            CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                date TEXT NOT NULL,
                start_time TEXT,
                end_time TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_events_date ON events(date);

            CREATE TABLE IF NOT EXISTS week_notes (
                id TEXT PRIMARY KEY,
                week_start TEXT NOT NULL,
                text TEXT NOT NULL,
                position INTEGER NOT NULL DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_week_notes_week ON week_notes(week_start);
            ",
        )?;
        Ok(())
    }

    /// Parse a UUID from a database string.
    fn parse_uuid(s: &str) -> Result<Uuid> {
        Uuid::parse_str(s).map_err(|e| CalendarError::InvalidData(e.to_string()))
    }

    /// Parse a date from ISO format (YYYY-MM-DD).
    fn parse_date(s: &str) -> Result<NaiveDate> {
        NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|e| CalendarError::InvalidData(e.to_string()))
    }

    /// Parse an optional time from HH:MM:SS format.
    fn parse_time(s: Option<String>) -> Result<Option<NaiveTime>> {
        match s {
            Some(t) => {
                let time = NaiveTime::parse_from_str(&t, "%H:%M:%S")
                    .map_err(|e| CalendarError::InvalidData(e.to_string()))?;
                Ok(Some(time))
            }
            None => Ok(None),
        }
    }

    /// Convert a row to an Event.
    fn row_to_event(row: &rusqlite::Row) -> rusqlite::Result<Event> {
        let id_str: String = row.get(0)?;
        let title: String = row.get(1)?;
        let description: Option<String> = row.get(2)?;
        let date_str: String = row.get(3)?;
        let start_time_str: Option<String> = row.get(4)?;
        let end_time_str: Option<String> = row.get(5)?;

        // Parse fields (unwrap is safe here due to schema constraints)
        let id = Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::nil());
        let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
        let start_time = start_time_str
            .and_then(|t| NaiveTime::parse_from_str(&t, "%H:%M:%S").ok());
        let end_time = end_time_str
            .and_then(|t| NaiveTime::parse_from_str(&t, "%H:%M:%S").ok());

        Ok(Event {
            id,
            title,
            description,
            date,
            start_time,
            end_time,
        })
    }
}

impl EventRepository for SqliteEventRepository {
    fn create(&self, event: &Event) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r"
            INSERT INTO events (id, title, description, date, start_time, end_time)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ",
            params![
                event.id.to_string(),
                event.title,
                event.description,
                event.date.format("%Y-%m-%d").to_string(),
                event.start_time.map(|t| t.format("%H:%M:%S").to_string()),
                event.end_time.map(|t| t.format("%H:%M:%S").to_string()),
            ],
        )?;
        Ok(())
    }

    fn get(&self, id: Uuid) -> Result<Option<Event>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, description, date, start_time, end_time FROM events WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id.to_string()])?;
        match rows.next()? {
            Some(row) => Ok(Some(Self::row_to_event(row)?)),
            None => Ok(None),
        }
    }

    fn get_by_date(&self, date: NaiveDate) -> Result<Vec<Event>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r"
            SELECT id, title, description, date, start_time, end_time
            FROM events
            WHERE date = ?1
            ORDER BY start_time NULLS LAST, title
            ",
        )?;

        let date_str = date.format("%Y-%m-%d").to_string();
        let events = stmt
            .query_map(params![date_str], Self::row_to_event)?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(events)
    }

    fn get_by_month(&self, year: i32, month: u32) -> Result<Vec<Event>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r"
            SELECT id, title, description, date, start_time, end_time
            FROM events
            WHERE date >= ?1 AND date < ?2
            ORDER BY date, start_time NULLS LAST, title
            ",
        )?;

        // Calculate month boundaries
        let start_date = NaiveDate::from_ymd_opt(year, month, 1)
            .ok_or_else(|| CalendarError::InvalidData("invalid month".to_string()))?;
        let end_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .ok_or_else(|| CalendarError::InvalidData("invalid month".to_string()))?;

        let events = stmt
            .query_map(
                params![
                    start_date.format("%Y-%m-%d").to_string(),
                    end_date.format("%Y-%m-%d").to_string(),
                ],
                Self::row_to_event,
            )?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(events)
    }

    fn update(&self, event: &Event) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute(
            r"
            UPDATE events
            SET title = ?2, description = ?3, date = ?4, start_time = ?5, end_time = ?6
            WHERE id = ?1
            ",
            params![
                event.id.to_string(),
                event.title,
                event.description,
                event.date.format("%Y-%m-%d").to_string(),
                event.start_time.map(|t| t.format("%H:%M:%S").to_string()),
                event.end_time.map(|t| t.format("%H:%M:%S").to_string()),
            ],
        )?;

        if rows_affected == 0 {
            return Err(CalendarError::NotFound(event.id));
        }
        Ok(())
    }

    fn delete(&self, id: Uuid) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute(
            "DELETE FROM events WHERE id = ?1",
            params![id.to_string()],
        )?;

        if rows_affected == 0 {
            return Err(CalendarError::NotFound(id));
        }
        Ok(())
    }

    fn get_all(&self) -> Result<Vec<Event>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r"
            SELECT id, title, description, date, start_time, end_time
            FROM events
            ORDER BY date, start_time NULLS LAST, title
            ",
        )?;

        let events = stmt
            .query_map([], Self::row_to_event)?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(events)
    }
}

// === Week Notes Implementation ===

use crate::week_note::WeekNote;

impl SqliteEventRepository {
    /// Convert a row to a WeekNote.
    fn row_to_week_note(row: &rusqlite::Row) -> rusqlite::Result<WeekNote> {
        let id_str: String = row.get(0)?;
        let week_start_str: String = row.get(1)?;
        let text: String = row.get(2)?;
        let position: u32 = row.get(3)?;

        let id = Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::nil());
        let week_start = NaiveDate::parse_from_str(&week_start_str, "%Y-%m-%d")
            .unwrap_or_else(|_| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());

        Ok(WeekNote {
            id,
            week_start,
            text,
            position,
        })
    }

    /// Create a new week note.
    pub fn create_week_note(&self, note: &WeekNote) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r"
            INSERT INTO week_notes (id, week_start, text, position)
            VALUES (?1, ?2, ?3, ?4)
            ",
            params![
                note.id.to_string(),
                note.week_start.format("%Y-%m-%d").to_string(),
                note.text,
                note.position,
            ],
        )?;
        Ok(())
    }

    /// Get all notes for a specific week.
    pub fn get_week_notes(&self, week_start: NaiveDate) -> Result<Vec<WeekNote>> {
        // Normalize to Monday
        let days_from_monday = week_start.weekday().num_days_from_monday();
        let monday = week_start - chrono::Duration::days(days_from_monday as i64);

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r"
            SELECT id, week_start, text, position
            FROM week_notes
            WHERE week_start = ?1
            ORDER BY position, id
            ",
        )?;

        let week_start_str = monday.format("%Y-%m-%d").to_string();
        let notes = stmt
            .query_map(params![week_start_str], Self::row_to_week_note)?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(notes)
    }

    /// Update an existing week note.
    pub fn update_week_note(&self, note: &WeekNote) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute(
            r"
            UPDATE week_notes
            SET text = ?2, position = ?3
            WHERE id = ?1
            ",
            params![note.id.to_string(), note.text, note.position,],
        )?;

        if rows_affected == 0 {
            return Err(CalendarError::NotFound(note.id));
        }
        Ok(())
    }

    /// Delete a week note.
    pub fn delete_week_note(&self, id: Uuid) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let rows_affected = conn.execute(
            "DELETE FROM week_notes WHERE id = ?1",
            params![id.to_string()],
        )?;

        if rows_affected == 0 {
            return Err(CalendarError::NotFound(id));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_repo() -> SqliteEventRepository {
        SqliteEventRepository::in_memory().unwrap()
    }

    #[test]
    fn create_and_get_event() {
        let repo = create_test_repo();
        let event = Event::new("Test Event", NaiveDate::from_ymd_opt(2025, 6, 15).unwrap());
        let id = event.id;

        repo.create(&event).unwrap();
        let retrieved = repo.get(id).unwrap().unwrap();

        assert_eq!(retrieved.title, "Test Event");
        assert_eq!(retrieved.id, id);
    }

    #[test]
    fn get_events_by_date() {
        let repo = create_test_repo();
        let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();

        repo.create(&Event::new("Event 1", date)).unwrap();
        repo.create(&Event::new("Event 2", date)).unwrap();
        repo.create(&Event::new("Other day", NaiveDate::from_ymd_opt(2025, 6, 16).unwrap()))
            .unwrap();

        let events = repo.get_by_date(date).unwrap();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn get_events_by_month() {
        let repo = create_test_repo();

        repo.create(&Event::new("June 1", NaiveDate::from_ymd_opt(2025, 6, 1).unwrap()))
            .unwrap();
        repo.create(&Event::new("June 15", NaiveDate::from_ymd_opt(2025, 6, 15).unwrap()))
            .unwrap();
        repo.create(&Event::new("July 1", NaiveDate::from_ymd_opt(2025, 7, 1).unwrap()))
            .unwrap();

        let june_events = repo.get_by_month(2025, 6).unwrap();
        assert_eq!(june_events.len(), 2);
    }

    #[test]
    fn update_event() {
        let repo = create_test_repo();
        let mut event = Event::new("Original", NaiveDate::from_ymd_opt(2025, 6, 15).unwrap());
        let id = event.id;

        repo.create(&event).unwrap();

        event.title = "Updated".to_string();
        repo.update(&event).unwrap();

        let retrieved = repo.get(id).unwrap().unwrap();
        assert_eq!(retrieved.title, "Updated");
    }

    #[test]
    fn delete_event() {
        let repo = create_test_repo();
        let event = Event::new("To Delete", NaiveDate::from_ymd_opt(2025, 6, 15).unwrap());
        let id = event.id;

        repo.create(&event).unwrap();
        assert!(repo.get(id).unwrap().is_some());

        repo.delete(id).unwrap();
        assert!(repo.get(id).unwrap().is_none());
    }

    #[test]
    fn timed_event_persistence() {
        let repo = create_test_repo();
        let event = Event::timed(
            "Meeting",
            NaiveDate::from_ymd_opt(2025, 6, 15).unwrap(),
            NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
            Some(NaiveTime::from_hms_opt(10, 30, 0).unwrap()),
        );
        let id = event.id;

        repo.create(&event).unwrap();
        let retrieved = repo.get(id).unwrap().unwrap();

        assert_eq!(retrieved.start_time, Some(NaiveTime::from_hms_opt(9, 30, 0).unwrap()));
        assert_eq!(retrieved.end_time, Some(NaiveTime::from_hms_opt(10, 30, 0).unwrap()));
    }

    // === Week Notes Tests ===

    #[test]
    fn create_and_get_week_notes() {
        let repo = create_test_repo();
        let monday = NaiveDate::from_ymd_opt(2025, 12, 29).unwrap(); // Monday

        let note1 = WeekNote::new(monday, "First note", 0);
        let note2 = WeekNote::new(monday, "Second note", 1);

        repo.create_week_note(&note1).unwrap();
        repo.create_week_note(&note2).unwrap();

        let notes = repo.get_week_notes(monday).unwrap();
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].text, "First note");
        assert_eq!(notes[1].text, "Second note");
    }

    #[test]
    fn week_notes_normalized_to_monday() {
        let repo = create_test_repo();
        let wednesday = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(); // Wednesday
        let monday = NaiveDate::from_ymd_opt(2025, 12, 29).unwrap(); // Monday of same week

        let note = WeekNote::new(wednesday, "Note for week", 0);
        repo.create_week_note(&note).unwrap();

        // Query with Monday should find it
        let notes = repo.get_week_notes(monday).unwrap();
        assert_eq!(notes.len(), 1);

        // Query with Wednesday should also find it (normalized)
        let notes = repo.get_week_notes(wednesday).unwrap();
        assert_eq!(notes.len(), 1);
    }

    #[test]
    fn update_week_note() {
        let repo = create_test_repo();
        let monday = NaiveDate::from_ymd_opt(2025, 12, 29).unwrap();

        let mut note = WeekNote::new(monday, "Original", 0);
        let id = note.id;

        repo.create_week_note(&note).unwrap();

        note.text = "Updated".to_string();
        repo.update_week_note(&note).unwrap();

        let notes = repo.get_week_notes(monday).unwrap();
        assert_eq!(notes[0].text, "Updated");
        assert_eq!(notes[0].id, id);
    }

    #[test]
    fn delete_week_note() {
        let repo = create_test_repo();
        let monday = NaiveDate::from_ymd_opt(2025, 12, 29).unwrap();

        let note = WeekNote::new(monday, "To delete", 0);
        let id = note.id;

        repo.create_week_note(&note).unwrap();
        assert_eq!(repo.get_week_notes(monday).unwrap().len(), 1);

        repo.delete_week_note(id).unwrap();
        assert_eq!(repo.get_week_notes(monday).unwrap().len(), 0);
    }
}
