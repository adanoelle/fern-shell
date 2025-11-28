//! # Notification Integration for Errors
//!
//! This module provides the [`Notifiable`] trait and [`Notification`] struct,
//! enabling errors to be displayed as desktop notifications.
//!
//! ## Design Rationale
//!
//! Errors in a desktop shell should be visible to users, not buried in logs.
//! By implementing [`Notifiable`] on error types, we can:
//!
//! 1. Display rich, actionable notifications when things go wrong
//! 2. Categorize notifications by severity for appropriate presentation
//! 3. Provide suggestions that help users fix problems themselves
//!
//! ## Integration Flow
//!
//! ```text
//! ConfigError implements Notifiable
//!         │
//!         ▼
//! error.to_notification()
//!         │
//!         ▼
//! NotifyPort::send(notification)
//!         │
//!         ▼
//! D-Bus adapter → Desktop notification service
//! ```
//!
//! ## Example
//!
//! ```rust,ignore
//! use fernctl::error::{ConfigError, Notifiable, Notification};
//!
//! let error = ConfigError::InvalidColor {
//!     value: "#gg0000".to_string(),
//!     span: None,
//!     source_code: None,
//! };
//!
//! // Convert to notification
//! let notification = error.to_notification();
//!
//! // Send via notification port
//! notify_port.send(notification)?;
//! ```

use super::Severity;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Trait for types that can be converted to user-facing notifications.
///
/// Implement this trait on error types to enable rich desktop notifications.
/// The trait provides all the information needed to display a helpful,
/// actionable notification to the user.
///
/// # Required Methods
///
/// | Method | Purpose |
/// |--------|---------|
/// | [`severity`](Notifiable::severity) | How serious is this? |
/// | [`title`](Notifiable::title) | Short summary for notification header |
/// | [`body`](Notifiable::body) | Detailed explanation |
///
/// # Optional Methods
///
/// | Method | Purpose | Default |
/// |--------|---------|---------|
/// | [`suggestion`](Notifiable::suggestion) | How to fix it | `None` |
/// | [`code`](Notifiable::code) | Error code for docs | `None` |
///
/// # Example Implementation
///
/// ```rust,ignore
/// use fernctl::error::{Notifiable, Severity};
///
/// struct MyError {
///     message: String,
/// }
///
/// impl Notifiable for MyError {
///     fn severity(&self) -> Severity {
///         Severity::Error
///     }
///
///     fn title(&self) -> String {
///         "Something Went Wrong".to_string()
///     }
///
///     fn body(&self) -> String {
///         self.message.clone()
///     }
///
///     fn suggestion(&self) -> Option<String> {
///         Some("Try restarting the shell".to_string())
///     }
///
///     fn code(&self) -> Option<&'static str> {
///         Some("my::error::code")
///     }
/// }
/// ```
pub trait Notifiable {
    /// Returns the severity level of this notification.
    ///
    /// Severity determines the visual presentation and behavior:
    ///
    /// - [`Info`](Severity::Info) — Subtle, may auto-dismiss
    /// - [`Warning`](Severity::Warning) — Visible, yellow/orange styling
    /// - [`Error`](Severity::Error) — Prominent, red styling, requires attention
    /// - [`Fatal`](Severity::Fatal) — Critical, may block other UI
    fn severity(&self) -> Severity;

    /// Returns a short title for the notification header.
    ///
    /// This should be concise (ideally under 50 characters) and immediately
    /// convey the nature of the notification.
    ///
    /// # Good Titles
    ///
    /// - "Invalid Color"
    /// - "Config Loaded"
    /// - "Module Failed: Clock"
    ///
    /// # Bad Titles
    ///
    /// - "Error" (too vague)
    /// - "The color value you specified in config.toml is not valid" (too long)
    fn title(&self) -> String;

    /// Returns the detailed body text of the notification.
    ///
    /// This should explain what happened. For errors, include specifics about
    /// what went wrong. Keep it readable — avoid raw error dumps.
    fn body(&self) -> String;

    /// Returns an optional suggestion for fixing the problem.
    ///
    /// When present, this should be an actionable instruction the user can
    /// follow. Be specific and helpful.
    ///
    /// # Good Suggestions
    ///
    /// - "Use hex format: #RRGGBB"
    /// - "Did you mean `color`?"
    /// - "Check that QuickShell is running"
    ///
    /// # Bad Suggestions
    ///
    /// - "Fix the error" (not actionable)
    /// - "See documentation" (too vague — link to specific docs instead)
    fn suggestion(&self) -> Option<String> {
        None
    }

    /// Returns an optional error code for documentation lookup.
    ///
    /// Error codes follow the pattern `category::subcategory::name` and can
    /// be used to find detailed documentation.
    ///
    /// # Example Codes
    ///
    /// - `fern::config::invalid_color`
    /// - `fern::io::permission_denied`
    /// - `fern::ipc::timeout`
    fn code(&self) -> Option<&'static str> {
        None
    }

    /// Converts this notifiable into a [`Notification`] struct.
    ///
    /// This is a convenience method that creates a notification with all
    /// fields populated from the trait methods.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let notification = error.to_notification();
    /// notify_port.send(notification)?;
    /// ```
    fn to_notification(&self) -> Notification {
        Notification {
            severity: self.severity(),
            title: self.title(),
            body: self.body(),
            suggestion: self.suggestion(),
            code: self.code().map(String::from),
            timestamp: SystemTime::now(),
        }
    }
}

/// A notification ready to be sent to the desktop notification service.
///
/// `Notification` is a plain data struct containing all information needed
/// to display a notification. It is serializable for IPC transmission.
///
/// # Creating Notifications
///
/// Notifications are typically created from types implementing [`Notifiable`]:
///
/// ```rust,ignore
/// let notification = error.to_notification();
/// ```
///
/// But you can also create them directly:
///
/// ```rust,ignore
/// let notification = Notification::new(
///     Severity::Info,
///     "Config Loaded",
///     "Successfully loaded configuration from config.toml",
/// );
/// ```
///
/// # Serialization
///
/// Notifications serialize to JSON for D-Bus transmission:
///
/// ```rust,ignore
/// let json = serde_json::to_string(&notification)?;
/// // {"severity":"info","title":"...","body":"...",...}
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    /// Severity level determining visual presentation.
    pub severity: Severity,

    /// Short title for the notification header.
    pub title: String,

    /// Detailed body text.
    pub body: String,

    /// Optional actionable suggestion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,

    /// Optional error code for documentation lookup.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// When the notification was created.
    #[serde(with = "system_time_serde")]
    pub timestamp: SystemTime,
}

impl Notification {
    /// Creates a new notification with the given severity, title, and body.
    ///
    /// # Arguments
    ///
    /// * `severity` — How serious is this notification
    /// * `title` — Short summary for the header
    /// * `body` — Detailed explanation
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::error::{Notification, Severity};
    ///
    /// let notification = Notification::new(
    ///     Severity::Info,
    ///     "Config Loaded",
    ///     "Successfully loaded configuration",
    /// );
    /// ```
    #[must_use]
    pub fn new(
        severity: Severity,
        title: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            title: title.into(),
            body: body.into(),
            suggestion: None,
            code: None,
            timestamp: SystemTime::now(),
        }
    }

    /// Creates an informational notification.
    ///
    /// Convenience constructor for [`Severity::Info`] notifications.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::error::Notification;
    ///
    /// let notification = Notification::info("Config Loaded", "All modules initialized");
    /// ```
    #[must_use]
    pub fn info(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self::new(Severity::Info, title, body)
    }

    /// Creates a warning notification.
    ///
    /// Convenience constructor for [`Severity::Warning`] notifications.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::error::Notification;
    ///
    /// let notification = Notification::warning(
    ///     "Unknown Key",
    ///     "Configuration contains unknown key 'colour'",
    /// );
    /// ```
    #[must_use]
    pub fn warning(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self::new(Severity::Warning, title, body)
    }

    /// Creates an error notification.
    ///
    /// Convenience constructor for [`Severity::Error`] notifications.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::error::Notification;
    ///
    /// let notification = Notification::error(
    ///     "Invalid Color",
    ///     "Color '#gg0000' is not valid hex",
    /// );
    /// ```
    #[must_use]
    pub fn error(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self::new(Severity::Error, title, body)
    }

    /// Adds a suggestion to this notification.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::error::Notification;
    ///
    /// let notification = Notification::error("Invalid Color", "...")
    ///     .with_suggestion("Use hex format: #RRGGBB");
    /// ```
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Adds an error code to this notification.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::error::Notification;
    ///
    /// let notification = Notification::error("Invalid Color", "...")
    ///     .with_code("fern::config::invalid_color");
    /// ```
    #[must_use]
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Returns `true` if this notification indicates an error condition.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fernctl::error::{Notification, Severity};
    ///
    /// let info = Notification::info("Test", "Test");
    /// let error = Notification::error("Test", "Test");
    ///
    /// assert!(!info.is_error());
    /// assert!(error.is_error());
    /// ```
    #[must_use]
    pub fn is_error(&self) -> bool {
        self.severity.is_error()
    }
}

impl std::fmt::Display for Notification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}: {}", self.severity.icon(), self.title, self.body)
    }
}

/// Serde support for `SystemTime`.
mod system_time_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_new() {
        let n = Notification::new(Severity::Error, "Title", "Body");
        assert_eq!(n.severity, Severity::Error);
        assert_eq!(n.title, "Title");
        assert_eq!(n.body, "Body");
        assert!(n.suggestion.is_none());
        assert!(n.code.is_none());
    }

    #[test]
    fn notification_builders() {
        let n = Notification::error("Title", "Body")
            .with_suggestion("Fix it")
            .with_code("fern::test");

        assert_eq!(n.suggestion, Some("Fix it".to_string()));
        assert_eq!(n.code, Some("fern::test".to_string()));
    }

    #[test]
    fn notification_display() {
        let n = Notification::warning("Test", "Test body");
        let display = n.to_string();
        assert!(display.contains("Test"));
        assert!(display.contains("Test body"));
    }

    #[test]
    fn notification_serialization_roundtrip() {
        let original = Notification::error("Title", "Body")
            .with_suggestion("Fix it")
            .with_code("fern::test");

        let json = serde_json::to_string(&original).expect("serialize");
        let restored: Notification = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(original.severity, restored.severity);
        assert_eq!(original.title, restored.title);
        assert_eq!(original.body, restored.body);
        assert_eq!(original.suggestion, restored.suggestion);
        assert_eq!(original.code, restored.code);
    }
}
