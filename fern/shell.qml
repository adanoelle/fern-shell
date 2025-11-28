// fern/shell.qml – Main panel entry point
import Quickshell
import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import "config" as Config

PanelWindow {
    id: root

    // Use Theme tokens for positioning and sizing
    anchors {
        top: Config.Theme.barPosition === "top"
        bottom: Config.Theme.barPosition === "bottom"
        left: true
        right: true
    }
    implicitHeight: Config.Theme.barHeight
    color: Config.Theme.barBackground

    // Terminal launcher process
    Process {
        id: terminalProc
        command: ["ghostty"]  // TODO: Make configurable via config.modules.terminal
    }

    // Bar content layout
    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: Config.Theme.barPadding
        anchors.rightMargin: Config.Theme.barPadding
        spacing: Config.Theme.moduleSpacing

        // Left modules
        RowLayout {
            Layout.alignment: Qt.AlignLeft
            spacing: Config.Theme.spacing.sm

            // Placeholder for workspaces module
            Text {
                text: "Workspaces"
                color: Config.Theme.foregroundDim
                font.family: Config.Theme.fontFamily
                font.pixelSize: Config.Theme.fontSize.sm
            }
        }

        // Center spacer
        Item { Layout.fillWidth: true }

        // Center modules
        RowLayout {
            Layout.alignment: Qt.AlignCenter
            spacing: Config.Theme.spacing.sm

            // Simple clock placeholder
            Text {
                id: clockText
                color: Config.Theme.foreground
                font.family: Config.Theme.fontFamily
                font.pixelSize: Config.Theme.fontSize.md

                Timer {
                    interval: 1000
                    running: true
                    repeat: true
                    triggeredOnStart: true
                    onTriggered: {
                        const fmt = Config.Theme.moduleConfig("clock").format || "HH:mm";
                        clockText.text = Qt.formatTime(new Date(), fmt);
                    }
                }
            }
        }

        // Center spacer
        Item { Layout.fillWidth: true }

        // Right modules
        RowLayout {
            Layout.alignment: Qt.AlignRight
            spacing: Config.Theme.spacing.sm

            // Terminal launcher button
            Rectangle {
                width: Config.Theme.buttonPadding * 2 + Config.Theme.iconSize
                height: width
                radius: Config.Theme.buttonRadius
                color: terminalButton.containsMouse
                    ? Config.Theme.buttonBackgroundHover
                    : Config.Theme.buttonBackground

                MouseArea {
                    id: terminalButton
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: terminalProc.running = true
                }

                Text {
                    anchors.centerIn: parent
                    text: ""  // Terminal icon (Nerd Font)
                    font.family: Config.Theme.fontMono
                    font.pixelSize: Config.Theme.iconSize
                    color: Config.Theme.foreground
                }
            }
        }
    }

    // React to theme changes
    Connections {
        target: Config.Theme
        function onThemeChanged() {
            console.log("Theme updated, bar will repaint");
        }
    }
}
