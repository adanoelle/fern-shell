pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Effects
import "../config" as Config
import "../components"

// Border component that creates the curved screen frame.
// Uses a mask technique to cut out the interior, leaving only the border.
Item {
    id: root

    required property Item bar

    anchors.fill: parent

    // Solid color rectangle that will be masked
    StyledRect {
        anchors.fill: parent
        color: Config.Theme.barBackground

        layer.enabled: true
        layer.effect: MultiEffect {
            maskSource: mask
            maskEnabled: true
            maskInverted: true
            maskThresholdMin: 0.5
            maskSpreadAtMin: 1
        }
    }

    // Mask: a rounded rectangle defining the interior cutout
    Item {
        id: mask

        anchors.fill: parent
        layer.enabled: true
        visible: false

        Rectangle {
            anchors.fill: parent
            // Margins define the border thickness
            anchors.margins: Config.Theme.borderThickness
            // Left margin is bar width (border continues behind bar)
            anchors.leftMargin: root.bar.implicitWidth
            // Corner rounding
            radius: Config.Theme.borderRounding
        }
    }
}
