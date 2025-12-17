# Adding Modules

Step-by-step guide to creating a new bar module.

## Module Structure

Each module is a QML file in `/fern/modules/`:

```qml
// fern/modules/MyModule.qml
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts
import "../config" as Config
import "../components"

ModuleContainer {
    id: root

    // Required for screen-specific data
    required property ShellScreen screen

    // Size to fit content
    implicitWidth: Config.Theme.barWidth - Config.Theme.spacing.sm * 2
    implicitHeight: content.implicitHeight + padding * 2

    // Your content here
    Item {
        id: content
        anchors.centerIn: parent
        // ...
    }
}
```

## Step 1: Create the File

Create `fern/modules/MyModule.qml` with the template above.

## Step 2: Register in qmldir

Add to `/fern/modules/qmldir`:

```
MyModule 1.0 MyModule.qml
```

## Step 3: Add to Bar.qml

Edit `/fern/bar/Bar.qml` to add your module:

```qml
// Add component
Component {
    id: myModuleComponent
    MyModule {
        screen: root.screen
    }
}

// Add to switch statement
sourceComponent: {
    switch (modelData) {
        case "workspaces":
            return workspacesComponent;
        case "clock":
            return clockComponent;
        case "mymodule":              // Add this
            return myModuleComponent;
        case "spacer":
            return spacerComponent;
        default:
            return null;
    }
}
```

## Step 4: Configure

Add to config:

```toml
[bar]
modules = ["workspaces", "mymodule", "spacer", "clock"]

[modules.mymodule]
# Your module's options
```

## Example: Battery Module

```qml
pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts
import "../config" as Config
import "../components"

ModuleContainer {
    id: root

    required property ShellScreen screen

    // Battery percentage (mock data for now)
    property int percentage: 75

    implicitWidth: Config.Theme.barWidth - Config.Theme.spacing.sm * 2
    implicitHeight: layout.implicitHeight + padding * 2

    ColumnLayout {
        id: layout
        anchors.centerIn: parent
        spacing: Config.Theme.spacing.xs

        // Battery icon
        Text {
            Layout.alignment: Qt.AlignHCenter
            text: root.percentage > 20 ? "󰁹" : "󰁺"
            font.family: Config.Theme.fontIcon
            font.pixelSize: Config.Theme.iconSize
            color: root.percentage > 20
                ? Config.Theme.foreground
                : Config.Theme.error
        }

        // Percentage
        Text {
            Layout.alignment: Qt.AlignHCenter
            text: root.percentage + "%"
            font.family: Config.Theme.fontMono
            font.pixelSize: Config.Theme.fontSize.sm
            color: Config.Theme.foregroundDim
        }
    }
}
```

## Best Practices

1. **Use `required property`** for explicit dependencies
2. **Use `readonly property`** for computed values
3. **Bind to Theme tokens** - never hardcode colors/sizes
4. **Use Behaviors** for smooth animations
5. **Use Loader** for expensive content that can be unloaded

## Testing

Run with live reload:

```bash
qs -p fern/shell.qml
```

Edit your module file - changes appear immediately.
