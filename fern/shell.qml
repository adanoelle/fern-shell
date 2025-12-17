// fern/shell.qml – Main shell entry point
// Full-screen window with curved border and left bar
import Quickshell
import Quickshell.Wayland
import QtQuick
import "config" as Config
import "bar"

ShellRoot {
    id: root

    // Create shell for each screen
    Variants {
        model: Quickshell.screens

        // Container for both windows per screen
        Scope {
            id: screenScope

            required property ShellScreen modelData

            // Track bar width for exclusive zone
            property int currentBarWidth: barWrapper.implicitWidth

            PanelWindow {
                id: panelWindow

                screen: screenScope.modelData

                // Full-screen anchoring for border effect
                anchors {
                    top: true
                    bottom: true
                    left: true
                    right: true
                }

                // Don't claim exclusive zone - let the exclusion window handle it
                WlrLayershell.exclusionMode: ExclusionMode.Ignore
                WlrLayershell.namespace: "fern-shell"

                // Transparent background - components provide visuals
                color: "transparent"

                // Input region - fixed at expanded bar width for stable hover
                // This prevents the region from shrinking during collapse animation
                Item {
                    id: inputRegion

                    anchors.top: parent.top
                    anchors.bottom: parent.bottom
                    anchors.left: parent.left
                    width: Config.Theme.barWidth  // Always 48px
                }

                // Mask input to the hover region - clicks elsewhere pass through
                mask: Region { item: inputRegion }

                // Layer for shadow effect
                Item {
                    anchors.fill: parent
                    // Could add shadow effect here like Caelestia

                    // Border: creates the curved frame around the screen
                    Border {
                        bar: barWrapper
                    }

                    // Backgrounds: fills the bar area
                    Backgrounds {
                        bar: barWrapper
                    }
                }

                // Hover detection layer (covers full screen for edge detection)
                BarInteractions {
                    anchors.fill: parent
                    bar: barWrapper

                    // The bar wrapper with expand/collapse logic
                    BarWrapper {
                        id: barWrapper

                        anchors.top: parent.top
                        anchors.bottom: parent.bottom
                        anchors.left: parent.left

                        screen: screenScope.modelData
                    }
                }
            }

            // Separate window for exclusive zone management
            PanelWindow {
                id: exclusionWindow

                screen: screenScope.modelData

                // Only anchor left side for exclusive zone
                anchors {
                    left: true
                    top: true
                    bottom: true
                }

                // Claim exclusive zone based on bar's current width
                // This reserves screen space dynamically as bar expands/collapses
                exclusiveZone: screenScope.currentBarWidth

                WlrLayershell.namespace: "fern-exclusion"

                // Invisible - just for reserving space
                color: "transparent"
                implicitWidth: 1
                visible: true
            }
        }
    }

    // React to theme changes
    Connections {
        target: Config.Theme

        function onThemeChanged() {
            console.log("Fern: Theme updated");
        }
    }

    Component.onCompleted: {
        console.log("Fern Shell started - curved border design");
    }
}
