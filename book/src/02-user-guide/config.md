# Config File Format

Complete reference for `config.toml`.

## File Location

The configuration file is located at `~/.config/fern/config.toml`.

## Schema

### `[appearance]`

| Key           | Type                              | Default                      | Description                 |
| ------------- | --------------------------------- | ---------------------------- | --------------------------- |
| `theme`       | `"dark"` \| `"light"` \| `"auto"` | `"dark"`                     | Color scheme                |
| `accent`      | hex color                         | `"#89b4fa"`                  | Accent color for highlights |
| `font_family` | string                            | `"Inter"`                    | Primary font                |
| `font_mono`   | string                            | `"JetBrainsMono Nerd Font"`  | Monospace font              |
| `font_icon`   | string                            | `"Material Symbols Rounded"` | Icon font                   |

### `[bar]`

| Key               | Type     | Default                             | Description                        |
| ----------------- | -------- | ----------------------------------- | ---------------------------------- |
| `position`        | `"left"` | `"left"`                            | Bar position (only left supported) |
| `width`           | integer  | `48`                                | Expanded bar width in pixels       |
| `collapsed_width` | integer  | `4`                                 | Collapsed indicator width          |
| `persistent`      | boolean  | `false`                             | Keep bar always visible            |
| `modules`         | array    | `["workspaces", "spacer", "clock"]` | Module order                       |

### `[border]`

| Key         | Type    | Default | Description                |
| ----------- | ------- | ------- | -------------------------- |
| `thickness` | integer | `4`     | Border thickness in pixels |
| `rounding`  | integer | `24`    | Corner radius in pixels    |

### `[modules.*]`

Module-specific configuration. See [Modules](modules.md) for details.

## Example

```toml
[appearance]
theme = "dark"
accent = "#cba6f7"  # Catppuccin Mauve

[bar]
width = 52
persistent = false
modules = ["workspaces", "spacer", "clock"]

[border]
thickness = 4
rounding = 20

[modules.workspaces]
count = 8
show_empty = false

[modules.clock]
format_time = "%H\n%M"
```
