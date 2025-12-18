//! OBS WebSocket client wrapper.
//!
//! This module provides a high-level interface to OBS Studio via the
//! obs-websocket protocol using the `obws` crate.

use crate::config::ObsConfig;
use crate::error::{Error, Result};
use crate::state::{ObsStats, StateTracker};
use obws::Client;

/// High-level OBS client wrapper.
///
/// Wraps the `obws::Client` and provides convenient methods for
/// common operations.
pub struct ObsClient {
    client: Client,
    config: ObsConfig,
}

impl ObsClient {
    /// Connects to OBS with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection fails or authentication fails.
    pub async fn connect(config: ObsConfig) -> Result<Self> {
        let client = Client::connect(&config.host, config.port, config.password.as_deref())
            .await
            .map_err(|e| Error::connection(&config.host, config.port, e.to_string()))?;

        Ok(Self { client, config })
    }

    /// Returns the configuration.
    #[must_use]
    pub fn config(&self) -> &ObsConfig {
        &self.config
    }

    // ========================================================================
    // Recording
    // ========================================================================

    /// Starts recording.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn start_recording(&self) -> Result<()> {
        self.client
            .recording()
            .start()
            .await
            .map_err(|e| Error::Request(e.to_string()))
    }

    /// Stops recording and returns the output file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn stop_recording(&self) -> Result<String> {
        self.client
            .recording()
            .stop()
            .await
            .map_err(|e| Error::Request(e.to_string()))
    }

    /// Toggles recording pause state.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn toggle_recording_pause(&self) -> Result<bool> {
        self.client
            .recording()
            .toggle_pause()
            .await
            .map_err(|e| Error::Request(e.to_string()))
    }

    /// Gets the current recording status.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn recording_status(&self) -> Result<RecordingStatus> {
        let status = self
            .client
            .recording()
            .status()
            .await
            .map_err(|e| Error::Request(e.to_string()))?;

        Ok(RecordingStatus {
            active: status.active,
            paused: status.paused,
            timecode: status.timecode.to_string(),
            bytes: status.bytes,
        })
    }

    // ========================================================================
    // Streaming
    // ========================================================================

    /// Starts streaming.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn start_streaming(&self) -> Result<()> {
        self.client
            .streaming()
            .start()
            .await
            .map_err(|e| Error::Request(e.to_string()))
    }

    /// Stops streaming.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn stop_streaming(&self) -> Result<()> {
        self.client
            .streaming()
            .stop()
            .await
            .map_err(|e| Error::Request(e.to_string()))
    }

    /// Gets the current streaming status.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn streaming_status(&self) -> Result<StreamingStatus> {
        let status = self
            .client
            .streaming()
            .status()
            .await
            .map_err(|e| Error::Request(e.to_string()))?;

        Ok(StreamingStatus {
            active: status.active,
            reconnecting: status.reconnecting,
            timecode: status.timecode.to_string(),
            bytes: status.bytes,
        })
    }

    // ========================================================================
    // Scenes
    // ========================================================================

    /// Gets the list of scenes.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get_scenes(&self) -> Result<Vec<String>> {
        let scenes = self
            .client
            .scenes()
            .list()
            .await
            .map_err(|e| Error::Request(e.to_string()))?;

        Ok(scenes.scenes.into_iter().map(|s| s.id.name).collect())
    }

    /// Gets the current scene name.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn current_scene(&self) -> Result<String> {
        let scene = self
            .client
            .scenes()
            .current_program_scene()
            .await
            .map_err(|e| Error::Request(e.to_string()))?;

        Ok(scene.id.name)
    }

    /// Sets the current scene.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn set_scene(&self, name: &str) -> Result<()> {
        self.client
            .scenes()
            .set_current_program_scene(name)
            .await
            .map_err(|e| Error::Request(e.to_string()))
    }

    // ========================================================================
    // Stats
    // ========================================================================

    /// Gets performance statistics.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get_stats(&self) -> Result<ObsStats> {
        let stats = self
            .client
            .general()
            .stats()
            .await
            .map_err(|e| Error::Request(e.to_string()))?;

        let mut obs_stats = ObsStats {
            cpu_usage: stats.cpu_usage,
            memory_mb: stats.memory_usage,
            available_disk_mb: Some(stats.available_disk_space),
            active_fps: stats.active_fps,
            average_frame_time_ms: stats.average_frame_render_time,
            render_missed_frames: stats.render_skipped_frames as u64,
            render_total_frames: stats.render_total_frames as u64,
            output_skipped_frames: stats.output_skipped_frames as u64,
            output_total_frames: stats.output_total_frames as u64,
            render_drop_percent: None,
            output_drop_percent: None,
        };

        obs_stats.calculate_percentages();
        Ok(obs_stats)
    }

    // ========================================================================
    // State Synchronization
    // ========================================================================

    /// Fetches the complete current state from OBS and updates the tracker.
    ///
    /// # Errors
    ///
    /// Returns an error if any request fails.
    pub async fn sync_state(&self, tracker: &mut StateTracker) -> Result<()> {
        tracker.set_connected();

        // Get recording status
        if let Ok(rec_status) = self.recording_status().await {
            if rec_status.active {
                if rec_status.paused {
                    tracker.state.recording.paused = true;
                }
                // Note: We can't easily get elapsed time from OBS timecode,
                // so we rely on the tracker's internal timer
                if !tracker.state.recording.active {
                    tracker.start_recording();
                }
            } else if tracker.state.recording.active {
                tracker.stop_recording();
            }
        }

        // Get streaming status
        if let Ok(stream_status) = self.streaming_status().await {
            if stream_status.active {
                if !tracker.state.streaming.active {
                    tracker.start_streaming();
                }
                tracker.state.streaming.reconnecting = stream_status.reconnecting;
            } else if tracker.state.streaming.active {
                tracker.stop_streaming();
            }
        }

        // Get current scene
        if let Ok(scene) = self.current_scene().await {
            tracker.set_scene(scene);
        }

        // Get scene list
        if let Ok(scenes) = self.get_scenes().await {
            tracker.set_scenes(scenes);
        }

        // Get stats if configured
        if self.config.show_stats {
            if let Ok(stats) = self.get_stats().await {
                tracker.set_stats(stats);
            }
        }

        Ok(())
    }
}

/// Recording status from OBS.
#[derive(Debug, Clone)]
pub struct RecordingStatus {
    /// Whether recording is active.
    pub active: bool,
    /// Whether recording is paused.
    pub paused: bool,
    /// Timecode string.
    pub timecode: String,
    /// Bytes recorded.
    pub bytes: u64,
}

/// Streaming status from OBS.
#[derive(Debug, Clone)]
pub struct StreamingStatus {
    /// Whether streaming is active.
    pub active: bool,
    /// Whether reconnecting.
    pub reconnecting: bool,
    /// Timecode string.
    pub timecode: String,
    /// Bytes sent.
    pub bytes: u64,
}
