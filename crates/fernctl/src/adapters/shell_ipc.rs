//! # Shell IPC
//!
//! Communication with QuickShell for configuration reload.

use crate::error::{FernctlError, Result};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::process::Command;

/// Finds the QuickShell process PID.
///
/// # Errors
///
/// Returns an error if QuickShell is not running.
pub fn find_quickshell_pid() -> Result<i32> {
    let output = Command::new("pgrep")
        .arg("-x")
        .arg("quickshell")
        .output()
        .map_err(|e| FernctlError::process_io("running pgrep", e))?;

    if !output.status.success() {
        return Err(FernctlError::process("QuickShell is not running"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let pid_str = stdout
        .lines()
        .next()
        .ok_or_else(|| FernctlError::process("QuickShell is not running"))?;

    pid_str
        .trim()
        .parse::<i32>()
        .map_err(|e| FernctlError::process(format!("parsing PID: {e}")))
}

/// Reloads QuickShell configuration by sending SIGHUP.
///
/// # Errors
///
/// Returns an error if QuickShell is not running or cannot be signaled.
pub fn reload_shell() -> Result<()> {
    let pid = find_quickshell_pid()?;

    signal::kill(Pid::from_raw(pid), Signal::SIGHUP)
        .map_err(|e| FernctlError::process(format!("sending SIGHUP: {e}")))?;

    tracing::info!("Sent SIGHUP to QuickShell (PID {})", pid);
    Ok(())
}

/// Checks if QuickShell is running.
#[must_use]
pub fn is_shell_running() -> bool {
    find_quickshell_pid().is_ok()
}

/// Gets the QuickShell process uptime in seconds.
///
/// Returns `None` if QuickShell is not running or uptime cannot be determined.
#[must_use]
pub fn shell_uptime() -> Option<u64> {
    let pid = find_quickshell_pid().ok()?;

    // Read process start time from /proc
    let stat_path = format!("/proc/{}/stat", pid);
    let stat = std::fs::read_to_string(stat_path).ok()?;

    // Field 22 is starttime in clock ticks since boot
    let fields: Vec<&str> = stat.split_whitespace().collect();
    let starttime_ticks: u64 = fields.get(21)?.parse().ok()?;

    // Get system uptime
    let uptime_str = std::fs::read_to_string("/proc/uptime").ok()?;
    let system_uptime_secs: f64 = uptime_str.split_whitespace().next()?.parse().ok()?;

    // Get clock ticks per second (usually 100)
    let ticks_per_sec = 100u64; // sysconf(_SC_CLK_TCK), typically 100

    let starttime_secs = starttime_ticks / ticks_per_sec;
    let process_uptime = system_uptime_secs as u64 - starttime_secs;

    Some(process_uptime)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_shell_running_returns_bool() {
        // Just verify it doesn't panic
        let _ = is_shell_running();
    }
}
