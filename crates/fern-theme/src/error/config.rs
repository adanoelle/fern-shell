//! # Configuration Error Types
//!
//! This module defines error types for configuration parsing and validation.
//! Each error variant includes:
//!
//! - A human-readable message
//! - A unique error code for documentation lookup
//! - Source location information (when available)
//! - Actionable suggestions for fixing the problem
//!
//! ## Error Categories
//!
//! | Category | Examples |
//! |----------|----------|
//! | **Parse Errors** | Invalid TOML syntax |
//! | **Type Errors** | String where number expected |
//! | **Value Errors** | Invalid color format, out of range |
//! | **Schema Errors** | Missing required field, unknown key |
//!
//! ## Integration with miette
//!
//! All errors implement [`miette::Diagnostic`] for rich terminal output.
//! When source code is available, errors display the problematic line:
//!
//! ```text
//! Error: fern::config::invalid_color
//!
//!   × Invalid color format: #gg0000
//!    ╭─[config.toml:5:10]
//!  4 │ [appearance]
//!  5 │ accent = "#gg0000"
//!    ·          ────────── this color value is not valid hex
//!    ╰────
//!   help: Colors must be hex format: #RRGGBB or #RRGGBBAA
//!         Examples: #89b4fa, #ff6b6bff
//! ```

use super::{Notifiable, Severity};
use miette::{Diagnostic, SourceSpan};
use std::path::PathBuf;
use thiserror::Error;

/// Location within a configuration file.
///
/// This struct captures where an error occurred, enabling rich error messages
/// that point to the exact location of the problem.
///
/// # Fields
///
/// - `file` — Path to the configuration file
/// - `line` — Line number (1-indexed)
/// - `column` — Column number (1-indexed)
/// - `key_path` — Dotted path to the key (e.g., "appearance.radius.md")
///
/// # Example
///
/// ```rust,ignore
/// let loc = ConfigLocation {
///     file: PathBuf::from("config.toml"),
///     line: Some(5),
///     column: Some(10),
///     key_path: "appearance.accent".to_string(),
/// };
/// ```
// TODO: Wire up ConfigLocation to ConfigError variants for rich source spans
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConfigLocation {
    /// Path to the configuration file.
    pub file: PathBuf,
    /// Line number (1-indexed), if known.
    pub line: Option<usize>,
    /// Column number (1-indexed), if known.
    pub column: Option<usize>,
    /// Dotted key path (e.g., "appearance.accent").
    pub key_path: String,
}

#[allow(dead_code)]
impl ConfigLocation {
    /// Creates a new config location.
    fn new(file: impl Into<PathBuf>, key_path: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            line: None,
            column: None,
            key_path: key_path.into(),
        }
    }

    /// Sets the line number.
    fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// Sets the column number.
    fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    /// Formats the location for display.
    fn display(&self) -> String {
        match (self.line, self.column) {
            (Some(l), Some(c)) => format!("{}:{}:{}", self.file.display(), l, c),
            (Some(l), None) => format!("{}:{}", self.file.display(), l),
            _ => self.file.display().to_string(),
        }
    }
}

/// Errors that occur during configuration parsing and validation.
///
/// `ConfigError` provides detailed, actionable error messages for configuration
/// problems. Each variant includes context about what went wrong and how to
/// fix it.
///
/// # Error Codes
///
/// Every error has a unique code (e.g., `fern::config::invalid_color`) that
/// can be looked up in documentation for detailed explanations.
///
/// # Severity Levels
///
/// Most configuration errors are [`Severity::Error`], but some are warnings:
///
/// | Variant | Severity | Behavior |
/// |---------|----------|----------|
/// | `UnknownKey` | Warning | Config loads, key ignored |
/// | `DeprecatedKey` | Warning | Config loads, warning shown |
/// | Others | Error | Config fails to load |
///
/// # Example
///
/// ```rust,ignore
/// use fern_theme::error::ConfigError;
///
/// let error = ConfigError::InvalidColor {
///     value: "#gg0000".to_string(),
///     span: None,
///     location: None,
/// };
///
/// // Rich error output with miette
/// println!("{:?}", miette::Report::new(error));
/// ```
#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum ConfigError {
    /// Invalid color format.
    ///
    /// The provided value is not a valid hex color. Colors must be in the
    /// format `#RRGGBB` or `#RRGGBBAA` where each character is a hex digit.
    ///
    /// # Valid Examples
    ///
    /// - `#89b4fa` — 6-digit hex (opaque)
    /// - `#89b4fa80` — 8-digit hex (with alpha)
    /// - `#fff` — 3-digit shorthand (expands to #ffffff)
    ///
    /// # Invalid Examples
    ///
    /// - `#gg0000` — Contains non-hex characters
    /// - `89b4fa` — Missing `#` prefix
    /// - `#89b` — Invalid length (not 3, 6, or 8)
    #[error("invalid color format: {value}")]
    #[diagnostic(code(fern::config::invalid_color))]
    InvalidColor {
        /// The invalid color value.
        value: String,
        /// Source span for highlighting.
        #[label("this color value is not valid hex")]
        span: Option<SourceSpan>,
        /// Location in config file.
        #[source_code]
        source_code: Option<String>,
    },

    /// Value is outside the valid range.
    ///
    /// Numeric values must fall within specified bounds. This error indicates
    /// the value is too large or too small.
    #[error("{key} must be between {min} and {max}, got {value}")]
    #[diagnostic(code(fern::config::out_of_range))]
    OutOfRange {
        /// The configuration key.
        key: String,
        /// The invalid value.
        value: i64,
        /// Minimum allowed value.
        min: i64,
        /// Maximum allowed value.
        max: i64,
        /// Source span for highlighting.
        #[label("value out of range")]
        span: Option<SourceSpan>,
        /// Location in config file.
        #[source_code]
        source_code: Option<String>,
    },

    /// Unknown configuration key.
    ///
    /// The configuration contains a key that is not recognized. This is a
    /// **warning**, not an error — the configuration will still load, but
    /// the unknown key will be ignored.
    ///
    /// # Common Causes
    ///
    /// - Typo in key name
    /// - Using a key from a different version
    /// - Copy-pasting from incompatible configuration
    #[error("unknown configuration key: {key}")]
    #[diagnostic(code(fern::config::unknown_key))]
    UnknownKey {
        /// The unrecognized key.
        key: String,
        /// Similar valid keys (for "did you mean?" suggestions).
        similar: Vec<String>,
        /// Source span for highlighting.
        #[label("unrecognized key")]
        span: Option<SourceSpan>,
        /// Location in config file.
        #[source_code]
        source_code: Option<String>,
    },

    /// Required field is missing.
    ///
    /// A required configuration field was not provided and has no default
    /// value.
    #[error("missing required field: {key}")]
    #[diagnostic(code(fern::config::missing_field))]
    MissingField {
        /// The missing key.
        key: String,
        /// Expected type description.
        expected_type: String,
    },

    /// Type mismatch in configuration value.
    ///
    /// The value has the wrong type. For example, a string was provided
    /// where a number was expected.
    #[error("type mismatch for {key}: expected {expected}, got {actual}")]
    #[diagnostic(code(fern::config::type_mismatch))]
    TypeMismatch {
        /// The configuration key.
        key: String,
        /// Expected type name.
        expected: String,
        /// Actual type name.
        actual: String,
        /// Source span for highlighting.
        #[label("wrong type")]
        span: Option<SourceSpan>,
        /// Location in config file.
        #[source_code]
        source_code: Option<String>,
    },

    /// TOML syntax error.
    ///
    /// The configuration file is not valid TOML. This is a **fatal** error
    /// because no configuration can be loaded.
    #[error("TOML syntax error")]
    #[diagnostic(code(fern::config::parse_error))]
    ParseError {
        /// The underlying TOML parse error.
        #[source]
        source: toml::de::Error,
        /// The source file content for error display.
        #[source_code]
        src: String,
    },

    /// Invalid theme name.
    ///
    /// The theme must be one of: "dark", "light", or "auto".
    #[error("invalid theme: {value}")]
    #[diagnostic(code(fern::config::invalid_theme))]
    InvalidTheme {
        /// The invalid theme value.
        value: String,
        /// Source span for highlighting.
        #[label("invalid theme")]
        span: Option<SourceSpan>,
        /// Location in config file.
        #[source_code]
        source_code: Option<String>,
    },

    /// Invalid bar position.
    ///
    /// The bar position must be "top" or "bottom".
    #[error("invalid bar position: {value}")]
    #[diagnostic(code(fern::config::invalid_position))]
    InvalidPosition {
        /// The invalid position value.
        value: String,
        /// Source span for highlighting.
        #[label("invalid position")]
        span: Option<SourceSpan>,
        /// Location in config file.
        #[source_code]
        source_code: Option<String>,
    },

    /// Deprecated configuration key.
    ///
    /// This key still works but will be removed in a future version.
    /// This is a **warning**.
    #[error("deprecated configuration key: {key}")]
    #[diagnostic(code(fern::config::deprecated))]
    DeprecatedKey {
        /// The deprecated key.
        key: String,
        /// The replacement key to use.
        replacement: String,
        /// Version when this key will be removed.
        removed_in: String,
        /// Source span for highlighting.
        #[label("deprecated")]
        span: Option<SourceSpan>,
        /// Location in config file.
        #[source_code]
        source_code: Option<String>,
    },

    /// Invalid font family.
    ///
    /// The specified font is not available on the system.
    #[error("font not found: {family}")]
    #[diagnostic(code(fern::config::font_not_found))]
    FontNotFound {
        /// The font family that was not found.
        family: String,
    },
}

impl ConfigError {
    /// Returns the severity level of this error.
    ///
    /// Most config errors are [`Severity::Error`], but some are warnings.
    #[must_use]
    pub fn severity(&self) -> Severity {
        match self {
            Self::UnknownKey { .. } | Self::DeprecatedKey { .. } => Severity::Warning,
            Self::ParseError { .. } => Severity::Fatal,
            _ => Severity::Error,
        }
    }

    /// Returns the error code for documentation lookup.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidColor { .. } => "fern::config::invalid_color",
            Self::OutOfRange { .. } => "fern::config::out_of_range",
            Self::UnknownKey { .. } => "fern::config::unknown_key",
            Self::MissingField { .. } => "fern::config::missing_field",
            Self::TypeMismatch { .. } => "fern::config::type_mismatch",
            Self::ParseError { .. } => "fern::config::parse_error",
            Self::InvalidTheme { .. } => "fern::config::invalid_theme",
            Self::InvalidPosition { .. } => "fern::config::invalid_position",
            Self::DeprecatedKey { .. } => "fern::config::deprecated",
            Self::FontNotFound { .. } => "fern::config::font_not_found",
        }
    }

    /// Returns a short title for notification display.
    #[must_use]
    pub fn title(&self) -> String {
        match self {
            Self::InvalidColor { .. } => "Invalid Color".to_string(),
            Self::OutOfRange { key, .. } => format!("Invalid {key}"),
            Self::UnknownKey { key, .. } => format!("Unknown Key: {key}"),
            Self::MissingField { key, .. } => format!("Missing: {key}"),
            Self::TypeMismatch { key, .. } => format!("Wrong Type: {key}"),
            Self::ParseError { .. } => "Config Parse Error".to_string(),
            Self::InvalidTheme { .. } => "Invalid Theme".to_string(),
            Self::InvalidPosition { .. } => "Invalid Position".to_string(),
            Self::DeprecatedKey { key, .. } => format!("Deprecated: {key}"),
            Self::FontNotFound { family, .. } => format!("Font Not Found: {family}"),
        }
    }
}

impl Notifiable for ConfigError {
    fn severity(&self) -> Severity {
        ConfigError::severity(self)
    }

    fn title(&self) -> String {
        ConfigError::title(self)
    }

    fn body(&self) -> String {
        self.to_string()
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::InvalidColor { .. } => {
                Some("Use hex format: #RRGGBB or #RRGGBBAA".to_string())
            }
            Self::OutOfRange { min, max, .. } => {
                Some(format!("Value must be between {min} and {max}"))
            }
            Self::UnknownKey { similar, .. } if !similar.is_empty() => {
                Some(format!("Did you mean: {}?", similar.join(", ")))
            }
            Self::MissingField { expected_type, .. } => {
                Some(format!("Expected type: {expected_type}"))
            }
            Self::TypeMismatch { expected, .. } => {
                Some(format!("Change the value to be a {expected}"))
            }
            Self::InvalidTheme { .. } => {
                Some("Theme must be one of: dark, light, auto".to_string())
            }
            Self::InvalidPosition { .. } => {
                Some("Position must be one of: top, bottom".to_string())
            }
            Self::DeprecatedKey { replacement, removed_in, .. } => {
                Some(format!("Use `{replacement}` instead (removing in {removed_in})"))
            }
            Self::FontNotFound { .. } => {
                Some("Install the font or use a different family".to_string())
            }
            _ => None,
        }
    }

    fn code(&self) -> Option<&'static str> {
        Some(ConfigError::code(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_color_has_correct_severity() {
        let err = ConfigError::InvalidColor {
            value: "#gg0000".to_string(),
            span: None,
            source_code: None,
        };
        assert_eq!(err.severity(), Severity::Error);
    }

    #[test]
    fn unknown_key_is_warning() {
        let err = ConfigError::UnknownKey {
            key: "colour".to_string(),
            similar: vec!["color".to_string()],
            span: None,
            source_code: None,
        };
        assert_eq!(err.severity(), Severity::Warning);
    }

    #[test]
    fn parse_error_is_fatal() {
        let toml_err = toml::from_str::<toml::Value>("invalid toml [[[")
            .expect_err("should fail");
        let err = ConfigError::ParseError {
            source: toml_err,
            src: "invalid toml [[[".to_string(),
        };
        assert_eq!(err.severity(), Severity::Fatal);
    }

    #[test]
    fn notifiable_suggestion() {
        let err = ConfigError::UnknownKey {
            key: "colour".to_string(),
            similar: vec!["color".to_string()],
            span: None,
            source_code: None,
        };
        assert_eq!(err.suggestion(), Some("Did you mean: color?".to_string()));
    }
}
