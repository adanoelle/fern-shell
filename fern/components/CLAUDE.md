# Fern Component Library Guide

## Purpose

The component library provides reusable, themed UI elements that ensure visual
consistency across Fern Shell. All components follow the same patterns for
theming, animations, and accessibility.

## Base Components

### FernRect - Base Container

```qml
// fern/components/FernRect.qml
import QtQuick
import "../config"

Rectangle {
    id: root

    // Theme integration
    color: Appearance.palette.surface
    radius: Appearance.radius.normal

    // Consistent animations
    Behavior on color {
        ColorAnimation {
            duration: Appearance.animation.normal
            easing.type: Easing.OutCubic
        }
    }

    Behavior on opacity {
        NumberAnimation {
            duration: Appearance.animation.fast
        }
    }

    // Elevation shadow (optional)
    property int elevation: 0
    layer.enabled: elevation > 0
    layer.effect: ElevationEffect {
        elevation: root.elevation
    }
}
```

### FernText - Typography

```qml
// fern/components/FernText.qml
import QtQuick
import "../config"

Text {
    id: root

    // Typography presets
    enum Style {
        Display,
        Headline,
        Title,
        Body,
        Label,
        Caption
    }

    property int style: FernText.Style.Body

    // Apply typography based on style
    font {
        family: Appearance.font.family
        pixelSize: {
            switch(style) {
                case FernText.Style.Display: return 32
                case FernText.Style.Headline: return 24
                case FernText.Style.Title: return 20
                case FernText.Style.Body: return 14
                case FernText.Style.Label: return 12
                case FernText.Style.Caption: return 10
            }
        }
        weight: style <= FernText.Style.Title ? Font.Medium : Font.Normal
    }

    // Theme integration
    color: Appearance.palette.foreground

    // Accessibility
    Accessible.role: Accessible.StaticText
    Accessible.name: text
}
```

### FernIcon - Icon System

```qml
// fern/components/FernIcon.qml
import QtQuick
import "../config"

Item {
    id: root

    required property string icon
    property int size: 24
    property color color: Appearance.palette.foreground
    property bool spinning: false

    implicitWidth: size
    implicitHeight: size

    Text {
        anchors.centerIn: parent
        text: IconProvider.get(root.icon)
        font.family: Appearance.font.iconFamily
        font.pixelSize: root.size
        color: root.color

        RotationAnimator on rotation {
            running: root.spinning
            from: 0
            to: 360
            duration: 1000
            loops: Animation.Infinite
        }
    }

    // Icon caching
    Component.onCompleted: {
        IconCache.preload(root.icon);
    }
}
```

## Container Components

### FernContainer - Layout Helper

```qml
// fern/components/FernContainer.qml
import QtQuick
import QtQuick.Layouts
import "../config"

RowLayout {
    id: root

    property int padding: Appearance.spacing.normal
    property bool vertical: false

    spacing: Appearance.spacing.small

    // Switch between row and column
    Component.onCompleted: {
        if (vertical) {
            // Transform to ColumnLayout
            Qt.createQmlObject(`
                import QtQuick.Layouts
                ColumnLayout {}
            `, parent);
        }
    }
}
```

### FernCard - Material Card

```qml
// fern/components/FernCard.qml
import QtQuick
import "../config"

FernRect {
    id: root

    property alias content: contentLoader.sourceComponent
    property int padding: Appearance.spacing.normal

    elevation: 2
    radius: Appearance.radius.medium

    Loader {
        id: contentLoader
        anchors {
            fill: parent
            margins: root.padding
        }
    }

    // Hover effect
    MouseArea {
        anchors.fill: parent
        hoverEnabled: true
        onEntered: root.elevation = 4
        onExited: root.elevation = 2
    }
}
```

## Interactive Components

### FernButton - Action Button

```qml
// fern/components/FernButton.qml
import QtQuick
import "../config"

FernRect {
    id: root

    property alias text: label.text
    property alias icon: iconItem.icon
    property bool primary: false

    signal clicked()

    implicitWidth: content.implicitWidth + Appearance.spacing.large * 2
    implicitHeight: 32

    color: {
        if (mouseArea.pressed) return Appearance.palette.accent
        if (mouseArea.containsMouse) return Qt.lighter(Appearance.palette.accent, 1.1)
        if (primary) return Appearance.palette.accent
        return Appearance.palette.surface
    }

    radius: Appearance.radius.small

    Row {
        id: content
        anchors.centerIn: parent
        spacing: Appearance.spacing.small

        FernIcon {
            id: iconItem
            size: 16
            visible: icon !== ""
        }

        FernText {
            id: label
            style: FernText.Style.Label
            color: primary ? Appearance.palette.background : Appearance.palette.foreground
        }
    }

    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true
        onClicked: root.clicked()
    }

    // Ripple effect
    RippleEffect {
        anchors.fill: parent
        trigger: mouseArea.pressed
    }
}
```

### FernSlider - Value Control

```qml
// fern/components/FernSlider.qml
import QtQuick
import QtQuick.Controls
import "../config"

Slider {
    id: root

    property color trackColor: Appearance.palette.surfaceVariant
    property color fillColor: Appearance.palette.accent

    implicitHeight: 24

    background: Rectangle {
        x: root.leftPadding
        y: root.topPadding + root.availableHeight / 2 - height / 2
        width: root.availableWidth
        height: 4
        radius: 2
        color: trackColor

        Rectangle {
            width: root.visualPosition * parent.width
            height: parent.height
            color: fillColor
            radius: 2
        }
    }

    handle: Rectangle {
        x: root.leftPadding + root.visualPosition * (root.availableWidth - width)
        y: root.topPadding + root.availableHeight / 2 - height / 2
        width: 20
        height: 20
        radius: 10
        color: root.pressed ? fillColor : Qt.lighter(fillColor, 1.2)

        Behavior on width {
            NumberAnimation { duration: 100 }
        }

        states: State {
            when: root.pressed
            PropertyChanges { target: handle; width: 24; height: 24 }
        }
    }
}
```

## Feedback Components

### FernTooltip - Information Display

```qml
// fern/components/FernTooltip.qml
import QtQuick
import "../config"

FernRect {
    id: root

    property alias text: label.text
    property Item target: null
    property int delay: 500

    visible: false
    color: Appearance.palette.surfaceVariant
    radius: Appearance.radius.small
    elevation: 8

    FernText {
        id: label
        style: FernText.Style.Caption
        padding: Appearance.spacing.small
    }

    Timer {
        id: showTimer
        interval: root.delay
        onTriggered: root.show()
    }

    Connections {
        target: root.target

        function onHoveredChanged() {
            if (target.hovered) {
                showTimer.start();
            } else {
                showTimer.stop();
                root.hide();
            }
        }
    }

    function show() {
        // Position above target
        x = target.x + (target.width - width) / 2;
        y = target.y - height - 4;
        visible = true;
    }

    function hide() {
        visible = false;
    }
}
```

### FernBadge - Status Indicator

```qml
// fern/components/FernBadge.qml
import QtQuick
import "../config"

Rectangle {
    id: root

    property alias text: label.text
    property int count: 0
    property color badgeColor: Appearance.palette.error

    visible: count > 0 || text !== ""

    implicitWidth: Math.max(16, label.implicitWidth + 4)
    implicitHeight: 16

    radius: height / 2
    color: badgeColor

    FernText {
        id: label
        anchors.centerIn: parent
        text: root.count > 99 ? "99+" : root.count > 0 ? root.count.toString() : root.text
        style: FernText.Style.Caption
        color: "white"
        font.bold: true
    }

    // Pulse animation for new items
    SequentialAnimation {
        id: pulseAnimation
        NumberAnimation { target: root; property: "scale"; to: 1.2; duration: 100 }
        NumberAnimation { target: root; property: "scale"; to: 1.0; duration: 100 }
    }

    onCountChanged: {
        if (count > 0) pulseAnimation.start();
    }
}
```

## Animation Components

### RippleEffect - Material Ripple

```qml
// fern/components/RippleEffect.qml
import QtQuick

Item {
    id: root

    property bool trigger: false
    property color color: Qt.rgba(1, 1, 1, 0.3)

    Rectangle {
        id: ripple
        anchors.centerIn: parent
        width: 0
        height: width
        radius: width / 2
        color: root.color
        opacity: 0

        ParallelAnimation {
            id: rippleAnimation

            NumberAnimation {
                target: ripple
                property: "width"
                to: Math.max(root.width, root.height) * 2
                duration: 400
                easing.type: Easing.OutCubic
            }

            NumberAnimation {
                target: ripple
                property: "opacity"
                from: 1
                to: 0
                duration: 400
            }
        }
    }

    onTriggerChanged: {
        if (trigger) {
            ripple.width = 0;
            rippleAnimation.start();
        }
    }
}
```

### LoadingSpinner - Progress Indicator

```qml
// fern/components/LoadingSpinner.qml
import QtQuick
import "../config"

Item {
    id: root

    property int size: 32
    property color color: Appearance.palette.accent
    property bool running: true

    implicitWidth: size
    implicitHeight: size

    Canvas {
        anchors.fill: parent

        onPaint: {
            var ctx = getContext("2d");
            ctx.clearRect(0, 0, width, height);
            ctx.strokeStyle = root.color;
            ctx.lineWidth = 3;
            ctx.lineCap = "round";
            ctx.beginPath();
            ctx.arc(width/2, height/2, width/2 - 3, 0, Math.PI * 1.5);
            ctx.stroke();
        }

        RotationAnimator on rotation {
            running: root.running
            from: 0
            to: 360
            duration: 1000
            loops: Animation.Infinite
        }
    }
}
```

## Theme Integration

### Using Theme Colors

```qml
FernRect {
    // Semantic colors
    color: Appearance.palette.error     // Error state
    color: Appearance.palette.warning   // Warning state
    color: Appearance.palette.success   // Success state
    color: Appearance.palette.info      // Information

    // Surface colors
    color: Appearance.palette.background
    color: Appearance.palette.surface
    color: Appearance.palette.surfaceVariant

    // Text colors
    color: Appearance.palette.foreground
    color: Appearance.palette.foregroundDim
}
```

### Responsive Sizing

```qml
FernText {
    // Responsive text based on screen
    font.pixelSize: {
        if (Screen.width < 1920) return 12
        if (Screen.width < 2560) return 14
        return 16
    }
}
```

## Component Guidelines

### DO ✅

- Always use Appearance for colors and spacing
- Implement keyboard navigation
- Add accessibility properties
- Cache heavy resources
- Use property aliases for customization
- Provide sensible defaults

### DON'T ❌

- Don't hardcode colors or sizes
- Don't ignore theme changes
- Don't create unnecessary bindings
- Don't forget error states
- Don't skip animations

## Testing Components

```bash
# Test component in isolation
qs -p fern/components/FernButton.qml

# Test with different themes
FERN_THEME=light qs -p fern/components/FernButton.qml

# Visual regression testing
qs-screenshot fern/components/FernButton.qml
```

## Creating New Components

### Component Checklist

- [ ] Extends appropriate base component
- [ ] Uses theme colors and spacing
- [ ] Has proper animations
- [ ] Includes accessibility properties
- [ ] Handles all states (hover, pressed, disabled)
- [ ] Documented with examples
- [ ] Tested in isolation

### Component Template

```qml
// fern/components/FernNewComponent.qml
import QtQuick
import "../config"

FernRect {
    id: root

    // Public API
    property string title: ""
    property bool active: false

    signal activated()

    // Size hints
    implicitWidth: 100
    implicitHeight: 40

    // Theme integration
    color: active ? Appearance.palette.accent : Appearance.palette.surface

    // Content
    FernText {
        anchors.centerIn: parent
        text: root.title
        style: FernText.Style.Body
    }

    // Interaction
    MouseArea {
        anchors.fill: parent
        onClicked: {
            root.active = !root.active;
            root.activated();
        }
    }

    // States
    states: [
        State {
            name: "active"
            when: root.active
            PropertyChanges {
                target: root
                elevation: 4
            }
        }
    ]

    // Transitions
    transitions: Transition {
        NumberAnimation {
            properties: "elevation"
            duration: Appearance.animation.normal
        }
    }
}
```

## Caelestia Component Reference

Study these Caelestia components:

| Component | File                                                | Key Features         |
| --------- | --------------------------------------------------- | -------------------- |
| Base      | `/caelestia/components/StyledRect.qml`              | Animation behaviors  |
| Text      | `/caelestia/components/StyledText.qml`              | Typography system    |
| Container | `/caelestia/components/containers/StyledWindow.qml` | Window styling       |
| Controls  | `/caelestia/components/controls/`                   | Interactive elements |

## Next Steps

1. Start with base components (FernRect, FernText, FernIcon)
2. Build container components
3. Add interactive components
4. Create feedback components
5. Implement animations
6. Test with different themes
7. Document usage examples

Remember: Components are the building blocks of your UI - make them flexible,
themeable, and delightful to use.
