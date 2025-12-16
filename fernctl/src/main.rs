//! # fernctl CLI
//!
//! Command-line interface for Fern Shell configuration management.
//!
//! `fernctl` is the primary tool for working with Fern Shell configuration files.
//! It provides commands for validation, format conversion, theme queries, and
//! live-reload file watching.
//!
//! ## Commands
//!
//! | Command | Description |
//! |---------|-------------|
//! | `validate` | Check configuration syntax and semantics |
//! | `convert` | Transform TOML to JSON for QuickShell |
//! | `query` | Extract specific theme values |
//! | `defaults` | Display default theme values |
//! | `watch` | Monitor config and auto-convert on changes |
//!
//! ## Quick Examples
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
//! # Watch for changes and auto-convert
//! fernctl watch
//!
//! # Show default dark theme
//! fernctl defaults dark
//! ```
//!
//! ## Configuration File Locations
//!
//! By default, `fernctl` looks for configuration at:
//!
//! - **Linux**: `~/.config/fern/config.toml`
//! - **macOS**: `~/Library/Application Support/fern/config.toml`
//! - **Windows**: `%APPDATA%\fern\config.toml`
//!
//! Override with the `--config` flag on any command.
//!
//! ## Exit Codes
//!
//! | Code | Meaning |
//! |------|---------|
//! | `0` | Success |
//! | `1` | Configuration error (parse, validation) |
//! | `2` | I/O error (file not found, permissions) |
//! | `3` | Watch system error |
//!
//! ## Environment Variables
//!
//! | Variable | Description |
//! |----------|-------------|
//! | `FERN_CONFIG` | Override default config file path |
//! | `NO_COLOR` | Disable colored output |
//! | `RUST_LOG` | Set log level (e.g., `debug`, `trace`) |

use clap::{Parser, Subcommand};
use fernctl::adapters::{FileSystemAdapter, TomlConfigAdapter};
use fernctl::error::Result;
use std::path::PathBuf;

mod commands_impl {
    //! Command implementations.
    //!
    //! We re-export from the library when available, but keep local
    //! implementations for commands that don't need library exposure.

    pub use fernctl::commands::convert;
    pub use fernctl::commands::query;
    pub use fernctl::commands::validate;

    #[cfg(feature = "watch")]
    pub use fernctl::commands::watch;
}

/// fernctl - Fern Shell Configuration Manager
///
/// A command-line tool for managing Fern Shell configuration files.
/// Validates TOML configuration, converts to JSON for QuickShell,
/// and provides theme query capabilities.
#[derive(Parser, Debug)]
#[command(name = "fernctl")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(after_help = "For more information, see: https://github.com/adanoelle/fern-shell")]
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
    /// Validate a configuration file without producing output.
    ///
    /// Checks both TOML syntax and semantic correctness (valid colors,
    /// ranges, etc.). Exits with code 0 if valid, 1 if invalid.
    Validate {
        /// Path to the configuration file.
        ///
        /// Defaults to ~/.config/fern/config.toml
        #[arg(short, long, env = "FERN_CONFIG")]
        config: Option<PathBuf>,
    },

    /// Convert TOML configuration to JSON.
    ///
    /// Validates the input file and writes a JSON file suitable for
    /// consumption by QuickShell. The conversion is atomic — if validation
    /// fails, no output is written.
    Convert {
        /// Input TOML file to convert.
        input: PathBuf,

        /// Output JSON file path.
        ///
        /// Defaults to the input filename with .json extension.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Query a specific theme value by path.
    ///
    /// Outputs the value to stdout for use in scripts. Available paths
    /// include colors.*, bar.*, typography.*, and radius.*.
    Query {
        /// The path to query (e.g., colors.background, bar.height).
        path: String,

        /// Path to the configuration file.
        #[arg(short, long, env = "FERN_CONFIG")]
        config: Option<PathBuf>,
    },

    /// Display default theme values.
    ///
    /// Outputs a complete theme as JSON, showing all default values.
    /// Useful for understanding the schema or bootstrapping a new config.
    Defaults {
        /// Theme variant: "dark" or "light".
        #[arg(default_value = "dark")]
        variant: String,
    },

    /// Watch configuration file and auto-convert on changes.
    ///
    /// Monitors the TOML config file and automatically converts to JSON
    /// when changes are detected. Sends desktop notifications on success
    /// or error. Press Ctrl+C to stop.
    #[cfg(feature = "watch")]
    Watch {
        /// Path to the configuration file to watch.
        ///
        /// Defaults to ~/.config/fern/config.toml
        #[arg(short, long, env = "FERN_CONFIG")]
        config: Option<PathBuf>,

        /// Output JSON file path.
        ///
        /// Defaults to the input filename with .json extension.
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Debounce duration in milliseconds.
        ///
        /// Wait this long after file changes before converting.
        /// Prevents multiple conversions when editors save incrementally.
        #[arg(short, long, default_value = "100")]
        debounce: u64,

        /// Suppress desktop notifications.
        ///
        /// By default, notifications are sent on success and error.
        #[arg(short, long)]
        quiet: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { config } => {
            let path = config.unwrap_or_else(default_config_path);
            cmd_validate(&path, cli.verbose)?;
        }
        Commands::Convert { input, output } => {
            let output = output.unwrap_or_else(|| commands_impl::convert::derive_output_path(&input));
            cmd_convert(&input, &output, cli.verbose)?;
        }
        Commands::Query { path, config } => {
            let config_path = config.unwrap_or_else(default_config_path);
            cmd_query(&config_path, &path, cli.verbose)?;
        }
        Commands::Defaults { variant } => {
            cmd_defaults(&variant)?;
        }
        #[cfg(feature = "watch")]
        Commands::Watch {
            config,
            output,
            debounce,
            quiet,
        } => {
            let config_path = config.unwrap_or_else(default_config_path);
            let output_path =
                output.unwrap_or_else(|| commands_impl::convert::derive_output_path(&config_path));
            cmd_watch(&config_path, &output_path, debounce, quiet, cli.verbose)?;
        }
    }

    Ok(())
}

/// Returns the default configuration file path.
///
/// Uses the XDG config directory on Linux, Application Support on macOS,
/// and %APPDATA% on Windows.
fn default_config_path() -> PathBuf {
    dirs::config_dir()
        .map(|p| p.join("fern").join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("config.toml"))
}

/// Validates a configuration file.
fn cmd_validate(path: &PathBuf, verbose: bool) -> Result<()> {
    let adapter = TomlConfigAdapter::new();
    let options = commands_impl::validate::ValidateOptions { verbose };

    let result = commands_impl::validate::run(path, options, &adapter)?;

    for warning in &result.warnings {
        eprintln!("Warning: {warning}");
    }

    println!("Configuration is valid.");
    Ok(())
}

/// Converts a TOML configuration to JSON.
fn cmd_convert(input: &PathBuf, output: &PathBuf, verbose: bool) -> Result<()> {
    let config_adapter = TomlConfigAdapter::new();
    let persist_adapter = FileSystemAdapter::new();
    let options = commands_impl::convert::ConvertOptions {
        verbose,
        pretty: true,
    };

    let result = commands_impl::convert::run(input, output, options, &config_adapter, &persist_adapter)?;

    for warning in &result.warnings {
        eprintln!("Warning: {warning}");
    }

    println!("Converted to: {}", output.display());
    Ok(())
}

/// Queries a theme value by path.
fn cmd_query(config_path: &PathBuf, query_path: &str, verbose: bool) -> Result<()> {
    let adapter = TomlConfigAdapter::new();
    let options = commands_impl::query::QueryOptions { verbose };

    match commands_impl::query::run(config_path, query_path, options, &adapter)? {
        Some(value) => {
            println!("{value}");
        }
        None => {
            eprintln!("Unknown path: {query_path}");
            commands_impl::query::print_available_paths();

            let suggestions = commands_impl::query::find_similar_paths(query_path);
            if !suggestions.is_empty() {
                eprintln!("\nDid you mean: {}?", suggestions.join(", "));
            }
        }
    }

    Ok(())
}

/// Shows default theme values.
fn cmd_defaults(variant: &str) -> Result<()> {
    use fernctl::domain::theme::Theme;

    let theme = match variant.to_lowercase().as_str() {
        "dark" => Theme::dark(),
        "light" => Theme::light(),
        _ => {
            eprintln!("Unknown variant: {variant}. Using 'dark'.");
            Theme::dark()
        }
    };

    let json = serde_json::to_string_pretty(&theme).map_err(|e| {
        fernctl::error::FernError::io(
            "serializing theme",
            std::io::Error::other(e.to_string()),
        )
    })?;

    println!("{json}");
    Ok(())
}

/// Watches configuration and auto-converts on changes.
#[cfg(feature = "watch")]
fn cmd_watch(
    config_path: &PathBuf,
    output_path: &PathBuf,
    debounce_ms: u64,
    quiet: bool,
    verbose: bool,
) -> Result<()> {
    let config_adapter = TomlConfigAdapter::new();
    let persist_adapter = FileSystemAdapter::new();

    let options = commands_impl::watch::WatchOptions {
        debounce_ms,
        notify_on_success: !quiet,
        notify_on_error: !quiet,
        verbose,
    };

    println!("Watching {} for changes...", config_path.display());
    println!("Output: {}", output_path.display());
    println!("Press Ctrl+C to stop.\n");

    commands_impl::watch::run(config_path, output_path, options, &config_adapter, &persist_adapter)
}
