import QtQuick
import QtQuick.Layouts
import "../config" as Config
import "../components"
import "../drawers"

// Vertical clock module for the bar.
// Displays time in a compact vertical format with optional date icon.
// Clicking opens a calendar/date popout.
ModuleContainer {
    id: root

    property color textColor: Config.Theme.accent

    // Size to fit content
    implicitWidth: Config.Theme.barWidth - Config.Theme.spacing.sm * 2
    implicitHeight: layout.implicitHeight + padding * 2

    // Update timer - only runs when module is visible
    Timer {
        id: updateTimer
        interval: 1000
        running: root.visible
        repeat: true
        triggeredOnStart: true

        onTriggered: root.updateTime()
    }

    function updateTime(): void {
        const now = new Date();
        const hours = now.getHours().toString().padStart(2, '0');
        const mins = now.getMinutes().toString().padStart(2, '0');
        timeText.text = hours + "\n" + mins;
    }

    // Cleanup on destruction
    Component.onDestruction: {
        updateTimer.stop();
    }

    // Click to toggle clock popout
    MouseArea {
        anchors.fill: parent
        onClicked: DrawerController.toggleDrawer("clock", root)
    }

    Column {
        id: layout

        anchors.centerIn: parent
        spacing: Config.Theme.spacing.xs

        // Calendar icon
        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "\ue935"  // calendar_month in Material Symbols
            font.family: Config.Theme.fontIcon
            font.pixelSize: Config.Theme.fontSize.lg
            color: root.textColor
        }

        // Time display (vertical: HH on top, mm below)
        Text {
            id: timeText

            anchors.horizontalCenter: parent.horizontalCenter
            horizontalAlignment: Text.AlignHCenter
            font.family: Config.Theme.fontMono
            font.pixelSize: Config.Theme.fontSize.sm
            font.weight: Font.Medium
            color: root.textColor
            lineHeight: 1.1

            // Initial value set by timer's triggeredOnStart
            text: "00\n00"
        }
    }
}
