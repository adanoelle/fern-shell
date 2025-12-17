# API Reference

Complete API documentation for Fern Shell.

## Rust API (fernctl)

The fernctl crate provides the Rust backend for configuration management and
system integration.

### Quick Links

- **[Full API Reference](../api/fernctl/index.html)** - Complete rustdoc

### Key Modules

| Module                                         | Description                                  |
| ---------------------------------------------- | -------------------------------------------- |
| [domain](../api/fernctl/domain/index.html)     | Pure business logic - themes, tokens, config |
| [ports](../api/fernctl/ports/index.html)       | Hexagonal architecture interfaces            |
| [adapters](../api/fernctl/adapters/index.html) | Concrete implementations                     |
| [commands](../api/fernctl/commands/index.html) | CLI command handlers                         |
| [error](../api/fernctl/error/index.html)       | Error types with diagnostics                 |

### Domain Types

```rust
// Theme composition
use fernctl::domain::Theme;

// Design tokens
use fernctl::domain::tokens::{ColorToken, SpacingToken, RadiusToken};

// User configuration
use fernctl::domain::UserConfig;
```

### Port Traits

```rust
// Inbound (what the domain accepts)
use fernctl::ports::inbound::{ConfigPort, QueryPort, ValidatePort};

// Outbound (what the domain needs)
use fernctl::ports::outbound::{PersistPort, NotifyPort, IpcPort};
```

### Adapter Implementations

```rust
// TOML configuration
use fernctl::adapters::TomlConfigAdapter;

// JSON serialization
use fernctl::adapters::JsonThemeAdapter;

// File system
use fernctl::adapters::FileSystemPersist;
```

## QML API

QML components don't have generated documentation. See the
[QML Components](../03-qml-components/README.md) section for manual
documentation.

### Key Singletons

| Singleton  | Import                      | Purpose       |
| ---------- | --------------------------- | ------------- |
| `Theme`    | `"../config" as Config`     | Design tokens |
| `Hyprland` | `"../services" as Services` | Hyprland IPC  |

### Common Patterns

```qml
// Access theme tokens
Config.Theme.accent
Config.Theme.barWidth
Config.Theme.spacing.md

// Access Hyprland state
Services.Hyprland.activeWsId
Services.Hyprland.workspaces
```

## CLI Reference

```bash
fernctl --help
```

| Command    | Description                  |
| ---------- | ---------------------------- |
| `validate` | Check config file for errors |
| `convert`  | Convert between formats      |
| `query`    | Query config values          |
| `watch`    | Watch for config changes     |

See `fernctl <command> --help` for detailed usage.
