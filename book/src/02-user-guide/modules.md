# Modules

Configure individual bar modules.

## Available Modules

| Module       | Description                   |
| ------------ | ----------------------------- |
| `workspaces` | Hyprland workspace indicators |
| `clock`      | Time and date display         |
| `spacer`     | Flexible space (no config)    |

## Workspaces

Shows workspace indicators as vertical pills.

```toml
[modules.workspaces]
count = 10          # Number of workspaces to show
show_empty = true   # Show empty workspace indicators
```

**Visual States:**

- **Active**: Tall pill with accent color
- **Occupied**: Small dot with dim color
- **Empty**: Small dot with overlay color (if `show_empty = true`)

## Clock

Displays time vertically.

```toml
[modules.clock]
format_time = "%H\n%M"    # Time format (newline for vertical)
format_date = "%a\n%d"    # Date format (optional)
```

**Format Codes:**

- `%H` - Hour (24h)
- `%M` - Minute
- `%a` - Weekday abbreviation
- `%d` - Day of month

## Module Order

Modules appear in the order listed:

```toml
[bar]
modules = ["workspaces", "spacer", "clock"]
```

- First module: Top of bar
- Last module: Bottom of bar
- `spacer`: Pushes modules apart
