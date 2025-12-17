import QtQuick
import "../config" as Config

// Base rectangle with animated color transitions.
// Use this as the foundation for themed components.
Rectangle {
    id: root

    color: "transparent"

    Behavior on color {
        ColorAnimation {
            duration: Config.Theme.duration.normal
            easing.type: Config.Theme.easing.standard
        }
    }

    Behavior on opacity {
        NumberAnimation {
            duration: Config.Theme.duration.normal
            easing.type: Config.Theme.easing.standard
        }
    }
}
