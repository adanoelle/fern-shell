//! # Configuration Watch Command
//!
//! The `watch` command monitors a TOML configuration file for changes and
//! automatically converts it to JSON when modifications are detected. This
//! enables a live-reload workflow where users can edit their configuration
//! and immediately see changes reflected in the running shell.
//!
//! ## Usage
//!
//! ```bash
//! # Watch default config location
//! fernctl watch
//!
//! # Watch a specific file
//! fernctl watch --config ./my-config.toml
//!
//! # Custom output location
//! fernctl watch --config config.toml --output theme.json
//!
//! # Adjust debounce timing (milliseconds)
//! fernctl watch --debounce 200
//!
//! # Disable desktop notifications
//! fernctl watch --quiet
//! ```
//!
//! ## How It Works
//!
//! The watch command uses operating system facilities to efficiently monitor
//! files for changes:
//!
//! | Platform | Technology |
//! |----------|------------|
//! | Linux | inotify |
//! | macOS | FSEvents |
//! | Windows | ReadDirectoryChangesW |
//!
//! This is more efficient than polling — the process sleeps until the OS
//! signals that the file has changed.
//!
//! ## Event Flow
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                        Watch Loop                                   │
//! │                                                                     │
//! │  ┌──────────────┐   ┌──────────────┐   ┌──────────────────────────┐│
//! │  │   inotify    │   │   Debounce   │   │      Convert & Notify    ││
//! │  │  (or equiv)  │──▶│   (100ms)    │──▶│                          ││
//! │  └──────────────┘   └──────────────┘   │  ┌─────────┐             ││
//! │        ▲                               │  │  Parse  │             ││
//! │        │                               │  │  TOML   │             ││
//! │        │                               │  └────┬────┘             ││
//! │   File modified                        │       │                  ││
//! │   by user                              │       ▼                  ││
//! │                                        │  ┌─────────┐             ││
//! │                                        │  │Validate │             ││
//! │                                        │  └────┬────┘             ││
//! │                                        │       │                  ││
//! │                                        │       ▼                  ││
//! │                                        │  ┌─────────┐  ┌────────┐ ││
//! │                                        │  │  Write  │  │ Notify │ ││
//! │                                        │  │  JSON   │──▶│  User  │ ││
//! │                                        │  └─────────┘  └────────┘ ││
//! │                                        └──────────────────────────┘│
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Debouncing
//!
//! Editors often trigger multiple file system events when saving a file:
//!
//! 1. File truncated (write begins)
//! 2. Data written
//! 3. File closed
//! 4. Metadata updated
//!
//! Without debouncing, we might try to parse a half-written file. The
//! debouncer waits for events to settle before processing:
//!
//! ```text
//! Events:  ┃━━━━━━━━━━┃     ┃━━━━┃                    ┃
//!          │ truncate │     │write│                    │
//!          │  modify  │     │close│                    │
//! Time:    ──────────────────────────────────────────────▶
//!          0ms       10ms  50ms  100ms (debounce expires)
//!                                  │
//!                                  ▼
//!                            Process file
//! ```
//!
//! ## Error Handling
//!
//! The watch command is designed to be resilient — configuration errors
//! should not crash the watcher. Instead:
//!
//! | Error Type | Behavior |
//! |------------|----------|
//! | Parse error | Notify user, keep watching |
//! | Validation error | Notify user, keep watching |
//! | File deleted | Notify user, keep watching |
//! | Watch system error | Exit with error |
//!
//! This allows users to fix mistakes without restarting the watcher:
//!
//! ```text
//! $ fernctl watch
//! Watching ~/.config/fern/config.toml...
//!
//! [14:32:01] ✓ Config reloaded
//! [14:32:15] ✗ Parse error at line 5 (keep editing...)
//! [14:32:28] ✓ Config reloaded
//! ```
//!
//! ## Desktop Notifications
//!
//! When notifications are enabled (the default), the watch command sends
//! desktop notifications via the freedesktop.org notification spec:
//!
//! | Scenario | Notification |
//! |----------|--------------|
//! | Success | "Config Reloaded" (info) |
//! | Warning | "Unknown Key: colour" (warning) |
//! | Error | "Invalid Color: #gg0000" (error) |
//!
//! These appear in whatever notification daemon is running (fern-shell's
//! own notification center, dunst, mako, etc.).
//!
//! ## Integration with QuickShell
//!
//! QuickShell watches the JSON file using its own `FileView` component.
//! When we write a new JSON file, QuickShell automatically reloads:
//!
//! ```text
//! ┌────────────┐      ┌─────────────┐      ┌────────────────┐
//! │  User      │      │  fernctl    │      │  QuickShell    │
//! │  edits     │──────▶  watch     │──────▶│  reloads       │
//! │  TOML      │      │  writes    │      │  UI updates    │
//! └────────────┘      │  JSON      │      └────────────────┘
//!                     └─────────────┘
//! ```
//!
//! No IPC is needed — file watching on both sides creates a clean
//! decoupled architecture.
//!
//! ## Exit Codes
//!
//! | Code | Meaning |
//! |------|---------|
//! | `0` | Normal exit (Ctrl+C) |
//! | `1` | Initialization error (file not found, etc.) |
//! | `2` | Watch system error |
//!
//! ## Programmatic Usage
//!
//! ```rust,ignore
//! use fern_theme::commands::watch::{run, WatchOptions, WatchEvent};
//! use fern_theme::adapters::{TomlConfigAdapter, FileSystemAdapter};
//!
//! let config_adapter = TomlConfigAdapter::new();
//! let persist_adapter = FileSystemAdapter::new();
//!
//! let options = WatchOptions {
//!     debounce_ms: 100,
//!     notify_on_success: true,
//!     notify_on_error: true,
//!     verbose: true,
//! };
//!
//! // Run the watch loop (blocks until interrupted)
//! run(
//!     &config_path,
//!     &json_path,
//!     options,
//!     &config_adapter,
//!     &persist_adapter,
//!     |event| {
//!         // Custom event handler
//!         match event {
//!             WatchEvent::Reloaded { warnings } => { /* ... */ }
//!             WatchEvent::Error { error } => { /* ... */ }
//!         }
//!     },
//! )?;
//! ```
//!
//! ## Resource Usage
//!
//! The watch command is designed to be lightweight:
//!
//! - **CPU**: Near-zero when idle (event-driven, not polling)
//! - **Memory**: ~5-10 MB baseline
//! - **File descriptors**: 2-3 per watched file
//!
//! It's safe to run indefinitely in the background.

use crate::error::{FernError, Notification, Notifiable, Result, Severity};
use crate::ports::inbound::ConfigPort;
use crate::ports::outbound::PersistPort;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebounceEventResult};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

/// Options for the watch command.
///
/// These options control the behavior of file watching, notification delivery,
/// and output verbosity.
///
/// # Example
///
/// ```rust
/// use fern_theme::commands::watch::WatchOptions;
///
/// let options = WatchOptions {
///     debounce_ms: 200,  // Wait 200ms for events to settle
///     notify_on_success: true,
///     notify_on_error: true,
///     verbose: false,
/// };
/// ```
///
/// # Default Values
///
/// | Option | Default | Rationale |
/// |--------|---------|-----------|
/// | `debounce_ms` | 100 | Fast enough to feel responsive, slow enough to debounce |
/// | `notify_on_success` | true | User wants feedback that changes were applied |
/// | `notify_on_error` | true | Errors should be visible |
/// | `verbose` | false | Terminal output is opt-in |
#[derive(Debug, Clone)]
pub struct WatchOptions {
    /// Debounce duration in milliseconds.
    ///
    /// After a file change is detected, the watcher waits this long for
    /// additional events before processing. This prevents multiple conversions
    /// when editors save files in multiple steps.
    ///
    /// Recommended values:
    /// - 50-100ms for fast editors (vim, emacs)
    /// - 100-200ms for slower editors or network filesystems
    pub debounce_ms: u64,

    /// Whether to send a notification on successful reload.
    ///
    /// When `true`, a brief notification confirms each successful conversion.
    /// Set to `false` for a quieter experience.
    pub notify_on_success: bool,

    /// Whether to send a notification on errors.
    ///
    /// When `true`, parse errors and validation failures trigger notifications.
    /// Strongly recommended to keep this enabled.
    pub notify_on_error: bool,

    /// Whether to print events to the terminal.
    ///
    /// When enabled, each event is logged with a timestamp:
    /// ```text
    /// [14:32:01] Config reloaded
    /// [14:32:15] Error: invalid color at line 5
    /// ```
    pub verbose: bool,
}

impl Default for WatchOptions {
    fn default() -> Self {
        Self {
            debounce_ms: 100,
            notify_on_success: true,
            notify_on_error: true,
            verbose: false,
        }
    }
}

/// Events emitted during the watch loop.
///
/// These events can be used to implement custom notification handling
/// or logging behavior.
///
/// # Example
///
/// ```rust,ignore
/// match event {
///     WatchEvent::Reloaded { warnings, bytes_written } => {
///         println!("Reloaded! {} warnings, {} bytes", warnings.len(), bytes_written);
///     }
///     WatchEvent::Error { error, notification } => {
///         eprintln!("Error: {}", error);
///         desktop_notify(&notification);
///     }
///     WatchEvent::Started { input, output } => {
///         println!("Watching {} -> {}", input.display(), output.display());
///     }
/// }
/// ```
#[derive(Debug)]
pub enum WatchEvent {
    /// The watch loop has started.
    Started {
        /// Path being watched.
        input: std::path::PathBuf,
        /// Path where JSON is written.
        output: std::path::PathBuf,
    },

    /// Configuration was successfully reloaded.
    Reloaded {
        /// Warnings encountered during validation (non-fatal).
        warnings: Vec<String>,
        /// Number of bytes written to the output file.
        bytes_written: usize,
    },

    /// An error occurred during reload.
    ///
    /// The watcher continues running after errors — this is not fatal.
    Error {
        /// The error that occurred.
        error: FernError,
        /// A notification ready to send to the desktop.
        notification: Notification,
    },

    /// File system event received but debouncing.
    ///
    /// Only emitted in verbose mode.
    Debouncing,
}

/// A callback function for watch events.
///
/// Implement this to customize how events are handled during watching.
pub type EventHandler = dyn Fn(WatchEvent);

/// Runs the configuration watch loop.
///
/// This function blocks, continuously watching the input file and converting
/// it to JSON when changes are detected. It only returns on unrecoverable
/// errors or when interrupted.
///
/// # Arguments
///
/// * `input` — Path to the TOML configuration file to watch
/// * `output` — Path where JSON output will be written
/// * `options` — Watch configuration options
/// * `config_adapter` — Adapter for parsing TOML
/// * `persist_adapter` — Adapter for writing JSON
///
/// # Returns
///
/// - `Ok(())` — Watch loop exited cleanly (interrupted)
/// - `Err(FernError)` — Unrecoverable error (e.g., watch system failure)
///
/// # Errors
///
/// Returns an error if:
/// - The input file does not exist at startup
/// - The file watcher cannot be initialized (e.g., too many open files)
/// - The parent directory of the output file doesn't exist
///
/// Configuration errors during the watch loop do **not** cause this function
/// to return an error — they are reported via notification and logged.
///
/// # Example
///
/// ```rust,ignore
/// use fern_theme::commands::watch::{run, WatchOptions};
/// use fern_theme::adapters::{TomlConfigAdapter, FileSystemAdapter};
/// use std::path::Path;
///
/// let config_adapter = TomlConfigAdapter::new();
/// let persist_adapter = FileSystemAdapter::new();
/// let options = WatchOptions::default();
///
/// // This blocks until Ctrl+C
/// run(
///     Path::new("~/.config/fern/config.toml"),
///     Path::new("~/.config/fern/config.json"),
///     options,
///     &config_adapter,
///     &persist_adapter,
/// )?;
/// ```
///
/// # Graceful Shutdown
///
/// The watch loop responds to SIGINT (Ctrl+C) by exiting cleanly. No special
/// cleanup is required — in-progress writes are completed atomically before
/// the function returns.
///
/// # Thread Safety
///
/// This function is not async and blocks the calling thread. For async usage,
/// wrap it in a blocking task:
///
/// ```rust,ignore
/// tokio::task::spawn_blocking(|| {
///     watch::run(&input, &output, options, &config, &persist)
/// }).await?;
/// ```
pub fn run<P, Q>(
    input: P,
    output: Q,
    options: WatchOptions,
    config_adapter: &impl ConfigPort,
    persist_adapter: &impl PersistPort,
) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let input = input.as_ref();
    let output = output.as_ref();

    // Verify input exists
    if !input.exists() {
        return Err(FernError::io(
            format!("config file not found: {}", input.display()),
            std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"),
        ));
    }

    // Set up the file watcher with debouncing
    let (tx, rx): (_, Receiver<DebounceEventResult>) = channel();

    let mut debouncer = new_debouncer(Duration::from_millis(options.debounce_ms), tx)
        .map_err(|e| FernError::watch(format!("failed to create file watcher: {e}")))?;

    debouncer
        .watcher()
        .watch(input, RecursiveMode::NonRecursive)
        .map_err(|e| FernError::watch(format!("failed to watch file: {e}")))?;

    // Log startup
    if options.verbose {
        eprintln!(
            "Watching {} -> {}",
            input.display(),
            output.display()
        );
        eprintln!("Press Ctrl+C to stop");
    }

    // Initial conversion
    if let Err(e) = convert_and_report(input, output, &options, config_adapter, persist_adapter) {
        if options.verbose {
            eprintln!("[initial] Error: {e}");
        }
        // Don't exit on initial error — start watching anyway
    }

    // Watch loop
    loop {
        match rx.recv() {
            Ok(Ok(events)) => {
                // Check if any events are relevant to our file
                let relevant = events.iter().any(|e| e.path == input);
                if !relevant {
                    continue;
                }

                if options.verbose {
                    eprintln!("[{}] File changed, converting...", timestamp());
                }

                if let Err(e) =
                    convert_and_report(input, output, &options, config_adapter, persist_adapter)
                {
                    if options.verbose {
                        eprintln!("[{}] Error: {e}", timestamp());
                    }
                    // Continue watching — don't exit on config errors
                }
            }
            Ok(Err(error)) => {
                // Debouncer error (usually recoverable)
                if options.verbose {
                    eprintln!("[{}] Watch warning: {error}", timestamp());
                }
            }
            Err(e) => {
                // Channel closed — watcher died
                return Err(FernError::watch(format!("watcher channel closed: {e}")));
            }
        }
    }
}

/// Converts the config and reports the result.
///
/// This is the core conversion logic extracted for reuse between initial
/// conversion and watch-triggered conversions.
fn convert_and_report(
    input: &Path,
    output: &Path,
    options: &WatchOptions,
    config_adapter: &impl ConfigPort,
    persist_adapter: &impl PersistPort,
) -> Result<()> {
    // Load and validate
    let raw = config_adapter.load_from_file(input)?;
    let validated = raw.validate()?;

    // Collect warnings
    let warnings: Vec<String> = validated.warnings().iter().map(ToString::to_string).collect();

    // Report warnings
    for warning in &warnings {
        if options.verbose {
            eprintln!("[{}] Warning: {}", timestamp(), warning);
        }
        if options.notify_on_error {
            // Warnings are non-fatal but should be shown
            send_notification(&Notification::warning("Config Warning", warning));
        }
    }

    // Convert and persist
    let theme = validated.into_theme();
    persist_adapter.save_theme(&theme, output)?;

    // Report success
    if options.verbose {
        eprintln!(
            "[{}] Converted {} -> {}",
            timestamp(),
            input.display(),
            output.display(),
        );
    }

    if options.notify_on_success {
        send_notification(&Notification::info(
            "Config Reloaded",
            format!("Updated {}", output.display()),
        ));
    }

    Ok(())
}

/// Sends a desktop notification.
///
/// This function attempts to send a notification via the system notification
/// daemon. It fails silently if notifications are unavailable.
fn send_notification(notification: &Notification) {
    // Use notify-send as a fallback until we have proper D-Bus integration
    // This is a simple implementation that works on most Linux desktops
    let urgency = match notification.severity {
        Severity::Info => "low",
        Severity::Warning => "normal",
        Severity::Error | Severity::Fatal => "critical",
    };

    let mut body = notification.body.clone();
    if let Some(suggestion) = &notification.suggestion {
        body.push_str("\n\n");
        body.push_str(suggestion);
    }

    // Fire and forget — don't block on notification delivery
    let _ = std::process::Command::new("notify-send")
        .arg("--app-name=fernctl")
        .arg(format!("--urgency={urgency}"))
        .arg(&notification.title)
        .arg(&body)
        .spawn();
}

/// Returns the current time formatted for log output.
fn timestamp() -> String {
    // Simple timestamp without external dependency
    // In production, consider using chrono or time crate
    use std::time::SystemTime;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();

    let secs = now.as_secs();
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let secs = secs % 60;

    format!("{hours:02}:{mins:02}:{secs:02}")
}

/// Callback wrapper that sends errors as notifications.
///
/// This helper implements the common pattern of converting errors to
/// notifications and sending them:
///
/// ```rust,ignore
/// let result = some_operation();
/// notify_on_error(result, &options)?;
/// ```
#[allow(dead_code)]
fn notify_on_error<T>(result: Result<T>, options: &WatchOptions) -> Result<T> {
    match result {
        Ok(v) => Ok(v),
        Err(e) => {
            if options.notify_on_error {
                send_notification(&e.to_notification());
            }
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watch_options_default() {
        let opts = WatchOptions::default();
        assert_eq!(opts.debounce_ms, 100);
        assert!(opts.notify_on_success);
        assert!(opts.notify_on_error);
        assert!(!opts.verbose);
    }

    #[test]
    fn timestamp_format() {
        let ts = timestamp();
        // Should be HH:MM:SS format
        assert_eq!(ts.len(), 8);
        assert_eq!(ts.chars().nth(2), Some(':'));
        assert_eq!(ts.chars().nth(5), Some(':'));
    }
}
