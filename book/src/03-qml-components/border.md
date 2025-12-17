# Border & Backgrounds

The border system creates a curved frame around the screen using an inverted
mask technique.

## The Challenge

Wayland layer-shell windows are rectangular. To create curved corners, we need a
creative approach.

## Inverted Mask Technique

`Border.qml` draws a full-screen rectangle and **cuts out** the interior:

```
Step 1: Full-screen colored rectangle
┌───────────────────────────────────┐
│█████████████████████████████████  │
│█████████████████████████████████  │
│█████████████████████████████████  │
└───────────────────────────────────┘

Step 2: Create mask shape (invisible)
┌───────────────────────────────────┐
│    ╔═════════════════════════╗    │
│    ║                         ║    │
│    ║      (mask interior)    ║    │
│    ╚═════════════════════════╝    │
└───────────────────────────────────┘

Step 3: Apply inverted mask
┌───────────────────────────────────┐
│████╔═════════════════════════╗    │
│████║                         ║    │
│████║   (transparent cutout)  ║    │
│████╚═════════════════════════╝    │
└───────────────────────────────────┘
```

## Implementation

```qml
// Border.qml
StyledRect {
    anchors.fill: parent
    color: Config.Theme.barBackground

    layer.enabled: true
    layer.effect: MultiEffect {
        maskSource: mask
        maskEnabled: true
        maskInverted: true    // Cut OUT the mask shape
    }
}

Item {
    id: mask
    layer.enabled: true
    visible: false

    Rectangle {
        anchors.fill: parent
        anchors.margins: Config.Theme.borderThickness  // 4px
        anchors.leftMargin: root.bar.implicitWidth     // Dynamic!
        radius: Config.Theme.borderRounding            // 24px
    }
}
```

## Dynamic Left Margin

The mask's `leftMargin` binds to `bar.implicitWidth`:

- **Collapsed**: leftMargin = 4px (thin border visible)
- **Expanded**: leftMargin = 48px (border "hugs" the bar)

This creates seamless integration between bar and border.

## Backgrounds

`Backgrounds.qml` is simpler - it fills the bar area:

```qml
StyledRect {
    width: root.bar.implicitWidth
    color: Config.Theme.barBackground
}
```

## Why GPU Layers?

```qml
layer.enabled: true
```

This renders the item to a GPU texture, enabling:

- Mask effects via shaders
- Smooth animations
- Hardware-accelerated compositing

## Files

- `/fern/bar/Border.qml` - Mask technique implementation
- `/fern/bar/Backgrounds.qml` - Bar background fill
- `/fern/components/StyledRect.qml` - Base rectangle with behaviors
