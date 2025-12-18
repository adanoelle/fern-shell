//! OBS state types for serialization and QML consumption.
//!
//! These types represent the current state of OBS Studio and are serialized
//! to JSON for the QML interface to consume via `FileView`.

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Complete OBS state written to the state file.
///
/// This is the top-level structure that gets serialized to
/// `~/.local/state/fern/obs-state.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObsState {
    /// Whether we're connected to OBS.
    pub connected: bool,

    /// Current recording state.
    pub recording: RecordingState,

    /// Current streaming state.
    pub streaming: StreamingState,

    /// Name of the currently active scene.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_scene: Option<String>,

    /// List of available scene names.
    #[serde(default)]
    pub scenes: Vec<String>,

    /// Performance statistics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<ObsStats>,

    /// Error message if connection failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Unix timestamp when this state was last updated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_secs: Option<u64>,
}

impl ObsState {
    /// Creates a new disconnected state.
    #[must_use]
    pub fn disconnected() -> Self {
        Self {
            connected: false,
            ..Default::default()
        }
    }

    /// Creates a disconnected state with an error message.
    #[must_use]
    pub fn with_error(error: impl Into<String>) -> Self {
        Self {
            connected: false,
            error: Some(error.into()),
            ..Default::default()
        }
    }

    /// Updates the timestamp to now.
    pub fn touch(&mut self) {
        self.updated_at_secs = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        );
    }
}

/// Recording state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordingState {
    /// Whether recording is currently active.
    pub active: bool,

    /// Whether recording is paused (only meaningful if active).
    pub paused: bool,

    /// Elapsed recording time in seconds.
    pub elapsed_secs: u64,

    /// Output file path (if known).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_path: Option<String>,

    /// Recording timecode string (e.g., "01:23:45").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timecode: Option<String>,
}

impl RecordingState {
    /// Creates a new idle recording state.
    #[must_use]
    pub fn idle() -> Self {
        Self::default()
    }

    /// Creates a new active recording state.
    #[must_use]
    pub fn active(elapsed_secs: u64) -> Self {
        Self {
            active: true,
            paused: false,
            elapsed_secs,
            output_path: None,
            timecode: Some(Self::format_timecode(elapsed_secs)),
        }
    }

    /// Creates a paused recording state.
    #[must_use]
    pub fn paused(elapsed_secs: u64) -> Self {
        Self {
            active: true,
            paused: true,
            elapsed_secs,
            output_path: None,
            timecode: Some(Self::format_timecode(elapsed_secs)),
        }
    }

    /// Formats seconds as HH:MM:SS timecode.
    #[must_use]
    pub fn format_timecode(secs: u64) -> String {
        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;

        if hours > 0 {
            format!("{hours:02}:{minutes:02}:{seconds:02}")
        } else {
            format!("{minutes:02}:{seconds:02}")
        }
    }
}

/// Streaming state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StreamingState {
    /// Whether streaming is currently active.
    pub active: bool,

    /// Elapsed streaming time in seconds.
    pub elapsed_secs: u64,

    /// Streaming timecode string (e.g., "01:23:45").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timecode: Option<String>,

    /// Current bitrate in kbps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bitrate_kbps: Option<u32>,

    /// Reconnecting status.
    pub reconnecting: bool,
}

impl StreamingState {
    /// Creates a new idle streaming state.
    #[must_use]
    pub fn idle() -> Self {
        Self::default()
    }

    /// Creates a new active streaming state.
    #[must_use]
    pub fn active(elapsed_secs: u64) -> Self {
        Self {
            active: true,
            elapsed_secs,
            timecode: Some(RecordingState::format_timecode(elapsed_secs)),
            bitrate_kbps: None,
            reconnecting: false,
        }
    }
}

/// OBS performance statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObsStats {
    /// CPU usage percentage (0-100).
    pub cpu_usage: f64,

    /// Memory usage in megabytes.
    pub memory_mb: f64,

    /// Available disk space in megabytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_disk_mb: Option<f64>,

    /// Active FPS (frames per second).
    pub active_fps: f64,

    /// Average frame render time in milliseconds.
    pub average_frame_time_ms: f64,

    /// Number of frames missed due to rendering lag.
    pub render_missed_frames: u64,

    /// Total number of rendered frames.
    pub render_total_frames: u64,

    /// Number of frames skipped due to encoding lag.
    pub output_skipped_frames: u64,

    /// Total number of output frames.
    pub output_total_frames: u64,

    /// Calculated render drop percentage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub render_drop_percent: Option<f64>,

    /// Calculated output drop percentage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_drop_percent: Option<f64>,
}

impl ObsStats {
    /// Calculates drop percentages from frame counts.
    pub fn calculate_percentages(&mut self) {
        if self.render_total_frames > 0 {
            self.render_drop_percent = Some(
                (self.render_missed_frames as f64 / self.render_total_frames as f64) * 100.0,
            );
        }
        if self.output_total_frames > 0 {
            self.output_drop_percent = Some(
                (self.output_skipped_frames as f64 / self.output_total_frames as f64) * 100.0,
            );
        }
    }
}

/// Internal state tracker with timing information.
///
/// This is used by the daemon to track state that includes
/// non-serializable fields like `Instant`.
#[derive(Debug)]
pub struct StateTracker {
    /// The serializable state.
    pub state: ObsState,

    /// When recording started (for elapsed time calculation).
    recording_started: Option<Instant>,

    /// When streaming started (for elapsed time calculation).
    streaming_started: Option<Instant>,
}

impl StateTracker {
    /// Creates a new state tracker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: ObsState::disconnected(),
            recording_started: None,
            streaming_started: None,
        }
    }

    /// Marks as connected.
    pub fn set_connected(&mut self) {
        self.state.connected = true;
        self.state.error = None;
    }

    /// Marks as disconnected with optional error.
    pub fn set_disconnected(&mut self, error: Option<String>) {
        self.state.connected = false;
        self.state.error = error;
        self.state.recording = RecordingState::idle();
        self.state.streaming = StreamingState::idle();
        self.recording_started = None;
        self.streaming_started = None;
    }

    /// Starts recording timer.
    pub fn start_recording(&mut self) {
        self.recording_started = Some(Instant::now());
        self.state.recording.active = true;
        self.state.recording.paused = false;
    }

    /// Stops recording timer.
    pub fn stop_recording(&mut self) {
        self.recording_started = None;
        self.state.recording = RecordingState::idle();
    }

    /// Pauses recording.
    pub fn pause_recording(&mut self) {
        self.state.recording.paused = true;
    }

    /// Resumes recording.
    pub fn resume_recording(&mut self) {
        self.state.recording.paused = false;
    }

    /// Starts streaming timer.
    pub fn start_streaming(&mut self) {
        self.streaming_started = Some(Instant::now());
        self.state.streaming.active = true;
    }

    /// Stops streaming timer.
    pub fn stop_streaming(&mut self) {
        self.streaming_started = None;
        self.state.streaming = StreamingState::idle();
    }

    /// Updates elapsed times and returns the current state.
    pub fn update_elapsed(&mut self) -> &ObsState {
        if let Some(started) = self.recording_started {
            if !self.state.recording.paused {
                let elapsed = started.elapsed().as_secs();
                self.state.recording.elapsed_secs = elapsed;
                self.state.recording.timecode = Some(RecordingState::format_timecode(elapsed));
            }
        }

        if let Some(started) = self.streaming_started {
            let elapsed = started.elapsed().as_secs();
            self.state.streaming.elapsed_secs = elapsed;
            self.state.streaming.timecode = Some(RecordingState::format_timecode(elapsed));
        }

        self.state.touch();
        &self.state
    }

    /// Sets the current scene.
    pub fn set_scene(&mut self, scene: impl Into<String>) {
        self.state.current_scene = Some(scene.into());
    }

    /// Sets the list of available scenes.
    pub fn set_scenes(&mut self, scenes: Vec<String>) {
        self.state.scenes = scenes;
    }

    /// Sets performance stats.
    pub fn set_stats(&mut self, mut stats: ObsStats) {
        stats.calculate_percentages();
        self.state.stats = Some(stats);
    }
}

impl Default for StateTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timecode_format() {
        assert_eq!(RecordingState::format_timecode(0), "00:00");
        assert_eq!(RecordingState::format_timecode(59), "00:59");
        assert_eq!(RecordingState::format_timecode(60), "01:00");
        assert_eq!(RecordingState::format_timecode(3599), "59:59");
        assert_eq!(RecordingState::format_timecode(3600), "01:00:00");
        assert_eq!(RecordingState::format_timecode(3661), "01:01:01");
    }

    #[test]
    fn obs_state_default_disconnected() {
        let state = ObsState::default();
        assert!(!state.connected);
        assert!(!state.recording.active);
        assert!(!state.streaming.active);
    }

    #[test]
    fn state_tracker_recording_lifecycle() {
        let mut tracker = StateTracker::new();

        tracker.set_connected();
        assert!(tracker.state.connected);

        tracker.start_recording();
        assert!(tracker.state.recording.active);
        assert!(!tracker.state.recording.paused);

        tracker.pause_recording();
        assert!(tracker.state.recording.paused);

        tracker.resume_recording();
        assert!(!tracker.state.recording.paused);

        tracker.stop_recording();
        assert!(!tracker.state.recording.active);
    }

    #[test]
    fn obs_stats_percentages() {
        let mut stats = ObsStats {
            render_missed_frames: 10,
            render_total_frames: 1000,
            output_skipped_frames: 5,
            output_total_frames: 500,
            ..Default::default()
        };

        stats.calculate_percentages();

        assert!((stats.render_drop_percent.unwrap() - 1.0).abs() < 0.001);
        assert!((stats.output_drop_percent.unwrap() - 1.0).abs() < 0.001);
    }
}
