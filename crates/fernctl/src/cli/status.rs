//! # Status Command
//!
//! Shows the status of all services or a specific service.

use crate::domain::KnownService;
use crate::error::Result;
use fern_core::state::ServiceRegistry;
use fern_core::FernPaths;
use std::fs;

/// Options for the status command.
#[derive(Debug, Clone)]
pub struct StatusOptions {
    /// Specific service to show, or None for all.
    pub service: Option<String>,
    /// Output format.
    pub format: OutputFormat,
    /// Show verbose output.
    pub verbose: bool,
}

/// Output format for commands.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable text output.
    #[default]
    Text,
    /// JSON output for scripting.
    Json,
}

/// Runs the status command.
///
/// # Errors
///
/// Returns an error if state files cannot be read.
pub fn run(options: StatusOptions) -> Result<()> {
    let paths = FernPaths::new();

    if let Some(service_name) = &options.service {
        // Show specific service
        if let Some(service) = KnownService::from_name(service_name) {
            show_service_status(&paths, service, &options)?;
        } else {
            eprintln!("Unknown service: {service_name}");
            eprintln!("Available services: obs, shell, theme-watcher");
        }
    } else {
        // Show all services
        show_all_status(&paths, &options)?;
    }

    Ok(())
}

fn show_service_status(paths: &FernPaths, service: KnownService, options: &StatusOptions) -> Result<()> {
    let state_path = paths.state_dir().join(service.state_file());

    let status = if state_path.exists() {
        match fs::read_to_string(&state_path) {
            Ok(content) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    format_service_json(&json, service, options.verbose)
                } else {
                    format!("{}: state file corrupt", service.display_name())
                }
            }
            Err(_) => format!("{}: cannot read state", service.display_name()),
        }
    } else {
        format!("{}: stopped (no state file)", service.display_name())
    };

    match options.format {
        OutputFormat::Text => println!("{status}"),
        OutputFormat::Json => {
            if let Ok(content) = fs::read_to_string(&state_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
                }
            }
        }
    }

    Ok(())
}

fn show_all_status(paths: &FernPaths, options: &StatusOptions) -> Result<()> {
    // Try to read the services registry first
    let registry_path = paths.services_registry();
    let registry: Option<ServiceRegistry> = if registry_path.exists() {
        fs::read_to_string(&registry_path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
    } else {
        None
    };

    match options.format {
        OutputFormat::Text => {
            println!("Fern Shell Services");
            println!("-------------------");

            for service in KnownService::all() {
                let state_path = paths.state_dir().join(service.state_file());
                let status = get_service_status_line(service, &state_path, &registry);
                println!("{status}");
            }
        }
        OutputFormat::Json => {
            let mut output = serde_json::Map::new();

            for service in KnownService::all() {
                let state_path = paths.state_dir().join(service.state_file());
                if let Ok(content) = fs::read_to_string(&state_path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        output.insert(service.name().to_string(), json);
                    }
                }
            }

            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::Value::Object(output))
                    .unwrap_or_default()
            );
        }
    }

    Ok(())
}

fn get_service_status_line(
    service: &KnownService,
    state_path: &std::path::Path,
    registry: &Option<ServiceRegistry>,
) -> String {
    let indicator = if state_path.exists() {
        if let Ok(content) = fs::read_to_string(state_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                let connected = json.get("connected").and_then(|v| v.as_bool()).unwrap_or(false);
                let active = json
                    .get("recording")
                    .and_then(|r| r.get("active"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                    || json
                        .get("streaming")
                        .and_then(|s| s.get("active"))
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                if connected || active {
                    "\x1b[32m●\x1b[0m" // Green dot
                } else {
                    "\x1b[33m○\x1b[0m" // Yellow empty dot
                }
            } else {
                "\x1b[31m✗\x1b[0m" // Red X
            }
        } else {
            "\x1b[31m✗\x1b[0m"
        }
    } else {
        // Check registry for status
        if let Some(reg) = registry {
            if let Some(info) = reg.find(service.name()) {
                if info.is_running() {
                    "\x1b[32m●\x1b[0m"
                } else {
                    "\x1b[90m○\x1b[0m" // Gray dot
                }
            } else {
                "\x1b[90m○\x1b[0m"
            }
        } else {
            "\x1b[90m○\x1b[0m"
        }
    };

    let status_text = if state_path.exists() {
        if let Ok(content) = fs::read_to_string(state_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                get_status_text_from_json(&json, service)
            } else {
                "corrupt".to_string()
            }
        } else {
            "unreadable".to_string()
        }
    } else {
        "stopped".to_string()
    };

    format!(
        "{} {:14} {}",
        indicator,
        service.display_name(),
        status_text
    )
}

fn format_service_json(json: &serde_json::Value, service: KnownService, verbose: bool) -> String {
    let status = get_status_text_from_json(json, &service);

    if verbose {
        format!(
            "{}: {}\n{}",
            service.display_name(),
            status,
            serde_json::to_string_pretty(json).unwrap_or_default()
        )
    } else {
        format!("{}: {}", service.display_name(), status)
    }
}

fn get_status_text_from_json(json: &serde_json::Value, service: &KnownService) -> String {
    match service {
        KnownService::Obs => {
            let connected = json.get("connected").and_then(|v| v.as_bool()).unwrap_or(false);
            if !connected {
                return "disconnected".to_string();
            }

            let recording = json
                .get("recording")
                .and_then(|r| r.get("active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let streaming = json
                .get("streaming")
                .and_then(|s| s.get("active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            match (recording, streaming) {
                (true, true) => "recording & streaming".to_string(),
                (true, false) => {
                    let timecode = json
                        .get("recording")
                        .and_then(|r| r.get("timecode"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("00:00");
                    format!("recording ({timecode})")
                }
                (false, true) => "streaming".to_string(),
                (false, false) => "ready".to_string(),
            }
        }
        KnownService::Shell => {
            // Shell logs don't have a simple status
            "running".to_string()
        }
        KnownService::ThemeWatcher => {
            let watching = json.get("watching").and_then(|v| v.as_bool()).unwrap_or(false);
            if watching {
                "watching".to_string()
            } else {
                "idle".to_string()
            }
        }
    }
}
