pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts
import Quickshell
import "../config" as Config
import "../components"
import "../modules"

// Vertical bar content with module layout.
// Modules are arranged in a ColumnLayout with configurable order.
StyledRect {
    id: root

    required property ShellScreen screen

    // Vertical padding for top/bottom modules
    readonly property int vPadding: Config.Theme.spacing.lg

    color: Config.Theme.barBackground

    ColumnLayout {
        id: layout

        anchors.fill: parent
        anchors.margins: Config.Theme.spacing.sm
        spacing: Config.Theme.moduleSpacing

        // Dynamic module loading based on config
        Repeater {
            id: moduleRepeater

            // Default module list, can be overridden by config
            model: Config.Theme.config.bar?.modules ?? ["workspaces", "spacer", "obs", "clock", "power"]

            Loader {
                id: moduleLoader

                required property string modelData
                required property int index

                Layout.alignment: Qt.AlignHCenter
                Layout.fillHeight: modelData === "spacer"

                // Add padding to first and last modules
                Layout.topMargin: index === 0 ? root.vPadding : 0
                Layout.bottomMargin: index === moduleRepeater.count - 1 ? root.vPadding : 0

                visible: active
                active: true

                sourceComponent: {
                    switch (modelData) {
                        case "workspaces":
                            return workspacesComponent;
                        case "clock":
                            return clockComponent;
                        case "obs":
                            return obsComponent;
                        case "power":
                            return powerComponent;
                        case "tray":
                            return trayComponent;
                        case "spacer":
                            return spacerComponent;
                        default:
                            console.warn("Unknown module:", modelData);
                            return null;
                    }
                }
            }
        }
    }

    // === MODULE COMPONENTS ===

    Component {
        id: workspacesComponent

        Workspaces {
            screen: root.screen
        }
    }

    Component {
        id: clockComponent

        Clock {}
    }

    Component {
        id: obsComponent

        Obs {}
    }

    Component {
        id: powerComponent

        Power {}
    }

    Component {
        id: trayComponent

        Tray {}
    }

    Component {
        id: spacerComponent

        Item {
            Layout.fillHeight: true
        }
    }
}
