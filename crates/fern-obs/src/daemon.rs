//! OBS daemon implementation.
//!
//! The daemon maintains a connection to OBS, handles events, and writes
//! state updates to the state file for the QML interface to consume.

use crate::client::ObsClient;
use crate::config::ObsConfig;
use crate::error::{Error, Result};
use crate::state::{ObsState, StateTracker};
use fern_core::FernPaths;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::{interval, sleep};
use tracing::{error, info, warn};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Number of fast reconnection attempts before switching to slow mode.
const FAST_RETRY_THRESHOLD: u32 = 10;

/// Slow retry interval in milliseconds (60 seconds).
const SLOW_RETRY_INTERVAL_MS: u64 = 60_000;

/// The OBS daemon.
///
/// Manages the connection to OBS and writes state updates.
pub struct Daemon {
    config: ObsConfig,
    state_path: PathBuf,
    tracker: StateTracker,
}

impl Daemon {
    /// Creates a new daemon with the given configuration.
    #[must_use]
    pub fn new(config: ObsConfig) -> Self {
        let paths = FernPaths::new();
        let state_path = paths.service_state("obs");

        Self {
            config,
            state_path,
            tracker: StateTracker::new(),
        }
    }

    /// Runs the daemon.
    ///
    /// This function runs indefinitely, maintaining a connection to OBS
    /// and writing state updates.
    ///
    /// # Errors
    ///
    /// Returns an error if the daemon encounters an unrecoverable error.
    pub async fn run(&mut self) -> Result<()> {
        // Ensure state directory exists
        if let Some(parent) = self.state_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::io("creating state directory", e))?;
        }

        // Write initial disconnected state
        self.write_state()?;

        info!(
            host = %self.config.host,
            port = self.config.port,
            state_file = %self.state_path.display(),
            "Starting OBS daemon"
        );

        let mut reconnect_attempts = 0u32;
        let mut in_slow_mode = false;

        loop {
            match self.run_connected().await {
                Ok(()) => {
                    // Clean shutdown requested
                    info!("Shutting down");
                    self.tracker.set_disconnected(None);
                    self.write_state()?;
                    break;
                }
                Err(e) => {
                    // Check if this was a connection that was established but then lost
                    // (vs never connected at all). If we were connected, reset counter.
                    let was_connected = self.tracker.state.connected;

                    reconnect_attempts += 1;
                    let max = self.config.max_reconnect_attempts;

                    if max > 0 && reconnect_attempts > max {
                        error!(max_attempts = max, "Max reconnection attempts exceeded");
                        return Err(e);
                    }

                    warn!(error = %e, "Connection error");
                    self.tracker.set_disconnected(Some(e.to_string()));
                    self.write_state()?;

                    // If we were connected before, reset to fast mode
                    if was_connected && in_slow_mode {
                        in_slow_mode = false;
                        reconnect_attempts = 1;
                        info!("Connection lost, resuming fast retry");
                    }

                    // Circuit breaker: switch to slow mode after threshold
                    let delay = if reconnect_attempts > FAST_RETRY_THRESHOLD {
                        if !in_slow_mode {
                            in_slow_mode = true;
                            warn!(
                                interval_secs = SLOW_RETRY_INTERVAL_MS / 1000,
                                "OBS appears to be unavailable, switching to slow retry"
                            );
                        }
                        Duration::from_millis(SLOW_RETRY_INTERVAL_MS)
                    } else {
                        Duration::from_millis(self.config.reconnect_interval_ms)
                    };

                    info!(
                        delay_secs = delay.as_secs(),
                        attempt = reconnect_attempts,
                        "Reconnecting"
                    );
                    sleep(delay).await;
                }
            }
        }

        Ok(())
    }

    /// Runs while connected to OBS.
    ///
    /// Returns when the connection is lost or shutdown is requested.
    async fn run_connected(&mut self) -> Result<()> {
        // Connect to OBS
        let client = ObsClient::connect(self.config.clone()).await?;

        info!("Connected to OBS");

        // Initial state sync
        client.sync_state(&mut self.tracker).await?;
        self.write_state()?;

        // Set up update interval
        let update_interval = Duration::from_millis(self.config.stats_interval_ms);
        let mut ticker = interval(update_interval);

        // Main event loop
        loop {
            tokio::select! {
                // Periodic state update
                _ = ticker.tick() => {
                    // Update elapsed times
                    self.tracker.update_elapsed();

                    // Sync state from OBS (this also updates stats)
                    if let Err(e) = client.sync_state(&mut self.tracker).await {
                        // Connection lost
                        return Err(e);
                    }

                    self.write_state()?;
                }

                // Handle shutdown signal
                _ = tokio::signal::ctrl_c() => {
                    info!("Received shutdown signal");
                    return Ok(());
                }
            }
        }
    }

    /// Writes the current state to the state file.
    ///
    /// Uses atomic write (write to temp, then rename) to prevent partial reads.
    /// Sets restrictive permissions (0600) for security.
    fn write_state(&mut self) -> Result<()> {
        self.tracker.update_elapsed();

        // Use compact JSON for state files (no pretty printing overhead)
        let json = serde_json::to_string(&self.tracker.state)?;

        // Atomic write: write to temp file, then rename
        let temp_path = self.state_path.with_extension("json.tmp");

        std::fs::write(&temp_path, &json).map_err(|e| Error::io("writing temp state file", e))?;

        // Set restrictive permissions (owner read/write only) before rename
        #[cfg(unix)]
        {
            let mut perms = std::fs::metadata(&temp_path)
                .map_err(|e| Error::io("reading temp file metadata", e))?
                .permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&temp_path, perms)
                .map_err(|e| Error::io("setting file permissions", e))?;
        }

        // Atomic rename (on same filesystem)
        std::fs::rename(&temp_path, &self.state_path)
            .map_err(|e| Error::io("renaming state file", e))?;

        Ok(())
    }
}

/// Sends a command to OBS via a one-shot connection.
///
/// This is used by CLI commands that don't need to maintain a connection.
pub async fn send_command(config: &ObsConfig, command: Command) -> Result<CommandResult> {
    let client = ObsClient::connect(config.clone()).await?;

    match command {
        Command::StartRecording => {
            client.start_recording().await?;
            Ok(CommandResult::Success("Recording started".into()))
        }
        Command::StopRecording => {
            let path = client.stop_recording().await?;
            Ok(CommandResult::Success(format!("Recording saved to: {path}")))
        }
        Command::TogglePause => {
            let paused = client.toggle_recording_pause().await?;
            let msg = if paused {
                "Recording paused"
            } else {
                "Recording resumed"
            };
            Ok(CommandResult::Success(msg.into()))
        }
        Command::StartStreaming => {
            client.start_streaming().await?;
            Ok(CommandResult::Success("Streaming started".into()))
        }
        Command::StopStreaming => {
            client.stop_streaming().await?;
            Ok(CommandResult::Success("Streaming stopped".into()))
        }
        Command::SetScene(name) => {
            client.set_scene(&name).await?;
            Ok(CommandResult::Success(format!("Scene set to: {name}")))
        }
        Command::GetStatus => {
            let mut tracker = StateTracker::new();
            client.sync_state(&mut tracker).await?;
            Ok(CommandResult::State(tracker.state))
        }
    }
}

/// Commands that can be sent to OBS.
#[derive(Debug, Clone)]
pub enum Command {
    /// Start recording.
    StartRecording,
    /// Stop recording.
    StopRecording,
    /// Toggle recording pause.
    TogglePause,
    /// Start streaming.
    StartStreaming,
    /// Stop streaming.
    StopStreaming,
    /// Set the current scene.
    SetScene(String),
    /// Get the current status.
    GetStatus,
}

/// Result of a command execution.
#[derive(Debug)]
pub enum CommandResult {
    /// Command succeeded with a message.
    Success(String),
    /// Command returned state information.
    State(ObsState),
}
