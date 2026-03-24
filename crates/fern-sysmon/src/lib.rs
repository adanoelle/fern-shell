//! # fern-sysmon - Hardware Sensor Monitoring for Fern Shell
//!
//! A library crate providing structured access to hardware sensors, GPU status,
//! and systemd unit state. Designed to be polled from fernctl's TUI event loop.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use fern_sysmon::SysMonitor;
//!
//! let mut monitor = SysMonitor::new(&["pipewire.service".into()])?;
//! let snapshot = monitor.snapshot()?;
//!
//! for chip in &snapshot.sensors {
//!     for reading in &chip.readings {
//!         println!("{}: {}{}", reading.label, reading.value, reading.unit);
//!     }
//! }
//! # Ok::<(), fern_sysmon::SysMonError>(())
//! ```
//!
//! ## Features
//!
//! - `nvidia` (default) - Enable NVIDIA GPU monitoring via NVML

#[cfg(feature = "nvidia")]
pub mod gpu;
pub mod sensors;
pub mod snapshot;
pub mod systemd;

pub use snapshot::{
    ActiveState, GpuStatus, SensorChip, SensorReading, SensorUnit, SystemSnapshot, SystemdUnit,
    Thresholds,
};

use chrono::Utc;

/// Error type for `fern-sysmon` operations.
#[derive(Debug, thiserror::Error)]
pub enum SysMonError {
    /// Error initializing or reading from libsensors.
    #[error("sensors error: {0}")]
    Sensors(String),

    /// Error communicating with NVIDIA GPU via NVML.
    #[error("GPU error: {0}")]
    Gpu(String),

    /// Error querying systemd via D-Bus.
    #[error("systemd error: {0}")]
    Systemd(String),
}

/// Central monitor that holds handles to sensor libraries and D-Bus.
///
/// Create once and call [`snapshot()`](SysMonitor::snapshot) repeatedly from
/// your event loop.
pub struct SysMonitor {
    /// libsensors handle.
    lm_sensors: lm_sensors::LMSensors,

    /// NVML GPU monitor (None if feature disabled or init failed).
    #[cfg(feature = "nvidia")]
    gpu_monitor: Option<gpu::GpuMonitor>,

    /// D-Bus connection to the system bus.
    dbus_connection: Option<zbus::blocking::Connection>,

    /// Which systemd units to track.
    tracked_units: Vec<String>,
}

impl SysMonitor {
    /// Creates a new system monitor.
    ///
    /// `tracked_units` is the list of systemd unit names to query
    /// (e.g. `["pipewire.service", "docker.service"]`).
    ///
    /// # Errors
    ///
    /// Returns an error if libsensors cannot be initialized.
    pub fn new(tracked_units: &[String]) -> Result<Self, SysMonError> {
        let lm_sensors = lm_sensors::Initializer::default()
            .initialize()
            .map_err(|e| SysMonError::Sensors(format!("init: {e}")))?;

        #[cfg(feature = "nvidia")]
        let gpu_monitor = gpu::GpuMonitor::new().ok();

        let dbus_connection = systemd::connect_system_bus().ok();

        Ok(Self {
            lm_sensors,
            #[cfg(feature = "nvidia")]
            gpu_monitor,
            dbus_connection,
            tracked_units: tracked_units.to_vec(),
        })
    }

    /// Takes a snapshot of all monitored hardware and services.
    ///
    /// This is designed to be fast (~1-5ms) and safe to call from a TUI
    /// event loop at 1Hz.
    ///
    /// # Errors
    ///
    /// Returns an error if libsensors reading fails. GPU and systemd errors
    /// are handled gracefully (returning None/empty).
    pub fn snapshot(&self) -> Result<SystemSnapshot, SysMonError> {
        let sensor_chips = sensors::read_sensors(&self.lm_sensors)?;

        #[cfg(feature = "nvidia")]
        let gpu = self
            .gpu_monitor
            .as_ref()
            .and_then(|gm| gm.read_gpu().ok());

        #[cfg(not(feature = "nvidia"))]
        let gpu = None;

        let units = self
            .dbus_connection
            .as_ref()
            .map(|conn| {
                systemd::query_units(conn, &self.tracked_units).unwrap_or_default()
            })
            .unwrap_or_default();

        Ok(SystemSnapshot {
            timestamp: Utc::now(),
            sensors: sensor_chips,
            gpu,
            units,
        })
    }

    /// Returns a reference to the list of tracked systemd units.
    #[must_use]
    pub fn tracked_units(&self) -> &[String] {
        &self.tracked_units
    }

    /// Updates the list of tracked systemd units.
    pub fn set_tracked_units(&mut self, units: Vec<String>) {
        self.tracked_units = units;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let err = SysMonError::Sensors("test error".to_string());
        assert_eq!(format!("{err}"), "sensors error: test error");
    }

    #[test]
    fn snapshot_types_are_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<SystemSnapshot>();
        assert_sync::<SystemSnapshot>();
        assert_send::<SysMonError>();
    }
}
