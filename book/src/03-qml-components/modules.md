# Module Components

Clock and Workspaces module implementations.

## Module Container

All modules use `ModuleContainer.qml` as a base - a pill-shaped container with
consistent styling:

```qml
// ModuleContainer.qml
Rectangle {
    color: Config.Theme.moduleBackground
    radius: Config.Theme.moduleRadius
    // ...
}
```

## Clock Module

`Clock.qml` displays time vertically with accent-colored text.

```qml
ModuleContainer {
    property color textColor: Config.Theme.accent

    ColumnLayout {
        Text {
            text: Qt.formatTime(new Date(), "HH")
            color: textColor
            font.family: Config.Theme.fontMono
        }
        Text {
            text: Qt.formatTime(new Date(), "mm")
            color: textColor
        }
    }

    Timer {
        interval: 1000
        running: true
        repeat: true
        onTriggered: // update time
    }
}
```

**Key Points:**

- Uses `Timer` to update every second
- Vertical layout with `ColumnLayout`
- Monospace font for alignment

## Workspaces Module

`Workspaces.qml` shows Hyprland workspace indicators.

```qml
ModuleContainer {
    required property ShellScreen screen

    readonly property int activeWsId:
        Services.Hyprland.activeWsIdForMonitor(screen)

    ColumnLayout {
        Repeater {
            model: wsCount

            Rectangle {
                readonly property bool isActive: wsId === activeWsId
                readonly property bool isOccupied:
                    Services.Hyprland.workspaceHasWindows(wsId)

                width: 8
                height: isActive ? 24 : 8
                radius: Config.Theme.radius.full

                color: isActive ? Config.Theme.accent
                     : isOccupied ? Config.Theme.foregroundDim
                     : Config.Theme.overlay
            }
        }
    }
}
```

**Visual States:**

| State    | Size | Color          |
| -------- | ---- | -------------- |
| Active   | 8x24 | Accent         |
| Occupied | 8x8  | Dim foreground |
| Empty    | 8x8  | Overlay        |

**Hyprland Integration:**

- `activeWsIdForMonitor(screen)` - Gets active workspace per monitor
- `workspaceHasWindows(wsId)` - Checks if workspace has windows

## Adding New Modules

See [Adding Modules](../04-developer-guide/adding-modules.md) for a step-by-step
guide.

## Files

- `/fern/modules/Clock.qml`
- `/fern/modules/Workspaces.qml`
- `/fern/components/ModuleContainer.qml`
- `/fern/services/Hyprland.qml`
