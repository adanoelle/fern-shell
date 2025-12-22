//! # Service Controller
//!
//! Controls service lifecycle (start, stop, restart).

use crate::domain::KnownService;
use crate::error::{FernctlError, Result};
use fern_core::FernPaths;
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};

/// Controls the lifecycle of known services.
pub struct ServiceController {
    /// Fern paths configuration.
    paths: FernPaths,
    /// Map of service name to managed child process.
    managed: HashMap<String, Child>,
}

impl ServiceController {
    /// Creates a new service controller.
    #[must_use]
    pub fn new(paths: FernPaths) -> Self {
        Self {
            paths,
            managed: HashMap::new(),
        }
    }

    /// Starts a service.
    ///
    /// # Errors
    ///
    /// Returns an error if the service cannot be started.
    pub fn start(&mut self, service: KnownService) -> Result<()> {
        if self.is_running(service) {
            tracing::info!("{} is already running", service.display_name());
            return Ok(());
        }

        tracing::info!("Starting {}...", service.display_name());

        let mut cmd = Command::new(service.binary());
        for arg in service.start_args() {
            cmd.arg(arg);
        }

        // Detach from terminal
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());

        let child = cmd
            .spawn()
            .map_err(|e| FernctlError::process_io(format!("starting {}", service.binary()), e))?;

        tracing::info!("{} started (PID {})", service.display_name(), child.id());
        self.managed.insert(service.name().to_string(), child);

        Ok(())
    }

    /// Stops a service.
    ///
    /// # Errors
    ///
    /// Returns an error if the service cannot be stopped.
    pub fn stop(&mut self, service: KnownService) -> Result<()> {
        tracing::info!("Stopping {}...", service.display_name());

        // First try to stop our managed child
        if let Some(mut child) = self.managed.remove(service.name()) {
            let _ = child.kill();
            let _ = child.wait();
            tracing::info!("{} stopped (managed)", service.display_name());
            return Ok(());
        }

        // Fall back to pkill for externally started processes
        let status = Command::new("pkill")
            .arg("-f")
            .arg(format!("^{}", service.binary()))
            .status()
            .map_err(|e| FernctlError::process_io("running pkill", e))?;

        if status.success() {
            tracing::info!("{} stopped", service.display_name());
        } else {
            tracing::debug!("{} was not running", service.display_name());
        }

        Ok(())
    }

    /// Restarts a service.
    ///
    /// # Errors
    ///
    /// Returns an error if the service cannot be restarted.
    pub fn restart(&mut self, service: KnownService) -> Result<()> {
        self.stop(service)?;
        std::thread::sleep(std::time::Duration::from_millis(500));
        self.start(service)
    }

    /// Checks if a service is running.
    #[must_use]
    pub fn is_running(&self, service: KnownService) -> bool {
        // Check managed processes first
        if let Some(child) = self.managed.get(service.name()) {
            // Check if process is still alive
            let pid = child.id();
            let proc_path = format!("/proc/{}", pid);
            if std::path::Path::new(&proc_path).exists() {
                return true;
            }
        }

        // Check via pgrep
        Command::new("pgrep")
            .arg("-x")
            .arg(service.binary())
            .stdout(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Gets the PID of a running service, if any.
    #[must_use]
    pub fn get_pid(&self, service: KnownService) -> Option<u32> {
        // Check managed processes first
        if let Some(child) = self.managed.get(service.name()) {
            let pid = child.id();
            let proc_path = format!("/proc/{}", pid);
            if std::path::Path::new(&proc_path).exists() {
                return Some(pid);
            }
        }

        // Check via pgrep
        let output = Command::new("pgrep")
            .arg("-x")
            .arg(service.binary())
            .output()
            .ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.lines().next()?.trim().parse().ok()
        } else {
            None
        }
    }

    /// Cleans up terminated child processes.
    pub fn reap_children(&mut self) {
        self.managed.retain(|name, child| {
            match child.try_wait() {
                Ok(Some(status)) => {
                    tracing::debug!("{} exited with status {:?}", name, status);
                    false // Remove from map
                }
                Ok(None) => true, // Still running
                Err(e) => {
                    tracing::error!("Error checking {}: {}", name, e);
                    false
                }
            }
        });
    }

    /// Returns the paths configuration.
    #[must_use]
    pub fn paths(&self) -> &FernPaths {
        &self.paths
    }
}

impl Drop for ServiceController {
    fn drop(&mut self) {
        // Clean up managed processes on drop
        for (name, mut child) in self.managed.drain() {
            tracing::debug!("Cleaning up managed process: {}", name);
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_controller_new() {
        let paths = FernPaths::new();
        let controller = ServiceController::new(paths);
        assert!(controller.managed.is_empty());
    }
}
