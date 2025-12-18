# API Reference

Complete API documentation for Fern Shell.

## Rust Crates

Fern Shell uses a Cargo workspace with specialized crates:

| Crate        | Description                              |
| ------------ | ---------------------------------------- |
| `fern-core`  | Shared types, paths, utilities           |
| `fern-theme` | Theme & config management CLI            |
| `fern-obs`   | OBS WebSocket bridge daemon              |

## fern-theme API

Theme and configuration management (formerly fernctl).

### Key Modules

| Module    | Description                                  |
| --------- | -------------------------------------------- |
| `domain`  | Pure business logic - themes, tokens, config |
| `ports`   | Hexagonal architecture interfaces            |
| `adapters`| Concrete implementations                     |
| `commands`| CLI command handlers                         |
| `error`   | Error types with diagnostics                 |

### Domain Types

```rust
// Theme composition
use fern_theme::domain::Theme;

// Design tokens
use fern_theme::domain::tokens::{ColorToken, SpacingToken, RadiusToken};

// User configuration
use fern_theme::domain::UserConfig;
```

### Port Traits

```rust
// Inbound (what the domain accepts)
use fern_theme::ports::inbound::{ConfigPort, QueryPort, ValidatePort};

// Outbound (what the domain needs)
use fern_theme::ports::outbound::{PersistPort, NotifyPort, IpcPort};
```

### Adapter Implementations

```rust
// TOML configuration
use fern_theme::adapters::TomlConfigAdapter;

// File system
use fern_theme::adapters::FileSystemAdapter;
```

## fern-obs API

OBS WebSocket bridge. See [fern-obs CLI](fern-obs.md) for command reference.

### Key Types

```rust
use fern_obs::state::{ObsState, RecordingState, StreamingState, ObsStats};
use fern_obs::client::ObsClient;
use fern_obs::daemon::Daemon;
use fern_obs::config::ObsConfig;
```

## fern-core API

Shared infrastructure.

```rust
use fern_core::paths::FernPaths;
use fern_core::state::{ServiceInfo, ServiceStatus};
```

## QML API

QML components don't have generated documentation. See the
[QML Components](../03-qml-components/README.md) section for manual
documentation.

### Key Singletons

| Singleton  | Import                      | Purpose             |
| ---------- | --------------------------- | ------------------- |
| `Theme`    | `"../config" as Config`     | Design tokens       |
| `Hyprland` | `"../services" as Services` | Hyprland IPC        |
| `Obs`      | `"../services" as Services` | OBS control         |

### Common Patterns

```qml
// Access theme tokens
Config.Theme.accent
Config.Theme.barWidth
Config.Theme.spacing.md

// Access Hyprland state
Services.Hyprland.activeWsId
Services.Hyprland.workspaces

// Access OBS state and control
Services.Obs.isRecording
Services.Obs.recordingTimecode
Services.Obs.toggleRecording()
```

## CLI Reference

### fern-theme

Theme and configuration management.

```bash
fern-theme --help
```

| Command    | Description                  |
| ---------- | ---------------------------- |
| `validate` | Check config file for errors |
| `convert`  | Convert TOML to JSON         |
| `query`    | Query config values          |
| `watch`    | Watch for config changes     |
| `defaults` | Show default theme values    |

### fern-obs

OBS WebSocket bridge. See [fern-obs CLI](fern-obs.md) for full reference.

```bash
fern-obs --help
```

| Command           | Description              |
| ----------------- | ------------------------ |
| `daemon`          | Start bridge daemon      |
| `start-recording` | Start OBS recording      |
| `stop-recording`  | Stop OBS recording       |
| `toggle-pause`    | Toggle recording pause   |
| `start-streaming` | Start OBS streaming      |
| `stop-streaming`  | Stop OBS streaming       |
| `scene`           | Switch scene             |
| `status`          | Show current OBS status  |
