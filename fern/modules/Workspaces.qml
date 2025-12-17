pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts
import Quickshell
import "../config" as Config
import "../components"
import "../services" as Services

// Workspace indicator module for the bar.
// Shows vertical pills representing workspaces:
//   - Active workspace: tall pill with accent color
//   - Occupied workspace: small dot with dim color
//   - Empty workspace: small dot with overlay color
ModuleContainer {
    id: root

    required property ShellScreen screen

    // Configuration
    readonly property int wsCount: Config.Theme.moduleConfig("workspaces").count ?? 10
    readonly property bool showEmpty: Config.Theme.moduleConfig("workspaces").show_empty ?? true

    // State
    readonly property int activeWsId: Services.Hyprland.activeWsIdForMonitor(screen)

    // Size to fit content
    implicitWidth: Config.Theme.barWidth - Config.Theme.spacing.sm * 2
    implicitHeight: layout.implicitHeight + padding * 2

    ColumnLayout {
        id: layout

        anchors.centerIn: parent
        spacing: Config.Theme.spacing.sm

        Repeater {
            model: root.wsCount

            Rectangle {
                id: wsIndicator

                required property int index
                readonly property int wsId: index + 1
                readonly property bool isActive: wsId === root.activeWsId
                readonly property bool isOccupied: Services.Hyprland.workspaceHasWindows(wsId)

                // Hide empty workspaces if configured
                visible: root.showEmpty || isActive || isOccupied

                Layout.alignment: Qt.AlignHCenter

                // Pill dimensions
                width: 8
                height: isActive ? 24 : 8
                radius: Config.Theme.radius.full

                // Color based on state
                color: isActive
                    ? Config.Theme.accent
                    : isOccupied
                        ? Config.Theme.foregroundDim
                        : Config.Theme.overlay

                // Smooth height animation
                Behavior on height {
                    NumberAnimation {
                        duration: Config.Theme.duration.normal
                        easing.type: Config.Theme.easing.standard
                    }
                }

                // Smooth color animation
                Behavior on color {
                    ColorAnimation {
                        duration: Config.Theme.duration.fast
                    }
                }

                // Click to switch workspace
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor

                    onClicked: {
                        Services.Hyprland.dispatch(`workspace ${wsIndicator.wsId}`);
                    }
                }
            }
        }
    }
}
