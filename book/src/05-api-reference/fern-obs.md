# fern-obs CLI

OBS WebSocket bridge daemon for Fern Shell.

## Overview

`fern-obs` maintains a persistent connection to OBS Studio via the obs-websocket
protocol and exposes state through a JSON file that QML components can watch.

## Installation

```bash
# Via Nix flake
nix run github:adanoelle/fern-shell#fern-obs

# Or add to your system packages
environment.systemPackages = [ inputs.fern-shell.packages.${system}.fern-obs ];
```

## Commands

### daemon

Start the OBS bridge daemon.

```bash
fern-obs daemon [OPTIONS]
```

**Options:**

| Option                 | Default     | Description                           |
| ---------------------- | ----------- | ------------------------------------- |
| `--host`               | `localhost` | OBS WebSocket host                    |
| `--port`               | `4455`      | OBS WebSocket port                    |
| `--password`           | (none)      | OBS WebSocket password                |
| `--stats-interval`     | `1000`      | Stats update interval (ms)            |
| `--reconnect-interval` | `5000`      | Reconnection delay (ms)               |
| `--max-reconnects`     | `0`         | Max reconnect attempts (0 = infinite) |
| `--no-stats`           | false       | Disable stats collection              |

**Examples:**

```bash
# Default connection
fern-obs daemon

# Remote OBS instance with password
fern-obs daemon --host 192.168.1.100 --password secret

# Lower stats frequency for reduced CPU
fern-obs daemon --stats-interval 5000
```

### start-recording

Start OBS recording.

```bash
fern-obs start-recording
# Alias: fern-obs rec
```

### stop-recording

Stop OBS recording.

```bash
fern-obs stop-recording
# Alias: fern-obs stop-rec
```

### toggle-pause

Toggle recording pause state.

```bash
fern-obs toggle-pause
# Alias: fern-obs pause
```

### start-streaming

Start OBS streaming.

```bash
fern-obs start-streaming
# Alias: fern-obs stream
```

### stop-streaming

Stop OBS streaming.

```bash
fern-obs stop-streaming
# Alias: fern-obs stop-stream
```

### scene

Switch to a specific scene.

```bash
fern-obs scene <NAME>
```

**Examples:**

```bash
fern-obs scene "Gaming"
fern-obs scene "Desktop"
```

### status

Get current OBS status.

```bash
fern-obs status [--json]
```

**Options:**

| Option   | Description              |
| -------- | ------------------------ |
| `--json` | Output as JSON           |

**Example Output:**

```
Connected: true
Scene: Gaming
Recording: active
  Duration: 01:23:45
Streaming: inactive
Stats:
  CPU: 2.5%
  FPS: 60.0
  Render drops: 0.01%
Scenes: Desktop, Gaming, BRB
```

## Environment Variables

| Variable       | Description                        |
| -------------- | ---------------------------------- |
| `OBS_HOST`     | Default OBS host                   |
| `OBS_PORT`     | Default OBS port                   |
| `OBS_PASSWORD` | Default OBS password               |

## State File

The daemon writes state to:

```
~/.local/state/fern/obs-state.json
```

**Schema:**

```json
{
  "connected": true,
  "recording": {
    "active": true,
    "paused": false,
    "elapsed_secs": 3600,
    "timecode": "01:00:00"
  },
  "streaming": {
    "active": false,
    "elapsed_secs": 0,
    "reconnecting": false
  },
  "current_scene": "Gaming",
  "scenes": ["Desktop", "Gaming", "BRB"],
  "stats": {
    "cpu_usage": 2.5,
    "memory_mb": 512.0,
    "active_fps": 60.0,
    "render_drop_percent": 0.01,
    "output_drop_percent": 0.0
  },
  "updated_at_secs": 1703001234
}
```

## QML Integration

The `Obs` service singleton reads the state file:

```qml
import "../services" as Services

// Check connection
if (Services.Obs.connected) {
    console.log("Recording:", Services.Obs.isRecording);
    console.log("Timecode:", Services.Obs.recordingTimecode);
}

// Control recording
Services.Obs.startRecording();
Services.Obs.togglePause();
Services.Obs.stopRecording();

// Switch scenes
Services.Obs.setScene("Gaming");
```

## Systemd Integration

Create a user service for auto-start:

```ini
# ~/.config/systemd/user/fern-obs.service
[Unit]
Description=Fern OBS Bridge
After=graphical-session.target

[Service]
ExecStart=/path/to/fern-obs daemon
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
```

Enable with:

```bash
systemctl --user enable --now fern-obs
```
