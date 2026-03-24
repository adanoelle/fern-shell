//! # Hardware State
//!
//! Holds a ring buffer of [`SystemSnapshot`]s for sparkline rendering
//! and provides access to the latest readings.

use fern_sysmon::SystemSnapshot;
use std::collections::VecDeque;

/// Default number of snapshots to retain (1/sec = 2 minutes).
const DEFAULT_CAPACITY: usize = 120;

/// Hardware monitoring state with historical data for sparkline graphs.
#[derive(Debug)]
pub struct HardwareState {
    /// Ring buffer of snapshots, newest at back.
    snapshots: VecDeque<SystemSnapshot>,

    /// Maximum number of snapshots to keep.
    capacity: usize,

    /// Currently selected chip index in the hardware panel.
    pub selected_chip: usize,
}

impl HardwareState {
    /// Creates a new hardware state with default capacity.
    #[must_use]
    pub fn new() -> Self {
        Self {
            snapshots: VecDeque::with_capacity(DEFAULT_CAPACITY),
            capacity: DEFAULT_CAPACITY,
            selected_chip: 0,
        }
    }

    /// Pushes a new snapshot, evicting the oldest if at capacity.
    pub fn push(&mut self, snapshot: SystemSnapshot) {
        if self.snapshots.len() >= self.capacity {
            self.snapshots.pop_front();
        }
        self.snapshots.push_back(snapshot);
    }

    /// Returns the latest snapshot, if any.
    #[must_use]
    pub fn latest(&self) -> Option<&SystemSnapshot> {
        self.snapshots.back()
    }

    /// Returns the number of stored snapshots.
    #[must_use]
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Returns whether there are no stored snapshots.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    /// Extracts sparkline data for a specific chip and reading label.
    ///
    /// Returns a `Vec<u64>` of values suitable for ratatui's `Sparkline` widget.
    /// Values are scaled to integer range for sparkline rendering.
    #[must_use]
    pub fn sparkline_data(&self, chip_name: &str, label: &str) -> Vec<u64> {
        self.snapshots
            .iter()
            .filter_map(|snapshot| {
                snapshot
                    .sensors
                    .iter()
                    .find(|c| c.name == chip_name)
                    .and_then(|chip| chip.readings.iter().find(|r| r.label == label))
                    .map(|r| r.value as u64)
            })
            .collect()
    }

    /// Extracts sparkline data for GPU temperature.
    #[must_use]
    pub fn gpu_temp_sparkline(&self) -> Vec<u64> {
        self.snapshots
            .iter()
            .filter_map(|s| s.gpu.as_ref().map(|g| g.temp_c as u64))
            .collect()
    }

    /// Extracts sparkline data for GPU power draw.
    #[must_use]
    pub fn gpu_power_sparkline(&self) -> Vec<u64> {
        self.snapshots
            .iter()
            .filter_map(|s| s.gpu.as_ref().map(|g| g.power_w as u64))
            .collect()
    }

    /// Returns the total number of chips in the latest snapshot.
    #[must_use]
    pub fn chip_count(&self) -> usize {
        self.latest().map_or(0, |s| s.sensors.len())
    }
}

impl Default for HardwareState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use fern_sysmon::{SensorChip, SensorReading, SensorUnit};

    fn make_snapshot(temp: f64) -> SystemSnapshot {
        SystemSnapshot {
            timestamp: Utc::now(),
            sensors: vec![SensorChip {
                name: "k10temp-pci-00c3".to_string(),
                adapter: "PCI adapter".to_string(),
                readings: vec![SensorReading {
                    label: "Tctl".to_string(),
                    value: temp,
                    unit: SensorUnit::Celsius,
                    thresholds: None,
                }],
            }],
            gpu: None,
            units: vec![],
        }
    }

    #[test]
    fn push_and_latest() {
        let mut state = HardwareState::new();
        assert!(state.is_empty());

        state.push(make_snapshot(70.0));
        assert_eq!(state.len(), 1);

        let latest = state.latest().unwrap();
        assert_eq!(latest.sensors[0].readings[0].value, 70.0);
    }

    #[test]
    fn capacity_eviction() {
        let mut state = HardwareState::new();

        for i in 0..150 {
            state.push(make_snapshot(f64::from(i)));
        }

        assert_eq!(state.len(), DEFAULT_CAPACITY);
        // Oldest should have been evicted; first remaining = 150 - 120 = 30
        let first = state.snapshots.front().unwrap();
        assert_eq!(first.sensors[0].readings[0].value, 30.0);
    }

    #[test]
    fn sparkline_data_extraction() {
        let mut state = HardwareState::new();

        for i in 0..5 {
            state.push(make_snapshot(f64::from(i * 10)));
        }

        let data = state.sparkline_data("k10temp-pci-00c3", "Tctl");
        assert_eq!(data, vec![0, 10, 20, 30, 40]);
    }

    #[test]
    fn sparkline_missing_chip() {
        let mut state = HardwareState::new();
        state.push(make_snapshot(50.0));

        let data = state.sparkline_data("nonexistent", "Tctl");
        assert!(data.is_empty());
    }
}
