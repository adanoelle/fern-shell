# QML Components Overview

Fern Shell's UI is built with QML (Qt Modeling Language) using the QuickShell
framework.

## Component Architecture

```
shell.qml                    # Entry point
├── bar/
│   ├── BarWrapper.qml      # Expand/collapse state machine
│   ├── BarInteractions.qml # Mouse hover detection
│   ├── Bar.qml             # Module layout
│   ├── Border.qml          # Curved screen frame
│   └── Backgrounds.qml     # Bar background
├── components/
│   ├── StyledRect.qml      # Base animated rectangle
│   └── ModuleContainer.qml # Pill-shaped container
├── modules/
│   ├── Clock.qml           # Time display
│   └── Workspaces.qml      # Workspace indicators
├── services/
│   └── Hyprland.qml        # Hyprland IPC integration
└── config/
    ├── Theme.qml           # Design token singleton
    └── ConfigLoader.qml    # TOML configuration loader
```

## Key Patterns

### Singleton Services

Theme and Hyprland are singletons accessed via imports:

```qml
import "../config" as Config
import "../services" as Services

// Usage
color: Config.Theme.accent
activeWorkspace: Services.Hyprland.activeWsId
```

### Required Properties

Components declare explicit dependencies:

```qml
Item {
    required property ShellScreen screen
    required property Item bar
}
```

### Reactive Bindings

State flows through property bindings, not imperative updates:

```qml
// This automatically updates when bar.implicitWidth changes
width: bar.implicitWidth
```

## Sections

- **[Bar System](bar.md)** - How the bar expands, collapses, and handles hover
- **[Border & Backgrounds](border.md)** - The mask technique for curved borders
- **[Modules](modules.md)** - Clock and Workspaces implementation
- **[Theme Tokens](theme.md)** - The three-tier token system
