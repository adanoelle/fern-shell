import QtQuick
import QtQuick.Layouts
import "../config" as Config
import "../components"
import "../services" as Services

// OBS recording/streaming control module for the bar.
// Shows status indicator with timecode, click to toggle recording.
ModuleContainer {
    id: root

    // Module state
    readonly property bool isConnected: Services.Obs.connected
    readonly property bool isRecording: Services.Obs.isRecording
    readonly property bool isPaused: Services.Obs.isPaused
    readonly property bool isStreaming: Services.Obs.isStreaming

    // Status indicator color
    property color indicatorColor: Services.Obs.statusColor

    // Size to fit content
    implicitWidth: Config.Theme.barWidth - Config.Theme.spacing.sm * 2
    implicitHeight: layout.implicitHeight + padding * 2

    // Click to toggle recording
    MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        acceptedButtons: Qt.LeftButton | Qt.RightButton

        onClicked: (mouse) => {
            if (mouse.button === Qt.LeftButton) {
                Services.Obs.toggleRecording();
            } else if (mouse.button === Qt.RightButton) {
                Services.Obs.togglePause();
            }
        }
    }

    Column {
        id: layout

        anchors.centerIn: parent
        spacing: Config.Theme.spacing.xs

        // Recording/streaming status icon
        Rectangle {
            id: statusIndicator

            anchors.horizontalCenter: parent.horizontalCenter
            width: 12
            height: 12
            radius: 6
            color: root.indicatorColor

            // Pulsing animation when recording
            SequentialAnimation on opacity {
                running: root.isRecording && !root.isPaused
                loops: Animation.Infinite

                NumberAnimation {
                    to: 0.4
                    duration: 800
                    easing.type: Easing.InOutSine
                }
                NumberAnimation {
                    to: 1.0
                    duration: 800
                    easing.type: Easing.InOutSine
                }
            }

            // Reset opacity when not recording
            states: State {
                when: !root.isRecording || root.isPaused
                PropertyChanges {
                    target: statusIndicator
                    opacity: 1.0
                }
            }
        }

        // Status icon (camera or broadcast)
        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: {
                if (!root.isConnected) return "\ue654";  // videocam_off
                if (root.isRecording) return "\ue04b";   // videocam (filled when recording)
                if (root.isStreaming) return "\uef71";   // cast (streaming)
                return "\ue04b";  // videocam (default)
            }
            font.family: Config.Theme.fontIcon
            font.pixelSize: Config.Theme.fontSize.lg
            color: root.indicatorColor
            opacity: root.isConnected ? 1.0 : 0.5
        }

        // Timecode display (when recording or streaming)
        Text {
            id: timecodeText

            visible: root.isRecording || root.isStreaming
            anchors.horizontalCenter: parent.horizontalCenter
            horizontalAlignment: Text.AlignHCenter
            font.family: Config.Theme.fontMono
            font.pixelSize: Config.Theme.fontSize.xs
            font.weight: Font.Medium
            color: root.indicatorColor
            lineHeight: 1.1

            // Format timecode vertically (MM on top, SS below)
            text: {
                const tc = root.isRecording
                    ? Services.Obs.recordingTimecode
                    : Services.Obs.streamingTimecode;
                // Parse HH:MM:SS or MM:SS format
                const parts = tc.split(":");
                if (parts.length === 3) {
                    // HH:MM:SS - show MM:SS
                    return parts[1] + "\n" + parts[2];
                } else if (parts.length === 2) {
                    // MM:SS
                    return parts[0] + "\n" + parts[1];
                }
                return "00\n00";
            }
        }

        // Scene indicator (small, when not recording)
        Text {
            visible: !root.isRecording && !root.isStreaming && root.isConnected
            anchors.horizontalCenter: parent.horizontalCenter
            horizontalAlignment: Text.AlignHCenter
            font.family: Config.Theme.fontFamily
            font.pixelSize: Config.Theme.fontSize.xs
            color: Config.Theme.textMuted
            text: {
                const scene = Services.Obs.currentScene;
                // Truncate long scene names
                return scene.length > 6 ? scene.substring(0, 5) + "…" : scene;
            }
        }
    }

    // Tooltip
    ToolTip {
        id: tooltip
        visible: mouseArea.containsMouse
        text: {
            if (!root.isConnected) return "OBS: Disconnected\nClick to start daemon";
            let tip = "OBS: " + Services.Obs.statusText;
            if (root.isRecording) {
                tip += "\nDuration: " + Services.Obs.recordingTimecode;
            }
            if (root.isStreaming) {
                tip += "\nStream: " + Services.Obs.streamingTimecode;
            }
            if (Services.Obs.currentScene) {
                tip += "\nScene: " + Services.Obs.currentScene;
            }
            tip += "\n\nLeft-click: Toggle recording";
            tip += "\nRight-click: Toggle pause";
            return tip;
        }
    }

    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true
        propagateComposedEvents: true

        onClicked: (mouse) => {
            if (!root.isConnected) {
                Services.Obs.startDaemon();
                return;
            }
            if (mouse.button === Qt.LeftButton) {
                Services.Obs.toggleRecording();
            } else if (mouse.button === Qt.RightButton) {
                Services.Obs.togglePause();
            }
        }
    }
}

// Simple tooltip component
component ToolTip: Rectangle {
    property string text: ""

    visible: false
    color: Config.Theme.surface
    border.color: Config.Theme.border
    border.width: 1
    radius: Config.Theme.radius.sm
    width: tooltipText.implicitWidth + Config.Theme.spacing.md * 2
    height: tooltipText.implicitHeight + Config.Theme.spacing.sm * 2

    // Position above the module
    x: (parent.width - width) / 2
    y: -height - Config.Theme.spacing.sm

    Text {
        id: tooltipText
        anchors.centerIn: parent
        text: parent.text
        font.family: Config.Theme.fontFamily
        font.pixelSize: Config.Theme.fontSize.sm
        color: Config.Theme.text
    }
}
