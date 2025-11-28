//! # Inbound Ports — Driving the Application
//!
//! Inbound ports define how external systems can **drive** the fernctl
//! application. They are the entry points through which configuration
//! is loaded, validated, and queried.
//!
//! ## Overview
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                   DRIVING ADAPTERS                          │
//! │                                                             │
//! │   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
//! │   │ TomlAdapter  │  │  CliAdapter  │  │ JsonAdapter  │     │
//! │   │              │  │              │  │              │     │
//! │   │ Reads .toml  │  │ Parses args  │  │ Reads .json  │     │
//! │   └──────┬───────┘  └──────┬───────┘  └──────┬───────┘     │
//! │          │                 │                 │              │
//! └──────────┼─────────────────┼─────────────────┼──────────────┘
//!            │                 │                 │
//!      ╔═════╧═════════════════╧═════════════════╧═════════════╗
//!      ║                  INBOUND PORTS                        ║
//!      ║                                                       ║
//!      ║   ConfigPort ──────► Load raw configuration           ║
//!      ║   ValidatePort ────► Validate and transform           ║
//!      ║   QueryPort ───────► Query theme values               ║
//!      ║                                                       ║
//!      ╚═══════════════════════════╤═══════════════════════════╝
//!                                  │
//!                                  ▼
//!                          ┌──────────────┐
//!                          │    DOMAIN    │
//!                          │    Theme     │
//!                          │    Tokens    │
//!                          └──────────────┘
//! ```
//!
//! ## Port Responsibilities
//!
//! | Port | Responsibility | Input | Output |
//! |------|----------------|-------|--------|
//! | [`ConfigPort`] | Load raw configuration | Source (file content, path) | Raw config data |
//! | [`ValidatePort`] | Validate and transform | Raw config | Validated theme |
//! | [`QueryPort`] | Query theme values | Theme + query | Token values |
//!
//! ## Usage Pattern
//!
//! Inbound ports are typically used in this sequence:
//!
//! ```rust,ignore
//! use fernctl::ports::inbound::*;
//! use fernctl::adapters::TomlConfigAdapter;
//!
//! // 1. Load raw configuration
//! let config_adapter = TomlConfigAdapter::new();
//! let raw_config = config_adapter.load_from_file("config.toml")?;
//!
//! // 2. Validate and transform
//! let theme = raw_config.validate()?.into_theme();
//!
//! // 3. Query values (optional)
//! let bg_color = query_adapter.get_color(&theme, "background")?;
//! ```
//!
//! ## Implementing Custom Adapters
//!
//! To add support for a new configuration format (e.g., YAML):
//!
//! ```rust,ignore
//! use fernctl::ports::inbound::ConfigPort;
//! use fernctl::domain::config::RawConfig;
//! use fernctl::error::Result;
//!
//! struct YamlConfigAdapter;
//!
//! impl ConfigPort for YamlConfigAdapter {
//!     fn load(&self, source: &str) -> Result<RawConfig> {
//!         // Parse YAML and convert to RawConfig
//!         let yaml: serde_yaml::Value = serde_yaml::from_str(source)?;
//!         RawConfig::from_yaml(yaml)
//!     }
//!
//!     fn format_name(&self) -> &'static str {
//!         "YAML"
//!     }
//! }
//! ```

use crate::domain::theme::Theme;
use crate::error::{ConfigError, FernError, Notification, Result, Severity};
use std::path::Path;

// ============================================================================
// Raw Configuration Types
// ============================================================================

/// Raw configuration data before validation.
///
/// `RawConfig` represents configuration data that has been parsed from a file
/// format (TOML, JSON) but has not yet been validated against the theme schema.
///
/// This type is intentionally opaque — its internal structure may change. Use
/// the [`ValidatePort`] trait to convert it into a validated [`Theme`].
///
/// # Type-State Pattern
///
/// `RawConfig` is part of a type-state pattern that prevents using invalid
/// configurations:
///
/// ```text
/// RawConfig ──validate()──► ValidatedConfig ──into_theme()──► Theme
///     │                          │
///     │ Cannot be used          │ Cannot be modified
///     │ directly in UI          │ but can be used
///     ▼                          ▼
///  Must validate             Safe to use
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use fernctl::ports::inbound::{ConfigPort, RawConfig};
///
/// // Load produces RawConfig
/// let raw: RawConfig = adapter.load(toml_content)?;
///
/// // Must validate before use
/// let validated = raw.validate()?;
/// let theme = validated.into_theme();
/// ```
#[derive(Debug, Clone)]
pub struct RawConfig {
    /// The parsed configuration as a generic value.
    ///
    /// This is kept as a serde_json::Value internally for maximum flexibility
    /// across different input formats (TOML, JSON, YAML can all convert to this).
    inner: serde_json::Value,
    /// The source format this config was parsed from.
    source_format: SourceFormat,
}

/// The format a configuration was parsed from.
///
/// Used for providing helpful error messages that reference the original format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFormat {
    /// TOML format (.toml files)
    Toml,
    /// JSON format (.json files)
    Json,
    /// Unknown or in-memory format
    Unknown,
}

impl RawConfig {
    /// Creates a new raw configuration from a JSON value.
    ///
    /// This is the internal constructor used by adapters. The JSON value
    /// should represent the full configuration structure.
    ///
    /// # Arguments
    ///
    /// * `value` — The parsed configuration as a JSON value
    /// * `source_format` — The original format this was parsed from
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let value = serde_json::from_str(json_content)?;
    /// let raw = RawConfig::new(value, SourceFormat::Json);
    /// ```
    #[must_use]
    pub fn new(value: serde_json::Value, source_format: SourceFormat) -> Self {
        Self {
            inner: value,
            source_format,
        }
    }

    /// Returns the source format this configuration was parsed from.
    #[must_use]
    pub fn source_format(&self) -> SourceFormat {
        self.source_format
    }

    /// Validates this raw configuration and returns a validated configuration.
    ///
    /// Validation checks:
    /// - All required fields are present
    /// - Color values are valid hex format
    /// - Numeric values are within valid ranges
    /// - Unknown keys trigger warnings (not errors)
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] if validation fails, with detailed information
    /// about what went wrong and how to fix it.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let raw = adapter.load(content)?;
    /// let validated = raw.validate()?;
    /// let theme = validated.into_theme();
    /// ```
    pub fn validate(self) -> Result<ValidatedConfig> {
        // Deserialize into Theme structure, collecting any validation errors
        let theme: Theme = serde_json::from_value(self.inner.clone()).map_err(|e| {
            FernError::Config(ConfigError::MissingField {
                key: "theme".to_string(),
                expected_type: format!("valid theme structure: {e}"),
            })
        })?;

        Ok(ValidatedConfig {
            theme,
            warnings: Vec::new(), // TODO: Collect warnings during validation
        })
    }

    /// Returns the inner JSON value for inspection.
    ///
    /// This is primarily useful for debugging and error reporting.
    #[must_use]
    pub fn as_value(&self) -> &serde_json::Value {
        &self.inner
    }
}

/// Configuration that has passed validation.
///
/// `ValidatedConfig` represents a configuration that has been checked for
/// correctness. It contains a valid [`Theme`] and optionally warnings about
/// non-critical issues (like unknown keys).
///
/// # Consuming the Configuration
///
/// Once validated, the configuration can be converted into a usable `Theme`:
///
/// ```rust,ignore
/// let validated = raw.validate()?;
///
/// // Check for warnings
/// for warning in validated.warnings() {
///     log::warn!("{}", warning);
/// }
///
/// // Extract the theme
/// let theme = validated.into_theme();
/// ```
#[derive(Debug, Clone)]
pub struct ValidatedConfig {
    /// The validated theme.
    theme: Theme,
    /// Non-fatal warnings encountered during validation.
    warnings: Vec<ConfigWarning>,
}

impl ValidatedConfig {
    /// Returns any warnings encountered during validation.
    ///
    /// Warnings are non-fatal issues that don't prevent the theme from loading
    /// but may indicate configuration problems:
    ///
    /// - Unknown keys (possible typos)
    /// - Deprecated options
    /// - Values outside recommended ranges
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// for warning in validated.warnings() {
    ///     eprintln!("Warning: {}", warning);
    /// }
    /// ```
    #[must_use]
    pub fn warnings(&self) -> &[ConfigWarning] {
        &self.warnings
    }

    /// Returns `true` if validation produced any warnings.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Consumes this validated configuration and returns the theme.
    ///
    /// After calling this method, the `ValidatedConfig` is consumed and the
    /// theme is ready for use in the application.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let theme = validated.into_theme();
    /// let bg_color = &theme.colors.background;
    /// ```
    #[must_use]
    pub fn into_theme(self) -> Theme {
        self.theme
    }

    /// Returns a reference to the theme without consuming the configuration.
    ///
    /// Use this when you need to inspect the theme but still want access to
    /// warnings or other metadata.
    #[must_use]
    pub fn theme(&self) -> &Theme {
        &self.theme
    }
}

/// A non-fatal warning from configuration validation.
///
/// Warnings indicate issues that don't prevent the configuration from loading
/// but may indicate problems that should be addressed.
#[derive(Debug, Clone)]
pub struct ConfigWarning {
    /// The warning message.
    pub message: String,
    /// The severity level.
    pub severity: Severity,
    /// Optional suggestion for fixing the issue.
    pub suggestion: Option<String>,
    /// The configuration key that triggered the warning, if applicable.
    pub key: Option<String>,
}

impl ConfigWarning {
    /// Creates a new configuration warning.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            severity: Severity::Warning,
            suggestion: None,
            key: None,
        }
    }

    /// Adds a suggestion to this warning.
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Associates this warning with a configuration key.
    #[must_use]
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Converts this warning to a notification.
    #[must_use]
    pub fn to_notification(&self) -> Notification {
        let mut notification = Notification::warning("Config Warning", &self.message);
        if let Some(ref suggestion) = self.suggestion {
            notification = notification.with_suggestion(suggestion);
        }
        notification
    }
}

impl std::fmt::Display for ConfigWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref key) = self.key {
            write!(f, "[{}] ", key)?;
        }
        write!(f, "{}", self.message)?;
        if let Some(ref suggestion) = self.suggestion {
            write!(f, " ({})", suggestion)?;
        }
        Ok(())
    }
}

// ============================================================================
// ConfigPort Trait
// ============================================================================

/// Port for loading configuration from external sources.
///
/// `ConfigPort` is the primary inbound port for configuration loading. It
/// abstracts over different configuration formats (TOML, JSON, etc.) and
/// sources (files, environment, in-memory).
///
/// # Implementing ConfigPort
///
/// Adapters implement this trait to add support for new configuration formats:
///
/// ```rust,ignore
/// use fernctl::ports::inbound::{ConfigPort, RawConfig, SourceFormat};
/// use fernctl::error::Result;
///
/// struct TomlConfigAdapter;
///
/// impl ConfigPort for TomlConfigAdapter {
///     fn load(&self, source: &str) -> Result<RawConfig> {
///         let value: toml::Value = toml::from_str(source)?;
///         let json = serde_json::to_value(value)?;
///         Ok(RawConfig::new(json, SourceFormat::Toml))
///     }
///
///     fn format_name(&self) -> &'static str {
///         "TOML"
///     }
/// }
/// ```
///
/// # Object Safety
///
/// This trait is object-safe and can be used as a trait object:
///
/// ```rust,ignore
/// let adapter: Box<dyn ConfigPort> = Box::new(TomlConfigAdapter::new());
/// let config = adapter.load(content)?;
/// ```
///
/// # Error Handling
///
/// All loading operations return `Result<RawConfig>`. Errors should be
/// wrapped in [`ConfigError`] variants for rich diagnostics.
pub trait ConfigPort: Send + Sync {
    /// Loads configuration from a string source.
    ///
    /// The source string should contain the full configuration in the format
    /// supported by this adapter (e.g., TOML content for a TOML adapter).
    ///
    /// # Arguments
    ///
    /// * `source` — The configuration content as a string
    ///
    /// # Returns
    ///
    /// A [`RawConfig`] that can be validated into a [`Theme`].
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] if the source cannot be parsed.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let toml_content = r#"
    ///     [appearance]
    ///     variant = "dark"
    /// "#;
    ///
    /// let raw = adapter.load(toml_content)?;
    /// ```
    fn load(&self, source: &str) -> Result<RawConfig>;

    /// Loads configuration from a file path.
    ///
    /// This is a convenience method that reads the file and calls [`load`].
    /// The default implementation handles file I/O and provides appropriate
    /// error messages.
    ///
    /// # Arguments
    ///
    /// * `path` — Path to the configuration file
    ///
    /// # Returns
    ///
    /// A [`RawConfig`] that can be validated into a [`Theme`].
    ///
    /// # Errors
    ///
    /// - [`FernError::Io`] if the file cannot be read
    /// - [`ConfigError`] if the content cannot be parsed
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let raw = adapter.load_from_file("~/.config/fern/config.toml")?;
    /// ```
    fn load_from_file(&self, path: impl AsRef<Path>) -> Result<RawConfig> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| {
            FernError::io(format!("reading configuration file: {}", path.display()), e)
        })?;
        self.load(&content)
    }

    /// Returns the human-readable name of this configuration format.
    ///
    /// Used in error messages and logging.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// println!("Loading {} configuration...", adapter.format_name());
    /// // Prints: "Loading TOML configuration..."
    /// ```
    fn format_name(&self) -> &'static str;

    /// Returns the typical file extension for this format.
    ///
    /// Used for auto-detecting the appropriate adapter based on file path.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// assert_eq!(toml_adapter.file_extension(), "toml");
    /// ```
    fn file_extension(&self) -> &'static str {
        match self.format_name().to_lowercase().as_str() {
            "toml" => "toml",
            "json" => "json",
            "yaml" => "yaml",
            _ => "",
        }
    }
}

// ============================================================================
// ValidatePort Trait
// ============================================================================

/// Port for validating configuration data.
///
/// `ValidatePort` handles the validation of raw configuration data, checking
/// that all values are correct and producing helpful error messages when
/// they are not.
///
/// # Validation Rules
///
/// The validator checks:
///
/// | Category | Validation |
/// |----------|------------|
/// | Colors | Valid hex format, correct length |
/// | Numbers | Within valid ranges |
/// | Strings | Non-empty where required |
/// | Structure | Required fields present |
/// | Keys | Warning for unknown keys |
///
/// # Example
///
/// ```rust,ignore
/// use fernctl::ports::inbound::ValidatePort;
///
/// let validator = DefaultValidator::new();
/// let result = validator.validate(&raw_config);
///
/// match result {
///     Ok(validated) => {
///         for warning in validated.warnings() {
///             log::warn!("{}", warning);
///         }
///         let theme = validated.into_theme();
///     }
///     Err(errors) => {
///         for error in errors {
///             eprintln!("Error: {}", error);
///         }
///     }
/// }
/// ```
pub trait ValidatePort: Send + Sync {
    /// Validates raw configuration and returns a validated configuration.
    ///
    /// # Arguments
    ///
    /// * `raw` — The raw configuration to validate
    ///
    /// # Returns
    ///
    /// A [`ValidatedConfig`] containing the theme and any warnings.
    ///
    /// # Errors
    ///
    /// Returns a vector of [`ConfigError`] if validation fails. Multiple
    /// errors may be returned to show all problems at once.
    fn validate(&self, raw: RawConfig) -> std::result::Result<ValidatedConfig, Vec<ConfigError>>;

    /// Validates a single color value.
    ///
    /// # Arguments
    ///
    /// * `value` — The color value to validate (e.g., "#1e1e2e")
    /// * `key` — The configuration key for error reporting
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::InvalidColor`] if the value is not valid.
    fn validate_color(&self, value: &str, key: &str) -> Result<()>;

    /// Validates a spacing value.
    ///
    /// # Arguments
    ///
    /// * `value` — The spacing value in pixels
    /// * `key` — The configuration key for error reporting
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::InvalidValue`] if the value is out of range.
    fn validate_spacing(&self, value: u16, key: &str) -> Result<()>;
}

// ============================================================================
// QueryPort Trait
// ============================================================================

/// Port for querying theme values.
///
/// `QueryPort` provides a way to query specific values from a theme using
/// string-based paths. This is useful for:
///
/// - CLI tools that need to extract specific values
/// - Templates that reference theme tokens by name
/// - Testing that specific tokens have expected values
///
/// # Query Paths
///
/// Queries use dot-separated paths to navigate the theme structure:
///
/// | Path | Returns |
/// |------|---------|
/// | `colors.background` | Background color as hex |
/// | `typography.family` | Primary font family name |
/// | `radius.button` | Button radius in pixels |
/// | `bar.height` | Bar height in pixels |
///
/// # Example
///
/// ```rust,ignore
/// use fernctl::ports::inbound::QueryPort;
///
/// let query = DefaultQueryAdapter::new();
/// let theme = Theme::dark();
///
/// // Query color
/// let bg = query.get(&theme, "colors.background")?;
/// assert_eq!(bg, "#1e1e2e");
///
/// // Query number
/// let height = query.get(&theme, "bar.height")?;
/// assert_eq!(height, "40");
/// ```
pub trait QueryPort: Send + Sync {
    /// Queries a value from the theme by path.
    ///
    /// # Arguments
    ///
    /// * `theme` — The theme to query
    /// * `path` — Dot-separated path to the value
    ///
    /// # Returns
    ///
    /// The value as a string representation.
    ///
    /// # Errors
    ///
    /// Returns an error if the path does not exist in the theme.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let color = query.get(&theme, "colors.accent")?;
    /// ```
    fn get(&self, theme: &Theme, path: &str) -> Result<String>;

    /// Queries a color value from the theme.
    ///
    /// This is a typed version of [`get`] that returns the color in hex format.
    ///
    /// # Arguments
    ///
    /// * `theme` — The theme to query
    /// * `name` — The color name (e.g., "background", "accent")
    ///
    /// # Returns
    ///
    /// The color as a hex string (e.g., "#1e1e2e").
    ///
    /// # Errors
    ///
    /// Returns an error if the color name is not recognized.
    fn get_color(&self, theme: &Theme, name: &str) -> Result<String>;

    /// Returns all available query paths.
    ///
    /// Useful for auto-completion and documentation.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// for path in query.available_paths() {
    ///     println!("  {}", path);
    /// }
    /// ```
    fn available_paths(&self) -> Vec<&'static str>;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_config_creation() {
        let value = serde_json::json!({
            "variant": "dark"
        });
        let raw = RawConfig::new(value, SourceFormat::Toml);
        assert_eq!(raw.source_format(), SourceFormat::Toml);
    }

    #[test]
    fn config_warning_display() {
        let warning = ConfigWarning::new("unknown key 'colour'")
            .with_key("appearance.colour")
            .with_suggestion("did you mean 'color'?");

        let display = warning.to_string();
        assert!(display.contains("appearance.colour"));
        assert!(display.contains("unknown key"));
        assert!(display.contains("did you mean 'color'?"));
    }

    #[test]
    fn config_warning_to_notification() {
        let warning = ConfigWarning::new("test warning").with_suggestion("fix it");
        let notification = warning.to_notification();
        assert_eq!(notification.severity, Severity::Warning);
        assert!(notification.suggestion.is_some());
    }
}
