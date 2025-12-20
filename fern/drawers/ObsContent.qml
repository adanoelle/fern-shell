pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts
import "../config" as Config
import "../services" as Services

// OBS Studio control drawer content.
// Shows connection status, recording/streaming controls, and scene selector.
Item {
    id: root

    // Content fills available width from DrawerPanel
    implicitWidth: parent.width
    implicitHeight: contentColumn.implicitHeight + contentPadding * 2

    // Padding value used by children
    readonly property real contentPadding: Config.Theme.spacing.sm
    readonly property real contentWidth: width - contentPadding * 2

    // Connection status
    readonly property bool isConnected: Services.Obs.connected

    Column {
        id: contentColumn

        anchors.centerIn: parent
        width: root.contentWidth
        spacing: Config.Theme.spacing.md

        // Status header
        Rectangle {
            width: parent.width
            height: statusRow.implicitHeight + Config.Theme.spacing.sm * 2
            radius: Config.Theme.radius.md
            color: root.isConnected ? Config.Theme.surface : Config.Theme.overlay

            Row {
                id: statusRow
                anchors.centerIn: parent
                spacing: Config.Theme.spacing.sm

                // Status indicator dot
                Rectangle {
                    anchors.verticalCenter: parent.verticalCenter
                    width: 8
                    height: 8
                    radius: 4
                    color: root.isConnected ? Config.Theme.success : Config.Theme.error
                }

                Text {
                    text: root.isConnected ? "Connected" : "Disconnected"
                    color: Config.Theme.foreground
                    font.family: Config.Theme.fontFamily
                    font.pixelSize: Config.Theme.fontSize.sm
                    font.weight: Font.Medium
                }
            }
        }

        // Start daemon button (only when disconnected)
        Rectangle {
            visible: !root.isConnected
            width: parent.width
            height: 40
            radius: Config.Theme.radius.md
            color: mouseAreaDaemon.containsMouse ? Config.Theme.surfaceHover : Config.Theme.surface

            Text {
                anchors.centerIn: parent
                text: "Start OBS Daemon"
                color: Config.Theme.foreground
                font.family: Config.Theme.fontFamily
                font.pixelSize: Config.Theme.fontSize.sm
            }

            MouseArea {
                id: mouseAreaDaemon
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: Services.Obs.startDaemon()
            }
        }

        // Recording/Streaming controls (only when connected)
        Column {
            visible: root.isConnected
            width: parent.width
            spacing: Config.Theme.spacing.sm

            // Recording control
            Rectangle {
                width: parent.width
                height: 48
                radius: Config.Theme.radius.md
                color: Services.Obs.isRecording
                    ? Config.Theme.error
                    : (mouseAreaRecord.containsMouse ? Config.Theme.surfaceHover : Config.Theme.surface)

                Row {
                    anchors.centerIn: parent
                    spacing: Config.Theme.spacing.sm

                    Text {
                        text: Services.Obs.isRecording ? "\ue047" : "\ue04b"  // stop or videocam
                        font.family: Config.Theme.fontIcon
                        font.pixelSize: 20
                        color: Services.Obs.isRecording ? Config.Theme.background : Config.Theme.foreground
                    }

                    Text {
                        text: Services.Obs.isRecording
                            ? "Stop (" + Services.Obs.recordingTimecode + ")"
                            : "Record"
                        color: Services.Obs.isRecording ? Config.Theme.background : Config.Theme.foreground
                        font.family: Config.Theme.fontFamily
                        font.pixelSize: Config.Theme.fontSize.md
                        font.weight: Font.Medium
                    }
                }

                MouseArea {
                    id: mouseAreaRecord
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: Services.Obs.toggleRecording()
                }
            }

            // Pause button (only when recording)
            Rectangle {
                visible: Services.Obs.isRecording
                width: parent.width
                height: 36
                radius: Config.Theme.radius.md
                color: Services.Obs.isPaused
                    ? Config.Theme.warning
                    : (mouseAreaPause.containsMouse ? Config.Theme.surfaceHover : Config.Theme.surface)

                Text {
                    anchors.centerIn: parent
                    text: Services.Obs.isPaused ? "Resume" : "Pause"
                    color: Services.Obs.isPaused ? Config.Theme.background : Config.Theme.foreground
                    font.family: Config.Theme.fontFamily
                    font.pixelSize: Config.Theme.fontSize.sm
                }

                MouseArea {
                    id: mouseAreaPause
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: Services.Obs.togglePause()
                }
            }

            // Streaming control
            Rectangle {
                width: parent.width
                height: 48
                radius: Config.Theme.radius.md
                color: Services.Obs.isStreaming
                    ? Config.Theme.success
                    : (mouseAreaStream.containsMouse ? Config.Theme.surfaceHover : Config.Theme.surface)

                Row {
                    anchors.centerIn: parent
                    spacing: Config.Theme.spacing.sm

                    Text {
                        text: Services.Obs.isStreaming ? "\ue047" : "\uef71"  // stop or cast
                        font.family: Config.Theme.fontIcon
                        font.pixelSize: 20
                        color: Services.Obs.isStreaming ? Config.Theme.background : Config.Theme.foreground
                    }

                    Text {
                        text: Services.Obs.isStreaming
                            ? "End Stream (" + Services.Obs.streamingTimecode + ")"
                            : "Go Live"
                        color: Services.Obs.isStreaming ? Config.Theme.background : Config.Theme.foreground
                        font.family: Config.Theme.fontFamily
                        font.pixelSize: Config.Theme.fontSize.md
                        font.weight: Font.Medium
                    }
                }

                MouseArea {
                    id: mouseAreaStream
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: Services.Obs.toggleStreaming()
                }
            }
        }

        // Scene selector (only when connected and has scenes)
        Column {
            visible: root.isConnected && Services.Obs.scenes.length > 0
            width: parent.width
            spacing: Config.Theme.spacing.xs

            Text {
                text: "Scenes"
                color: Config.Theme.foregroundDim
                font.family: Config.Theme.fontFamily
                font.pixelSize: Config.Theme.fontSize.xs
                font.weight: Font.Medium
            }

            Repeater {
                model: Services.Obs.scenes

                Rectangle {
                    required property string modelData
                    readonly property bool isActive: modelData === Services.Obs.currentScene

                    width: contentColumn.width
                    height: 32
                    radius: Config.Theme.radius.sm
                    color: isActive
                        ? Config.Theme.accent
                        : (sceneMouseArea.containsMouse ? Config.Theme.surfaceHover : Config.Theme.surface)

                    Text {
                        anchors.centerIn: parent
                        text: modelData
                        color: parent.isActive ? Config.Theme.background : Config.Theme.foreground
                        font.family: Config.Theme.fontFamily
                        font.pixelSize: Config.Theme.fontSize.sm
                        elide: Text.ElideRight
                    }

                    MouseArea {
                        id: sceneMouseArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: Services.Obs.setScene(parent.modelData)
                    }
                }
            }
        }
    }
}
