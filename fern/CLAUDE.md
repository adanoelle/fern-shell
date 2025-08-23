# Fern QML Development Guide

## Overview

This directory contains the QML-based user interface for Fern Shell. All UI
components are written in QML using the QuickShell framework, which provides
Wayland/Hyprland integration and system access.

## QuickShell Fundamentals

### Key Components

- **PanelWindow**: Base window type for panels/bars
- **LazyLoader**: Performance-optimized component loading
- **Process**: Run and monitor system processes
- **Variants**: Multiple panel configurations

### System Integration APIs

```qml
// D-Bus access
import Quickshell.DBus

// Process management
import Quickshell.Io

// Hyprland IPC
import Quickshell.Hyprland

// System services
import Quickshell.Services
```

## Current Structure

```
fern/
├── shell.qml          # Main panel definition
└── modules/           # Individual panel modules (to be created)
    ├── Clock.qml
    ├── Workspaces.qml
    ├── SystemTray.qml
    ├── Audio.qml
    ├── Network.qml
    └── Battery.qml
```

## Module Development Template

### Basic Module Structure

```qml
// fern/modules/ModuleName.qml
import Quickshell
import QtQuick
import QtQuick.Layouts

Item {
    id: root

    // Configuration properties
    property color backgroundColor: "transparent"
    property color textColor: "white"
    property int updateInterval: 1000

    // Module-specific properties
    property var moduleData: ({})

    // Size hints for panel layout
    implicitWidth: contentLayout.implicitWidth
    implicitHeight: parent.height

    // Visual container
    Rectangle {
        anchors.fill: parent
        color: mouseArea.containsMouse ? Qt.lighter(backgroundColor, 1.2) : backgroundColor
        radius: 4

        // Content layout
        RowLayout {
            id: contentLayout
            anchors.centerIn: parent
            spacing: 4

            // Icon
            Text {
                font.family: "Material Symbols Rounded"
                font.pixelSize: 16
                color: textColor
            }

            // Value display
            Text {
                id: valueText
                font.family: "JetBrainsMono Nerd Font"
                font.pixelSize: 14
                color: textColor
            }
        }

        // Interaction
        MouseArea {
            id: mouseArea
            anchors.fill: parent
            hoverEnabled: true
            onClicked: root.handleClick()
        }
    }

    // Update timer
    Timer {
        interval: root.updateInterval
        running: true
        repeat: true
        onTriggered: root.updateData()
    }

    // Module logic
    function updateData() {
        // Fetch and update module data
    }

    function handleClick() {
        // Handle user interaction
    }

    Component.onCompleted: {
        updateData()
    }
}
```

## Specific Module Patterns

### Workspaces Module (Hyprland Integration)

```qml
import Quickshell.Hyprland

HyprlandWorkspaces {
    model: Hyprland.workspaces

    delegate: Rectangle {
        property bool active: Hyprland.activeWorkspace.id === modelData.id
        property bool occupied: modelData.windows > 0

        width: 24; height: 24
        radius: 4
        color: active ? "#3daee9" : (occupied ? "#4d4d4d" : "transparent")
        border.width: occupied ? 0 : 1
        border.color: "#4d4d4d"

        MouseArea {
            anchors.fill: parent
            onClicked: Hyprland.dispatch("workspace", modelData.id)
        }
    }
}
```

### Clock Module

```qml
import QtQuick

Item {
    property string timeFormat: "hh:mm"
    property string dateFormat: "ddd, MMM d"

    Timer {
        interval: 1000
        running: true
        repeat: true
        onTriggered: {
            timeLabel.text = Qt.formatTime(new Date(), timeFormat)
            dateLabel.text = Qt.formatDate(new Date(), dateFormat)
        }
    }

    // Click to toggle between time/date display
    MouseArea {
        anchors.fill: parent
        onClicked: showDate = !showDate
    }
}
```

### System Tray (StatusNotifierItem)

```qml
import Quickshell.Services.SystemTray

SystemTray {
    // Automatically manages tray items
    onItemAdded: {
        // Create icon for new tray item
    }

    onItemRemoved: {
        // Remove icon
    }

    delegate: Image {
        source: modelData.icon
        width: 16; height: 16

        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.LeftButton | Qt.RightButton
            onClicked: (mouse) => {
                if (mouse.button === Qt.LeftButton)
                    modelData.activate()
                else
                    modelData.openMenu()
            }
        }
    }
}
```

### Audio Module (PipeWire via D-Bus)

```qml
import Quickshell.DBus

DBusInterface {
    service: "org.freedesktop.PipeWire"
    path: "/org/freedesktop/PipeWire"
    interface: "org.freedesktop.PipeWire.Device"

    property real volume: 0
    property bool muted: false

    function setVolume(value) {
        call("SetVolume", value)
    }

    function toggleMute() {
        call("SetMute", !muted)
    }
}
```

## Performance Considerations

### Use LazyLoader for Heavy Components

```qml
LazyLoader {
    id: trayLoader
    active: config.enableSystemTray

    source: Component {
        SystemTrayModule {
            // Heavy component only loaded when needed
        }
    }
}
```

### Batch Updates

```qml
// Bad: Multiple property changes trigger multiple repaints
text: value1
color: value2
visible: value3

// Good: Single binding updates all at once
property var state: ({
    text: value1,
    color: value2,
    visible: value3
})
```

### Minimize Timers

```qml
// Instead of multiple timers, use one central timer
Timer {
    id: updateTimer
    interval: 1000
    running: true
    repeat: true
    onTriggered: {
        clockModule.update()
        statsModule.update()
        // etc.
    }
}
```

## Styling Guidelines

### Color Palette

```qml
// Define in shell.qml or theme file
QtObject {
    id: theme

    // Base colors
    property color background: "#202020"
    property color foreground: "#ffffff"
    property color accent: "#3daee9"

    // State colors
    property color hover: Qt.lighter(background, 1.2)
    property color active: accent
    property color inactive: "#4d4d4d"

    // Semantic colors
    property color error: "#da4453"
    property color warning: "#f67400"
    property color success: "#27ae60"
}
```

### Consistent Spacing

```qml
// Use consistent spacing values
property int smallSpacing: 4
property int mediumSpacing: 8
property int largeSpacing: 12
```

### Icon Guidelines

- Use Material Symbols Rounded for consistency
- Standard size: 16px for panel, 20px for larger displays
- Ensure icons align with text baseline

## Testing Modules

### Isolated Module Testing

```bash
# Test individual module
qs -p fern/modules/Clock.qml

# Test with mock data
QS_TEST_MODE=1 qs -p fern/modules/Workspaces.qml
```

### Integration Testing

```bash
# Test full panel
qs -p fern/shell.qml

# Test with different configurations
QS_CONFIG=minimal qs -p fern/shell.qml
```

## Common Issues & Solutions

### Module Not Updating

- Check Timer is running and has correct interval
- Verify signal connections are established
- Ensure Component.onCompleted initializes data

### Layout Issues

- Use implicitWidth/Height for size hints
- Avoid fixed sizes unless necessary
- Test with different panel heights

### Performance Problems

- Profile with QML Profiler
- Use LazyLoader for complex components
- Batch property updates
- Minimize JavaScript in bindings

## D-Bus Integration Examples

### Network Manager

```qml
DBusInterface {
    service: "org.freedesktop.NetworkManager"
    path: "/org/freedesktop/NetworkManager"
    interface: "org.freedesktop.NetworkManager"

    property string primaryConnection: ""
    property int connectivity: 0

    onConnectivityChanged: {
        // Update network status display
    }
}
```

### UPower (Battery)

```qml
DBusInterface {
    service: "org.freedesktop.UPower"
    path: "/org/freedesktop/UPower/devices/battery_BAT0"
    interface: "org.freedesktop.UPower.Device"

    property real percentage: 0
    property int state: 0  // 1=charging, 2=discharging
    property int timeToEmpty: 0
    property int timeToFull: 0
}
```

## Future Enhancements

### Configuration System

```qml
// Planned: User configuration via JSON/YAML
ConfigLoader {
    source: "~/.config/fern/config.json"

    onLoaded: {
        // Apply user preferences to modules
    }
}
```

### Module Registry

```qml
// Dynamic module loading based on config
Repeater {
    model: config.modules

    delegate: Loader {
        source: "modules/" + modelData.type + ".qml"
        property var moduleConfig: modelData.config
    }
}
```

### Animation System

```qml
// Smooth transitions for state changes
Behavior on opacity {
    NumberAnimation { duration: 200 }
}

Behavior on width {
    NumberAnimation { duration: 150; easing.type: Easing.OutCubic }
}
```

## Development Commands

```bash
# Watch for QML changes (auto-reload)
qs -p fern/shell.qml

# Check for QML errors
qmllint fern/**/*.qml

# Format QML (if qmlformat available)
qmlformat -i fern/**/*.qml

# Profile performance
QSG_VISUALIZE=overdraw qs -p fern/shell.qml
```

## Resources

- [QuickShell Docs](https://quickshell.outfoxxed.me/docs/)
- [QML Types Reference](https://doc.qt.io/qt-6/qmltypes.html)
- [Material Symbols](https://fonts.google.com/icons)
- [Hyprland IPC Protocol](https://wiki.hyprland.org/IPC/)
