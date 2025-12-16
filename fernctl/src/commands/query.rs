//! # Theme Query Command
//!
//! The `query` command retrieves specific values from a configuration file.
//! This is useful for shell scripts, debugging, and integration with other tools.
//!
//! ## Usage
//!
//! ```bash
//! # Query a specific value
//! fernctl query colors.background
//!
//! # Query from a specific config file
//! fernctl query bar.height --config ./my-config.toml
//!
//! # Use in shell scripts
//! ACCENT=$(fernctl query colors.accent)
//! echo "Accent color is: $ACCENT"
//! ```
//!
//! ## Query Paths
//!
//! Values are addressed using dot-separated paths that mirror the configuration
//! structure:
//!
//! ```text
//! config.toml                        Query Path
//! ───────────                        ──────────
//! [colors]
//! background = "#1e1e2e"      →     colors.background
//! accent = "#89b4fa"          →     colors.accent
//!
//! [bar]
//! height = 32                 →     bar.height
//! position = "top"            →     bar.position
//!
//! [typography]
//! family = "Inter"            →     typography.family
//! ```
//!
//! ## Available Paths
//!
//! ### Theme
//!
//! | Path | Type | Description |
//! |------|------|-------------|
//! | `variant` | string | Theme variant ("dark" or "light") |
//!
//! ### Colors
//!
//! | Path | Type | Description |
//! |------|------|-------------|
//! | `colors.background` | hex | Primary background color |
//! | `colors.foreground` | hex | Primary text color |
//! | `colors.accent` | hex | Accent/highlight color |
//! | `colors.surface` | hex | Surface/card background |
//! | `colors.error` | hex | Error state color |
//! | `colors.warning` | hex | Warning state color |
//! | `colors.success` | hex | Success state color |
//! | `colors.info` | hex | Informational state color |
//!
//! ### Bar
//!
//! | Path | Type | Description |
//! |------|------|-------------|
//! | `bar.height` | integer | Bar height in pixels |
//! | `bar.position` | string | Bar position ("top" or "bottom") |
//!
//! ### Typography
//!
//! | Path | Type | Description |
//! |------|------|-------------|
//! | `typography.family` | string | Primary font family |
//! | `typography.mono` | string | Monospace font family |
//!
//! ### Radius
//!
//! | Path | Type | Description |
//! |------|------|-------------|
//! | `radius.sm` | integer | Small border radius (px) |
//! | `radius.md` | integer | Medium border radius (px) |
//! | `radius.lg` | integer | Large border radius (px) |
//!
//! ## Output Format
//!
//! Query results are printed to stdout without any decoration, making them
//! suitable for shell script consumption:
//!
//! ```bash
//! $ fernctl query colors.accent
//! #89b4fa
//!
//! $ fernctl query bar.height
//! 32
//! ```
//!
//! ## Error Handling
//!
//! If a path is not recognized, the command prints available paths to stderr
//! and exits with code 0 (this is intentional — unknown paths are not errors,
//! they might be from a newer config version).
//!
//! ```bash
//! $ fernctl query colors.prple
//! Unknown path: colors.prple
//!
//! Available paths:
//!   colors.background, colors.foreground, colors.accent, ...
//!
//! Did you mean: colors.purple?
//! ```
//!
//! ## Programmatic Usage
//!
//! ```rust,ignore
//! use fernctl::commands::query::{run, QueryOptions};
//! use fernctl::adapters::TomlConfigAdapter;
//!
//! let adapter = TomlConfigAdapter::new();
//! let options = QueryOptions { verbose: false };
//!
//! if let Some(value) = run(&config_path, "colors.accent", options, &adapter)? {
//!     println!("Accent: {}", value);
//! }
//! ```

use crate::error::Result;
use crate::ports::inbound::ConfigPort;
use std::path::Path;

/// Options for the query command.
///
/// # Example
///
/// ```rust
/// use fernctl::commands::query::QueryOptions;
///
/// let options = QueryOptions {
///     verbose: true,
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct QueryOptions {
    /// Whether to print additional context about the query.
    pub verbose: bool,
}

/// All available query paths.
///
/// This constant is used for help text and "did you mean?" suggestions.
pub const AVAILABLE_PATHS: &[&str] = &[
    "variant",
    "bar.height",
    "bar.position",
    "colors.background",
    "colors.foreground",
    "colors.accent",
    "colors.surface",
    "colors.error",
    "colors.warning",
    "colors.success",
    "colors.info",
    "typography.family",
    "typography.mono",
    "radius.sm",
    "radius.md",
    "radius.lg",
];

/// Queries a theme value by path.
///
/// Loads the configuration, validates it, and extracts the value at the
/// specified path. Returns `None` if the path is not recognized.
///
/// # Arguments
///
/// * `config_path` — Path to the configuration file
/// * `query_path` — Dot-separated path to the value (e.g., "colors.accent")
/// * `options` — Query options
/// * `adapter` — Configuration adapter for loading TOML
///
/// # Returns
///
/// - `Ok(Some(value))` — Path found, value returned as string
/// - `Ok(None)` — Path not recognized
/// - `Err(FernError)` — Configuration error
///
/// # Example
///
/// ```rust,ignore
/// use fernctl::commands::query::{run, QueryOptions};
/// use fernctl::adapters::TomlConfigAdapter;
///
/// let adapter = TomlConfigAdapter::new();
/// let options = QueryOptions::default();
///
/// match run(&config_path, "colors.accent", options, &adapter)? {
///     Some(value) => println!("{}", value),
///     None => eprintln!("Unknown path"),
/// }
/// ```
pub fn run<P: AsRef<Path>>(
    config_path: P,
    query_path: &str,
    options: QueryOptions,
    adapter: &impl ConfigPort,
) -> Result<Option<String>> {
    let config_path = config_path.as_ref();

    if options.verbose {
        eprintln!("Querying: {} from {}", query_path, config_path.display());
    }

    // Load and validate configuration
    let raw = adapter.load_from_file(config_path)?;
    let validated = raw.validate()?;
    let theme = validated.into_theme();

    // Extract value by path
    let value = match query_path {
        "variant" => Some(theme.variant.name().to_string()),
        "bar.height" => Some(theme.bar.height.to_string()),
        "bar.position" => Some(theme.bar.position.name().to_string()),
        "colors.background" => Some(theme.colors.background.to_hex()),
        "colors.foreground" => Some(theme.colors.foreground.to_hex()),
        "colors.accent" => Some(theme.colors.accent.to_hex()),
        "colors.surface" => Some(theme.colors.surface.to_hex()),
        "colors.error" => Some(theme.colors.error.to_hex()),
        "colors.warning" => Some(theme.colors.warning.to_hex()),
        "colors.success" => Some(theme.colors.success.to_hex()),
        "colors.info" => Some(theme.colors.info.to_hex()),
        "typography.family" => Some(theme.typography.family.name().to_string()),
        "typography.mono" => Some(theme.typography.mono.name().to_string()),
        "radius.sm" => Some(theme.radius.sm.to_string()),
        "radius.md" => Some(theme.radius.md.to_string()),
        "radius.lg" => Some(theme.radius.lg.to_string()),
        _ => None,
    };

    Ok(value)
}

/// Finds similar paths for "did you mean?" suggestions.
///
/// Uses string similarity to find paths that are close to the given
/// unknown path, helping users correct typos.
///
/// # Arguments
///
/// * `unknown` — The unrecognized path
///
/// # Returns
///
/// A vector of similar paths, sorted by similarity (most similar first).
/// Returns at most 3 suggestions.
///
/// # Example
///
/// ```rust
/// use fernctl::commands::query::find_similar_paths;
///
/// let suggestions = find_similar_paths("colors.backgroud");
/// assert!(suggestions.contains(&"colors.background".to_string()));
/// ```
#[must_use]
pub fn find_similar_paths(unknown: &str) -> Vec<String> {
    let mut candidates: Vec<(f64, &str)> = AVAILABLE_PATHS
        .iter()
        .map(|&path| (strsim::jaro_winkler(unknown, path), path))
        .filter(|(score, _)| *score > 0.6)
        .collect();

    candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    candidates
        .into_iter()
        .take(3)
        .map(|(_, path)| path.to_string())
        .collect()
}

/// Prints help text showing all available query paths.
///
/// This is called when an unknown path is queried, providing guidance
/// to the user about what paths are valid.
pub fn print_available_paths() {
    eprintln!("\nAvailable paths:");
    eprintln!("  variant");
    eprintln!("  bar.height, bar.position");
    eprintln!(
        "  colors.background, colors.foreground, colors.accent, colors.surface"
    );
    eprintln!("  colors.error, colors.warning, colors.success, colors.info");
    eprintln!("  typography.family, typography.mono");
    eprintln!("  radius.sm, radius.md, radius.lg");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_similar_detects_typo() {
        let suggestions = find_similar_paths("colors.backgroud");
        assert!(suggestions.contains(&"colors.background".to_string()));
    }

    #[test]
    fn find_similar_returns_empty_for_gibberish() {
        let suggestions = find_similar_paths("xyzzy123");
        assert!(suggestions.is_empty());
    }

    #[test]
    fn find_similar_limits_to_three() {
        let suggestions = find_similar_paths("colors");
        assert!(suggestions.len() <= 3);
    }
}
