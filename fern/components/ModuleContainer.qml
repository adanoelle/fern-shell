import QtQuick
import "../config" as Config

// Pill-shaped container for bar modules.
// Provides consistent styling with full radius (9999px) for pill effect.
Rectangle {
    id: root

    // Full pill radius from theme
    radius: Config.Theme.radius.full

    // Module background color
    color: Config.Theme.moduleBackground

    // Content padding
    property int padding: Config.Theme.modulePadding

    // Default content item for children
    default property alias content: contentItem.data

    // Animated color transitions
    Behavior on color {
        ColorAnimation {
            duration: Config.Theme.duration.normal
            easing.type: Config.Theme.easing.standard
        }
    }

    // Content container with padding
    Item {
        id: contentItem
        anchors.fill: parent
        anchors.margins: root.padding
    }
}
