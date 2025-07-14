// fern/shell.qml – MVP bar
import Quickshell
import QtQuick

PanelWindow {
    anchors { top: true; left: true; right: true }
    implicitHeight: 32
    color: "#202020"

    Text {
        anchors.centerIn: parent
        text: "Fern bar – Mhmmmm"
        color: "white"
        font.pixelSize: 14
    }
}

