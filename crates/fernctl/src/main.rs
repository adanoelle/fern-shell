//! # fernctl CLI
//!
//! Command-line interface for the Fern Shell control plane.
//!
//! ## Usage
//!
//! ```bash
//! # Show service status
//! fernctl status
//! fernctl status obs
//!
//! # View logs
//! fernctl logs
//! fernctl logs -f
//! fernctl logs --service obs
//!
//! # Launch TUI dashboard
//! fernctl tui
//!
//! # Control services
//! fernctl obs start
//! fernctl obs stop
//! fernctl obs restart
//!
//! # Reload shell configuration
//! fernctl reload
//!
//! # Theme management
//! fernctl theme apply catppuccin-mocha
//! fernctl theme current
//! ```

use clap::{Parser, Subcommand};
use fernctl::cli::{logs, obs, reload, status, theme};
use fernctl::error::Result;

#[cfg(feature = "tui")]
use fernctl::tui::TuiApp;

/// fernctl - Fern Shell Control Plane
///
/// A unified tool for managing Fern Shell services, viewing logs,
/// and controlling configuration.
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

    /// Output format: text, json
    #[arg(long, global = true, default_value = "text")]
    output: String,
}

/// Available commands
#[derive(Subcommand, Debug)]
enum Commands {
    /// Launch interactive TUI dashboard.
    ///
    /// Opens a full-screen terminal UI with service status, logs,
    /// and configuration panels. Use Tab to navigate, q to quit.
    #[cfg(feature = "tui")]
    Tui,

    /// Show service status.
    ///
    /// Displays the current status of all services or a specific service.
    /// Shows connection state, activity, and health information.
    Status {
        /// Specific service to show (obs, shell, theme-watcher).
        service: Option<String>,
    },

    /// View aggregated logs.
    ///
    /// Shows log entries from all Fern Shell services. Use -f to follow
    /// logs in real-time (like tail -f).
    Logs {
        /// Follow mode - stream logs as they arrive.
        #[arg(short, long)]
        follow: bool,

        /// Filter by service name.
        #[arg(short, long)]
        service: Option<String>,

        /// Number of lines to show.
        #[arg(short = 'n', long, default_value = "50")]
        lines: usize,
    },

    /// Reload QuickShell configuration.
    ///
    /// Sends a reload signal to QuickShell, causing it to re-read
    /// the configuration file without restarting.
    Reload,

    /// OBS daemon control.
    ///
    /// Start, stop, restart, or check status of the fern-obs daemon
    /// that bridges OBS Studio to the shell.
    Obs {
        /// Action to perform.
        #[command(subcommand)]
        action: ObsCommands,
    },

    /// Theme management.
    ///
    /// Apply themes, validate configuration, or show the current theme.
    Theme {
        /// Action to perform.
        #[command(subcommand)]
        action: ThemeCommands,
    },
}

/// OBS subcommands
#[derive(Subcommand, Debug)]
enum ObsCommands {
    /// Start the OBS daemon.
    Start,
    /// Stop the OBS daemon.
    Stop,
    /// Restart the OBS daemon.
    Restart,
    /// Show OBS daemon status.
    Status,
}

/// Theme subcommands
#[derive(Subcommand, Debug)]
enum ThemeCommands {
    /// Apply a theme by name.
    Apply {
        /// Theme name to apply.
        name: String,
    },
    /// Validate the current configuration.
    Validate,
    /// Show the current theme.
    Current,
    /// Query a specific theme value.
    Query {
        /// Path to query (e.g., colors.background, bar.height).
        path: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up tracing based on verbosity
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("fernctl=debug")
            .init();
    }

    let output_format = match cli.output.as_str() {
        "json" => status::OutputFormat::Json,
        _ => status::OutputFormat::Text,
    };

    match cli.command {
        #[cfg(feature = "tui")]
        Commands::Tui => {
            let mut app = TuiApp::new();
            app.run()?;
        }

        Commands::Status { service } => {
            status::run(status::StatusOptions {
                service,
                format: output_format,
                verbose: cli.verbose,
            })?;
        }

        Commands::Logs {
            follow,
            service,
            lines,
        } => {
            logs::run(logs::LogsOptions {
                follow,
                service,
                lines,
                level: None,
            })?;
        }

        Commands::Reload => {
            reload::run()?;
        }

        Commands::Obs { action } => {
            let obs_action = match action {
                ObsCommands::Start => obs::ObsAction::Start,
                ObsCommands::Stop => obs::ObsAction::Stop,
                ObsCommands::Restart => obs::ObsAction::Restart,
                ObsCommands::Status => obs::ObsAction::Status,
            };
            obs::run(obs_action)?;
        }

        Commands::Theme { action } => {
            let theme_action = match action {
                ThemeCommands::Apply { name } => theme::ThemeAction::Apply { name },
                ThemeCommands::Validate => theme::ThemeAction::Validate,
                ThemeCommands::Current => theme::ThemeAction::Current,
                ThemeCommands::Query { path } => theme::ThemeAction::Query { path },
            };
            theme::run(theme::ThemeOptions {
                action: theme_action,
                verbose: cli.verbose,
                json: output_format == status::OutputFormat::Json,
            })?;
        }
    }

    Ok(())
}
