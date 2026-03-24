//! # System Snapshot Types
//!
//! Data types representing a point-in-time capture of hardware sensor readings,
//! GPU status, and systemd unit state.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A point-in-time snapshot of all monitored system state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    /// When this snapshot was taken.
    pub timestamp: DateTime<Utc>,

    /// All lm_sensors chip readings.
    pub sensors: Vec<SensorChip>,

    /// NVIDIA GPU status (None if no GPU or feature disabled).
    pub gpu: Option<GpuStatus>,

    /// Tracked systemd unit statuses.
    pub units: Vec<SystemdUnit>,
}

/// A single hardware monitoring chip (e.g. k10temp, nct6798, nvme).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorChip {
    /// Chip name from libsensors (e.g. "k10temp-pci-00c3").
    pub name: String,

    /// Adapter type (e.g. "PCI adapter", "ISA adapter").
    pub adapter: String,

    /// All sensor readings from this chip.
    pub readings: Vec<SensorReading>,
}

/// A single sensor reading (temperature, fan speed, voltage, power, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    /// Human-readable label (e.g. "Tctl", "fan1", "in0").
    pub label: String,

    /// Current value in the unit's native scale.
    pub value: f64,

    /// What unit this reading is in.
    pub unit: SensorUnit,

    /// High/critical thresholds if the sensor reports them.
    pub thresholds: Option<Thresholds>,
}

/// The unit of measurement for a sensor reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SensorUnit {
    /// Temperature in degrees Celsius.
    Celsius,

    /// Fan speed in rotations per minute.
    Rpm,

    /// Voltage in volts.
    Volts,

    /// Power in watts.
    Watts,

    /// Current in amps.
    Amps,

    /// Relative humidity percentage.
    Humidity,

    /// Generic/unknown unit.
    Other,
}

impl fmt::Display for SensorUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Celsius => write!(f, "\u{00b0}C"),
            Self::Rpm => write!(f, " RPM"),
            Self::Volts => write!(f, " V"),
            Self::Watts => write!(f, " W"),
            Self::Amps => write!(f, " A"),
            Self::Humidity => write!(f, " %RH"),
            Self::Other => Ok(()),
        }
    }
}

/// Threshold values for a sensor reading.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Thresholds {
    /// Warning threshold (sensor-specific meaning).
    pub high: Option<f64>,

    /// Critical threshold.
    pub crit: Option<f64>,
}

impl SensorReading {
    /// Returns whether this reading exceeds the high threshold.
    #[must_use]
    pub fn is_high(&self) -> bool {
        self.thresholds
            .and_then(|t| t.high)
            .map_or(false, |high| self.value >= high)
    }

    /// Returns whether this reading exceeds the critical threshold.
    #[must_use]
    pub fn is_critical(&self) -> bool {
        self.thresholds
            .and_then(|t| t.crit)
            .map_or(false, |crit| self.value >= crit)
    }
}

/// NVIDIA GPU status from NVML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuStatus {
    /// GPU name (e.g. "NVIDIA GeForce RTX 3070").
    pub name: String,

    /// GPU temperature in degrees Celsius.
    pub temp_c: f64,

    /// Current power draw in watts.
    pub power_w: f64,

    /// Fan speed as a percentage (0-100).
    pub fan_pct: u32,

    /// GPU utilization as a percentage (0-100).
    pub util_pct: u32,

    /// Used video memory in megabytes.
    pub mem_used_mb: u64,

    /// Total video memory in megabytes.
    pub mem_total_mb: u64,
}

/// Active state of a systemd unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActiveState {
    /// Unit is currently active/running.
    Active,

    /// Unit is being activated.
    Activating,

    /// Unit is not active.
    Inactive,

    /// Unit is being deactivated.
    Deactivating,

    /// Unit has failed.
    Failed,

    /// Unit is being reloaded.
    Reloading,
}

impl fmt::Display for ActiveState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Activating => write!(f, "activating"),
            Self::Inactive => write!(f, "inactive"),
            Self::Deactivating => write!(f, "deactivating"),
            Self::Failed => write!(f, "failed"),
            Self::Reloading => write!(f, "reloading"),
        }
    }
}

impl ActiveState {
    /// Parses an active state from a systemd string.
    #[must_use]
    pub fn from_str_lossy(s: &str) -> Self {
        match s {
            "active" => Self::Active,
            "activating" => Self::Activating,
            "inactive" => Self::Inactive,
            "deactivating" => Self::Deactivating,
            "failed" => Self::Failed,
            "reloading" => Self::Reloading,
            _ => Self::Inactive,
        }
    }
}

/// Status of a tracked systemd unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdUnit {
    /// Unit name (e.g. "sensor-logger.timer", "pipewire.service").
    pub name: String,

    /// Current active state.
    pub active_state: ActiveState,

    /// Sub-state (e.g. "running", "waiting", "dead", "listening").
    pub sub_state: String,

    /// Human-readable description.
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensor_unit_display() {
        assert_eq!(format!("{}", SensorUnit::Celsius), "\u{00b0}C");
        assert_eq!(format!("{}", SensorUnit::Rpm), " RPM");
        assert_eq!(format!("{}", SensorUnit::Watts), " W");
    }

    #[test]
    fn threshold_checks() {
        let reading = SensorReading {
            label: "Tctl".to_string(),
            value: 85.0,
            unit: SensorUnit::Celsius,
            thresholds: Some(Thresholds {
                high: Some(80.0),
                crit: Some(95.0),
            }),
        };

        assert!(reading.is_high());
        assert!(!reading.is_critical());

        let critical = SensorReading {
            value: 96.0,
            ..reading.clone()
        };
        assert!(critical.is_critical());
    }

    #[test]
    fn threshold_none_is_safe() {
        let reading = SensorReading {
            label: "temp1".to_string(),
            value: 100.0,
            unit: SensorUnit::Celsius,
            thresholds: None,
        };

        assert!(!reading.is_high());
        assert!(!reading.is_critical());
    }

    #[test]
    fn active_state_roundtrip() {
        assert_eq!(ActiveState::from_str_lossy("active"), ActiveState::Active);
        assert_eq!(ActiveState::from_str_lossy("failed"), ActiveState::Failed);
        assert_eq!(
            ActiveState::from_str_lossy("garbage"),
            ActiveState::Inactive
        );
    }
}
