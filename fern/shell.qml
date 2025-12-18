// fern/shell.qml – Main shell entry point
// Full-screen window with curved border, left bar, and drawer panels
import Quickshell
import Quickshell.Wayland
import QtQuick
import "config" as Config
import "services" as Services
import "bar"
import "drawers"

ShellRoot {
    id: root

    // Create shell for each screen
    Variants {
        model: Quickshell.screens

        // Container for both windows per screen
        Scope {
            id: screenScope

            required property ShellScreen modelData

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
                // Enable keyboard focus when drawer is open (for Escape key)
                WlrLayershell.keyboardFocus: DrawerController.hasDrawer ? WlrKeyboardFocus.OnDemand : WlrKeyboardFocus.None

                // Transparent background - components provide visuals
                color: "transparent"

                // Input region - bar area + active drawer areas
                // When drawer is open, expands to full width to capture all input
                Item {
                    id: inputRegion

                    anchors.top: parent.top
                    anchors.bottom: parent.bottom
                    anchors.left: parent.left
                    // When any drawer is active, expand to full width for input capture
                    width: DrawerController.hasDrawer
                        ? parent.width  // Full width when drawer open
                        : Config.Theme.barWidth
                }

                // Mask input to the active region
                mask: Region { item: inputRegion }

                // Layer for shadow effect
                Item {
                    anchors.fill: parent

                    // Border: creates the curved frame around the screen
                    Border {
                        bar: barWrapper
                    }

                    // Backgrounds: fills the bar area
                    Backgrounds {
                        bar: barWrapper
                    }
                }

                // Bar - fixed width, always visible
                BarWrapper {
                    id: barWrapper

                    anchors.top: parent.top
                    anchors.bottom: parent.bottom
                    anchors.left: parent.left

                    screen: screenScope.modelData
                }

                // Drawers - popouts, notifications, etc.
                Drawers {
                    id: drawers

                    bar: barWrapper
                    screen: screenScope.modelData
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

                // Exclusive zone accounts for bar width minus outer gap
                // This makes the gap between bar and windows equal to gaps between windows
                exclusiveZone: Config.Theme.barWidth - Config.Theme.gapsOut

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
            applyHyprlandConfig();
        }
    }

    // Apply Hyprland configuration on startup
    function applyHyprlandConfig(): void {
        Services.Hyprland.configureGaps(Config.Theme.gapsIn, Config.Theme.gapsOut);
        Services.Hyprland.configureRounding(Config.Theme.borderRounding);
    }

    Component.onCompleted: {
        console.log("Fern Shell started - fixed bar with drawer system");
        applyHyprlandConfig();
    }
}
