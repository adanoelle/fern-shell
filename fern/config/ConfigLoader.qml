pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io
import "../services" as Services

// ConfigLoader handles reading and watching the JSON config file.
// The config.toml is converted to config.json by Nix at build time.
// This singleton watches for changes and updates Theme accordingly.

Singleton {
    id: root

    // Path to the config file
    // Use XDG_CONFIG_HOME or fallback to ~/.config
    readonly property string configPath: {
        const xdgConfig = Quickshell.env("XDG_CONFIG_HOME") || (Quickshell.env("HOME") + "/.config");
        return xdgConfig + "/fern/config.json";
    }

    // Fallback path (shipped with package)
    readonly property string defaultConfigPath: Qt.resolvedUrl("../config.json")

    // Current loaded config
    property var config: ({})

    // Loading state
    property bool loaded: false
    property string error: ""

    // Note: configChanged() signal is auto-generated from the config property

    // File watcher for live reload
    FileView {
        id: configFile
        path: root.configPath
        watchChanges: true

        onTextChanged: {
            if (text && text.length > 0) {
                root.parseConfig(text);
            } else if (root.loaded === false) {
                // File doesn't exist or is empty, load defaults
                root.loadDefaultConfig();
            }
        }
    }

    // Maximum config file size (1MB) - prevents DoS from malicious files
    readonly property int maxConfigSize: 1024 * 1024

    // Parse JSON config string
    function parseConfig(jsonText: string) {
        // Size limit check to prevent memory issues
        if (jsonText.length > maxConfigSize) {
            error = "Config file too large (max " + (maxConfigSize / 1024) + "KB)";
            Services.Log.error("ConfigLoader", error, { size: jsonText.length, max: maxConfigSize });
            loadDefaultConfig();
            return;
        }

        try {
            const parsed = JSON.parse(jsonText);
            config = parsed;
            loaded = true;
            error = "";
            Services.Log.info("ConfigLoader", "Loaded config", {
                path: configPath,
                variant: parsed.variant ?? "none"
            });
            Theme.applyConfig(config);
        } catch (e) {
            error = "Failed to parse config: " + e.message;
            Services.Log.error("ConfigLoader", "Failed to parse config", { error: e.message });
            loadDefaultConfig();
        }
    }

    // Load default/fallback config
    function loadDefaultConfig() {
        Services.Log.info("ConfigLoader", "Loading default configuration");
        config = getDefaultConfig();
        loaded = true;
        Theme.applyConfig(config);
    }

    // Default configuration (matches config.toml defaults)
    function getDefaultConfig(): var {
        return {
            appearance: {
                theme: "dark",
                accent: "#89b4fa",
                font_family: "Inter",
                font_mono: "JetBrainsMono Nerd Font",
                font_icon: "Material Symbols Rounded",
                radius: {
                    none: 0,
                    sm: 4,
                    md: 8,
                    lg: 12,
                    full: 9999
                }
            },
            bar: {
                height: 40,
                position: "top",
                margin: 0,
                modules_left: ["workspaces"],
                modules_center: ["clock"],
                modules_right: ["tray"]
            },
            modules: {
                clock: {
                    format: "HH:mm",
                    show_date: false,
                    date_format: "ddd, MMM d"
                },
                workspaces: {
                    show_empty: false,
                    count: 10
                },
                tray: {
                    icon_size: 16
                }
            }
        };
    }

    // Reload config from file
    function reload() {
        configFile.reload();
    }

    Component.onCompleted: {
        Services.Log.info("ConfigLoader", "Initialized", { path: configPath });
        // Try to load config file, fall back to defaults
        if (!configFile.text) {
            Services.Log.info("ConfigLoader", "File not found, using defaults");
            loadDefaultConfig();
        }
    }
}
