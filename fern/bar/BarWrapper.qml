pragma ComponentBehavior: Bound
import QtQuick
import Quickshell
import "../config" as Config

// Bar wrapper handling expand/collapse animation and visibility state.
// Uses a state machine to animate between collapsed (thin edge) and
// expanded (full bar) states based on hover and persistence settings.
Item {
    id: root

    required property ShellScreen screen
    property bool isHovered: false

    // === CONFIGURATION ===

    readonly property int collapsedWidth: Config.Theme.barCollapsedWidth
    readonly property int expandedWidth: Config.Theme.barWidth
    readonly property bool persistent: Config.Theme.barPersistent

    // === COMPUTED STATE ===

    readonly property bool shouldBeVisible: persistent || isHovered
    readonly property int exclusiveZone: shouldBeVisible ? expandedWidth : collapsedWidth

    onIsHoveredChanged: {
        console.log("Fern: BarWrapper.isHovered =", isHovered, "shouldBeVisible =", shouldBeVisible);
    }

    // Start collapsed
    implicitWidth: collapsedWidth
    implicitHeight: parent?.height ?? 0

    // === STATE MACHINE ===

    states: State {
        name: "visible"
        when: root.shouldBeVisible

        PropertyChanges {
            root.implicitWidth: root.expandedWidth
        }
    }

    transitions: [
        // Expand animation (entering)
        Transition {
            from: ""
            to: "visible"

            NumberAnimation {
                target: root
                property: "implicitWidth"
                duration: Config.Theme.duration.slow
                easing.type: Config.Theme.easing.enter
            }
        },
        // Collapse animation (exiting)
        Transition {
            from: "visible"
            to: ""

            NumberAnimation {
                target: root
                property: "implicitWidth"
                duration: Config.Theme.duration.normal
                easing.type: Config.Theme.easing.exit
            }
        }
    ]

    // === CONTENT ===

    // Bar content - loaded when visible (or about to be)
    Loader {
        id: barContent

        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        width: root.expandedWidth

        // Keep active during transitions
        active: root.shouldBeVisible || root.implicitWidth > root.collapsedWidth

        // Fade content with expansion
        opacity: (root.implicitWidth - root.collapsedWidth) /
                 (root.expandedWidth - root.collapsedWidth)

        sourceComponent: Bar {
            screen: root.screen
        }
    }

}
