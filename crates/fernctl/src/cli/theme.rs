//! # Theme CLI Commands
//!
//! Theme management commands for fernctl.
//!
//! ## Available Commands
//!
//! - `theme apply <name>` - Apply a theme preset
//! - `theme validate` - Validate current configuration
//! - `theme current` - Show current theme info
//! - `theme query <path>` - Query specific theme values

use crate::error::{FernctlError, Result};
use fern_core::FernPaths;
use fern_theme::adapters::{FileSystemAdapter, TomlConfigAdapter};
use fern_theme::commands::{convert, query, validate};

/// Theme action to perform.
#[derive(Debug, Clone)]
pub enum ThemeAction {
    /// Apply a theme by name.
    Apply {
        /// Theme name to apply.
        name: String,
    },
    /// Validate the current configuration.
    Validate,
    /// Show current theme information.
    Current,
    /// Query a specific theme value.
    Query {
        /// Path to query (e.g., colors.background).
        path: String,
    },
}

/// Options for theme commands.
#[derive(Debug, Clone)]
pub struct ThemeOptions {
    /// Theme action to perform.
    pub action: ThemeAction,
    /// Whether to output verbose information.
    pub verbose: bool,
    /// Whether to output as JSON.
    pub json: bool,
}

/// Runs the theme command.
///
/// # Errors
///
/// Returns an error if the theme operation fails.
pub fn run(options: ThemeOptions) -> Result<()> {
    match options.action {
        ThemeAction::Apply { name } => run_apply(&name, options.verbose),
        ThemeAction::Validate => run_validate(options.verbose),
        ThemeAction::Current => run_current(options.json),
        ThemeAction::Query { path } => run_query(&path, options.verbose),
    }
}

/// Applies a theme by name.
fn run_apply(name: &str, verbose: bool) -> Result<()> {
    let paths = FernPaths::new();
    let config_toml = paths.config_toml();
    let config_json = paths.config_json();

    if verbose {
        eprintln!("Config TOML: {}", config_toml.display());
        eprintln!("Config JSON: {}", config_json.display());
    }

    // Check if config file exists
    if !config_toml.exists() {
        return Err(FernctlError::config(format!(
            "Config file not found: {}. Create a config.toml first.",
            config_toml.display()
        )));
    }

    // Read the current config
    let content = std::fs::read_to_string(&config_toml)
        .map_err(|e| FernctlError::io("reading config", e))?;

    // Update the theme in the config
    let updated = update_theme_in_toml(&content, name)?;

    // Write back the config
    std::fs::write(&config_toml, &updated)
        .map_err(|e| FernctlError::io("writing config", e))?;

    if verbose {
        eprintln!("Updated config with theme: {}", name);
    }

    // Convert to JSON
    let config_adapter = TomlConfigAdapter::new();
    let persist_adapter = FileSystemAdapter::new();
    let convert_options = convert::ConvertOptions {
        verbose,
        pretty: true,
    };

    convert::run(&config_toml, &config_json, convert_options, &config_adapter, &persist_adapter)
        .map_err(|e| FernctlError::config(format!("Failed to convert config: {}", e)))?;

    println!("Theme applied: {}", name);
    println!("Config written to: {}", config_json.display());

    Ok(())
}

/// Updates the theme value in a TOML config string.
fn update_theme_in_toml(content: &str, theme: &str) -> Result<String> {
    // Parse the TOML
    let mut doc = content.parse::<toml_edit::DocumentMut>()
        .map_err(|e| FernctlError::parse("config TOML", e.to_string()))?;

    // Ensure appearance section exists
    if doc.get("appearance").is_none() {
        doc["appearance"] = toml_edit::Item::Table(toml_edit::Table::new());
    }

    // Set the theme
    doc["appearance"]["theme"] = toml_edit::value(theme);

    Ok(doc.to_string())
}

/// Validates the current configuration.
fn run_validate(verbose: bool) -> Result<()> {
    let paths = FernPaths::new();
    let config_toml = paths.config_toml();

    if !config_toml.exists() {
        return Err(FernctlError::config(format!(
            "Config file not found: {}",
            config_toml.display()
        )));
    }

    let adapter = TomlConfigAdapter::new();
    let options = validate::ValidateOptions { verbose };

    match validate::run(&config_toml, options, &adapter) {
        Ok(result) => {
            for warning in &result.warnings {
                eprintln!("\x1b[33mWarning:\x1b[0m {}", warning);
            }
            println!("\x1b[32m✓\x1b[0m Configuration is valid.");
            if verbose {
                println!("Theme variant: {}", result.theme.variant.name());
                println!("Bar height: {}px", result.theme.bar.height);
                println!("Bar position: {}", result.theme.bar.position.name());
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("\x1b[31m✗\x1b[0m Configuration is invalid.");
            Err(FernctlError::config(format!("{}", e)))
        }
    }
}

/// Shows the current theme information.
fn run_current(json: bool) -> Result<()> {
    let paths = FernPaths::new();
    let config_json = paths.config_json();

    if !config_json.exists() {
        return Err(FernctlError::config(format!(
            "Config JSON not found: {}. Run 'fernctl theme validate' first.",
            config_json.display()
        )));
    }

    let content = std::fs::read_to_string(&config_json)
        .map_err(|e| FernctlError::io("reading config JSON", e))?;

    if json {
        println!("{}", content);
        return Ok(());
    }

    // Parse and display key values
    let config: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| FernctlError::parse("config JSON", e.to_string()))?;

    println!("Current Theme Configuration");
    println!("===========================");

    // Appearance
    if let Some(appearance) = config.get("appearance") {
        if let Some(theme) = appearance.get("theme").and_then(|v| v.as_str()) {
            println!("Theme: {}", theme);
        }
        if let Some(variant) = appearance.get("variant").and_then(|v| v.as_str()) {
            println!("Variant: {}", variant);
        }
    }

    // Colors
    if let Some(colors) = config.get("colors") {
        println!("\nColors:");
        if let Some(bg) = colors.get("background").and_then(|v| v.as_str()) {
            println!("  Background: {}", bg);
        }
        if let Some(fg) = colors.get("foreground").and_then(|v| v.as_str()) {
            println!("  Foreground: {}", fg);
        }
        if let Some(accent) = colors.get("accent").and_then(|v| v.as_str()) {
            println!("  Accent: {}", accent);
        }
    }

    // Bar
    if let Some(bar) = config.get("bar") {
        println!("\nBar:");
        if let Some(height) = bar.get("height").and_then(|v| v.as_i64()) {
            println!("  Height: {}px", height);
        }
        if let Some(position) = bar.get("position").and_then(|v| v.as_str()) {
            println!("  Position: {}", position);
        }
    }

    Ok(())
}

/// Queries a specific theme value.
fn run_query(path: &str, verbose: bool) -> Result<()> {
    let paths = FernPaths::new();
    let config_toml = paths.config_toml();

    if !config_toml.exists() {
        return Err(FernctlError::config(format!(
            "Config file not found: {}",
            config_toml.display()
        )));
    }

    let adapter = TomlConfigAdapter::new();
    let options = query::QueryOptions { verbose };

    match query::run(&config_toml, path, options, &adapter)? {
        Some(value) => {
            println!("{}", value);
            Ok(())
        }
        None => {
            eprintln!("Unknown path: {}", path);
            query::print_available_paths();

            let suggestions = query::find_similar_paths(path);
            if !suggestions.is_empty() {
                eprintln!("\nDid you mean: {}?", suggestions.join(", "));
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_theme_in_toml_new_section() {
        let content = "";
        let result = update_theme_in_toml(content, "catppuccin").unwrap();
        assert!(result.contains("[appearance]"));
        assert!(result.contains("theme = \"catppuccin\""));
    }

    #[test]
    fn test_update_theme_in_toml_existing() {
        let content = r#"
[appearance]
theme = "old-theme"
variant = "mocha"
"#;
        let result = update_theme_in_toml(content, "catppuccin").unwrap();
        assert!(result.contains("theme = \"catppuccin\""));
        assert!(result.contains("variant = \"mocha\""));
    }
}
