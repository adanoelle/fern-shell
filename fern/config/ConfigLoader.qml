pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io

// ConfigLoader handles reading and watching the JSON config file.
// The config.toml is converted to config.json by Nix at build time.
// This singleton watches for changes and updates Theme accordingly.

Singleton {
    id: root

    // Path to the config file
    readonly property string configPath: Quickshell.configDir + "/fern/config.json"

    // Fallback path (shipped with package)
    readonly property string defaultConfigPath: Qt.resolvedUrl("../config.json")

    // Current loaded config
    property var config: ({})

    // Loading state
    property bool loaded: false
    property string error: ""

    // Signal when config changes
    signal configChanged()

    // File watcher for live reload
    FileView {
        id: configFile
        path: root.configPath
        watchChanges: true

        onTextChanged: {
            if (text) {
                root.parseConfig(text);
            }
        }

        onErrorChanged: {
            if (error) {
                console.warn("ConfigLoader: Cannot read config file:", error);
                root.loadDefaultConfig();
            }
        }
    }

    // Parse JSON config string
    function parseConfig(jsonText: string) {
        try {
            const parsed = JSON.parse(jsonText);
            config = parsed;
            loaded = true;
            error = "";
            console.log("ConfigLoader: Configuration loaded successfully");
            configChanged();
            Theme.applyConfig(config);
        } catch (e) {
            error = "Failed to parse config: " + e.message;
            console.error("ConfigLoader:", error);
            loadDefaultConfig();
        }
    }

    // Load default/fallback config
    function loadDefaultConfig() {
        console.log("ConfigLoader: Loading default configuration");
        config = getDefaultConfig();
        loaded = true;
        configChanged();
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
        // Try to load config file, fall back to defaults
        if (!configFile.text) {
            loadDefaultConfig();
        }
    }
}
