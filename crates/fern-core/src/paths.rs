//! # XDG-compliant paths for Fern Shell
//!
//! This module provides standardized paths for configuration, state, and
//! runtime files following the XDG Base Directory Specification.
//!
//! ## Directory Structure
//!
//! ```text
//! ~/.config/fern/           # Configuration (XDG_CONFIG_HOME)
//! ├── config.toml           # User configuration
//! └── config.json           # Generated JSON for QuickShell
//!
//! ~/.local/state/fern/      # Runtime state (XDG_STATE_HOME)
//! ├── services.json         # Service registry state
//! ├── obs-state.json        # OBS bridge state
//! └── theme-state.json      # Theme service state
//!
//! ~/.local/share/fern/      # Persistent data (XDG_DATA_HOME)
//! └── themes/               # User themes
//! ```

use std::path::PathBuf;

/// Fern Shell directory paths following XDG specification.
///
/// # Example
///
/// ```rust
/// use fern_core::FernPaths;
///
/// let paths = FernPaths::new();
/// println!("Config dir: {:?}", paths.config_dir());
/// println!("State dir: {:?}", paths.state_dir());
/// ```
#[derive(Debug, Clone)]
pub struct FernPaths {
    config_dir: PathBuf,
    state_dir: PathBuf,
    data_dir: PathBuf,
}

impl FernPaths {
    /// Creates a new `FernPaths` instance using XDG directories.
    ///
    /// Falls back to sensible defaults if XDG directories cannot be determined.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config_dir: dirs::config_dir()
                .map(|p| p.join("fern"))
                .unwrap_or_else(|| PathBuf::from(".config/fern")),
            state_dir: dirs::state_dir()
                .or_else(dirs::data_local_dir)
                .map(|p| p.join("fern"))
                .unwrap_or_else(|| PathBuf::from(".local/state/fern")),
            data_dir: dirs::data_dir()
                .map(|p| p.join("fern"))
                .unwrap_or_else(|| PathBuf::from(".local/share/fern")),
        }
    }

    /// Returns the configuration directory (`~/.config/fern/`).
    #[must_use]
    pub fn config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    /// Returns the state directory (`~/.local/state/fern/`).
    ///
    /// Used for runtime state files that should persist across restarts
    /// but are not user configuration.
    #[must_use]
    pub fn state_dir(&self) -> &PathBuf {
        &self.state_dir
    }

    /// Returns the data directory (`~/.local/share/fern/`).
    ///
    /// Used for persistent user data like custom themes.
    #[must_use]
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }

    /// Returns the path to the user's TOML configuration file.
    #[must_use]
    pub fn config_toml(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    /// Returns the path to the generated JSON configuration file.
    ///
    /// This is consumed by QuickShell and should not be edited manually.
    #[must_use]
    pub fn config_json(&self) -> PathBuf {
        self.config_dir.join("config.json")
    }

    /// Returns the path to a service's state file.
    ///
    /// # Arguments
    ///
    /// * `service` - The service name (e.g., "obs", "theme")
    ///
    /// # Example
    ///
    /// ```rust
    /// use fern_core::FernPaths;
    ///
    /// let paths = FernPaths::new();
    /// let obs_state = paths.service_state("obs");
    /// // Returns: ~/.local/state/fern/obs-state.json
    /// ```
    #[must_use]
    pub fn service_state(&self, service: &str) -> PathBuf {
        self.state_dir.join(format!("{service}-state.json"))
    }

    /// Returns the path to the service registry state file.
    ///
    /// This file tracks all running services and their status.
    #[must_use]
    pub fn services_registry(&self) -> PathBuf {
        self.state_dir.join("services.json")
    }

    /// Ensures all required directories exist, creating them if necessary.
    ///
    /// # Errors
    ///
    /// Returns an error if directories cannot be created.
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.config_dir)?;
        std::fs::create_dir_all(&self.state_dir)?;
        std::fs::create_dir_all(&self.data_dir)?;
        Ok(())
    }
}

impl Default for FernPaths {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_are_reasonable() {
        let paths = FernPaths::new();

        // Should end with "fern"
        assert!(paths.config_dir().ends_with("fern"));
        assert!(paths.state_dir().ends_with("fern"));
        assert!(paths.data_dir().ends_with("fern"));
    }

    #[test]
    fn service_state_path_format() {
        let paths = FernPaths::new();
        let obs_state = paths.service_state("obs");

        assert!(obs_state.to_string_lossy().contains("obs-state.json"));
    }

    #[test]
    fn config_files_in_config_dir() {
        let paths = FernPaths::new();

        assert!(paths.config_toml().starts_with(paths.config_dir()));
        assert!(paths.config_json().starts_with(paths.config_dir()));
    }
}
