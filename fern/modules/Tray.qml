pragma ComponentBehavior: Bound
import QtQuick
import Quickshell.Services.SystemTray
import "../config" as Config
import "../components"
import "../drawers"

// System tray module for the bar.
// Displays tray icons vertically, clicking opens menu popout.
ModuleContainer {
    id: root

    // Expose items for popout positioning
    readonly property alias items: repeater

    // Size to fit content
    implicitWidth: Config.Theme.barWidth - Config.Theme.spacing.sm * 2
    implicitHeight: layout.implicitHeight + padding * 2

    Column {
        id: layout

        anchors.centerIn: parent
        spacing: Config.Theme.spacing.sm

        Repeater {
            id: repeater

            model: SystemTray.items

            // Individual tray item
            Item {
                id: trayItem

                required property SystemTrayItem modelData
                required property int index

                width: iconSize
                height: iconSize

                readonly property int iconSize: Config.Theme.fontSize.lg + 4

                // Icon display
                Image {
                    id: icon

                    anchors.fill: parent
                    source: {
                        let iconPath = trayItem.modelData.icon;
                        // Handle icon paths with query strings
                        if (iconPath.includes("?path=")) {
                            const [name, path] = iconPath.split("?path=");
                            iconPath = `file://${path}/${name.slice(name.lastIndexOf("/") + 1)}`;
                        }
                        return iconPath;
                    }
                    sourceSize: Qt.size(trayItem.iconSize, trayItem.iconSize)
                    fillMode: Image.PreserveAspectFit
                }

                // Click handling
                MouseArea {
                    anchors.fill: parent
                    acceptedButtons: Qt.LeftButton | Qt.RightButton

                    onClicked: event => {
                        if (event.button === Qt.LeftButton) {
                            // Left click - activate or show menu
                            trayItem.modelData.activate();
                        } else {
                            // Right click - secondary action or show menu popout
                            const globalPos = trayItem.mapToGlobal(0, trayItem.height / 2);
                            DrawerController.toggleDrawer(`tray-${trayItem.index}`, globalPos.y);
                        }
                    }
                }

                // Hover effect
                Rectangle {
                    anchors.fill: parent
                    anchors.margins: -2
                    radius: Config.Theme.radius.sm
                    color: parent.containsMouse ? Config.Theme.surfaceHover : "transparent"
                    z: -1

                    property bool containsMouse: false
                }
            }
        }
    }
}
