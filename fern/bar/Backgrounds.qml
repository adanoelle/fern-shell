pragma ComponentBehavior: Bound
import QtQuick
import "../config" as Config
import "../components"

// Backgrounds component that draws the bar background.
// The bar background extends from the left edge, filling behind the bar content.
Item {
    id: root

    required property Item bar

    anchors.fill: parent

    // Bar background - fills the bar area
    StyledRect {
        id: barBackground

        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.left: parent.left

        // Width matches the bar's current width
        width: root.bar.implicitWidth

        color: Config.Theme.barBackground

        // Subtle gradient overlay for depth (optional)
        Rectangle {
            anchors.fill: parent
            anchors.leftMargin: parent.width - Config.Theme.borderThickness
            color: Config.Theme.surface
            opacity: 0.3
        }
    }
}
