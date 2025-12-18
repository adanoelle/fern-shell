# Rust Services

Building backend integrations with the Fern Shell Rust crates.

## Workspace Structure

Fern Shell uses a Cargo workspace with specialized crates:

```
crates/
├── fern-core/      # Shared types, paths, utilities
├── fern-theme/     # Theme & config management CLI
└── fern-obs/       # OBS WebSocket bridge daemon
```

### fern-core

Shared infrastructure used by all crates:

- `FernPaths` - XDG-compliant path utilities
- `ServiceInfo`, `ServiceStatus` - Service state types
- Common error types and utilities

### fern-theme

Theme and configuration management (formerly fernctl). Uses hexagonal architecture:

```
Domain (pure logic)
  ↓
Ports (interfaces)
  ↓
Adapters (implementations)
```

### fern-obs

OBS Studio integration via WebSocket. Writes state to a JSON file for QML consumption.

## Adding a New Adapter

### Step 1: Define the Port

In `crates/fern-theme/src/ports/outbound.rs`:

```rust
/// Port for battery status monitoring
pub trait BatteryPort {
    /// Get current battery percentage (0-100)
    fn get_percentage(&self) -> Result<u8, FernError>;

    /// Check if charging
    fn is_charging(&self) -> Result<bool, FernError>;
}
```

### Step 2: Implement the Adapter

Create `crates/fern-theme/src/adapters/battery.rs`:

```rust
use crate::error::FernError;
use crate::ports::outbound::BatteryPort;

/// UPower-based battery adapter
pub struct UPowerBattery {
    // Connection state
}

impl UPowerBattery {
    pub fn new() -> Result<Self, FernError> {
        // Initialize UPower connection
        Ok(Self { })
    }
}

impl BatteryPort for UPowerBattery {
    fn get_percentage(&self) -> Result<u8, FernError> {
        // Read from UPower D-Bus
        Ok(75)
    }

    fn is_charging(&self) -> Result<bool, FernError> {
        Ok(false)
    }
}
```

### Step 3: Register in mod.rs

Add to `crates/fern-theme/src/adapters/mod.rs`:

```rust
pub mod battery;
pub use battery::UPowerBattery;
```

### Step 4: Feature Gate (Optional)

In `Cargo.toml`:

```toml
[features]
battery = ["dep:zbus"]

[dependencies]
zbus = { version = "4", optional = true }
```

## Error Handling

Use the `FernError` type with miette integration:

```rust
use crate::error::{FernError, ConfigError};

fn do_something() -> Result<(), FernError> {
    // Convert errors with ?
    let data = read_file()?;

    // Create domain errors
    if data.is_empty() {
        return Err(ConfigError::Empty {
            path: path.to_path_buf(),
        }.into());
    }

    Ok(())
}
```

## D-Bus Integration

For system services, use `zbus`:

```rust
use zbus::blocking::Connection;

pub struct DBusAdapter {
    connection: Connection,
}

impl DBusAdapter {
    pub fn new() -> Result<Self, FernError> {
        let connection = Connection::system()
            .map_err(|e| FernError::Dbus(e.to_string()))?;
        Ok(Self { connection })
    }
}
```

## Testing

Use trait objects for mocking:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct MockBattery;

    impl BatteryPort for MockBattery {
        fn get_percentage(&self) -> Result<u8, FernError> {
            Ok(50)
        }

        fn is_charging(&self) -> Result<bool, FernError> {
            Ok(true)
        }
    }

    #[test]
    fn test_battery_logic() {
        let battery = MockBattery;
        assert_eq!(battery.get_percentage().unwrap(), 50);
    }
}
```

## Files

**fern-core:**
- `crates/fern-core/src/paths.rs` - XDG path utilities
- `crates/fern-core/src/state.rs` - Service state types
- `crates/fern-core/src/error.rs` - Shared error types

**fern-theme:**
- `crates/fern-theme/src/ports/` - Interface definitions
- `crates/fern-theme/src/adapters/` - Concrete implementations
- `crates/fern-theme/src/domain/` - Design tokens and themes
- `crates/fern-theme/src/error/` - Error types

**fern-obs:**
- `crates/fern-obs/src/client.rs` - OBS WebSocket client
- `crates/fern-obs/src/daemon.rs` - Persistent daemon
- `crates/fern-obs/src/state.rs` - State types for JSON output
