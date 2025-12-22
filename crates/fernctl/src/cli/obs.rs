//! # OBS Command
//!
//! Control the OBS daemon (fern-obs).

use crate::domain::KnownService;
use crate::error::{FernctlError, Result};
use std::process::Command;

/// OBS subcommand action.
#[derive(Debug, Clone, Copy)]
pub enum ObsAction {
    /// Start the OBS daemon.
    Start,
    /// Stop the OBS daemon.
    Stop,
    /// Restart the OBS daemon.
    Restart,
    /// Show OBS status.
    Status,
}

/// Runs the OBS command.
///
/// # Errors
///
/// Returns an error if the daemon cannot be controlled.
pub fn run(action: ObsAction) -> Result<()> {
    let service = KnownService::Obs;

    match action {
        ObsAction::Start => start_daemon(service),
        ObsAction::Stop => stop_daemon(service),
        ObsAction::Restart => {
            stop_daemon(service)?;
            std::thread::sleep(std::time::Duration::from_millis(500));
            start_daemon(service)
        }
        ObsAction::Status => {
            // Use the status command
            crate::cli::status::run(crate::cli::status::StatusOptions {
                service: Some("obs".to_string()),
                format: crate::cli::status::OutputFormat::Text,
                verbose: true,
            })
        }
    }
}

fn start_daemon(service: KnownService) -> Result<()> {
    println!("Starting {}...", service.display_name());

    let mut cmd = Command::new(service.binary());
    for arg in service.start_args() {
        cmd.arg(arg);
    }

    // Spawn detached
    cmd.spawn()
        .map_err(|e| FernctlError::process_io(format!("starting {}", service.binary()), e))?;

    println!("{} started.", service.display_name());
    Ok(())
}

fn stop_daemon(service: KnownService) -> Result<()> {
    println!("Stopping {}...", service.display_name());

    // Use pkill to stop the daemon
    let status = Command::new("pkill")
        .arg("-f")
        .arg(format!("^{}", service.binary()))
        .status()
        .map_err(|e| FernctlError::process_io("running pkill", e))?;

    if status.success() {
        println!("{} stopped.", service.display_name());
    } else {
        println!("{} was not running.", service.display_name());
    }

    Ok(())
}
