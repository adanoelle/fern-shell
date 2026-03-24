//! # Hardware Sensor Reading via libsensors
//!
//! Iterates over all lm_sensors chips and their features to produce
//! structured [`SensorChip`] readings.

use crate::snapshot::{SensorChip, SensorReading, SensorUnit, Thresholds};
use crate::SysMonError;

/// Reads all available sensor chips and their readings from libsensors.
///
/// # Errors
///
/// Returns an error if libsensors cannot be initialized.
pub fn read_sensors(
    sensors: &lm_sensors::LMSensors,
) -> std::result::Result<Vec<SensorChip>, SysMonError> {
    let mut chips = Vec::new();

    for chip in sensors.chip_iter(None) {
        let chip_name = format!("{chip}");
        let adapter = chip
            .bus()
            .and_then(|bus| sensors.bus_name(&bus).ok())
            .map_or_else(String::new, |name| format!("{name}"));

        let mut readings = Vec::new();

        for feature in chip.feature_iter() {
            let label = feature
                .name()
                .transpose()
                .ok()
                .flatten()
                .unwrap_or("unknown")
                .to_string();

            let kind = feature.kind();
            let unit = feature_kind_to_unit(kind);

            // Read the primary value and thresholds from sub-features
            let mut value: Option<f64> = None;
            let mut high: Option<f64> = None;
            let mut crit: Option<f64> = None;

            for sub_feature in feature.sub_feature_iter() {
                let sf_name = format!("{sub_feature}");
                if let Ok(v) = sub_feature.value() {
                    let v = f64::from(v);
                    if sf_name.contains("_input") || value.is_none() {
                        // First sub-feature or explicit _input is the primary value
                        if sf_name.contains("_input") {
                            value = Some(v);
                        } else if value.is_none() {
                            value = Some(v);
                        }
                    }
                    if sf_name.contains("_max") || sf_name.contains("_high") {
                        high = Some(v);
                    }
                    if sf_name.contains("_crit") && !sf_name.contains("_crit_alarm") {
                        crit = Some(v);
                    }
                }
            }

            if let Some(val) = value {
                let thresholds = if high.is_some() || crit.is_some() {
                    Some(Thresholds { high, crit })
                } else {
                    None
                };

                readings.push(SensorReading {
                    label,
                    value: val,
                    unit,
                    thresholds,
                });
            }
        }

        if !readings.is_empty() {
            chips.push(SensorChip {
                name: chip_name,
                adapter,
                readings,
            });
        }
    }

    Ok(chips)
}

/// Maps an lm_sensors feature kind to our [`SensorUnit`].
fn feature_kind_to_unit(kind: Option<lm_sensors::feature::Kind>) -> SensorUnit {
    match kind {
        Some(lm_sensors::feature::Kind::Temperature) => SensorUnit::Celsius,
        Some(lm_sensors::feature::Kind::Fan) => SensorUnit::Rpm,
        Some(lm_sensors::feature::Kind::Voltage) => SensorUnit::Volts,
        Some(lm_sensors::feature::Kind::Power) => SensorUnit::Watts,
        Some(lm_sensors::feature::Kind::Current) => SensorUnit::Amps,
        Some(lm_sensors::feature::Kind::Humidity) => SensorUnit::Humidity,
        _ => SensorUnit::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_kind_mapping() {
        assert_eq!(
            feature_kind_to_unit(Some(lm_sensors::feature::Kind::Temperature)),
            SensorUnit::Celsius
        );
        assert_eq!(
            feature_kind_to_unit(Some(lm_sensors::feature::Kind::Fan)),
            SensorUnit::Rpm
        );
        assert_eq!(feature_kind_to_unit(None), SensorUnit::Other);
    }
}
