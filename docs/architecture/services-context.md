# Fern Shell Services: Architecture Context

This document captures the exploration and context for building Rust services for the Fern shell, specifically focusing on media integration (Cider/Apple Music) and audio visualization.

## Background

Fern Shell is a QML-based desktop shell for Hyprland, using QuickShell as its runtime. The existing architecture uses Rust daemons that communicate with the QML frontend via JSON state files watched by `FileView`.

### Current Pattern (fern-obs example)

```
Rust Daemon → JSON state file → FileView → QML properties
           ← CLI commands ← Quickshell.execDetached ← QML
```

This works well for infrequent updates but has latency concerns for:
- High-frequency data (60fps audio visualization)
- Time-sensitive commands (play/pause should feel instant)

## Services Under Consideration

### 1. fern-cider: Apple Music Integration

Cider is an Electron-based Apple Music client with:
- **Socket.io API** on `localhost:10767` for real-time events
- **REST API** for commands (`/api/v1/playback/*`)
- **MPRIS support** (standard Linux media player interface)

#### Cider API Capabilities

| Feature | Socket.io Event | REST Endpoint |
|---------|-----------------|---------------|
| Track changed | `playbackStatus.nowPlayingItemDidChange` | `GET /now-playing` |
| Play state | `playbackStatus.playbackStateDidChange` | - |
| Position | `playbackStatus.playbackTimeDidChange` | - |
| Play/Pause | - | `POST /play`, `POST /pause` |
| Skip | - | `POST /next`, `POST /previous` |
| Rating | - | `POST /setRating/{-1,0,1}` |
| Shuffle/Repeat | - | `POST /toggleShuffle`, `POST /toggleRepeat` |

Cider-specific data not available via MPRIS:
- `inLibrary`, `inFavorites` status
- Rating (-1/0/1)
- Full queue visibility

### 2. fern-viz: Audio Visualization

Real-time audio spectrum analysis requiring:
- Audio capture (PipeWire or cpal)
- FFT processing (spectrum-analyzer crate)
- 60fps updates to QML

## Architecture Options Evaluated

### Option A: State File (Current Pattern)

```
Daemon → ~/.local/state/fern/X-state.json → FileView → QML
      ← spawn CLI process ← execDetached ← QML
```

| Aspect | Assessment |
|--------|------------|
| Command latency | ~200ms (process spawn + execution) |
| State latency | ~50ms (file write + inotify) |
| Complexity | Low |
| Debugging | Easy (cat/jq the state file) |
| Crash isolation | Good (daemon survives shell crash) |

**Verdict:** Fine for infrequent state updates, too slow for commands.

### Option B: State File + Direct HTTP Commands

```
Daemon → state file → FileView → QML
      ← HTTP POST ← XMLHttpRequest ← QML
```

| Aspect | Assessment |
|--------|------------|
| Command latency | ~30ms (HTTP to Cider directly) |
| State latency | ~50ms |
| Complexity | Low |

**Verdict:** Good compromise. Commands bypass the daemon entirely.

### Option C: MPRIS + State File for Extras

```
QuickShell MPRIS → D-Bus → Cider (commands + basic state)
Daemon → state file → FileView → QML (Cider-specific extras)
```

| Aspect | Assessment |
|--------|------------|
| Command latency | ~5ms (D-Bus) |
| State latency | ~50ms for extras, instant for MPRIS |
| Complexity | Low |

**Verdict:** Elegant for basic features. QuickShell has built-in MPRIS.

### Option D: Unix Socket (Bidirectional)

```
Daemon ←→ Unix socket ←→ QML Socket
```

| Aspect | Assessment |
|--------|------------|
| Command latency | ~1ms |
| State latency | ~1ms (push-based) |
| Complexity | Medium |
| Debugging | Harder (need to tap socket) |

**Verdict:** Best latency, single connection handles both directions.

### Option E: CXX-Qt Plugin (In-Process)

```
QML ←→ CXX-Qt FFI ←→ Rust (in QuickShell process)
```

| Aspect | Assessment |
|--------|------------|
| Command latency | <1ms |
| State latency | <1ms |
| Complexity | High (CMake + Cargo + Qt bindings) |
| Crash isolation | None (Rust crash = shell crash) |

**Verdict:** Maximum performance, maximum complexity.

## Reference Implementation: Caelestia Shell

Caelestia is a mature QuickShell-based desktop shell with:
- **Players.qml**: MPRIS service using `Quickshell.Services.Mpris`
- **Audio.qml**: PipeWire integration + Cava visualization
- **C++ Plugin**: Native Qt plugin wrapping libcava for 60fps FFT

Key insights from Caelestia:
1. QuickShell's built-in MPRIS is sufficient for basic media control
2. High-frequency visualization requires native code (C++ plugin)
3. IpcHandler enables external process communication

## QuickShell Capabilities

### Quickshell.Io Module

| Type | Description |
|------|-------------|
| `FileView` | Watch and read files reactively |
| `Process` | Spawn and manage child processes |
| `Socket` | Unix socket client/server |
| `SocketServer` | Unix socket server |
| `IpcHandler` | Handler for `qs ipc call` commands |

### Quickshell.Services.Mpris

Built-in MPRIS support providing:
- `Mpris.players` - list of all media players
- `MprisPlayer` object with properties: `trackTitle`, `trackArtist`, `isPlaying`, `position`, etc.
- Methods: `play()`, `pause()`, `next()`, `previous()`, `togglePlaying()`

## Rust Ecosystem

### Audio Capture

| Crate | Description |
|-------|-------------|
| `pipewire` | Native PipeWire bindings (Linux) |
| `cpal` | Cross-platform audio I/O |

### FFT / Spectrum Analysis

| Crate | Description |
|-------|-------------|
| `spectrum-analyzer` | Fast, no_std FFT with window functions |
| `rustfft` | Pure Rust FFT implementation |

### Socket.io

| Crate | Description |
|-------|-------------|
| `rust_socketio` | Socket.io client |

### Qt Integration

| Crate | Description |
|-------|-------------|
| `cxx-qt` | Safe Rust ↔ Qt/QML bindings (KDAB) |
| `cxx` | Safe Rust ↔ C++ FFI |

## Open Questions

1. **PipeWire vs cpal for audio capture?**
   - PipeWire: Better for capturing system audio output (monitor)
   - cpal: Simpler API, cross-platform, but primarily for input devices

2. **Socket message format?**
   - Newline-delimited JSON is simple and debuggable
   - MessagePack would be more efficient but harder to debug

3. **Visualization update rate?**
   - 60fps ideal, 30fps acceptable
   - State file approach maxes out around 20fps practically

4. **CXX-Qt maturity?**
   - API is still evolving (0.7.x)
   - KDAB actively maintains it
   - May need to pin version carefully

## Next Steps

See ADR-001 for the architectural decisions made based on this exploration.
