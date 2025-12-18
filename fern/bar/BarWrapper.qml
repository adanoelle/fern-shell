pragma ComponentBehavior: Bound
import QtQuick
import Quickshell
import "../config" as Config

// Bar wrapper - fixed width container for bar content.
// Previously handled expand/collapse animation, now simplified to always-visible.
Item {
    id: root

    required property ShellScreen screen

    // Fixed dimensions
    implicitWidth: Config.Theme.barWidth
    implicitHeight: parent?.height ?? 0

    // Bar content - always active
    Bar {
        id: barContent

        anchors.fill: parent
        screen: root.screen
    }
}
