//! # State Watcher
//!
//! Watches state files for changes and emits actions.

use crate::domain::action::ConfigSummary;
use crate::domain::{Action, LogEntry};
use crate::error::{FernctlError, Result};
use fern_core::FernPaths;
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind, Debouncer};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

/// State change event from file watcher.
#[derive(Debug, Clone)]
pub enum StateChange {
    /// OBS state file changed.
    ObsState(serde_json::Value),
    /// Shell log file changed.
    ShellLogs(Vec<LogEntry>),
    /// Config file changed.
    ConfigChanged(ConfigSummary),
    /// A watched file was deleted.
    FileDeleted(PathBuf),
}

/// Watches state files for changes.
pub struct StateWatcher {
    /// Debounced file watcher.
    _debouncer: Debouncer<RecommendedWatcher>,
    /// Receiver for file events.
    rx: Receiver<StateChange>,
    /// Paths configuration.
    paths: FernPaths,
}

impl StateWatcher {
    /// Creates a new state watcher.
    ///
    /// # Arguments
    ///
    /// * `paths` - Fern paths configuration
    /// * `debounce_ms` - Debounce duration in milliseconds
    ///
    /// # Errors
    ///
    /// Returns an error if the watcher cannot be created.
    pub fn new(paths: FernPaths, debounce_ms: u64) -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let paths_clone = paths.clone();

        // Create debounced watcher
        let debouncer = new_debouncer(
            Duration::from_millis(debounce_ms),
            move |res: std::result::Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>| {
                match res {
                    Ok(events) => {
                        for event in events {
                            if event.kind == DebouncedEventKind::Any {
                                Self::handle_file_change(&tx, &paths_clone, &event.path);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Watch error: {}", e);
                    }
                }
            },
        )
        .map_err(|e| FernctlError::watch_notify("creating debouncer", e))?;

        Ok(Self {
            _debouncer: debouncer,
            rx,
            paths,
        })
    }

    /// Starts watching the state and config directories.
    ///
    /// # Errors
    ///
    /// Returns an error if directories cannot be watched.
    pub fn start(&mut self) -> Result<()> {
        // Watch state directory
        self._debouncer
            .watcher()
            .watch(self.paths.state_dir(), RecursiveMode::NonRecursive)
            .map_err(|e| FernctlError::watch_notify("watching state directory", e))?;

        // Watch config directory
        self._debouncer
            .watcher()
            .watch(self.paths.config_dir(), RecursiveMode::NonRecursive)
            .map_err(|e| FernctlError::watch_notify("watching config directory", e))?;

        tracing::info!(
            "Watching {} and {}",
            self.paths.state_dir().display(),
            self.paths.config_dir().display()
        );

        Ok(())
    }

    /// Tries to receive a state change without blocking.
    ///
    /// Returns `None` if no change is available.
    pub fn try_recv(&self) -> Option<StateChange> {
        self.rx.try_recv().ok()
    }

    /// Receives a state change, blocking until one is available or timeout.
    ///
    /// Returns `None` on timeout.
    pub fn recv_timeout(&self, timeout: Duration) -> Option<StateChange> {
        self.rx.recv_timeout(timeout).ok()
    }

    /// Handles a file change event.
    fn handle_file_change(tx: &Sender<StateChange>, _paths: &FernPaths, path: &PathBuf) {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        match file_name {
            "obs-state.json" => {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        let _ = tx.send(StateChange::ObsState(json));
                    }
                }
            }
            "shell-log.json" => {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Ok(logs) = parse_shell_logs(&content) {
                        let _ = tx.send(StateChange::ShellLogs(logs));
                    }
                }
            }
            "config.json" => {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        let summary = ConfigSummary::from_json(&json);
                        let _ = tx.send(StateChange::ConfigChanged(summary));
                    }
                }
            }
            _ => {}
        }
    }

    /// Converts a state change to an action.
    pub fn to_action(change: StateChange) -> Option<Action> {
        match change {
            StateChange::ObsState(json) => {
                // Parse OBS state into ServiceInfo
                let connected = json.get("connected").and_then(|v| v.as_bool()).unwrap_or(false);
                let status = if connected {
                    fern_core::ServiceStatus::Running
                } else {
                    fern_core::ServiceStatus::Stopped
                };
                let info = fern_core::ServiceInfo::new("obs", status);
                Some(Action::ServiceStateChanged {
                    name: "obs".to_string(),
                    info,
                })
            }
            StateChange::ShellLogs(logs) => {
                // Sync all logs from the file
                Some(Action::LogsSync(logs))
            }
            StateChange::ConfigChanged(summary) => Some(Action::ConfigChanged(summary)),
            StateChange::FileDeleted(_) => None,
        }
    }
}

/// Parses shell logs from JSON content.
fn parse_shell_logs(content: &str) -> std::result::Result<Vec<LogEntry>, serde_json::Error> {
    // Try parsing as array first
    if let Ok(entries) = serde_json::from_str::<Vec<LogEntry>>(content) {
        return Ok(entries);
    }

    // Try parsing as object with entries field
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(entries) = json.get("entries").and_then(|e| e.as_array()) {
            let parsed: Vec<LogEntry> = entries
                .iter()
                .filter_map(|e| serde_json::from_value(e.clone()).ok())
                .collect();
            return Ok(parsed);
        }
    }

    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_summary_from_json() {
        let json = serde_json::json!({
            "appearance": {
                "theme": "catppuccin",
                "variant": "mocha",
                "accent": "#89b4fa"
            },
            "bar": {
                "position": "top",
                "height": 40
            }
        });

        let summary = ConfigSummary::from_json(&json);
        assert_eq!(summary.theme, Some("catppuccin".to_string()));
        assert_eq!(summary.variant, Some("mocha".to_string()));
        assert_eq!(summary.bar_position, Some("top".to_string()));
        assert_eq!(summary.bar_height, Some(40));
        assert_eq!(summary.accent_color, Some("#89b4fa".to_string()));
    }
}
