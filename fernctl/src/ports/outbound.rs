//! # Outbound Ports — Driven Services
//!
//! Outbound ports define how the domain can **use** external services. They
//! represent capabilities the application needs but doesn't implement itself:
//! persistence, notifications, inter-process communication.
//!
//! ## Overview
//!
//! ```text
//!                          ┌──────────────┐
//!                          │    DOMAIN    │
//!                          │    Theme     │
//!                          │  Validation  │
//!                          └──────┬───────┘
//!                                 │
//!      ╔══════════════════════════╧══════════════════════════════╗
//!      ║                   OUTBOUND PORTS                        ║
//!      ║                                                         ║
//!      ║   PersistPort ────► Save themes to disk                 ║
//!      ║   NotifyPort ─────► Show desktop notifications          ║
//!      ║   IpcPort ────────► Communicate with QuickShell         ║
//!      ║                                                         ║
//!      ╚═════╤════════════════════╤════════════════════╤═════════╝
//!            │                    │                    │
//!      ┌─────▼──────┐       ┌─────▼──────┐       ┌─────▼──────┐
//!      │FileAdapter │       │NotifAdapter│       │DbusAdapter │
//!      │            │       │            │       │            │
//!      │ Writes to  │       │ Sends via  │       │ Calls via  │
//!      │ filesystem │       │  D-Bus     │       │  D-Bus     │
//!      └─────┬──────┘       └─────┬──────┘       └─────┬──────┘
//!            │                    │                    │
//!      ┌─────▼──────┐       ┌─────▼──────┐       ┌─────▼──────┐
//!      │ Filesystem │       │   Desktop  │       │ QuickShell │
//!      │            │       │Notification│       │            │
//!      │ ~/.config/ │       │  Service   │       │   D-Bus    │
//!      │   fern/    │       │            │       │  Service   │
//!      └────────────┘       └────────────┘       └────────────┘
//! ```
//!
//! ## Dependency Inversion
//!
//! Outbound ports demonstrate the **Dependency Inversion Principle**:
//!
//! - The domain depends on *abstractions* (port traits)
//! - Adapters depend on the domain (they implement port traits)
//! - External systems are accessed only through adapters
//!
//! This means the domain never knows about file systems, D-Bus, or any
//! specific technology. It just calls trait methods.
//!
//! ```rust,ignore
//! // Domain code - doesn't know about D-Bus
//! fn reload_theme(ipc: &impl IpcPort, theme: &Theme) -> Result<()> {
//!     ipc.send_reload(theme)?;
//!     Ok(())
//! }
//!
//! // Adapter - implements the port using D-Bus
//! struct DbusIpcAdapter { /* D-Bus connection */ }
//!
//! impl IpcPort for DbusIpcAdapter {
//!     fn send_reload(&self, theme: &Theme) -> Result<()> {
//!         // D-Bus specific code here
//!     }
//! }
//! ```
//!
//! ## Port Responsibilities
//!
//! | Port | Responsibility | External System |
//! |------|----------------|-----------------|
//! | [`PersistPort`] | Save/load theme files | Filesystem |
//! | [`NotifyPort`] | Send desktop notifications | Notification daemon |
//! | [`IpcPort`] | Communicate with QuickShell | D-Bus |
//!
//! ## Testing with Mocks
//!
//! Because ports are traits, they're easy to mock for testing:
//!
//! ```rust,ignore
//! use std::sync::atomic::{AtomicUsize, Ordering};
//!
//! struct MockNotifyPort {
//!     call_count: AtomicUsize,
//! }
//!
//! impl NotifyPort for MockNotifyPort {
//!     fn send(&self, _: Notification) -> Result<()> {
//!         self.call_count.fetch_add(1, Ordering::SeqCst);
//!         Ok(())
//!     }
//! }
//!
//! #[test]
//! fn sends_notification_on_error() {
//!     let mock = MockNotifyPort { call_count: AtomicUsize::new(0) };
//!     // ... test code that should send a notification ...
//!     assert_eq!(mock.call_count.load(Ordering::SeqCst), 1);
//! }
//! ```

use crate::domain::theme::Theme;
use crate::error::{Notification, Result};
use std::path::Path;

// ============================================================================
// PersistPort Trait
// ============================================================================

/// Port for persisting configuration and themes to storage.
///
/// `PersistPort` abstracts over different storage backends, enabling the
/// domain to save and load themes without knowing the underlying storage
/// mechanism.
///
/// # Implementations
///
/// - **Filesystem** — Writes JSON files to `~/.config/fern/`
/// - **XDG** — Follows XDG Base Directory Specification
/// - **Memory** — In-memory storage for testing
///
/// # File Structure
///
/// The recommended file structure for persisted themes:
///
/// ```text
/// ~/.config/fern/
/// ├── config.toml      # User configuration (source of truth)
/// ├── config.json      # Generated JSON for QuickShell
/// └── themes/
///     ├── dark.json    # Built-in dark theme
///     ├── light.json   # Built-in light theme
///     └── custom.json  # User-created themes
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use fernctl::ports::outbound::PersistPort;
/// use fernctl::adapters::FileSystemAdapter;
///
/// let adapter = FileSystemAdapter::new();
///
/// // Save a theme
/// adapter.save_theme(&theme, "~/.config/fern/config.json")?;
///
/// // Load a theme
/// let loaded = adapter.load_theme("~/.config/fern/config.json")?;
/// ```
pub trait PersistPort: Send + Sync {
    /// Saves a theme to the specified path.
    ///
    /// The theme is serialized to JSON format for QuickShell compatibility.
    ///
    /// # Arguments
    ///
    /// * `theme` — The theme to save
    /// * `path` — The file path to write to
    ///
    /// # Errors
    ///
    /// Returns [`FernError::Io`] if the file cannot be written.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// adapter.save_theme(&theme, "config.json")?;
    /// ```
    fn save_theme(&self, theme: &Theme, path: impl AsRef<Path>) -> Result<()>;

    /// Loads a theme from the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` — The file path to read from
    ///
    /// # Returns
    ///
    /// The loaded [`Theme`].
    ///
    /// # Errors
    ///
    /// - [`FernError::Io`] if the file cannot be read
    /// - [`ConfigError`] if the file content is invalid
    fn load_theme(&self, path: impl AsRef<Path>) -> Result<Theme>;

    /// Returns the default configuration directory.
    ///
    /// This follows platform conventions:
    /// - Linux: `~/.config/fern/`
    /// - macOS: `~/Library/Application Support/fern/`
    /// - Windows: `%APPDATA%\fern\`
    ///
    /// # Returns
    ///
    /// The path to the configuration directory, or `None` if it cannot be
    /// determined.
    fn config_dir(&self) -> Option<std::path::PathBuf>;

    /// Ensures the configuration directory exists.
    ///
    /// Creates the directory and any parent directories if they don't exist.
    ///
    /// # Errors
    ///
    /// Returns [`FernError::Io`] if the directory cannot be created.
    fn ensure_config_dir(&self) -> Result<std::path::PathBuf>;

    /// Checks if a configuration file exists.
    ///
    /// # Arguments
    ///
    /// * `path` — The file path to check
    ///
    /// # Returns
    ///
    /// `true` if the file exists and is readable.
    fn exists(&self, path: impl AsRef<Path>) -> bool;
}

// ============================================================================
// NotifyPort Trait
// ============================================================================

/// Port for sending desktop notifications.
///
/// `NotifyPort` provides a way to display notifications to the user. This is
/// used for:
///
/// - Configuration errors that need user attention
/// - Successful theme reloads
/// - Warnings about deprecated options
///
/// # Notification Levels
///
/// | Level | Use Case | User Impact |
/// |-------|----------|-------------|
/// | Info | Success messages | Brief, auto-dismiss |
/// | Warning | Potential issues | Visible but non-blocking |
/// | Error | Problems requiring action | Persistent, prominent |
///
/// # Desktop Integration
///
/// The notification adapter should use the system notification service:
///
/// - **Linux**: D-Bus `org.freedesktop.Notifications`
/// - **macOS**: `NSUserNotificationCenter`
/// - **Windows**: Toast notifications
///
/// # Example
///
/// ```rust,ignore
/// use fernctl::ports::outbound::NotifyPort;
/// use fernctl::error::Notification;
///
/// // Send an info notification
/// let notification = Notification::info(
///     "Theme Loaded",
///     "Dark theme applied successfully",
/// );
/// adapter.send(notification)?;
///
/// // Send an error notification
/// let notification = Notification::error(
///     "Invalid Color",
///     "Color '#gg0000' is not valid",
/// ).with_suggestion("Use hex format: #RRGGBB");
/// adapter.send(notification)?;
/// ```
///
/// # Suppressing Notifications
///
/// Users may want to disable notifications. Adapters should respect a
/// configuration option for this:
///
/// ```rust,ignore
/// if !config.notifications_enabled {
///     return Ok(()); // Silently succeed
/// }
/// ```
pub trait NotifyPort: Send + Sync {
    /// Sends a notification to the desktop notification service.
    ///
    /// # Arguments
    ///
    /// * `notification` — The notification to display
    ///
    /// # Errors
    ///
    /// Returns an error if the notification cannot be sent. However,
    /// notification failures are typically non-fatal — the application
    /// should continue even if notifications are unavailable.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let notification = Notification::info("Test", "Hello world");
    /// if let Err(e) = adapter.send(notification) {
    ///     // Log but don't fail
    ///     log::warn!("Could not send notification: {}", e);
    /// }
    /// ```
    fn send(&self, notification: Notification) -> Result<()>;

    /// Sends an info-level notification.
    ///
    /// Convenience method for sending simple informational notifications.
    ///
    /// # Arguments
    ///
    /// * `title` — The notification title
    /// * `body` — The notification body text
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// adapter.info("Theme Loaded", "Configuration applied")?;
    /// ```
    fn info(&self, title: impl Into<String>, body: impl Into<String>) -> Result<()> {
        self.send(Notification::info(title, body))
    }

    /// Sends a warning-level notification.
    ///
    /// Convenience method for sending warning notifications.
    ///
    /// # Arguments
    ///
    /// * `title` — The notification title
    /// * `body` — The notification body text
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// adapter.warning("Deprecated Option", "The 'colour' key is deprecated")?;
    /// ```
    fn warning(&self, title: impl Into<String>, body: impl Into<String>) -> Result<()> {
        self.send(Notification::warning(title, body))
    }

    /// Sends an error-level notification.
    ///
    /// Convenience method for sending error notifications.
    ///
    /// # Arguments
    ///
    /// * `title` — The notification title
    /// * `body` — The notification body text
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// adapter.error("Config Error", "Invalid color format")?;
    /// ```
    fn error(&self, title: impl Into<String>, body: impl Into<String>) -> Result<()> {
        self.send(Notification::error(title, body))
    }

    /// Checks if notifications are available on this system.
    ///
    /// # Returns
    ///
    /// `true` if the notification service is available and responding.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if adapter.is_available() {
    ///     adapter.send(notification)?;
    /// } else {
    ///     eprintln!("{}", notification); // Fallback to stderr
    /// }
    /// ```
    fn is_available(&self) -> bool {
        true // Adapters can override with actual check
    }
}

// ============================================================================
// IpcPort Trait
// ============================================================================

/// Port for inter-process communication with QuickShell.
///
/// `IpcPort` provides communication between `fernctl` and the running
/// QuickShell instance. This enables:
///
/// - Live theme reloading without restarting the shell
/// - Querying current shell state
/// - Sending commands to the shell
///
/// # Communication Protocol
///
/// The default implementation uses D-Bus for IPC:
///
/// ```text
/// fernctl                           QuickShell
///   │                                   │
///   │    D-Bus: org.fern.Shell          │
///   │ ───────────────────────────────►  │
///   │    Method: ReloadTheme            │
///   │    Params: { path: "..." }        │
///   │                                   │
///   │    Response: { success: true }    │
///   │ ◄───────────────────────────────  │
///   │                                   │
/// ```
///
/// # Alternative Transports
///
/// While D-Bus is the primary transport, adapters can implement other
/// mechanisms:
///
/// - **Unix Socket**: Direct socket communication
/// - **File Watching**: QuickShell watches a file for changes
/// - **Signals**: Unix signals for simple commands
///
/// # Example
///
/// ```rust,ignore
/// use fernctl::ports::outbound::IpcPort;
/// use fernctl::adapters::DbusIpcAdapter;
///
/// let ipc = DbusIpcAdapter::new()?;
///
/// // Reload theme
/// ipc.reload_theme(&theme)?;
///
/// // Check if shell is running
/// if ipc.is_shell_running() {
///     ipc.send_command("refresh")?;
/// }
/// ```
///
/// # Error Handling
///
/// IPC operations can fail for various reasons:
///
/// | Error | Cause | Recovery |
/// |-------|-------|----------|
/// | Not running | QuickShell not started | Start the shell |
/// | Timeout | Shell not responding | Restart the shell |
/// | Protocol error | Version mismatch | Update fernctl |
pub trait IpcPort: Send + Sync {
    /// Signals QuickShell to reload its theme.
    ///
    /// This sends the updated theme to QuickShell for live application.
    /// The shell will re-read the configuration and update all components.
    ///
    /// # Arguments
    ///
    /// * `theme` — The new theme to apply
    ///
    /// # Errors
    ///
    /// - [`FernError::Ipc`] if QuickShell is not running
    /// - [`FernError::Ipc`] if the reload times out
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Save theme to disk first
    /// persist.save_theme(&theme, config_path)?;
    ///
    /// // Then signal reload
    /// ipc.reload_theme(&theme)?;
    /// ```
    fn reload_theme(&self, theme: &Theme) -> Result<()>;

    /// Sends a raw command to QuickShell.
    ///
    /// Commands are shell-specific strings that trigger actions.
    ///
    /// # Built-in Commands
    ///
    /// | Command | Effect |
    /// |---------|--------|
    /// | `refresh` | Refresh all modules |
    /// | `restart` | Restart the shell |
    /// | `quit` | Gracefully exit |
    ///
    /// # Arguments
    ///
    /// * `command` — The command string to send
    ///
    /// # Errors
    ///
    /// Returns an error if the command cannot be sent or is rejected.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// ipc.send_command("refresh")?;
    /// ```
    fn send_command(&self, command: &str) -> Result<()>;

    /// Checks if QuickShell is currently running.
    ///
    /// This performs a lightweight check (e.g., pinging the D-Bus service)
    /// to determine if the shell is available for commands.
    ///
    /// # Returns
    ///
    /// `true` if QuickShell is running and responding.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if !ipc.is_shell_running() {
    ///     eprintln!("QuickShell is not running");
    ///     return Ok(()); // Theme saved, will be applied on next start
    /// }
    /// ```
    fn is_shell_running(&self) -> bool;

    /// Returns the version of the running QuickShell instance.
    ///
    /// Useful for compatibility checks and debugging.
    ///
    /// # Returns
    ///
    /// The version string, or `None` if the shell is not running or
    /// the version cannot be determined.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if let Some(version) = ipc.shell_version() {
    ///     println!("QuickShell version: {}", version);
    /// }
    /// ```
    fn shell_version(&self) -> Option<String>;

    /// Pings the shell to check connectivity.
    ///
    /// This is a more thorough check than [`is_shell_running`], verifying
    /// that the shell can receive and respond to messages.
    ///
    /// # Returns
    ///
    /// The round-trip time in milliseconds, or an error if the ping failed.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// match ipc.ping() {
    ///     Ok(ms) => println!("Shell responded in {}ms", ms),
    ///     Err(e) => eprintln!("Shell not responding: {}", e),
    /// }
    /// ```
    fn ping(&self) -> Result<u64>;
}

// ============================================================================
// Null Implementations for Testing
// ============================================================================

/// A no-op implementation of [`NotifyPort`] for testing.
///
/// This adapter accepts all notifications but doesn't actually send them.
/// Useful for unit tests where notification delivery isn't being tested.
///
/// # Example
///
/// ```rust
/// use fernctl::ports::outbound::{NotifyPort, NullNotifyPort};
/// use fernctl::error::Notification;
///
/// let notifier = NullNotifyPort;
/// let notification = Notification::info("Test", "Body");
///
/// // This succeeds but does nothing
/// notifier.send(notification).unwrap();
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct NullNotifyPort;

impl NotifyPort for NullNotifyPort {
    fn send(&self, _notification: Notification) -> Result<()> {
        Ok(())
    }

    fn is_available(&self) -> bool {
        false
    }
}

/// A no-op implementation of [`IpcPort`] for testing.
///
/// This adapter simulates a disconnected shell. All operations succeed
/// but [`is_shell_running`] returns `false`.
///
/// # Example
///
/// ```rust
/// use fernctl::ports::outbound::{IpcPort, NullIpcPort};
///
/// let ipc = NullIpcPort;
/// assert!(!ipc.is_shell_running());
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct NullIpcPort;

impl IpcPort for NullIpcPort {
    fn reload_theme(&self, _theme: &Theme) -> Result<()> {
        Ok(())
    }

    fn send_command(&self, _command: &str) -> Result<()> {
        Ok(())
    }

    fn is_shell_running(&self) -> bool {
        false
    }

    fn shell_version(&self) -> Option<String> {
        None
    }

    fn ping(&self) -> Result<u64> {
        Err(crate::error::FernError::ipc("shell not running"))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_notify_port_accepts_notifications() {
        let notifier = NullNotifyPort;
        let notification = Notification::info("Test", "Body");
        assert!(notifier.send(notification).is_ok());
    }

    #[test]
    fn null_notify_port_reports_unavailable() {
        let notifier = NullNotifyPort;
        assert!(!notifier.is_available());
    }

    #[test]
    fn null_ipc_port_reports_not_running() {
        let ipc = NullIpcPort;
        assert!(!ipc.is_shell_running());
        assert!(ipc.shell_version().is_none());
    }

    #[test]
    fn null_ipc_port_ping_fails() {
        let ipc = NullIpcPort;
        assert!(ipc.ping().is_err());
    }
}
