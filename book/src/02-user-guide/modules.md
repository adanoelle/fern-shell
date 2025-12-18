# Modules

Configure individual bar modules.

## Available Modules

| Module       | Description                        |
| ------------ | ---------------------------------- |
| `workspaces` | Hyprland workspace indicators      |
| `clock`      | Time and date display              |
| `obs`        | OBS Studio recording/streaming     |
| `spacer`     | Flexible space (no config)         |

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

## OBS

Controls OBS Studio recording and streaming via WebSocket.

**Requirements:**

- OBS Studio with obs-websocket plugin (included in OBS 28+)
- `fern-obs daemon` running in the background

**Features:**

- Recording status indicator with pulsing animation
- Live timecode display
- Left-click to toggle recording
- Right-click to toggle pause

**Visual States:**

- **Recording**: Pulsing red indicator with timecode
- **Paused**: Orange indicator (no pulse)
- **Streaming**: Green indicator with timecode
- **Ready**: Blue indicator showing current scene
- **Disconnected**: Dimmed icon

**Starting the Daemon:**

```bash
# Start with default settings (localhost:4455)
fern-obs daemon

# Custom OBS connection
fern-obs daemon --host 192.168.1.100 --port 4455 --password secret
```

See [fern-obs CLI Reference](../05-api-reference/fern-obs.md) for all commands.

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
