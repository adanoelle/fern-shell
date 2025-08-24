# Fern Modules Development Guide

## Purpose

This directory contains the bar modules that provide functionality to Fern
Shell. Each module is a self-contained QML component that can be dynamically
loaded based on user configuration.

## Quick Start: New Module Template

```qml
// fern/modules/MyModule.qml
import QtQuick
import "../components"
import "../services"

FernModule {
    id: root

    // Required for all modules
    required property var config
    required property ShellScreen screen

    // Module-specific properties
    property string displayText: ""
    property real updateInterval: config.interval || 1000

    // Size hints for bar layout
    implicitWidth: content.implicitWidth + padding * 2
    implicitHeight: parent.height

    // Module content
    FernContainer {
        id: content
        anchors.centerIn: parent

        FernIcon {
            icon: "schedule"
            size: 16
        }

        FernText {
            text: root.displayText
        }
    }

    // Update logic
    Timer {
        interval: root.updateInterval
        running: true
        repeat: true
        onTriggered: root.update()
    }

    // Module methods
    function update() {
        // Fetch new data
        displayText = MyService.getData();
    }

    Component.onCompleted: {
        update();
    }
}
```

## Module Types & Patterns

### 1. Status Module (Static Display)

**Examples**: Clock, CPU usage, Memory **Pattern**: Timer-based updates,
read-only display

```qml
// Clock module pattern
Timer {
    interval: 1000
    running: visible
    repeat: true
    onTriggered: {
        timeText = Qt.formatTime(new Date(), config.format);
    }
}
```

### 2. Interactive Module

**Examples**: Workspaces, Volume, Brightness **Pattern**: User interaction
changes system state

```qml
// Workspace module pattern
MouseArea {
    anchors.fill: parent
    onClicked: HyprlandService.switchWorkspace(index)
    onWheel: (wheel) => {
        if (wheel.angleDelta.y > 0)
            HyprlandService.previousWorkspace();
        else
            HyprlandService.nextWorkspace();
    }
}
```

### 3. Notification Module

**Examples**: System tray, Network status, Battery **Pattern**: React to system
events, show alerts

```qml
// Battery module pattern
Connections {
    target: PowerService

    function onBatteryLevelChanged(level) {
        if (level < 10 && !lowBatteryWarned) {
            NotificationService.show("Low Battery", "Critical");
            lowBatteryWarned = true;
        }
    }
}
```

### 4. Dynamic List Module

**Examples**: Window list, App launcher, Media players **Pattern**: Variable
number of items

```qml
// Window list pattern
Repeater {
    model: HyprlandService.windows

    delegate: WindowItem {
        window: modelData
        onClicked: window.focus()
    }
}
```

## Configuration Integration

### Module Config Schema

```javascript
// Expected in user's config.json
{
    "modules": {
        "clock": {
            "enabled": true,
            "format": "hh:mm",
            "showDate": false,
            "interval": 1000
        }
    }
}
```

### Accessing Config

```qml
// In your module
property var moduleConfig: Config.modules.clock || {}
property string format: moduleConfig.format || "hh:mm:ss"
property bool showDate: moduleConfig.showDate ?? false
```

## Service Integration

### Using Service Singletons

```qml
import "../services"

// Read system state
property int workspace: HyprlandService.activeWorkspace
property real volume: AudioService.volume
property bool muted: AudioService.muted

// Call service methods
onClicked: AudioService.toggleMute()
```

### Creating Module-Specific Services

```qml
// For complex logic, create a service
QtObject {
    id: moduleService

    property var cache: ({})

    function fetchData() {
        // Complex logic here
        return processedData;
    }
}
```

## Performance Guidelines

### DO ✅

```qml
// Use lazy loading
Loader {
    active: root.visible && config.enabled
    sourceComponent: HeavyComponent {}
}

// Batch property updates
function updateAll() {
    // Single binding update
    displayData = {
        text: computedText,
        icon: computedIcon,
        color: computedColor
    };
}

// Use appropriate timers
Timer {
    interval: 60000  // 1 minute for weather
    interval: 1000   // 1 second for clock
    interval: 100    // 100ms for animations
}
```

### DON'T ❌

```qml
// Avoid frequent bindings
text: ExpensiveFunction()  // Called on every update

// Don't create timers for each item
Repeater {
    delegate: Item {
        Timer { ... }  // Creates N timers!
    }
}

// Don't poll when events exist
Timer {  // Bad for workspace changes
    onTriggered: checkIfWorkspaceChanged()
}
```

## Testing Your Module

### Manual Testing

```bash
# Test module in isolation
qs -p fern/modules/MyModule.qml

# Test with mock data
QS_MOCK=1 qs -p fern/modules/MyModule.qml

# Test in full bar
qs -p fern/shell.qml
```

### Module Checklist

- [ ] Handles all config options
- [ ] Gracefully handles missing services
- [ ] Updates only when visible
- [ ] Cleans up timers/connections
- [ ] Responds to theme changes
- [ ] Works on different screen sizes
- [ ] Memory usage is stable

## Common Patterns

### Popout Pattern

```qml
// Module triggers popout
onClicked: {
    PopoutService.show("audio", {
        anchor: root,
        content: AudioPopout {}
    });
}
```

### State Machine Pattern

```qml
// For complex state
states: [
    State {
        name: "connected"
        PropertyChanges { target: icon; color: "green" }
    },
    State {
        name: "disconnected"
        PropertyChanges { target: icon; color: "red" }
    }
]
```

### Animation Pattern

```qml
// Smooth transitions
Behavior on width {
    NumberAnimation {
        duration: 200
        easing.type: Easing.OutCubic
    }
}
```

## Module Registry

When creating a new module:

1. Add to `fern/modules/` directory
2. Register in module loader:

```qml
// fern/ModuleLoader.qml
property var availableModules: {
    "clock": Qt.createComponent("modules/Clock.qml"),
    "workspaces": Qt.createComponent("modules/Workspaces.qml"),
    "mymodule": Qt.createComponent("modules/MyModule.qml")  // Add here
}
```

3. Add default config:

```javascript
// config/defaults.json
{
    "modules": {
        "mymodule": {
            "enabled": false,
            "position": 5
        }
    }
}
```

## Debugging

### Console Output

```qml
Component.onCompleted: {
    console.log("MyModule loaded with config:", JSON.stringify(config));
}

function update() {
    console.time("MyModule.update");
    // ... update logic
    console.timeEnd("MyModule.update");
}
```

### Visual Debugging

```qml
// Show bounds
Rectangle {
    anchors.fill: parent
    color: "transparent"
    border.color: "red"
    border.width: 1
    visible: Debug.showBounds
}
```

## Anti-Patterns to Avoid

### ❌ Direct DOM Manipulation

```qml
// Bad: Bypasses Qt's rendering
Component.onCompleted: {
    document.getElementById("something")...
}
```

### ❌ Synchronous Network Calls

```qml
// Bad: Blocks UI
property string data: Http.getSync(url)
```

### ❌ Global State Mutation

```qml
// Bad: Side effects
onClicked: {
    GlobalConfig.theme = "dark"  // Use signals instead
}
```

## Caelestia Reference

Study these Caelestia modules for patterns:

| Module Type  | Caelestia File                                                | Key Pattern                  |
| ------------ | ------------------------------------------------------------- | ---------------------------- |
| Workspaces   | `/caelestia/modules/bar/components/workspaces/Workspaces.qml` | Scroll handling, per-monitor |
| Clock        | `/caelestia/modules/bar/components/Clock.qml`                 | Format switching             |
| System Tray  | `/caelestia/modules/bar/components/Tray.qml`                  | Icon management              |
| Status Icons | `/caelestia/modules/bar/components/StatusIcons.qml`           | Modular indicators           |

## Next Steps

1. Choose a module type from patterns above
2. Copy the Quick Start template
3. Implement your logic
4. Test with `qs -p`
5. Add to module registry
6. Document configuration options

Remember: Start simple, test often, optimize later.
