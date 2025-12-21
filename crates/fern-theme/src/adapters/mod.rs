//! # Adapters — External System Integration
//!
//! Adapters implement the port traits to connect the domain to external systems.
//!
//! ## Overview
//!
//! ```text
//!              PORTS                          ADAPTERS
//!        ┌──────────────┐              ┌─────────────────────┐
//!        │  ConfigPort  │◄─────────────│  TomlConfigAdapter  │
//!        └──────────────┘              │  JsonConfigAdapter  │
//!                                      └─────────────────────┘
//!        ┌──────────────┐              ┌─────────────────────┐
//!        │  PersistPort │◄─────────────│  FileSystemAdapter  │
//!        └──────────────┘              └─────────────────────┘
//!        ┌──────────────┐              ┌─────────────────────┐
//!        │  NotifyPort  │◄─────────────│  DbusNotifyAdapter  │
//!        └──────────────┘              └─────────────────────┘
//!        ┌──────────────┐              ┌─────────────────────┐
//!        │   IpcPort    │◄─────────────│  DbusIpcAdapter     │
//!        └──────────────┘              └─────────────────────┘
//! ```
//!
//! ## Available Adapters
//!
//! | Adapter | Port | Description |
//! |---------|------|-------------|
//! | `TomlConfigAdapter` | `ConfigPort` | Load config from TOML files |
//! | `JsonConfigAdapter` | `ConfigPort` | Load config from JSON files |
//! | `FileSystemAdapter` | `PersistPort` | Save themes to filesystem |
//! | `DbusNotifyAdapter` | `NotifyPort` | Send notifications via D-Bus |
//! | `DbusIpcAdapter` | `IpcPort` | Communicate with QuickShell via D-Bus |
//!
//! ## Feature Flags
//!
//! Some adapters are behind feature flags:
//!
//! - `dbus` — Enables `DbusNotifyAdapter` and `DbusIpcAdapter`
//! - `watch` — Enables file watching capabilities
//!
//! ## Example
//!
//! ```rust,ignore
//! use fern_theme::adapters::{TomlConfigAdapter, FileSystemAdapter};
//! use fern_theme::ports::inbound::ConfigPort;
//! use fern_theme::ports::outbound::PersistPort;
//!
//! // Load configuration
//! let config_adapter = TomlConfigAdapter::new();
//! let raw = config_adapter.load_from_file("config.toml")?;
//! let theme = raw.validate()?.into_theme();
//!
//! // Save as JSON for QuickShell
//! let persist_adapter = FileSystemAdapter::new();
//! persist_adapter.save_theme(&theme, "config.json")?;
//! ```

// Adapters will be implemented in a future PR.
// This module provides the structure for external system integration.

use crate::domain::theme::Theme;
use crate::error::{FernError, Result};
use crate::ports::inbound::{ConfigPort, RawConfig, SourceFormat};
use crate::ports::outbound::PersistPort;
use std::path::Path;

// ============================================================================
// TomlConfigAdapter
// ============================================================================

/// Adapter for loading configuration from TOML files.
///
/// This is the primary configuration format for Fern Shell. Users write
/// their configuration in TOML, and this adapter parses it into the domain
/// types.
///
/// # Example
///
/// ```rust,ignore
/// use fern_theme::adapters::TomlConfigAdapter;
/// use fern_theme::ports::inbound::ConfigPort;
///
/// let adapter = TomlConfigAdapter::new();
/// let raw = adapter.load_from_file("~/.config/fern/config.toml")?;
/// let theme = raw.validate()?.into_theme();
/// ```
#[derive(Debug, Clone, Default)]
pub struct TomlConfigAdapter;

impl TomlConfigAdapter {
    /// Creates a new TOML configuration adapter.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ConfigPort for TomlConfigAdapter {
    fn load(&self, source: &str) -> Result<RawConfig> {
        // Parse TOML to a generic Value
        let toml_value: toml::Value = toml::from_str(source).map_err(|e| {
            FernError::Config(crate::error::ConfigError::ParseError {
                source: e,
                src: source.to_string(),
            })
        })?;

        // Convert TOML Value to JSON Value for unified internal representation
        let json_value = toml_to_json(toml_value);

        Ok(RawConfig::new(json_value, SourceFormat::Toml))
    }

    fn format_name(&self) -> &'static str {
        "TOML"
    }
}

/// Converts a TOML value to a JSON value.
///
/// This allows the domain to work with a single internal representation
/// regardless of the input format.
fn toml_to_json(toml: toml::Value) -> serde_json::Value {
    match toml {
        toml::Value::String(s) => serde_json::Value::String(s),
        toml::Value::Integer(i) => serde_json::Value::Number(i.into()),
        toml::Value::Float(f) => {
            serde_json::Number::from_f64(f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
        toml::Value::Boolean(b) => serde_json::Value::Bool(b),
        toml::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(toml_to_json).collect())
        }
        toml::Value::Table(table) => {
            let map = table
                .into_iter()
                .map(|(k, v)| (k, toml_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
        toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
    }
}

// ============================================================================
// JsonConfigAdapter
// ============================================================================

/// Adapter for loading configuration from JSON files.
///
/// JSON configuration is primarily used internally (generated from TOML)
/// but can also be written directly by users who prefer JSON.
///
/// # Example
///
/// ```rust,ignore
/// use fern_theme::adapters::JsonConfigAdapter;
/// use fern_theme::ports::inbound::ConfigPort;
///
/// let adapter = JsonConfigAdapter::new();
/// let raw = adapter.load_from_file("config.json")?;
/// ```
#[derive(Debug, Clone, Default)]
pub struct JsonConfigAdapter;

impl JsonConfigAdapter {
    /// Creates a new JSON configuration adapter.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ConfigPort for JsonConfigAdapter {
    fn load(&self, source: &str) -> Result<RawConfig> {
        let json_value: serde_json::Value = serde_json::from_str(source).map_err(|e| {
            FernError::Config(crate::error::ConfigError::MissingField {
                key: "root".to_string(),
                expected_type: format!("valid JSON: {e}"),
            })
        })?;

        Ok(RawConfig::new(json_value, SourceFormat::Json))
    }

    fn format_name(&self) -> &'static str {
        "JSON"
    }
}

// ============================================================================
// FileSystemAdapter
// ============================================================================

/// Adapter for persisting themes to the filesystem.
///
/// This adapter handles saving and loading theme files, following platform
/// conventions for configuration directories.
///
/// # Example
///
/// ```rust,ignore
/// use fern_theme::adapters::FileSystemAdapter;
/// use fern_theme::ports::outbound::PersistPort;
///
/// let adapter = FileSystemAdapter::new();
/// adapter.save_theme(&theme, "config.json")?;
/// ```
#[derive(Debug, Clone, Default)]
pub struct FileSystemAdapter;

impl FileSystemAdapter {
    /// Creates a new filesystem adapter.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl PersistPort for FileSystemAdapter {
    fn save_theme(&self, theme: &Theme, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let json = serde_json::to_string_pretty(theme).map_err(|e| {
            FernError::io(format!("serializing theme: {e}"), std::io::Error::other(e.to_string()))
        })?;

        std::fs::write(path, json)
            .map_err(|e| FernError::io(format!("writing {}", path.display()), e))?;

        Ok(())
    }

    fn load_theme(&self, path: impl AsRef<Path>) -> Result<Theme> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| FernError::io(format!("reading {}", path.display()), e))?;

        let theme: Theme = serde_json::from_str(&content).map_err(|e| {
            FernError::Config(crate::error::ConfigError::MissingField {
                key: "theme".to_string(),
                expected_type: format!("valid theme JSON: {e}"),
            })
        })?;

        Ok(theme)
    }

    fn config_dir(&self) -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|p| p.join("fern"))
    }

    fn ensure_config_dir(&self) -> Result<std::path::PathBuf> {
        let dir = self.config_dir().ok_or_else(|| {
            FernError::io(
                "determining config directory",
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "could not determine config directory",
                ),
            )
        })?;

        std::fs::create_dir_all(&dir)
            .map_err(|e| FernError::io(format!("creating {}", dir.display()), e))?;

        Ok(dir)
    }

    fn exists(&self, path: impl AsRef<Path>) -> bool {
        path.as_ref().exists()
    }
}
