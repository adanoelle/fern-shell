//! # Log Types
//!
//! Types for log entries and log buffering.

use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Log severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Trace-level debugging information.
    Trace,
    /// Debug-level information.
    Debug,
    /// Informational message.
    Info,
    /// Warning message.
    Warn,
    /// Error message.
    Error,
}

impl LogLevel {
    /// Returns the ANSI color code for this log level.
    #[must_use]
    pub const fn color(&self) -> &'static str {
        match self {
            Self::Trace => "\x1b[90m",   // Gray
            Self::Debug => "\x1b[36m",   // Cyan
            Self::Info => "\x1b[32m",    // Green
            Self::Warn => "\x1b[33m",    // Yellow
            Self::Error => "\x1b[31m",   // Red
        }
    }

    /// Returns the short label for this log level.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

/// A single log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp when the log was created.
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,

    /// Log severity level.
    pub level: LogLevel,

    /// Source component (e.g., "Obs", "ConfigLoader").
    pub source: String,

    /// Log message.
    pub message: String,

    /// Optional structured data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl LogEntry {
    /// Creates a new log entry with the current timestamp.
    #[must_use]
    pub fn new(level: LogLevel, source: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            source: source.into(),
            message: message.into(),
            data: None,
        }
    }

    /// Creates an info log entry.
    #[must_use]
    pub fn info(source: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(LogLevel::Info, source, message)
    }

    /// Creates a warning log entry.
    #[must_use]
    pub fn warn(source: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(LogLevel::Warn, source, message)
    }

    /// Creates an error log entry.
    #[must_use]
    pub fn error(source: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(LogLevel::Error, source, message)
    }

    /// Adds structured data to the log entry.
    #[must_use]
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Returns the timestamp formatted for display.
    #[must_use]
    pub fn formatted_time(&self) -> String {
        self.timestamp
            .with_timezone(&Local)
            .format("%H:%M:%S")
            .to_string()
    }

    /// Returns a formatted string for CLI display.
    #[must_use]
    pub fn format_cli(&self) -> String {
        format!(
            "{} [{}] {}: {}",
            self.formatted_time(),
            self.level.label(),
            self.source,
            self.message
        )
    }

    /// Returns a formatted string with ANSI colors.
    #[must_use]
    pub fn format_colored(&self) -> String {
        format!(
            "\x1b[90m{}\x1b[0m {}[{:5}]\x1b[0m \x1b[1m{}\x1b[0m: {}",
            self.formatted_time(),
            self.level.color(),
            self.level.label(),
            self.source,
            self.message
        )
    }

    /// Checks if this entry matches the given filter string.
    #[must_use]
    pub fn matches_filter(&self, filter: &str) -> bool {
        if filter.is_empty() {
            return true;
        }
        let filter_lower = filter.to_lowercase();
        self.source.to_lowercase().contains(&filter_lower)
            || self.message.to_lowercase().contains(&filter_lower)
            || self.level.label().to_lowercase().contains(&filter_lower)
    }
}

/// Ring buffer for log entries.
#[derive(Debug, Clone)]
pub struct LogBuffer {
    /// Log entries (newest at the end).
    entries: VecDeque<LogEntry>,

    /// Maximum number of entries to keep.
    capacity: usize,

    /// Currently selected entry index (for scrolling).
    selected: Option<usize>,

    /// Current filter string.
    filter: String,
}

impl LogBuffer {
    /// Default capacity for the log buffer.
    pub const DEFAULT_CAPACITY: usize = 500;

    /// Creates a new log buffer with the given capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(capacity),
            capacity,
            selected: None,
            filter: String::new(),
        }
    }

    /// Creates a new log buffer with the default capacity.
    #[must_use]
    pub fn with_default_capacity() -> Self {
        Self::new(Self::DEFAULT_CAPACITY)
    }

    /// Adds a log entry to the buffer.
    ///
    /// If the buffer is at capacity, the oldest entry is removed.
    pub fn push(&mut self, entry: LogEntry) {
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
            // Adjust selected index if it was pointing to the removed entry
            if let Some(idx) = self.selected {
                self.selected = idx.checked_sub(1);
            }
        }
        self.entries.push_back(entry);
    }

    /// Clears all entries from the buffer.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.selected = None;
    }

    /// Returns the number of entries in the buffer.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the buffer is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns all entries in the buffer.
    #[must_use]
    pub fn entries(&self) -> &VecDeque<LogEntry> {
        &self.entries
    }

    /// Returns entries matching the current filter.
    pub fn filtered_entries(&self) -> impl Iterator<Item = &LogEntry> {
        self.entries
            .iter()
            .filter(|e| e.matches_filter(&self.filter))
    }

    /// Returns the number of entries matching the current filter.
    #[must_use]
    pub fn filtered_count(&self) -> usize {
        self.filtered_entries().count()
    }

    /// Sets the filter string.
    pub fn set_filter(&mut self, filter: impl Into<String>) {
        self.filter = filter.into();
        self.selected = None;
    }

    /// Returns the current filter string.
    #[must_use]
    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// Returns the selected entry index.
    #[must_use]
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Scrolls the selection by the given offset.
    pub fn scroll(&mut self, offset: i32) {
        let count = self.filtered_count();
        if count == 0 {
            self.selected = None;
            return;
        }

        let current = self.selected.unwrap_or(count.saturating_sub(1));
        let new_idx = if offset > 0 {
            (current + offset as usize).min(count.saturating_sub(1))
        } else {
            current.saturating_sub((-offset) as usize)
        };
        self.selected = Some(new_idx);
    }

    /// Selects the next entry.
    pub fn select_next(&mut self) {
        self.scroll(1);
    }

    /// Selects the previous entry.
    pub fn select_prev(&mut self) {
        self.scroll(-1);
    }

    /// Jumps to the newest entry.
    pub fn jump_to_end(&mut self) {
        let count = self.filtered_count();
        self.selected = if count > 0 {
            Some(count.saturating_sub(1))
        } else {
            None
        };
    }

    /// Jumps to the oldest entry.
    pub fn jump_to_start(&mut self) {
        self.selected = if self.filtered_count() > 0 {
            Some(0)
        } else {
            None
        };
    }

    /// Syncs the buffer with a list of entries from a file.
    ///
    /// Only adds entries that are newer than the most recent entry in the buffer.
    /// This allows streaming new logs without duplicating existing ones.
    pub fn sync(&mut self, entries: Vec<LogEntry>) {
        // Get the timestamp of the newest entry we have
        let last_timestamp = self.entries.back().map(|e| e.timestamp);

        // Add only entries newer than our last one
        for entry in entries {
            let is_new = match last_timestamp {
                Some(last) => entry.timestamp > last,
                None => true, // Buffer is empty, add all
            };

            if is_new {
                self.push(entry);
            }
        }
    }
}

impl Default for LogBuffer {
    fn default() -> Self {
        Self::with_default_capacity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_buffer_push_and_capacity() {
        let mut buffer = LogBuffer::new(3);

        buffer.push(LogEntry::info("test", "message 1"));
        buffer.push(LogEntry::info("test", "message 2"));
        buffer.push(LogEntry::info("test", "message 3"));
        assert_eq!(buffer.len(), 3);

        // Adding a fourth should evict the first
        buffer.push(LogEntry::info("test", "message 4"));
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.entries[0].message, "message 2");
    }

    #[test]
    fn log_buffer_filter() {
        let mut buffer = LogBuffer::new(10);

        buffer.push(LogEntry::info("Obs", "connected"));
        buffer.push(LogEntry::warn("Config", "missing value"));
        buffer.push(LogEntry::error("Obs", "disconnected"));

        assert_eq!(buffer.filtered_count(), 3);

        buffer.set_filter("Obs");
        assert_eq!(buffer.filtered_count(), 2);

        buffer.set_filter("error");
        assert_eq!(buffer.filtered_count(), 1);

        buffer.set_filter("");
        assert_eq!(buffer.filtered_count(), 3);
    }

    #[test]
    fn log_entry_matches_filter() {
        let entry = LogEntry::info("ConfigLoader", "Loaded config successfully");

        assert!(entry.matches_filter(""));
        assert!(entry.matches_filter("config"));
        assert!(entry.matches_filter("Config"));
        assert!(entry.matches_filter("Loaded"));
        assert!(entry.matches_filter("INFO"));
        assert!(!entry.matches_filter("error"));
    }
}
