pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io

// Singleton service for OBS Studio integration.
// Reads state from fern-obs daemon via state file and provides control methods.
Singleton {
    id: root

    // === STATE FILE PATH ===
    // XDG state directory + fern/obs-state.json
    readonly property string statePath: {
        const xdgState = Quickshell.env("XDG_STATE_HOME") || (Quickshell.env("HOME") + "/.local/state");
        return xdgState + "/fern/obs-state.json";
    }

    // === CONNECTION STATE ===
    readonly property bool connected: state.connected ?? false
    readonly property string errorMessage: state.error ?? ""

    // === RECORDING STATE ===
    readonly property bool isRecording: state.recording?.active ?? false
    readonly property bool isPaused: state.recording?.paused ?? false
    readonly property int recordingElapsed: state.recording?.elapsed_secs ?? 0
    readonly property string recordingTimecode: state.recording?.timecode ?? "00:00"

    // === STREAMING STATE ===
    readonly property bool isStreaming: state.streaming?.active ?? false
    readonly property int streamingElapsed: state.streaming?.elapsed_secs ?? 0
    readonly property string streamingTimecode: state.streaming?.timecode ?? "00:00"
    readonly property bool isReconnecting: state.streaming?.reconnecting ?? false

    // === SCENE STATE ===
    readonly property string currentScene: state.current_scene ?? ""
    readonly property var scenes: state.scenes ?? []

    // === STATS ===
    readonly property var stats: state.stats ?? null
    readonly property real cpuUsage: stats?.cpu_usage ?? 0
    readonly property real activeFps: stats?.active_fps ?? 0
    readonly property real renderDropPercent: stats?.render_drop_percent ?? 0
    readonly property real outputDropPercent: stats?.output_drop_percent ?? 0

    // === INTERNAL STATE ===
    property var state: ({})
    property bool stateLoaded: false

    // === STATE FILE WATCHER ===
    FileView {
        id: stateFile
        path: root.statePath
        watchChanges: true

        onTextChanged: {
            if (text && text.length > 0) {
                root.parseState(text);
            } else {
                // File doesn't exist or daemon not running
                root.state = { connected: false };
                root.stateLoaded = false;
            }
        }
    }

    // Parse JSON state from file
    function parseState(jsonText: string) {
        try {
            const parsed = JSON.parse(jsonText);
            state = parsed;
            stateLoaded = true;
        } catch (e) {
            Log.error("Obs", "Failed to parse state", { error: e.message });
            state = { connected: false, error: "Failed to parse state file" };
        }
    }

    // === CONTROL METHODS ===

    // Start recording
    function startRecording(): void {
        Quickshell.execDetached("fern-obs", ["start-recording"]);
    }

    // Stop recording
    function stopRecording(): void {
        Quickshell.execDetached("fern-obs", ["stop-recording"]);
    }

    // Toggle recording (start if stopped, stop if started)
    function toggleRecording(): void {
        if (isRecording) {
            stopRecording();
        } else {
            startRecording();
        }
    }

    // Toggle recording pause
    function togglePause(): void {
        Quickshell.execDetached("fern-obs", ["toggle-pause"]);
    }

    // Start streaming
    function startStreaming(): void {
        Quickshell.execDetached("fern-obs", ["start-streaming"]);
    }

    // Stop streaming
    function stopStreaming(): void {
        Quickshell.execDetached("fern-obs", ["stop-streaming"]);
    }

    // Toggle streaming
    function toggleStreaming(): void {
        if (isStreaming) {
            stopStreaming();
        } else {
            startStreaming();
        }
    }

    // Set scene by name
    function setScene(sceneName: string): void {
        Quickshell.execDetached("fern-obs", ["scene", sceneName]);
    }

    // Start the daemon if not running
    function startDaemon(): void {
        Quickshell.execDetached("fern-obs", ["daemon"]);
    }

    // === CONVENIENCE PROPERTIES ===

    // True if any activity (recording or streaming)
    readonly property bool isActive: isRecording || isStreaming

    // Status text for display
    readonly property string statusText: {
        if (!connected) return "Disconnected";
        if (isRecording && isStreaming) return "Recording & Streaming";
        if (isRecording) return isPaused ? "Paused" : "Recording";
        if (isStreaming) return isReconnecting ? "Reconnecting" : "Streaming";
        return "Ready";
    }

    // Recording/streaming indicator color
    readonly property string statusColor: {
        if (!connected) return "#6c7086";  // Overlay0
        if (isRecording) return isPaused ? "#fab387" : "#f38ba8";  // Peach or Red
        if (isStreaming) return isReconnecting ? "#fab387" : "#a6e3a1";  // Peach or Green
        return "#89b4fa";  // Blue (ready)
    }

    Component.onCompleted: {
        Log.info("Obs", "Service initialized", { statePath: statePath });
    }
}
