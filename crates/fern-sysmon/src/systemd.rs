//! # Systemd Unit Monitoring via D-Bus
//!
//! Queries systemd unit status using the D-Bus API through zbus.

use crate::snapshot::{ActiveState, SystemdUnit};
use crate::SysMonError;
use zbus::blocking::Connection;
use zbus::zvariant::OwnedObjectPath;

/// Queries the status of a list of systemd units.
///
/// # Errors
///
/// Returns an error if the D-Bus connection or queries fail.
pub fn query_units(
    connection: &Connection,
    unit_names: &[String],
) -> std::result::Result<Vec<SystemdUnit>, SysMonError> {
    let mut units = Vec::with_capacity(unit_names.len());

    for unit_name in unit_names {
        match query_single_unit(connection, unit_name) {
            Ok(unit) => units.push(unit),
            Err(_) => {
                // Unit not found or query failed -- report as inactive
                units.push(SystemdUnit {
                    name: unit_name.clone(),
                    active_state: ActiveState::Inactive,
                    sub_state: "unknown".to_string(),
                    description: String::new(),
                });
            }
        }
    }

    Ok(units)
}

/// Queries a single systemd unit's status via D-Bus.
fn query_single_unit(
    connection: &Connection,
    unit_name: &str,
) -> std::result::Result<SystemdUnit, SysMonError> {
    // Call org.freedesktop.systemd1.Manager.GetUnit to get the unit object path
    let object_path: OwnedObjectPath = connection
        .call_method(
            Some("org.freedesktop.systemd1"),
            "/org/freedesktop/systemd1",
            Some("org.freedesktop.systemd1.Manager"),
            "GetUnit",
            &(unit_name,),
        )
        .map_err(|e| SysMonError::Systemd(format!("GetUnit({unit_name}): {e}")))?
        .body()
        .deserialize()
        .map_err(|e| SysMonError::Systemd(format!("deserialize path for {unit_name}: {e}")))?;

    // Read properties from the unit object
    let active_state: String = get_property(connection, &object_path, "ActiveState")?;
    let sub_state: String = get_property(connection, &object_path, "SubState")?;
    let description: String =
        get_property(connection, &object_path, "Description").unwrap_or_default();

    Ok(SystemdUnit {
        name: unit_name.to_string(),
        active_state: ActiveState::from_str_lossy(&active_state),
        sub_state,
        description,
    })
}

/// Reads a single property from a systemd unit via D-Bus.
fn get_property(
    connection: &Connection,
    object_path: &OwnedObjectPath,
    property: &str,
) -> std::result::Result<String, SysMonError> {
    let reply = connection
        .call_method(
            Some("org.freedesktop.systemd1"),
            object_path.as_str(),
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.freedesktop.systemd1.Unit", property),
        )
        .map_err(|e| SysMonError::Systemd(format!("Get {property}: {e}")))?;

    let variant: zbus::zvariant::OwnedValue = reply
        .body()
        .deserialize()
        .map_err(|e| SysMonError::Systemd(format!("deserialize {property}: {e}")))?;

    // The value is a variant containing a string
    let value: String = variant
        .try_into()
        .map_err(|e: zbus::zvariant::Error| SysMonError::Systemd(format!("cast {property}: {e}")))?;

    Ok(value)
}

/// Creates a blocking D-Bus connection to the system bus.
///
/// # Errors
///
/// Returns an error if the system bus is not available.
pub fn connect_system_bus() -> std::result::Result<Connection, SysMonError> {
    Connection::system().map_err(|e| SysMonError::Systemd(format!("system bus: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_state_parsing() {
        assert_eq!(ActiveState::from_str_lossy("active"), ActiveState::Active);
        assert_eq!(ActiveState::from_str_lossy("failed"), ActiveState::Failed);
        assert_eq!(
            ActiveState::from_str_lossy("inactive"),
            ActiveState::Inactive
        );
    }
}
