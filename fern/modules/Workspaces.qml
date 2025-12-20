pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts
import Quickshell
import "../config" as Config
import "../components"
import "../services" as Services

// Workspace indicator module for the bar.
// Shows workspace pills with app icons:
//   - Active workspace: tall pill with accent color + app icon
//   - Occupied workspace: small pill with app icon
//   - Empty workspace: hidden (not shown)
ModuleContainer {
    id: root

    required property ShellScreen screen

    // Configuration
    readonly property int wsCount: Config.Theme.moduleConfig("workspaces").count ?? 10

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

            // Each workspace indicator
            Item {
                id: wsIndicator

                required property int index
                readonly property int wsId: index + 1
                readonly property bool isActive: wsId === root.activeWsId
                readonly property var recentWindow: Services.Hyprland.mostRecentWindowForWorkspace(wsId)
                readonly property bool isOccupied: recentWindow !== null

                // Only show workspaces with windows (or active)
                visible: isOccupied || isActive

                Layout.alignment: Qt.AlignHCenter
                Layout.preferredWidth: pill.width
                Layout.preferredHeight: pill.height

                // Background pill
                Rectangle {
                    id: pill

                    anchors.centerIn: parent
                    width: wsIndicator.isActive ? 32 : 24
                    height: wsIndicator.isActive ? 32 : 24
                    radius: Config.Theme.radius.md

                    color: wsIndicator.isActive
                        ? Config.Theme.accent
                        : Config.Theme.surface

                    Behavior on width {
                        NumberAnimation {
                            duration: Config.Theme.duration.normal
                            easing.type: Config.Theme.easing.standard
                        }
                    }
                    Behavior on height {
                        NumberAnimation {
                            duration: Config.Theme.duration.normal
                            easing.type: Config.Theme.easing.standard
                        }
                    }
                    Behavior on color {
                        ColorAnimation {
                            duration: Config.Theme.duration.fast
                        }
                    }
                }

                // App icon (if occupied)
                Image {
                    id: appIcon

                    visible: wsIndicator.isOccupied
                    anchors.centerIn: parent
                    width: wsIndicator.isActive ? 20 : 16
                    height: width
                    sourceSize.width: width
                    sourceSize.height: height

                    source: {
                        if (!wsIndicator.recentWindow) return "";
                        const appClass = wsIndicator.recentWindow.lastIpcObject?.class ?? "";
                        const entry = DesktopEntries.heuristicLookup(appClass);
                        const iconName = entry?.icon ?? "application-x-executable";
                        return Quickshell.iconPath(iconName, "application-x-executable");
                    }

                    Behavior on width {
                        NumberAnimation {
                            duration: Config.Theme.duration.normal
                            easing.type: Config.Theme.easing.standard
                        }
                    }
                }

                // Workspace number (shown when no icon or as fallback)
                Text {
                    visible: !wsIndicator.isOccupied && wsIndicator.isActive
                    anchors.centerIn: parent
                    text: wsIndicator.wsId.toString()
                    font.family: Config.Theme.fontFamily
                    font.pixelSize: Config.Theme.fontSize.sm
                    font.weight: Font.Bold
                    color: Config.Theme.background
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
