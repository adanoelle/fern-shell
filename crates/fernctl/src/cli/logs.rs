//! # Logs Command
//!
//! View and follow aggregated logs from Fern Shell services.

use crate::domain::{LogBuffer, LogEntry};
use crate::error::{FernctlError, Result};
use fern_core::FernPaths;
use std::fs;

/// Options for the logs command.
#[derive(Debug, Clone)]
pub struct LogsOptions {
    /// Follow mode (like `tail -f`).
    pub follow: bool,
    /// Filter by service name.
    pub service: Option<String>,
    /// Number of lines to show.
    pub lines: usize,
    /// Filter by log level.
    pub level: Option<String>,
}

impl Default for LogsOptions {
    fn default() -> Self {
        Self {
            follow: false,
            service: None,
            lines: 50,
            level: None,
        }
    }
}

/// Runs the logs command.
///
/// # Errors
///
/// Returns an error if log files cannot be read.
pub fn run(options: LogsOptions) -> Result<()> {
    let paths = FernPaths::new();
    let mut buffer = LogBuffer::with_default_capacity();

    // Load shell logs
    let shell_log_path = paths.service_state("shell-log");
    if shell_log_path.exists() {
        load_log_file(&shell_log_path, &mut buffer)?;
    }

    // Apply service filter
    if let Some(ref service) = options.service {
        buffer.set_filter(service);
    }

    // Display logs
    let entries: Vec<_> = buffer.filtered_entries().collect();
    let start = entries.len().saturating_sub(options.lines);

    for entry in entries.iter().skip(start) {
        println!("{}", entry.format_colored());
    }

    if options.follow {
        println!("\n\x1b[90m--- Following logs (Ctrl+C to exit) ---\x1b[0m\n");
        follow_logs(&paths, &options)?;
    }

    Ok(())
}

fn load_log_file(path: &std::path::Path, buffer: &mut LogBuffer) -> Result<()> {
    let content = fs::read_to_string(path).map_err(|e| FernctlError::state_io("reading logs", e))?;

    // Try parsing as JSON array of log entries
    if let Ok(entries) = serde_json::from_str::<Vec<LogEntry>>(&content) {
        for entry in entries {
            buffer.push(entry);
        }
    } else if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
        // Try parsing as object with entries array
        if let Some(entries) = json.get("entries").and_then(|e| e.as_array()) {
            for entry_json in entries {
                if let Ok(entry) = serde_json::from_value::<LogEntry>(entry_json.clone()) {
                    buffer.push(entry);
                }
            }
        }
    }

    Ok(())
}

fn follow_logs(paths: &FernPaths, options: &LogsOptions) -> Result<()> {
    use notify::{RecursiveMode, Watcher};
    use std::sync::mpsc;
    use std::time::Duration;

    let (tx, rx) = mpsc::channel();

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    })
    .map_err(|e| FernctlError::watch_notify("creating watcher", e))?;

    let state_dir = paths.state_dir();
    watcher
        .watch(state_dir, RecursiveMode::NonRecursive)
        .map_err(|e| FernctlError::watch_notify("watching state directory", e))?;

    let shell_log_path = paths.service_state("shell-log");
    let mut last_len = shell_log_path
        .metadata()
        .map(|m| m.len())
        .unwrap_or(0);

    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(event) => {
                if event
                    .paths
                    .iter()
                    .any(|p| p.file_name().map_or(false, |n| n == "shell-log.json"))
                {
                    // Check if file grew
                    if let Ok(metadata) = shell_log_path.metadata() {
                        if metadata.len() > last_len {
                            // Reload and show new entries
                            let mut buffer = LogBuffer::with_default_capacity();
                            if load_log_file(&shell_log_path, &mut buffer).is_ok() {
                                if let Some(ref service) = options.service {
                                    buffer.set_filter(service);
                                }
                                // Show only new entries (rough approximation)
                                let entries: Vec<_> = buffer.filtered_entries().collect();
                                if let Some(entry) = entries.last() {
                                    println!("{}", entry.format_colored());
                                }
                            }
                            last_len = metadata.len();
                        }
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    Ok(())
}
