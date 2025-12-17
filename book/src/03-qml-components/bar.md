# Bar System

The bar system handles expansion, collapse, and hover detection.

## Component Hierarchy

```
shell.qml
└── Scope (per screen)
    ├── panelWindow (visual layer)
    │   ├── inputRegion (fixed 48px hover zone)
    │   ├── Border
    │   ├── Backgrounds
    │   └── BarInteractions (MouseArea)
    │       └── BarWrapper (state machine)
    │           └── Bar (module layout)
    └── exclusionWindow (space reservation)
```

## Dual-Window Architecture

Fern Shell uses **two windows per screen**:

| Window            | Purpose                                       |
| ----------------- | --------------------------------------------- |
| `panelWindow`     | Full-screen visual layer (border, bar, hover) |
| `exclusionWindow` | 1px invisible window for exclusive zone       |

**Why separate?**

- Visual effects need full-screen coverage
- Exclusive zone is rectangular (just the bar area)
- Prevents compositor bugs with complex geometry

## BarWrapper State Machine

`BarWrapper.qml` uses QML's declarative state system:

```qml
states: State {
    name: "visible"
    when: root.shouldBeVisible
    PropertyChanges {
        root.implicitWidth: root.expandedWidth
    }
}

transitions: [
    Transition {
        from: ""
        to: "visible"
        NumberAnimation {
            property: "implicitWidth"
            duration: 300
        }
    }
]
```

**States:**

- `""` (default): Collapsed, `implicitWidth = 4px`
- `"visible"`: Expanded, `implicitWidth = 48px`

## Hover Detection

`BarInteractions.qml` is a full-screen MouseArea with fixed thresholds:

```qml
MouseArea {
    // Fixed 48px + 15px tolerance (never animated)
    if (containsMouse && mouseX < Config.Theme.barWidth + 15) {
        bar.isHovered = true;
    }
}
```

### Anti-Bounce Mechanism

**Problem**: If the hover zone shrank during collapse animation, the mouse could
exit mid-animation, causing flicker.

**Solution**: Fixed input region that never changes:

```qml
// shell.qml
Item {
    id: inputRegion
    width: Config.Theme.barWidth  // Always 48px
}
mask: Region { item: inputRegion }
```

### Edge Detection

When the mouse is at the screen edge (x < 5px), don't collapse:

```qml
onContainsMouseChanged: {
    if (!containsMouse) {
        if (lastMouseX > 5) {
            collapseTimer.start();
        }
        // If at edge, don't collapse
    }
}
```

## Exclusive Zone Flow

```
bar.implicitWidth changes (4 → 48)
  → screenScope.currentBarWidth updates
    → exclusionWindow.exclusiveZone updates
      → Compositor pushes windows right
```

This happens automatically through QML property bindings.

## Files

- `/fern/shell.qml` - Entry point, dual-window setup
- `/fern/bar/BarWrapper.qml` - State machine
- `/fern/bar/BarInteractions.qml` - Hover detection
- `/fern/bar/Bar.qml` - Module layout
