import QtQuick
import "../config" as Config
import "../components"
import "../drawers"

// Power button module for the bar.
// Displays a power icon that triggers a session/power popout.
ModuleContainer {
    id: root

    property color iconColor: Config.Theme.error

    // Size to fit content
    implicitWidth: Config.Theme.barWidth - Config.Theme.spacing.sm * 2
    implicitHeight: icon.implicitHeight + padding * 2

    // Click to toggle session drawer (right edge)
    MouseArea {
        anchors.fill: parent
        onClicked: DrawerController.toggleDrawer("session", root)
    }

    // Power icon
    Text {
        id: icon

        anchors.centerIn: parent
        text: "\ue8ac"  // power_settings_new in Material Symbols
        font.family: Config.Theme.fontIcon
        font.pixelSize: Config.Theme.fontSize.lg
        color: root.iconColor
    }
}
