pragma ComponentBehavior: Bound
import QtQuick
import Quickshell
import "../config" as Config

// Session/power menu content with power actions.
// Used inside a DrawerPanel with edge: "right"
//
// Actions:
// - Logout: End user session
// - Shutdown: Power off system
// - Suspend: Sleep/standby mode
// - Reboot: Restart system
Column {
    id: root

    spacing: Config.Theme.spacing.lg
    padding: Config.Theme.spacing.md

    // Session action button component
    component SessionButton: Rectangle {
        id: button

        required property string icon
        required property string label
        required property var command  // Array of strings for execDetached

        width: 72
        height: 72
        radius: Config.Theme.radius.lg
        color: button.activeFocus ? Config.Theme.surfaceActive
             : mouseArea.containsMouse ? Config.Theme.surfaceHover
             : Config.Theme.surface

        // Make focusable for keyboard navigation
        activeFocusOnTab: true

        // Keyboard handlers
        Keys.onEscapePressed: DrawerController.hideDrawer()
        Keys.onReturnPressed: executeCommand()
        Keys.onEnterPressed: executeCommand()

        function executeCommand() {
            console.log("Fern: Executing", button.label, button.command);
            Quickshell.exec(button.command);
            DrawerController.hideDrawer();
        }

        // Icon
        Text {
            anchors.centerIn: parent
            anchors.verticalCenterOffset: -8
            text: button.icon
            font.family: Config.Theme.fontIcon
            font.pixelSize: 28
            color: Config.Theme.foreground
        }

        // Label below icon
        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.bottom: parent.bottom
            anchors.bottomMargin: 6
            text: button.label
            font.family: Config.Theme.fontFamily
            font.pixelSize: Config.Theme.fontSize.xs
            color: Config.Theme.foregroundDim
        }

        MouseArea {
            id: mouseArea
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: button.executeCommand()
        }

        Behavior on color {
            ColorAnimation { duration: Config.Theme.duration.fast }
        }
    }

    // Logout
    SessionButton {
        id: logoutButton
        icon: "\ue9ba"  // logout
        label: "Logout"
        command: ["loginctl", "terminate-user", ""]

        KeyNavigation.down: shutdownButton

        // Grab focus when session drawer opens
        Connections {
            target: DrawerController
            function onCurrentDrawerChanged() {
                if (DrawerController.currentDrawer === "session") {
                    logoutButton.forceActiveFocus();
                }
            }
        }
    }

    // Shutdown
    SessionButton {
        id: shutdownButton
        icon: "\ue8ac"  // power_settings_new
        label: "Shutdown"
        command: ["systemctl", "poweroff"]

        KeyNavigation.up: logoutButton
        KeyNavigation.down: suspendButton
    }

    // Animated GIF - playful visual element (kurukuru = spinning)
    AnimatedImage {
        width: 72
        height: 72
        sourceSize.width: width
        sourceSize.height: height

        playing: visible
        asynchronous: true
        speed: 0.7
        source: Qt.resolvedUrl("../assets/kurukuru.gif")
    }

    // Suspend/Sleep
    SessionButton {
        id: suspendButton
        icon: "\uef44"  // bedtime (moon icon)
        label: "Suspend"
        command: ["systemctl", "suspend"]

        KeyNavigation.up: shutdownButton
        KeyNavigation.down: rebootButton
    }

    // Reboot
    SessionButton {
        id: rebootButton
        icon: "\uf053"  // restart_alt
        label: "Reboot"
        command: ["systemctl", "reboot"]

        KeyNavigation.up: suspendButton
    }
}
