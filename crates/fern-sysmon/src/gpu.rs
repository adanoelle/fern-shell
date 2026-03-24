//! # NVIDIA GPU Monitoring via NVML
//!
//! Reads GPU temperature, power draw, fan speed, utilization, and memory
//! usage from the NVIDIA Management Library. Feature-gated behind `nvidia`.

use crate::snapshot::GpuStatus;
use crate::SysMonError;

/// Handle to the NVML library for GPU queries.
pub struct GpuMonitor {
    nvml: nvml_wrapper::Nvml,
}

impl GpuMonitor {
    /// Initializes NVML.
    ///
    /// # Errors
    ///
    /// Returns an error if the NVML library cannot be loaded (e.g. no NVIDIA
    /// driver installed).
    pub fn new() -> std::result::Result<Self, SysMonError> {
        let nvml = nvml_wrapper::Nvml::init().map_err(|e| SysMonError::Gpu(e.to_string()))?;
        Ok(Self { nvml })
    }

    /// Reads the status of the first GPU (index 0).
    ///
    /// # Errors
    ///
    /// Returns an error if NVML queries fail.
    pub fn read_gpu(&self) -> std::result::Result<GpuStatus, SysMonError> {
        let device = self
            .nvml
            .device_by_index(0)
            .map_err(|e| SysMonError::Gpu(e.to_string()))?;

        let name = device
            .name()
            .map_err(|e| SysMonError::Gpu(e.to_string()))?;

        let temp_c = f64::from(
            device
                .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                .map_err(|e| SysMonError::Gpu(e.to_string()))?,
        );

        let power_w = f64::from(
            device
                .power_usage()
                .map_err(|e| SysMonError::Gpu(e.to_string()))?,
        ) / 1000.0; // milliwatts -> watts

        let fan_pct = device.fan_speed(0).unwrap_or(0);

        let utilization = device
            .utilization_rates()
            .map_err(|e| SysMonError::Gpu(e.to_string()))?;

        let mem_info = device
            .memory_info()
            .map_err(|e| SysMonError::Gpu(e.to_string()))?;

        Ok(GpuStatus {
            name,
            temp_c,
            power_w,
            fan_pct,
            util_pct: utilization.gpu,
            mem_used_mb: mem_info.used / (1024 * 1024),
            mem_total_mb: mem_info.total / (1024 * 1024),
        })
    }
}
