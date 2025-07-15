// fern/shell.qml – MVP bar
import Quickshell
import QtQuick
import QtQuick.Layouts
import Quickshell.Io

PanelWindow {
    anchors { top: true; left: true; right: true }
    implicitHeight: 40
    color: "#202020"

    Process {
        id: ghosttyProc
        command: ["ghostty"]
    }


    RowLayout {
        anchors.verticalCenter: parent.verticalCenter
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: 12

        Rectangle {
            width: 28; height: 28; radius: 4
            color: hovered? "#404040" : "transparent"

            property bool hovered: false
            MouseArea {
                anchors.fill: parent
                hoverEnabled: true
                onEntered: parent.hovered = true
                onExited:  parent.hovered = false
                onClicked: ghosttyProc.running = true;
            }

            Text {
                anchors.centerIn: parent
                text: ""
                font.family: "JetBrainsMono Nerd Font"
                color: "white"
                font.pixelSize: 16
            }
        }
    }

    function launchTerminal() { ghosttyProc.running = true; }
}

