//! # Reload Command
//!
//! Reload QuickShell configuration.

use crate::error::{FernctlError, Result};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::process::Command;

/// Runs the reload command.
///
/// Finds the QuickShell process and sends SIGHUP to trigger a config reload.
///
/// # Errors
///
/// Returns an error if QuickShell is not running or cannot be signaled.
pub fn run() -> Result<()> {
    println!("Reloading Fern Shell configuration...");

    // Find QuickShell PID
    let pid = find_quickshell_pid()?;

    // Send SIGHUP
    signal::kill(Pid::from_raw(pid), Signal::SIGHUP)
        .map_err(|e| FernctlError::process(format!("sending SIGHUP: {e}")))?;

    println!("Reload signal sent to QuickShell (PID {pid})");
    Ok(())
}

fn find_quickshell_pid() -> Result<i32> {
    let output = Command::new("pgrep")
        .arg("-x")
        .arg("quickshell")
        .output()
        .map_err(|e| FernctlError::process_io("running pgrep", e))?;

    if !output.status.success() {
        return Err(FernctlError::process("QuickShell is not running"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let pid_str = stdout.lines().next().ok_or_else(|| {
        FernctlError::process("QuickShell is not running")
    })?;

    pid_str
        .trim()
        .parse::<i32>()
        .map_err(|e| FernctlError::process(format!("parsing PID: {e}")))
}
