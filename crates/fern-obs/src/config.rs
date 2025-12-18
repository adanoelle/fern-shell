//! Configuration for fern-obs.

use serde::{Deserialize, Serialize};

/// Configuration for connecting to OBS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsConfig {
    /// OBS WebSocket host.
    #[serde(default = "default_host")]
    pub host: String,

    /// OBS WebSocket port.
    #[serde(default = "default_port")]
    pub port: u16,

    /// OBS WebSocket password (if authentication is enabled).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    /// How often to poll for stats (in milliseconds).
    #[serde(default = "default_stats_interval")]
    pub stats_interval_ms: u64,

    /// How long to wait before reconnecting after disconnect (in milliseconds).
    #[serde(default = "default_reconnect_interval")]
    pub reconnect_interval_ms: u64,

    /// Maximum number of reconnection attempts (0 = unlimited).
    #[serde(default)]
    pub max_reconnect_attempts: u32,

    /// Whether to show stats in the state file.
    #[serde(default = "default_show_stats")]
    pub show_stats: bool,
}

fn default_host() -> String {
    "localhost".to_string()
}

fn default_port() -> u16 {
    4455
}

fn default_stats_interval() -> u64 {
    1000 // 1 second
}

fn default_reconnect_interval() -> u64 {
    5000 // 5 seconds
}

fn default_show_stats() -> bool {
    true
}

impl Default for ObsConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            password: None,
            stats_interval_ms: default_stats_interval(),
            reconnect_interval_ms: default_reconnect_interval(),
            max_reconnect_attempts: 0,
            show_stats: default_show_stats(),
        }
    }
}

impl ObsConfig {
    /// Creates a new config with the specified host and port.
    #[must_use]
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            ..Default::default()
        }
    }

    /// Sets the password.
    #[must_use]
    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Returns the WebSocket URL.
    #[must_use]
    pub fn websocket_url(&self) -> String {
        format!("ws://{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = ObsConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 4455);
        assert!(config.password.is_none());
    }

    #[test]
    fn websocket_url() {
        let config = ObsConfig::new("192.168.1.100", 4456);
        assert_eq!(config.websocket_url(), "ws://192.168.1.100:4456");
    }
}
