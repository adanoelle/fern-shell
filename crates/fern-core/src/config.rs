//! # Common Configuration Utilities
//!
//! Shared configuration loading patterns for the Fern ecosystem.
//!
//! This module provides utilities for loading and parsing configuration files
//! that are shared across multiple `fern-*` crates.

use crate::error::{Error, Result};
use crate::paths::FernPaths;
use serde::de::DeserializeOwned;
use std::path::Path;

/// Loads and parses a JSON configuration file.
///
/// # Arguments
///
/// * `path` - Path to the JSON file
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed.
///
/// # Example
///
/// ```rust,ignore
/// use fern_core::config::load_json;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct MyConfig {
///     enabled: bool,
/// }
///
/// let config: MyConfig = load_json("config.json")?;
/// ```
pub fn load_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path).map_err(|e| Error::Io {
        context: format!("reading {}", path.display()),
        source: e,
    })?;

    serde_json::from_str(&content).map_err(|e| Error::Parse {
        context: format!("parsing {}", path.display()),
        message: e.to_string(),
    })
}

/// Saves a value as a JSON configuration file.
///
/// # Arguments
///
/// * `path` - Path to write the JSON file
/// * `value` - The value to serialize
///
/// # Errors
///
/// Returns an error if the file cannot be written or the value cannot be serialized.
pub fn save_json<T: serde::Serialize>(path: impl AsRef<Path>, value: &T) -> Result<()> {
    let path = path.as_ref();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::Io {
            context: format!("creating directory {}", parent.display()),
            source: e,
        })?;
    }

    let content = serde_json::to_string_pretty(value).map_err(|e| Error::Parse {
        context: format!("serializing to {}", path.display()),
        message: e.to_string(),
    })?;

    std::fs::write(path, content).map_err(|e| Error::Io {
        context: format!("writing {}", path.display()),
        source: e,
    })
}

/// Loads a service's state file.
///
/// # Arguments
///
/// * `service` - The service name (e.g., "obs", "theme")
///
/// # Errors
///
/// Returns an error if the state file cannot be read or parsed.
/// Returns `Ok(None)` if the state file doesn't exist.
///
/// # Example
///
/// ```rust,ignore
/// use fern_core::config::load_service_state;
///
/// #[derive(Deserialize)]
/// struct ObsState {
///     recording: bool,
/// }
///
/// if let Some(state) = load_service_state::<ObsState>("obs")? {
///     println!("Recording: {}", state.recording);
/// }
/// ```
pub fn load_service_state<T: DeserializeOwned>(service: &str) -> Result<Option<T>> {
    let paths = FernPaths::new();
    let state_path = paths.service_state(service);

    if !state_path.exists() {
        return Ok(None);
    }

    load_json(&state_path).map(Some)
}

/// Saves a service's state file.
///
/// # Arguments
///
/// * `service` - The service name (e.g., "obs", "theme")
/// * `state` - The state to save
///
/// # Errors
///
/// Returns an error if the state file cannot be written.
pub fn save_service_state<T: serde::Serialize>(service: &str, state: &T) -> Result<()> {
    let paths = FernPaths::new();
    paths.ensure_dirs().map_err(|e| Error::Io {
        context: "creating state directory".into(),
        source: e,
    })?;

    save_json(paths.service_state(service), state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestConfig {
        name: String,
        value: i32,
    }

    #[test]
    fn load_json_success() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"name": "test", "value": 42}}"#).unwrap();

        let config: TestConfig = load_json(file.path()).unwrap();

        assert_eq!(config.name, "test");
        assert_eq!(config.value, 42);
    }

    #[test]
    fn load_json_file_not_found() {
        let result: Result<TestConfig> = load_json("/nonexistent/path.json");

        assert!(result.is_err());
    }

    #[test]
    fn save_and_load_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.json");

        let original = TestConfig {
            name: "roundtrip".into(),
            value: 123,
        };

        save_json(&path, &original).unwrap();
        let loaded: TestConfig = load_json(&path).unwrap();

        assert_eq!(original, loaded);
    }
}
