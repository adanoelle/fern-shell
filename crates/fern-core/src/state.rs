//! # Service State Types
//!
//! Types for representing service status and health across the Fern ecosystem.
//!
//! These types are serialized to JSON state files that are read by:
//! - The QML shell (via `FileView`)
//! - The `fernctl` control plane
//! - The TUI dashboard
//!
//! ## State File Format
//!
//! Each service writes its state to `~/.local/state/fern/{service}-state.json`:
//!
//! ```json
//! {
//!   "name": "obs",
//!   "status": "running",
//!   "pid": 12345,
//!   "uptime_secs": 3600,
//!   "health": {
//!     "ok": true,
//!     "message": null,
//!     "last_check_secs": 1704067200
//!   },
//!   "data": { /* service-specific state */ }
//! }
//! ```

use serde::{Deserialize, Serialize};

/// Information about a running service.
///
/// This is the common envelope for all service state files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service name (e.g., "obs", "theme", "audio").
    pub name: String,

    /// Current service status.
    pub status: ServiceStatus,

    /// Process ID if the service is running as a daemon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,

    /// Uptime in seconds since the service started.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_secs: Option<u64>,

    /// Last health check result.
    pub health: HealthCheck,
}

impl ServiceInfo {
    /// Creates a new `ServiceInfo` with the given name and status.
    #[must_use]
    pub fn new(name: impl Into<String>, status: ServiceStatus) -> Self {
        Self {
            name: name.into(),
            status,
            pid: None,
            uptime_secs: None,
            health: HealthCheck::unknown(),
        }
    }

    /// Creates a `ServiceInfo` for a running service.
    #[must_use]
    pub fn running(name: impl Into<String>, pid: u32) -> Self {
        Self {
            name: name.into(),
            status: ServiceStatus::Running,
            pid: Some(pid),
            uptime_secs: Some(0),
            health: HealthCheck::ok(),
        }
    }

    /// Creates a `ServiceInfo` for a stopped service.
    #[must_use]
    pub fn stopped(name: impl Into<String>) -> Self {
        Self::new(name, ServiceStatus::Stopped)
    }

    /// Creates a `ServiceInfo` for a disabled service.
    #[must_use]
    pub fn disabled(name: impl Into<String>) -> Self {
        Self::new(name, ServiceStatus::Disabled)
    }

    /// Returns `true` if the service is currently running.
    #[must_use]
    pub fn is_running(&self) -> bool {
        matches!(self.status, ServiceStatus::Running)
    }

    /// Returns `true` if the service is healthy.
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.health.ok
    }
}

/// Service operational status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceStatus {
    /// Service is running normally.
    Running,

    /// Service is stopped (can be started).
    Stopped,

    /// Service is in the process of starting.
    Starting,

    /// Service is in the process of stopping.
    Stopping,

    /// Service encountered an error.
    Failed(String),

    /// Service is disabled by configuration.
    Disabled,
}

impl ServiceStatus {
    /// Returns `true` if this is a `Running` status.
    #[must_use]
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Returns `true` if this is a `Failed` status.
    #[must_use]
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed(_))
    }

    /// Returns the error message if this is a `Failed` status.
    #[must_use]
    pub fn error_message(&self) -> Option<&str> {
        match self {
            Self::Failed(msg) => Some(msg),
            _ => None,
        }
    }
}

impl std::fmt::Display for ServiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Running => write!(f, "running"),
            Self::Stopped => write!(f, "stopped"),
            Self::Starting => write!(f, "starting"),
            Self::Stopping => write!(f, "stopping"),
            Self::Failed(msg) => write!(f, "failed: {msg}"),
            Self::Disabled => write!(f, "disabled"),
        }
    }
}

/// Health check result for a service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Whether the health check passed.
    pub ok: bool,

    /// Optional message describing the health status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Unix timestamp of the last health check (seconds since epoch).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_check_secs: Option<u64>,
}

impl HealthCheck {
    /// Creates a healthy status.
    #[must_use]
    pub fn ok() -> Self {
        Self {
            ok: true,
            message: None,
            last_check_secs: None,
        }
    }

    /// Creates a healthy status with a message.
    #[must_use]
    pub fn ok_with_message(message: impl Into<String>) -> Self {
        Self {
            ok: true,
            message: Some(message.into()),
            last_check_secs: None,
        }
    }

    /// Creates an unhealthy status with an error message.
    #[must_use]
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            message: Some(message.into()),
            last_check_secs: None,
        }
    }

    /// Creates an unknown health status (not yet checked).
    #[must_use]
    pub fn unknown() -> Self {
        Self {
            ok: false,
            message: Some("not yet checked".into()),
            last_check_secs: None,
        }
    }

    /// Sets the last check timestamp to now.
    #[must_use]
    pub fn with_timestamp(mut self) -> Self {
        self.last_check_secs = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        );
        self
    }
}

impl Default for HealthCheck {
    fn default() -> Self {
        Self::unknown()
    }
}

/// Service registry containing all known services.
///
/// This is written to `~/.local/state/fern/services.json` by `fernctl`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceRegistry {
    /// List of all registered services.
    pub services: Vec<ServiceInfo>,

    /// Unix timestamp when the registry was last updated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at_secs: Option<u64>,
}

impl ServiceRegistry {
    /// Creates an empty service registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Finds a service by name.
    #[must_use]
    pub fn find(&self, name: &str) -> Option<&ServiceInfo> {
        self.services.iter().find(|s| s.name == name)
    }

    /// Finds a service by name (mutable).
    #[must_use]
    pub fn find_mut(&mut self, name: &str) -> Option<&mut ServiceInfo> {
        self.services.iter_mut().find(|s| s.name == name)
    }

    /// Adds or updates a service in the registry.
    pub fn upsert(&mut self, info: ServiceInfo) {
        if let Some(existing) = self.find_mut(&info.name) {
            *existing = info;
        } else {
            self.services.push(info);
        }
        self.touch();
    }

    /// Updates the `updated_at_secs` timestamp to now.
    pub fn touch(&mut self) {
        self.updated_at_secs = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        );
    }

    /// Returns all running services.
    pub fn running(&self) -> impl Iterator<Item = &ServiceInfo> {
        self.services.iter().filter(|s| s.is_running())
    }

    /// Returns all unhealthy services.
    pub fn unhealthy(&self) -> impl Iterator<Item = &ServiceInfo> {
        self.services.iter().filter(|s| !s.is_healthy())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_info_running() {
        let info = ServiceInfo::running("obs", 1234);

        assert!(info.is_running());
        assert_eq!(info.pid, Some(1234));
        assert!(info.is_healthy());
    }

    #[test]
    fn service_status_display() {
        assert_eq!(ServiceStatus::Running.to_string(), "running");
        assert_eq!(ServiceStatus::Stopped.to_string(), "stopped");
        assert_eq!(
            ServiceStatus::Failed("connection lost".into()).to_string(),
            "failed: connection lost"
        );
    }

    #[test]
    fn service_registry_upsert() {
        let mut registry = ServiceRegistry::new();

        registry.upsert(ServiceInfo::running("obs", 1234));
        assert_eq!(registry.services.len(), 1);

        // Update existing
        registry.upsert(ServiceInfo::stopped("obs"));
        assert_eq!(registry.services.len(), 1);
        assert!(!registry.find("obs").unwrap().is_running());

        // Add new
        registry.upsert(ServiceInfo::running("theme", 5678));
        assert_eq!(registry.services.len(), 2);
    }

    #[test]
    fn health_check_with_timestamp() {
        let health = HealthCheck::ok().with_timestamp();

        assert!(health.ok);
        assert!(health.last_check_secs.is_some());
    }
}
