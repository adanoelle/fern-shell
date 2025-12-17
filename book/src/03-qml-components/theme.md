# Theme Tokens

`Theme.qml` is a singleton that provides design tokens to all components.

## Three-Tier Architecture

```
┌─────────────────────────────────────┐
│  Tier 1: Primitives                 │
│  _darkPalette, _lightPalette        │
│  (internal, not configurable)       │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│  Tier 2: Semantic Tokens            │
│  background, surface, accent, etc.  │
│  (configurable via config.toml)     │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│  Tier 3: Component Tokens           │
│  barBackground, moduleRadius, etc.  │
│  (internal mappings)                │
└─────────────────────────────────────┘
```

## Tier 1: Primitives

Built-in color palettes (Catppuccin):

```qml
readonly property var _darkPalette: ({
    base: "#1e1e2e",
    surface: "#313244",
    text: "#cdd6f4",
    blue: "#89b4fa",
    // ...
})

readonly property var _lightPalette: ({
    base: "#eff1f5",
    surface: "#e6e9ef",
    text: "#4c4f69",
    blue: "#1e66f5",
    // ...
})
```

## Tier 2: Semantic Tokens

Configurable through `config.toml`:

```qml
// Colors
readonly property color background: _palette.base
readonly property color surface: _palette.surface
readonly property color accent: config.appearance?.accent ?? _palette.blue

// Typography
readonly property string fontFamily: config.appearance?.font_family ?? "Inter"

// Spacing (fixed scale)
readonly property var spacing: ({
    xs: 2, sm: 4, md: 8, lg: 12, xl: 16, xxl: 24
})

// Animation
readonly property var duration: ({
    instant: 0, fast: 100, normal: 200, slow: 300
})
```

## Tier 3: Component Tokens

Internal mappings used by components:

```qml
// Bar
readonly property color barBackground: background
readonly property int barWidth: config.bar?.width ?? 48

// Module
readonly property color moduleBackground: surface
readonly property int moduleRadius: radius.md

// Button
readonly property color buttonBackgroundHover: surfaceHover
```

## Using Tokens

Import the config module and access via `Config.Theme`:

```qml
import "../config" as Config

Rectangle {
    color: Config.Theme.barBackground
    radius: Config.Theme.moduleRadius

    Text {
        color: Config.Theme.foreground
        font.family: Config.Theme.fontFamily
        font.pixelSize: Config.Theme.fontSize.md
    }
}
```

## Why Three Tiers?

| Tier | Who Controls      | Purpose                        |
| ---- | ----------------- | ------------------------------ |
| 1    | Fern developers   | Consistent base colors         |
| 2    | Users             | Customization without breaking |
| 3    | Component authors | Semantic meaning for UI        |

**Benefits:**

- Users can't break layouts by changing primitives
- Components use meaningful names, not raw colors
- Theme switching is automatic (just change `_palette`)

## File

- `/fern/config/Theme.qml`
