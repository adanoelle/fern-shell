//! # fern-obs CLI
//!
//! Command-line interface for the OBS WebSocket bridge daemon.
//!
//! ## Usage
//!
//! ```bash
//! # Start the daemon
//! fern-obs daemon
//!
//! # With custom connection settings
//! fern-obs daemon --host 192.168.1.100 --port 4455 --password secret
//!
//! # Recording controls
//! fern-obs start-recording
//! fern-obs stop-recording
//! fern-obs toggle-pause
//!
//! # Streaming controls
//! fern-obs start-streaming
//! fern-obs stop-streaming
//!
//! # Scene control
//! fern-obs scene "Gaming"
//!
//! # Get current status
//! fern-obs status
//! ```

use clap::{Parser, Subcommand};
use fern_obs::config::ObsConfig;
use fern_obs::daemon::{send_command, Command, CommandResult, Daemon};
use fern_obs::error::Result;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// fern-obs - OBS WebSocket bridge for Fern Shell
///
/// Connects to OBS Studio via obs-websocket and exposes state
/// for the Fern Shell QML interface.
#[derive(Parser, Debug)]
#[command(name = "fern-obs")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// OBS WebSocket host
    #[arg(long, default_value = "localhost", global = true, env = "OBS_HOST")]
    host: String,

    /// OBS WebSocket port
    #[arg(long, default_value_t = 4455, global = true, env = "OBS_PORT")]
    port: u16,

    /// OBS WebSocket password
    #[arg(long, global = true, env = "OBS_PASSWORD")]
    password: Option<String>,

    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,
}

/// Available commands
#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the OBS bridge daemon
    ///
    /// Maintains a persistent connection to OBS and writes state updates
    /// to ~/.local/state/fern/obs-state.json
    Daemon {
        /// How often to update stats (milliseconds)
        #[arg(long, default_value_t = 1000)]
        stats_interval: u64,

        /// How long to wait before reconnecting (milliseconds)
        #[arg(long, default_value_t = 5000)]
        reconnect_interval: u64,

        /// Maximum reconnection attempts (0 = unlimited)
        #[arg(long, default_value_t = 0)]
        max_reconnects: u32,

        /// Disable stats collection
        #[arg(long)]
        no_stats: bool,
    },

    /// Start recording
    #[command(alias = "rec")]
    StartRecording,

    /// Stop recording
    #[command(alias = "stop-rec")]
    StopRecording,

    /// Toggle recording pause
    #[command(alias = "pause")]
    TogglePause,

    /// Start streaming
    #[command(alias = "stream")]
    StartStreaming,

    /// Stop streaming
    #[command(alias = "stop-stream")]
    StopStreaming,

    /// Set the current scene
    Scene {
        /// Name of the scene to switch to
        name: String,
    },

    /// Get current OBS status
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber with env filter
    // Default to "info" level, can be overridden with RUST_LOG env var
    // e.g., RUST_LOG=debug or RUST_LOG=fern_obs=trace
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(
            fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false)
                .compact(),
        )
        .init();

    let cli = Cli::parse();

    let base_config = ObsConfig {
        host: cli.host,
        port: cli.port,
        password: cli.password,
        ..Default::default()
    };

    match cli.command {
        Commands::Daemon {
            stats_interval,
            reconnect_interval,
            max_reconnects,
            no_stats,
        } => {
            let config = ObsConfig {
                stats_interval_ms: stats_interval,
                reconnect_interval_ms: reconnect_interval,
                max_reconnect_attempts: max_reconnects,
                show_stats: !no_stats,
                ..base_config
            };

            let mut daemon = Daemon::new(config);
            daemon.run().await
        }

        Commands::StartRecording => {
            let result = send_command(&base_config, Command::StartRecording).await?;
            print_result(result, false);
            Ok(())
        }

        Commands::StopRecording => {
            let result = send_command(&base_config, Command::StopRecording).await?;
            print_result(result, false);
            Ok(())
        }

        Commands::TogglePause => {
            let result = send_command(&base_config, Command::TogglePause).await?;
            print_result(result, false);
            Ok(())
        }

        Commands::StartStreaming => {
            let result = send_command(&base_config, Command::StartStreaming).await?;
            print_result(result, false);
            Ok(())
        }

        Commands::StopStreaming => {
            let result = send_command(&base_config, Command::StopStreaming).await?;
            print_result(result, false);
            Ok(())
        }

        Commands::Scene { name } => {
            let result = send_command(&base_config, Command::SetScene(name)).await?;
            print_result(result, false);
            Ok(())
        }

        Commands::Status { json } => {
            let result = send_command(&base_config, Command::GetStatus).await?;
            print_result(result, json);
            Ok(())
        }
    }
}

fn print_result(result: CommandResult, as_json: bool) {
    match result {
        CommandResult::Success(msg) => {
            println!("{msg}");
        }
        CommandResult::State(state) => {
            if as_json {
                if let Ok(json) = serde_json::to_string_pretty(&state) {
                    println!("{json}");
                }
            } else {
                println!("Connected: {}", state.connected);

                if let Some(scene) = &state.current_scene {
                    println!("Scene: {scene}");
                }

                println!(
                    "Recording: {}{}",
                    if state.recording.active {
                        "active"
                    } else {
                        "inactive"
                    },
                    if state.recording.paused {
                        " (paused)"
                    } else {
                        ""
                    }
                );

                if state.recording.active {
                    if let Some(tc) = &state.recording.timecode {
                        println!("  Duration: {tc}");
                    }
                }

                println!(
                    "Streaming: {}{}",
                    if state.streaming.active {
                        "active"
                    } else {
                        "inactive"
                    },
                    if state.streaming.reconnecting {
                        " (reconnecting)"
                    } else {
                        ""
                    }
                );

                if state.streaming.active {
                    if let Some(tc) = &state.streaming.timecode {
                        println!("  Duration: {tc}");
                    }
                }

                if let Some(stats) = &state.stats {
                    println!("Stats:");
                    println!("  CPU: {:.1}%", stats.cpu_usage);
                    println!("  FPS: {:.1}", stats.active_fps);
                    if let Some(drop) = stats.render_drop_percent {
                        println!("  Render drops: {:.2}%", drop);
                    }
                    if let Some(drop) = stats.output_drop_percent {
                        println!("  Output drops: {:.2}%", drop);
                    }
                }

                if !state.scenes.is_empty() {
                    println!("Scenes: {}", state.scenes.join(", "));
                }
            }
        }
    }
}
