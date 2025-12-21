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

    // ========================================================================
    // Timecode formatting tests
    // ========================================================================

    #[test]
    fn timecode_format_zero() {
        assert_eq!(RecordingState::format_timecode(0), "00:00");
    }

    #[test]
    fn timecode_format_seconds() {
        assert_eq!(RecordingState::format_timecode(1), "00:01");
        assert_eq!(RecordingState::format_timecode(59), "00:59");
    }

    #[test]
    fn timecode_format_minutes() {
        assert_eq!(RecordingState::format_timecode(60), "01:00");
        assert_eq!(RecordingState::format_timecode(90), "01:30");
        assert_eq!(RecordingState::format_timecode(3599), "59:59");
    }

    #[test]
    fn timecode_format_hours() {
        assert_eq!(RecordingState::format_timecode(3600), "01:00:00");
        assert_eq!(RecordingState::format_timecode(3661), "01:01:01");
        assert_eq!(RecordingState::format_timecode(86399), "23:59:59");
    }

    #[test]
    fn timecode_format_large_values() {
        // 100 hours
        assert_eq!(RecordingState::format_timecode(360000), "100:00:00");
    }

    // ========================================================================
    // ObsState tests
    // ========================================================================

    #[test]
    fn obs_state_default_disconnected() {
        let state = ObsState::default();
        assert!(!state.connected);
        assert!(!state.recording.active);
        assert!(!state.streaming.active);
        assert!(state.current_scene.is_none());
        assert!(state.scenes.is_empty());
        assert!(state.stats.is_none());
        assert!(state.error.is_none());
    }

    #[test]
    fn obs_state_disconnected_constructor() {
        let state = ObsState::disconnected();
        assert!(!state.connected);
        assert!(state.error.is_none());
    }

    #[test]
    fn obs_state_with_error() {
        let state = ObsState::with_error("Connection refused");
        assert!(!state.connected);
        assert_eq!(state.error.as_deref(), Some("Connection refused"));
    }

    #[test]
    fn obs_state_with_error_string_ownership() {
        let error = String::from("Network error");
        let state = ObsState::with_error(error);
        assert_eq!(state.error.as_deref(), Some("Network error"));
    }

    #[test]
    fn obs_state_touch_sets_timestamp() {
        let mut state = ObsState::default();
        assert!(state.updated_at_secs.is_none());

        state.touch();
        assert!(state.updated_at_secs.is_some());
        assert!(state.updated_at_secs.unwrap() > 0);
    }

    // ========================================================================
    // RecordingState tests
    // ========================================================================

    #[test]
    fn recording_state_idle() {
        let state = RecordingState::idle();
        assert!(!state.active);
        assert!(!state.paused);
        assert_eq!(state.elapsed_secs, 0);
        assert!(state.output_path.is_none());
        assert!(state.timecode.is_none());
    }

    #[test]
    fn recording_state_active() {
        let state = RecordingState::active(125);
        assert!(state.active);
        assert!(!state.paused);
        assert_eq!(state.elapsed_secs, 125);
        assert_eq!(state.timecode.as_deref(), Some("02:05"));
    }

    #[test]
    fn recording_state_paused() {
        let state = RecordingState::paused(3700);
        assert!(state.active);
        assert!(state.paused);
        assert_eq!(state.elapsed_secs, 3700);
        assert_eq!(state.timecode.as_deref(), Some("01:01:40"));
    }

    // ========================================================================
    // StreamingState tests
    // ========================================================================

    #[test]
    fn streaming_state_idle() {
        let state = StreamingState::idle();
        assert!(!state.active);
        assert_eq!(state.elapsed_secs, 0);
        assert!(state.timecode.is_none());
        assert!(state.bitrate_kbps.is_none());
        assert!(!state.reconnecting);
    }

    #[test]
    fn streaming_state_active() {
        let state = StreamingState::active(300);
        assert!(state.active);
        assert_eq!(state.elapsed_secs, 300);
        assert_eq!(state.timecode.as_deref(), Some("05:00"));
        assert!(!state.reconnecting);
    }

    // ========================================================================
    // ObsStats tests
    // ========================================================================

    #[test]
    fn obs_stats_default() {
        let stats = ObsStats::default();
        assert_eq!(stats.cpu_usage, 0.0);
        assert_eq!(stats.memory_mb, 0.0);
        assert!(stats.render_drop_percent.is_none());
        assert!(stats.output_drop_percent.is_none());
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

    #[test]
    fn obs_stats_percentages_zero_frames() {
        let mut stats = ObsStats::default();
        stats.calculate_percentages();

        // Should not calculate percentages when total is zero
        assert!(stats.render_drop_percent.is_none());
        assert!(stats.output_drop_percent.is_none());
    }

    #[test]
    fn obs_stats_percentages_no_drops() {
        let mut stats = ObsStats {
            render_missed_frames: 0,
            render_total_frames: 1000,
            output_skipped_frames: 0,
            output_total_frames: 1000,
            ..Default::default()
        };

        stats.calculate_percentages();

        assert!((stats.render_drop_percent.unwrap() - 0.0).abs() < 0.001);
        assert!((stats.output_drop_percent.unwrap() - 0.0).abs() < 0.001);
    }

    // ========================================================================
    // StateTracker tests
    // ========================================================================

    #[test]
    fn state_tracker_new() {
        let tracker = StateTracker::new();
        assert!(!tracker.state.connected);
    }

    #[test]
    fn state_tracker_default() {
        let tracker = StateTracker::default();
        assert!(!tracker.state.connected);
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
    fn state_tracker_streaming_lifecycle() {
        let mut tracker = StateTracker::new();

        tracker.set_connected();
        assert!(tracker.state.connected);

        tracker.start_streaming();
        assert!(tracker.state.streaming.active);

        tracker.stop_streaming();
        assert!(!tracker.state.streaming.active);
    }

    #[test]
    fn state_tracker_disconnected_clears_state() {
        let mut tracker = StateTracker::new();

        // Set up some active state
        tracker.set_connected();
        tracker.start_recording();
        tracker.start_streaming();
        tracker.set_scene("Gaming");

        // Disconnect with error
        tracker.set_disconnected(Some("Connection lost".into()));

        assert!(!tracker.state.connected);
        assert_eq!(tracker.state.error.as_deref(), Some("Connection lost"));
        assert!(!tracker.state.recording.active);
        assert!(!tracker.state.streaming.active);
    }

    #[test]
    fn state_tracker_disconnected_without_error() {
        let mut tracker = StateTracker::new();
        tracker.set_connected();
        tracker.set_disconnected(None);

        assert!(!tracker.state.connected);
        assert!(tracker.state.error.is_none());
    }

    #[test]
    fn state_tracker_scene_management() {
        let mut tracker = StateTracker::new();

        tracker.set_scene("Gaming");
        assert_eq!(tracker.state.current_scene.as_deref(), Some("Gaming"));

        tracker.set_scene("Desktop");
        assert_eq!(tracker.state.current_scene.as_deref(), Some("Desktop"));
    }

    #[test]
    fn state_tracker_scenes_list() {
        let mut tracker = StateTracker::new();

        let scenes = vec!["Gaming".into(), "Desktop".into(), "BRB".into()];
        tracker.set_scenes(scenes);

        assert_eq!(tracker.state.scenes.len(), 3);
        assert_eq!(tracker.state.scenes[0], "Gaming");
        assert_eq!(tracker.state.scenes[1], "Desktop");
        assert_eq!(tracker.state.scenes[2], "BRB");
    }

    #[test]
    fn state_tracker_set_stats() {
        let mut tracker = StateTracker::new();

        let stats = ObsStats {
            cpu_usage: 15.5,
            memory_mb: 512.0,
            render_missed_frames: 5,
            render_total_frames: 1000,
            ..Default::default()
        };

        tracker.set_stats(stats);

        let saved_stats = tracker.state.stats.as_ref().unwrap();
        assert!((saved_stats.cpu_usage - 15.5).abs() < 0.001);
        assert!((saved_stats.memory_mb - 512.0).abs() < 0.001);
        // Percentages should be calculated
        assert!(saved_stats.render_drop_percent.is_some());
    }

    #[test]
    fn state_tracker_update_elapsed() {
        let mut tracker = StateTracker::new();
        tracker.set_connected();
        tracker.start_recording();
        tracker.start_streaming();

        // Update elapsed - this should set timestamp
        let state = tracker.update_elapsed();
        assert!(state.updated_at_secs.is_some());
    }

    // ========================================================================
    // Serialization tests
    // ========================================================================

    #[test]
    fn obs_state_serialization_roundtrip() {
        let mut state = ObsState {
            connected: true,
            recording: RecordingState::active(120),
            streaming: StreamingState::active(300),
            current_scene: Some("Gaming".into()),
            scenes: vec!["Gaming".into(), "Desktop".into()],
            stats: Some(ObsStats {
                cpu_usage: 10.0,
                memory_mb: 256.0,
                active_fps: 60.0,
                ..Default::default()
            }),
            error: None,
            updated_at_secs: None,
        };
        state.touch();

        let json = serde_json::to_string(&state).expect("serialization failed");
        let deserialized: ObsState = serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(deserialized.connected, state.connected);
        assert_eq!(deserialized.recording.active, state.recording.active);
        assert_eq!(deserialized.recording.elapsed_secs, state.recording.elapsed_secs);
        assert_eq!(deserialized.streaming.active, state.streaming.active);
        assert_eq!(deserialized.current_scene, state.current_scene);
        assert_eq!(deserialized.scenes, state.scenes);
        assert!(deserialized.stats.is_some());
    }

    #[test]
    fn obs_state_serialization_skips_none_fields() {
        let state = ObsState::disconnected();
        let json = serde_json::to_string(&state).expect("serialization failed");

        // These optional fields should not appear in JSON when None
        assert!(!json.contains("current_scene"));
        assert!(!json.contains("stats"));
        assert!(!json.contains("error"));
        assert!(!json.contains("updated_at_secs"));
    }

    #[test]
    fn recording_state_serialization() {
        let state = RecordingState::active(3661);
        let json = serde_json::to_string(&state).expect("serialization failed");

        assert!(json.contains("\"active\":true"));
        assert!(json.contains("\"elapsed_secs\":3661"));
        assert!(json.contains("\"timecode\":\"01:01:01\""));
    }

    #[test]
    fn streaming_state_serialization() {
        let state = StreamingState::active(600);
        let json = serde_json::to_string(&state).expect("serialization failed");

        assert!(json.contains("\"active\":true"));
        assert!(json.contains("\"elapsed_secs\":600"));
        assert!(json.contains("\"timecode\":\"10:00\""));
    }

    #[test]
    fn obs_state_deserialization_minimal() {
        // Minimal valid JSON - all optional fields missing
        let json = r#"{"connected":false,"recording":{"active":false,"paused":false,"elapsed_secs":0},"streaming":{"active":false,"elapsed_secs":0,"reconnecting":false},"scenes":[]}"#;
        let state: ObsState = serde_json::from_str(json).expect("deserialization failed");

        assert!(!state.connected);
        assert!(state.current_scene.is_none());
    }
}
