pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io

// Singleton service for structured logging.
// Maintains a ring buffer of log entries and writes them to a state file
// for aggregation by fernctl.
Singleton {
    id: root

    // === STATE FILE PATH ===
    readonly property string statePath: {
        const xdgState = Quickshell.env("XDG_STATE_HOME") || (Quickshell.env("HOME") + "/.local/state");
        return xdgState + "/fern/shell-log.json";
    }

    // Maximum number of log entries to keep (ring buffer)
    readonly property int maxEntries: 100

    // Current log entries
    property var entries: []

    // Debounce timer for batching writes
    Timer {
        id: writeDebounce
        interval: 500
        repeat: false
        onTriggered: root.writeState()
    }

    // === PUBLIC LOGGING API ===

    // Log an info message
    function info(module: string, message: string, data: var) {
        logEntry("info", module, message, data);
    }

    // Log a warning message (also outputs to console)
    function warn(module: string, message: string, data: var) {
        logEntry("warn", module, message, data);
        console.warn(`[${module}] ${message}`);
    }

    // Log an error message (also outputs to console)
    function error(module: string, message: string, data: var) {
        logEntry("error", module, message, data);
        console.error(`[${module}] ${message}`);
    }

    // Log a debug message (only when FERN_DEBUG is set)
    function debug(module: string, message: string, data: var) {
        if (Quickshell.env("FERN_DEBUG")) {
            logEntry("debug", module, message, data);
            console.log(`[${module}] ${message}`);
        }
    }

    // === INTERNAL IMPLEMENTATION ===

    // Add a log entry to the ring buffer
    function logEntry(level: string, module: string, message: string, data: var) {
        const entry = {
            timestamp: new Date().toISOString(),
            level: level,
            module: module,
            message: message,
            data: data || {}
        };

        entries.push(entry);

        // Ring buffer: remove oldest entries if over limit
        if (entries.length > maxEntries) {
            entries = entries.slice(-maxEntries);
        }

        // Debounce file writes
        writeDebounce.restart();
    }

    // Write state to file using atomic write pattern
    function writeState() {
        const state = {
            version: 1,
            updated: new Date().toISOString(),
            entries: entries
        };

        const json = JSON.stringify(state, null, 2);
        const tempPath = statePath + ".tmp";

        // Write to temp file, then rename for atomicity
        writeProcess.command = ["sh", "-c",
            `mkdir -p "$(dirname '${statePath}')" && ` +
            `printf '%s' '${json.replace(/'/g, "'\\''")}' > '${tempPath}' && ` +
            `chmod 600 '${tempPath}' && ` +
            `mv '${tempPath}' '${statePath}'`
        ];
        writeProcess.running = true;
    }

    // Process for atomic file writes
    Process {
        id: writeProcess
        command: ["true"]
        running: false
    }

    // Cleanup on destruction
    Component.onDestruction: {
        writeDebounce.stop();
        // Final flush
        if (entries.length > 0) {
            writeState();
        }
    }

    Component.onCompleted: {
        console.log("Log: Service initialized, writing to", statePath);
    }
}
