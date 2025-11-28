//! # fernctl CLI
//!
//! Command-line interface for Fern Shell configuration management.
//!
//! ## Usage
//!
//! ```bash
//! # Validate configuration
//! fernctl validate
//!
//! # Convert TOML to JSON
//! fernctl convert config.toml -o config.json
//!
//! # Query theme values
//! fernctl query colors.background
//!
//! # Reload QuickShell theme
//! fernctl reload
//! ```

use clap::{Parser, Subcommand};
use fernctl::adapters::{FileSystemAdapter, TomlConfigAdapter};
use fernctl::error::Result;
use fernctl::ports::inbound::ConfigPort;
use fernctl::ports::outbound::PersistPort;
use std::path::PathBuf;

/// fernctl - Fern Shell Configuration Manager
///
/// A command-line tool for managing Fern Shell configuration files.
/// Validates TOML configuration, converts to JSON for QuickShell,
/// and provides theme query capabilities.
#[derive(Parser, Debug)]
#[command(name = "fernctl")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

/// Available commands
#[derive(Subcommand, Debug)]
enum Commands {
    /// Validate a configuration file
    Validate {
        /// Path to the configuration file (default: ~/.config/fern/config.toml)
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    /// Convert TOML configuration to JSON
    Convert {
        /// Input TOML file
        input: PathBuf,

        /// Output JSON file (default: same name with .json extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Query a theme value by path
    Query {
        /// The path to query (e.g., colors.background, bar.height)
        path: String,

        /// Path to the configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    /// Show the default theme values
    Defaults {
        /// Theme variant (dark, light)
        #[arg(default_value = "dark")]
        variant: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { config } => {
            let path = config.unwrap_or_else(default_config_path);
            validate_config(&path, cli.verbose)?;
        }
        Commands::Convert { input, output } => {
            let output = output.unwrap_or_else(|| input.with_extension("json"));
            convert_config(&input, &output, cli.verbose)?;
        }
        Commands::Query { path, config } => {
            let config_path = config.unwrap_or_else(default_config_path);
            query_theme(&config_path, &path, cli.verbose)?;
        }
        Commands::Defaults { variant } => {
            show_defaults(&variant)?;
        }
    }

    Ok(())
}

/// Returns the default configuration file path.
fn default_config_path() -> PathBuf {
    dirs::config_dir()
        .map(|p| p.join("fern").join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("config.toml"))
}

/// Validates a configuration file.
fn validate_config(path: &PathBuf, verbose: bool) -> Result<()> {
    if verbose {
        eprintln!("Validating: {}", path.display());
    }

    let adapter = TomlConfigAdapter::new();
    let raw = adapter.load_from_file(path)?;
    let validated = raw.validate()?;

    for warning in validated.warnings() {
        eprintln!("Warning: {}", warning);
    }

    let theme = validated.into_theme();

    if verbose {
        eprintln!("Theme variant: {}", theme.variant.name());
        eprintln!("Bar height: {}px", theme.bar.height);
    }

    println!("Configuration is valid.");
    Ok(())
}

/// Converts a TOML configuration to JSON.
fn convert_config(input: &PathBuf, output: &PathBuf, verbose: bool) -> Result<()> {
    if verbose {
        eprintln!("Converting: {} -> {}", input.display(), output.display());
    }

    let config_adapter = TomlConfigAdapter::new();
    let persist_adapter = FileSystemAdapter::new();

    let raw = config_adapter.load_from_file(input)?;
    let validated = raw.validate()?;
    let theme = validated.into_theme();

    persist_adapter.save_theme(&theme, output)?;

    println!("Converted to: {}", output.display());
    Ok(())
}

/// Queries a theme value by path.
fn query_theme(config_path: &PathBuf, query_path: &str, verbose: bool) -> Result<()> {
    if verbose {
        eprintln!("Querying: {} from {}", query_path, config_path.display());
    }

    let adapter = TomlConfigAdapter::new();
    let raw = adapter.load_from_file(config_path)?;
    let validated = raw.validate()?;
    let theme = validated.into_theme();

    // Simple query implementation
    let value = match query_path {
        "variant" => theme.variant.name().to_string(),
        "bar.height" => theme.bar.height.to_string(),
        "bar.position" => theme.bar.position.name().to_string(),
        "colors.background" => theme.colors.background.to_hex(),
        "colors.foreground" => theme.colors.foreground.to_hex(),
        "colors.accent" => theme.colors.accent.to_hex(),
        "colors.surface" => theme.colors.surface.to_hex(),
        "colors.error" => theme.colors.error.to_hex(),
        "colors.warning" => theme.colors.warning.to_hex(),
        "colors.success" => theme.colors.success.to_hex(),
        "colors.info" => theme.colors.info.to_hex(),
        "typography.family" => theme.typography.family.name().to_string(),
        "typography.mono" => theme.typography.mono.name().to_string(),
        "radius.sm" => theme.radius.sm.to_string(),
        "radius.md" => theme.radius.md.to_string(),
        "radius.lg" => theme.radius.lg.to_string(),
        _ => {
            eprintln!("Unknown path: {}", query_path);
            eprintln!("\nAvailable paths:");
            eprintln!("  variant");
            eprintln!("  bar.height, bar.position");
            eprintln!("  colors.background, colors.foreground, colors.accent, colors.surface");
            eprintln!("  colors.error, colors.warning, colors.success, colors.info");
            eprintln!("  typography.family, typography.mono");
            eprintln!("  radius.sm, radius.md, radius.lg");
            return Ok(());
        }
    };

    println!("{}", value);
    Ok(())
}

/// Shows default theme values.
fn show_defaults(variant: &str) -> Result<()> {
    use fernctl::domain::theme::Theme;

    let theme = match variant.to_lowercase().as_str() {
        "dark" => Theme::dark(),
        "light" => Theme::light(),
        _ => {
            eprintln!("Unknown variant: {}. Using 'dark'.", variant);
            Theme::dark()
        }
    };

    let json = serde_json::to_string_pretty(&theme).map_err(|e| {
        fernctl::error::FernError::io(
            "serializing theme",
            std::io::Error::other(e.to_string()),
        )
    })?;

    println!("{}", json);
    Ok(())
}
