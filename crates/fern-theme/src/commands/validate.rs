//! # Configuration Validation Command
//!
//! The `validate` command checks a configuration file for correctness without
//! producing any output files. It performs both syntactic validation (is it
//! valid TOML?) and semantic validation (are the values sensible?).
//!
//! ## Usage
//!
//! ```bash
//! # Validate default config location
//! fernctl validate
//!
//! # Validate specific file
//! fernctl validate --config ./my-config.toml
//!
//! # With verbose output
//! fernctl validate -v
//! ```
//!
//! ## Validation Stages
//!
//! Validation proceeds in three stages:
//!
//! ```text
//! ┌─────────────┐    ┌──────────────┐    ┌────────────────┐
//! │   Stage 1   │───▶│   Stage 2    │───▶│    Stage 3     │
//! │   Parsing   │    │  Type Check  │    │   Semantics    │
//! └─────────────┘    └──────────────┘    └────────────────┘
//!     TOML syntax       Value types        Business rules
//!     Well-formed       Color formats      Range limits
//!     Structure         Required fields    Consistency
//! ```
//!
//! ### Stage 1: Parsing
//!
//! Checks that the file is syntactically valid TOML:
//!
//! - Proper key-value syntax
//! - Balanced brackets and quotes
//! - Valid escape sequences
//!
//! **Errors at this stage:**
//! - `fern::config::parse_error` — Invalid TOML syntax
//!
//! ### Stage 2: Type Checking
//!
//! Validates that values have the expected types:
//!
//! - Numbers where numbers are expected
//! - Strings where strings are expected
//! - Required fields are present
//!
//! **Errors at this stage:**
//! - `fern::config::type_mismatch` — Wrong value type
//! - `fern::config::missing_field` — Required field absent
//!
//! ### Stage 3: Semantic Validation
//!
//! Checks business rules and constraints:
//!
//! - Color values are valid hex (`#RRGGBB` or `#RRGGBBAA`)
//! - Numbers are within allowed ranges
//! - Enum values are recognized (`"top"`, `"bottom"`, etc.)
//!
//! **Errors at this stage:**
//! - `fern::config::invalid_color` — Malformed color value
//! - `fern::config::out_of_range` — Number outside limits
//! - `fern::config::unknown_key` — Unrecognized configuration key
//!
//! ## Warnings vs Errors
//!
//! Not all problems are fatal. The validator distinguishes:
//!
//! | Severity | Behavior | Example |
//! |----------|----------|---------|
//! | **Error** | Validation fails | Invalid color format |
//! | **Warning** | Validation passes, issue reported | Unknown key |
//!
//! Warnings allow forward compatibility — new config files may have keys
//! that older `fernctl` versions don't recognize.
//!
//! ## Exit Codes
//!
//! | Code | Meaning |
//! |------|---------|
//! | `0` | Validation passed (possibly with warnings) |
//! | `1` | Validation failed with errors |
//!
//! ## Programmatic Usage
//!
//! ```rust,ignore
//! use fern_theme::commands::validate::{run, ValidateOptions};
//! use fern_theme::adapters::TomlConfigAdapter;
//!
//! let adapter = TomlConfigAdapter::new();
//! let options = ValidateOptions { verbose: true };
//!
//! match run(&path, options, &adapter) {
//!     Ok(result) => {
//!         println!("Valid! {} warnings", result.warnings.len());
//!     }
//!     Err(e) => {
//!         eprintln!("Invalid: {}", e);
//!     }
//! }
//! ```

use crate::domain::theme::Theme;
use crate::error::Result;
use crate::ports::inbound::ConfigPort;
use std::path::Path;

/// Options for the validate command.
///
/// These options control the behavior and output of validation.
///
/// # Example
///
/// ```rust
/// use fern_theme::commands::validate::ValidateOptions;
///
/// let options = ValidateOptions {
///     verbose: true,
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct ValidateOptions {
    /// Whether to print detailed information during validation.
    ///
    /// When enabled, prints:
    /// - The file being validated
    /// - Theme variant detected
    /// - Key configuration values
    pub verbose: bool,
}

/// Result of a successful validation.
///
/// Contains the validated theme and any warnings encountered.
/// Warnings indicate non-fatal issues that don't prevent the
/// configuration from being used.
///
/// # Example
///
/// ```rust,ignore
/// let result = validate::run(&path, options, &adapter)?;
///
/// for warning in &result.warnings {
///     eprintln!("Warning: {}", warning);
/// }
///
/// // Use the validated theme
/// let theme = result.theme;
/// ```
#[derive(Debug)]
pub struct ValidateResult {
    /// The validated theme.
    pub theme: Theme,

    /// Non-fatal warnings encountered during validation.
    ///
    /// Common warnings include:
    /// - Unknown configuration keys (possibly typos)
    /// - Deprecated keys (still work, but will be removed)
    pub warnings: Vec<String>,
}

/// Validates a configuration file.
///
/// This function performs comprehensive validation of a TOML configuration
/// file, checking both syntax and semantics. It returns a [`ValidateResult`]
/// containing the parsed theme and any warnings.
///
/// # Arguments
///
/// * `path` — Path to the configuration file to validate
/// * `options` — Configuration for validation behavior
/// * `adapter` — The configuration parser to use
///
/// # Returns
///
/// - `Ok(ValidateResult)` — Configuration is valid
/// - `Err(FernError)` — Configuration has errors
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read ([`FernError::Io`](crate::error::FernError::Io))
/// - TOML syntax is invalid ([`ConfigError::ParseError`](crate::error::ConfigError::ParseError))
/// - Values fail validation ([`ConfigError::InvalidColor`](crate::error::ConfigError::InvalidColor), etc.)
///
/// # Example
///
/// ```rust,ignore
/// use fern_theme::commands::validate::{run, ValidateOptions};
/// use fern_theme::adapters::TomlConfigAdapter;
/// use std::path::Path;
///
/// let adapter = TomlConfigAdapter::new();
/// let options = ValidateOptions::default();
///
/// match run(Path::new("config.toml"), options, &adapter) {
///     Ok(result) => {
///         println!("Configuration is valid!");
///         println!("Theme variant: {}", result.theme.variant.name());
///
///         if !result.warnings.is_empty() {
///             println!("\nWarnings:");
///             for warning in &result.warnings {
///                 println!("  - {}", warning);
///             }
///         }
///     }
///     Err(e) => {
///         // Error includes rich diagnostic information
///         eprintln!("{:?}", miette::Report::new(e));
///         std::process::exit(1);
///     }
/// }
/// ```
pub fn run<P: AsRef<Path>>(
    path: P,
    options: ValidateOptions,
    adapter: &impl ConfigPort,
) -> Result<ValidateResult> {
    let path = path.as_ref();

    if options.verbose {
        eprintln!("Validating: {}", path.display());
    }

    // Load and parse the configuration
    let raw = adapter.load_from_file(path)?;

    // Validate the parsed configuration
    let validated = raw.validate()?;

    // Collect warnings
    let warnings: Vec<String> = validated.warnings().iter().map(ToString::to_string).collect();

    // Convert to theme
    let theme = validated.into_theme();

    if options.verbose {
        eprintln!("Theme variant: {}", theme.variant.name());
        eprintln!("Bar height: {}px", theme.bar.height);
        eprintln!("Bar position: {}", theme.bar.position.name());
    }

    Ok(ValidateResult { theme, warnings })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_options_default() {
        let options = ValidateOptions::default();
        assert!(!options.verbose);
    }
}
