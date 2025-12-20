pragma ComponentBehavior: Bound
import QtQuick
import Quickshell
import "../config" as Config

// Drawer area - the space inside the border, to the right of the bar.
// Contains drawers (popouts), notifications, and other overlay content.
Item {
    id: root

    required property Item bar
    required property ShellScreen screen

    // Expose drawer container for input region calculation
    readonly property alias drawerContainer: drawerContainer

    // Drawer area fills screen minus border thickness and bar width
    anchors.fill: parent
    anchors.margins: Config.Theme.borderThickness
    anchors.leftMargin: bar.implicitWidth

    // Click-to-close scrim - covers screen when any drawer is open
    // Also handles Escape key to close
    FocusScope {
        id: scrim
        anchors.fill: parent
        visible: DrawerController.hasDrawer
        focus: DrawerController.hasDrawer

        Keys.onEscapePressed: DrawerController.hideDrawer()

        // Grab focus when drawer opens
        onVisibleChanged: {
            if (visible) forceActiveFocus();
        }

        MouseArea {
            anchors.fill: parent
            onClicked: DrawerController.hideDrawer()
        }
    }

    // Drawer container - positioned relative to bar elements
    Item {
        id: drawerContainer

        anchors.left: parent.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: 300  // Max drawer width

        // Clock drawer - uses DrawerPanel for clean encapsulation
        DrawerPanel {
            id: clockPopout
            name: "clock"
            contentWidth: 220

            Column {
                id: clockContent
                width: parent.width
                spacing: Config.Theme.spacing.md

                // Date header
                Text {
                    width: parent.width
                    text: {
                        const now = new Date();
                        return now.toLocaleDateString(Qt.locale(), "dddd, MMMM d");
                    }
                    color: Config.Theme.foreground
                    font.family: Config.Theme.fontFamily
                    font.pixelSize: Config.Theme.fontSize.md
                    font.weight: Font.Medium
                }

                // Large time display
                Text {
                    id: timeDisplay
                    width: parent.width
                    text: formatTime()
                    color: Config.Theme.accent
                    font.family: Config.Theme.fontMono
                    font.pixelSize: Config.Theme.fontSize.xxl
                    font.weight: Font.Bold

                    function formatTime(): string {
                        const now = new Date();
                        const hours = now.getHours();
                        const mins = now.getMinutes().toString().padStart(2, '0');
                        const ampm = hours >= 12 ? 'PM' : 'AM';
                        const hour12 = hours % 12 || 12;
                        return `${hour12}:${mins} ${ampm}`;
                    }

                    Timer {
                        interval: 1000
                        running: clockPopout.isVisible
                        repeat: true
                        triggeredOnStart: true
                        onTriggered: timeDisplay.text = timeDisplay.formatTime()
                    }
                }

                // Separator
                Rectangle {
                    width: parent.width
                    height: 1
                    color: Config.Theme.overlay
                    opacity: 0.3
                }

                // Calendar placeholder
                Text {
                    width: parent.width
                    text: "Calendar coming soon..."
                    color: Config.Theme.foregroundDim
                    font.family: Config.Theme.fontFamily
                    font.pixelSize: Config.Theme.fontSize.sm
                    font.italic: true
                }
            }
        }

        // OBS control drawer
        DrawerPanel {
            id: obsDrawer
            name: "obs"
            contentWidth: 180

            ObsContent {}
        }

        // Future: Dynamic tray menu drawers via Repeater
        // Repeater {
        //     model: SystemTray.items.values
        //     DrawerPanel {
        //         required property SystemTrayItem modelData
        //         required property int index
        //         name: `traymenu${index}`
        //         contentWidth: 200
        //         TrayMenuContent { trayItem: modelData.menu }
        //     }
        // }
    }

    // Right-edge drawer container - for session, notifications, etc.
    Item {
        id: rightDrawerContainer

        anchors.right: parent.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: 120  // Session drawer width

        // Session/Power drawer - right edge, vertically centered
        DrawerPanel {
            id: sessionDrawer
            name: "session"
            edge: "right"
            centered: true
            contentWidth: 88  // Fits 72px button + padding

            SessionContent {}
        }
    }

    // Note: Click-outside is now handled by HyprlandFocusGrab in DrawerPanel
}
