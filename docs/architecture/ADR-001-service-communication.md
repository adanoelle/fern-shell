# ADR-001: Service Communication Architecture

**Status:** Accepted
**Date:** 2024-12-23
**Deciders:** Ada

## Context

Fern Shell needs Rust services for:
1. **fern-cider**: Apple Music integration via Cider client
2. **fern-viz**: Real-time audio visualization (60fps)

The existing pattern (JSON state files + CLI commands) has limitations:
- Command latency (~200ms) is unacceptable for play/pause
- State file updates (~50ms) are too slow for 60fps visualization

We evaluated five architectural options ranging from simple (state files) to complex (CXX-Qt in-process FFI).

## Decision

We will use a **hybrid architecture**:

### fern-cider: Unix Socket (Bidirectional)

```
┌─────────────────────────────────────────────────────────┐
│                  QuickShell Process                     │
│  ┌───────────────────────────────────────────────────┐  │
│  │  Cider.qml                                        │  │
│  │                                                   │  │
│  │  Socket {                                         │  │
│  │    path: $XDG_RUNTIME_DIR/fern/cider.sock        │  │
│  │    onMessageReceived: state = JSON.parse(msg)    │  │
│  │  }                                                │  │
│  │                                                   │  │
│  │  function play() { socket.write('{"cmd":"play"}') │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                         ↕ Unix Socket
┌─────────────────────────────────────────────────────────┐
│                   fern-cider daemon                     │
│                                                         │
│  • Socket.io client → Cider (port 10767)               │
│  • Unix socket server → QML clients                    │
│  • Bidirectional: push state, receive commands         │
└─────────────────────────────────────────────────────────┘
                         ↕ Socket.io
┌─────────────────────────────────────────────────────────┐
│                      Cider App                          │
└─────────────────────────────────────────────────────────┘
```

### fern-viz: CXX-Qt Plugin (In-Process)

```
┌─────────────────────────────────────────────────────────┐
│                  QuickShell Process                     │
│  ┌───────────────────────────────────────────────────┐  │
│  │  QML Layer                                        │  │
│  │    import Fern.Viz 1.0                            │  │
│  │    VizService { bars: 64 }                        │  │
│  └───────────────────────────────────────────────────┘  │
│                         ↕ CXX-Qt FFI                    │
│  ┌───────────────────────────────────────────────────┐  │
│  │  Rust (in-process)                                │  │
│  │    • PipeWire/cpal audio capture                  │  │
│  │    • spectrum-analyzer FFT                        │  │
│  │    • 60fps property updates                       │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Rationale

### Why Unix Socket for fern-cider?

| Consideration | Decision Driver |
|---------------|-----------------|
| Command latency | ~1ms vs ~200ms (CLI) or ~30ms (HTTP) |
| State updates | Push-based, no polling |
| Complexity | Medium - simpler than CXX-Qt |
| Debugging | `socat` can tap the socket |
| Crash isolation | Daemon survives shell restart |
| External tools | Can still write state file for fernctl |

Alternatives rejected:
- **State file + CLI**: Too slow for commands
- **State file + HTTP**: Acceptable but socket is cleaner
- **MPRIS only**: Doesn't provide Cider-specific features (rating, library status)
- **CXX-Qt**: Overkill for ~10 updates/second

### Why CXX-Qt for fern-viz?

| Consideration | Decision Driver |
|---------------|-----------------|
| Update rate | 60fps requires in-process updates |
| Latency | <1ms property updates |
| Audio capture | Needs direct PipeWire access |

Alternatives rejected:
- **State file**: Max ~20fps, too much I/O overhead
- **Unix socket**: Would work but adds complexity for QML consumption
- **Separate process + shared memory**: More complex than CXX-Qt

### Why Not CXX-Qt for Everything?

1. **Complexity**: CXX-Qt requires CMake + Cargo coordination
2. **Crash coupling**: Rust panic in CXX-Qt crashes the shell
3. **Build time**: Slower iteration during development
4. **Overkill**: fern-cider doesn't need sub-millisecond updates

## Consequences

### Positive

- Play/pause feels instant (~1ms command latency)
- Visualizer runs at native 60fps
- Clean separation: network daemon vs in-process audio
- Can develop fern-cider without CXX-Qt complexity
- fern-viz is isolated - can ship fern-cider first

### Negative

- Two different IPC patterns to maintain
- CXX-Qt adds build complexity for fern-viz
- Need to learn CXX-Qt API (still maturing, 0.7.x)
- Debugging fern-viz requires Qt tools

### Neutral

- Both patterns are valid for other future services
- Socket pattern could be used for fern-notify later
- CXX-Qt pattern could be used for other high-frequency services

## Implementation Plan

### Phase 1: fern-cider (Socket-based)

1. Create crate structure with tokio async runtime
2. Implement Cider Socket.io client
3. Add Unix socket server for QML
4. Create QML service using `Quickshell.Io.Socket`
5. Wire up to existing Media widget

### Phase 2: fern-viz (CXX-Qt)

1. Set up CXX-Qt build infrastructure
2. Implement audio capture (cpal initially, PipeWire later)
3. Implement FFT processor with spectrum-analyzer
4. Expose `VizService` QObject to QML
5. Create visualizer widget

### Phase 3: Integration

1. Connect visualizer to media widget
2. Add beat detection for animations
3. Polish and optimize

## Technical Specifications

### fern-cider Socket Protocol

**Socket path:** `$XDG_RUNTIME_DIR/fern/cider.sock`

**Message format:** Newline-delimited JSON

**State message (daemon → QML):**
```json
{
  "connected": true,
  "playing": true,
  "track": {
    "name": "Voyager",
    "artist": "Daft Punk",
    "album": "Discovery",
    "artwork_url": "https://...",
    "duration_secs": 247,
    "position_secs": 83
  },
  "shuffle": false,
  "repeat": 0,
  "rating": 1,
  "in_library": true,
  "in_favorites": false
}
```

**Command messages (QML → daemon):**
```json
{"cmd": "play"}
{"cmd": "pause"}
{"cmd": "toggle"}
{"cmd": "next"}
{"cmd": "previous"}
{"cmd": "seek", "position_ms": 50000}
{"cmd": "setRating", "value": 1}
{"cmd": "toggleShuffle"}
{"cmd": "toggleRepeat"}
```

### fern-viz QML Interface

```qml
VizService {
    // Configuration
    property int bars: 64

    // State (read-only)
    readonly property var values: [0.0, 0.1, ...]  // QVector<f64>
    readonly property bool active: false
    readonly property real bpm: 120.0  // Future: beat detection

    // Methods
    function start()
    function stop()
}
```

## References

- [Context Document](./services-context.md)
- [QuickShell.Io Documentation](https://quickshell.org/docs/types/Quickshell.Io/)
- [CXX-Qt Book](https://kdab.github.io/cxx-qt/book/)
- [Cider RPC Documentation](https://cider.sh/docs/client/rpc)
- [Caelestia Shell](https://github.com/caelestia-dots/shell) - Reference implementation
