# Quick Tour

This 5-minute walkthrough introduces the core concepts of Fern Shell.

## The Bar

Fern Shell provides a vertical bar on the left edge of your screen:

- **Collapsed**: A thin 4px accent line
- **Expanded**: Full 48px bar with modules (hover to expand)

The bar automatically hides when not in use and expands when you move your mouse
to the left edge.

## Modules

The bar contains modules that display information:

| Module         | Description                         |
| -------------- | ----------------------------------- |
| **Workspaces** | Shows Hyprland workspace indicators |
| **Clock**      | Displays current time vertically    |
| **Spacer**     | Flexible space between modules      |

Modules are arranged top-to-bottom in the order specified in your config.

## Configuration

Fern Shell uses TOML for configuration. The config file is typically at
`~/.config/fern/config.toml`:

```toml
[appearance]
theme = "dark"
accent = "#89b4fa"

[bar]
position = "left"
width = 48
modules = ["workspaces", "spacer", "clock"]
```

## Live Reload

One of Fern Shell's key features is live reload:

1. Edit any QML file in `fern/`
2. Save the file
3. Changes appear immediately (no restart needed)

This makes styling and development much faster than traditional approaches.

## The Border

Fern Shell also draws a subtle curved border around your screen, creating a
polished visual effect that integrates with the bar.

## Next Steps

- **[Configuration](../02-user-guide/README.md)** - Customize colors, fonts, and
  behavior
- **[QML Components](../03-qml-components/README.md)** - Understand how the bar
  works
- **[Developer Guide](../04-developer-guide/README.md)** - Add new modules
