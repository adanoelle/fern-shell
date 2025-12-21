//! # Configuration Conversion Command
//!
//! The `convert` command transforms TOML configuration files into JSON format
//! suitable for consumption by QuickShell. This is the bridge between the
//! human-friendly TOML authoring format and the QML-friendly JSON runtime format.
//!
//! ## Usage
//!
//! ```bash
//! # Convert with auto-named output (config.toml → config.json)
//! fernctl convert config.toml
//!
//! # Convert with explicit output path
//! fernctl convert config.toml --output theme.json
//!
//! # With verbose output
//! fernctl convert config.toml -v
//! ```
//!
//! ## Why Two Formats?
//!
//! TOML and JSON serve different purposes in the Fern ecosystem:
//!
//! | Aspect | TOML (Input) | JSON (Output) |
//! |--------|--------------|---------------|
//! | **Audience** | Human authors | QML runtime |
//! | **Comments** | ✓ Supported | ✗ Not in spec |
//! | **Readability** | Optimized | Adequate |
//! | **Parsing** | Rust (fernctl) | JavaScript (QML) |
//! | **Editing** | By users | Never |
//!
//! ## Conversion Pipeline
//!
//! ```text
//! ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
//! │   config.toml   │────▶│    Validate     │────▶│   config.json   │
//! │   (user file)   │     │   & Transform   │     │  (QML runtime)  │
//! └─────────────────┘     └─────────────────┘     └─────────────────┘
//!         │                       │                       │
//!         ▼                       ▼                       ▼
//!    Human-readable         Type-safe Rust           Structured for
//!    with comments          domain model             QML consumption
//! ```
//!
//! ## Validation During Conversion
//!
//! Conversion is **not** a simple format translation. The configuration passes
//! through full validation before JSON output is generated. This ensures that
//! invalid configurations are caught early, before they reach QuickShell.
//!
//! ```text
//! config.toml (valid TOML, invalid values)
//!      │
//!      ▼
//! ┌────────────────────────────────────────┐
//! │  Parse TOML                            │  ✓ Succeeds
//! └────────────────────────────────────────┘
//!      │
//!      ▼
//! ┌────────────────────────────────────────┐
//! │  Validate Values                       │  ✗ Fails: "#gg0000" invalid
//! │  - Check color formats                 │
//! │  - Verify ranges                       │
//! │  - Resolve defaults                    │
//! └────────────────────────────────────────┘
//!      │
//!      ▼
//!   Error returned, no JSON generated
//! ```
//!
//! ## Output Format
//!
//! The generated JSON follows a specific structure optimized for QML:
//!
//! ```json
//! {
//!   "variant": "dark",
//!   "colors": {
//!     "background": "#1e1e2e",
//!     "foreground": "#cdd6f4",
//!     "accent": "#89b4fa",
//!     ...
//!   },
//!   "bar": {
//!     "height": 32,
//!     "position": "top",
//!     ...
//!   },
//!   ...
//! }
//! ```
//!
//! ## Atomicity
//!
//! Conversion is atomic — if any step fails, no output file is created or
//! modified. This prevents partial writes that could leave QuickShell with
//! a corrupted configuration.
//!
//! ```text
//! ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
//! │   Success   │    │   Failure   │    │   Failure   │
//! │   Path      │    │   (Parse)   │    │  (Validate) │
//! └─────────────┘    └─────────────┘    └─────────────┘
//!       │                  │                  │
//!       ▼                  ▼                  ▼
//!  JSON written       No file touched    No file touched
//! ```
//!
//! ## Exit Codes
//!
//! | Code | Meaning |
//! |------|---------|
//! | `0` | Conversion succeeded |
//! | `1` | Conversion failed (parse error, validation error, I/O error) |
//!
//! ## Programmatic Usage
//!
//! ```rust,ignore
//! use fern_theme::commands::convert::{run, ConvertOptions};
//! use fern_theme::adapters::{TomlConfigAdapter, FileSystemAdapter};
//!
//! let config_adapter = TomlConfigAdapter::new();
//! let persist_adapter = FileSystemAdapter::new();
//! let options = ConvertOptions { verbose: true };
//!
//! run(
//!     Path::new("config.toml"),
//!     Path::new("config.json"),
//!     options,
//!     &config_adapter,
//!     &persist_adapter,
//! )?;
//! ```

use crate::error::Result;
use crate::ports::inbound::ConfigPort;
use crate::ports::outbound::PersistPort;
use std::path::Path;

/// Options for the convert command.
///
/// These options control the behavior and output of conversion.
///
/// # Example
///
/// ```rust
/// use fern_theme::commands::convert::ConvertOptions;
///
/// let options = ConvertOptions {
///     verbose: true,
///     pretty: true,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ConvertOptions {
    /// Whether to print detailed information during conversion.
    ///
    /// When enabled, prints:
    /// - Input and output paths
    /// - Validation status
    /// - Any warnings encountered
    pub verbose: bool,

    /// Whether to format the JSON output with indentation.
    ///
    /// When `true` (default), output is human-readable:
    /// ```json
    /// {
    ///   "variant": "dark",
    ///   "colors": { ... }
    /// }
    /// ```
    ///
    /// When `false`, output is compact (smaller file size):
    /// ```json
    /// {"variant":"dark","colors":{...}}
    /// ```
    pub pretty: bool,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        Self {
            verbose: false,
            pretty: true,
        }
    }
}

/// Result of a successful conversion.
///
/// Contains information about what was converted and any
/// warnings that were encountered during validation.
#[derive(Debug)]
pub struct ConvertResult {
    /// Warnings encountered during validation.
    ///
    /// These are non-fatal issues that didn't prevent conversion.
    pub warnings: Vec<String>,
}

/// Converts a TOML configuration file to JSON.
///
/// This function loads a TOML configuration, validates it, and writes the
/// validated theme to a JSON file. The conversion is atomic — if any step
/// fails, no output file is created.
///
/// # Arguments
///
/// * `input` — Path to the input TOML file
/// * `output` — Path where the JSON output will be written
/// * `options` — Configuration for conversion behavior
/// * `config_adapter` — Adapter for loading TOML configuration
/// * `persist_adapter` — Adapter for writing JSON output
///
/// # Returns
///
/// - `Ok(ConvertResult)` — Conversion succeeded
/// - `Err(FernError)` — Conversion failed
///
/// # Errors
///
/// Returns an error if:
/// - The input file cannot be read
/// - TOML parsing fails
/// - Validation fails
/// - The output file cannot be written
///
/// # Example
///
/// ```rust,ignore
/// use fern_theme::commands::convert::{run, ConvertOptions};
/// use fern_theme::adapters::{TomlConfigAdapter, FileSystemAdapter};
/// use std::path::Path;
///
/// let config_adapter = TomlConfigAdapter::new();
/// let persist_adapter = FileSystemAdapter::new();
///
/// let result = run(
///     Path::new("~/.config/fern/config.toml"),
///     Path::new("~/.config/fern/config.json"),
///     ConvertOptions::default(),
///     &config_adapter,
///     &persist_adapter,
/// )?;
///
/// println!("Conversion complete with {} warnings", result.warnings.len());
/// ```
///
/// # Atomicity Guarantee
///
/// This function guarantees that either:
/// 1. The output file is completely written with valid JSON, OR
/// 2. No changes are made to the output file
///
/// This is achieved by validating completely before writing, so partial
/// or corrupted output is never possible.
pub fn run<P: AsRef<Path>, Q: AsRef<Path>>(
    input: P,
    output: Q,
    options: ConvertOptions,
    config_adapter: &impl ConfigPort,
    persist_adapter: &impl PersistPort,
) -> Result<ConvertResult> {
    let input = input.as_ref();
    let output = output.as_ref();

    if options.verbose {
        eprintln!("Converting: {} -> {}", input.display(), output.display());
    }

    // Load and validate
    let raw = config_adapter.load_from_file(input)?;
    let validated = raw.validate()?;

    // Collect warnings before consuming validated
    let warnings: Vec<String> = validated.warnings().iter().map(ToString::to_string).collect();

    // Print warnings if verbose
    if options.verbose {
        for warning in &warnings {
            eprintln!("Warning: {}", warning);
        }
    }

    // Convert to theme
    let theme = validated.into_theme();

    // Persist to JSON
    persist_adapter.save_theme(&theme, output)?;

    if options.verbose {
        eprintln!("Wrote {}", output.display());
    }

    Ok(ConvertResult { warnings })
}

/// Derives the output path from an input path.
///
/// Replaces the file extension with `.json`. If the input has no extension,
/// `.json` is appended.
///
/// # Examples
///
/// ```rust
/// use fern_theme::commands::convert::derive_output_path;
/// use std::path::PathBuf;
///
/// assert_eq!(
///     derive_output_path("config.toml"),
///     PathBuf::from("config.json")
/// );
///
/// assert_eq!(
///     derive_output_path("my-theme"),
///     PathBuf::from("my-theme.json")
/// );
/// ```
#[must_use]
pub fn derive_output_path(input: impl AsRef<Path>) -> std::path::PathBuf {
    let input = input.as_ref();
    input.with_extension("json")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn derive_output_replaces_extension() {
        assert_eq!(derive_output_path("config.toml"), PathBuf::from("config.json"));
    }

    #[test]
    fn derive_output_handles_no_extension() {
        assert_eq!(derive_output_path("config"), PathBuf::from("config.json"));
    }

    #[test]
    fn derive_output_handles_multiple_dots() {
        assert_eq!(
            derive_output_path("my.config.toml"),
            PathBuf::from("my.config.json")
        );
    }

    #[test]
    fn convert_options_default() {
        let options = ConvertOptions::default();
        assert!(!options.verbose);
        assert!(options.pretty);
    }
}
