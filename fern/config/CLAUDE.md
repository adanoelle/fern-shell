# Fern Configuration System Guide

## Purpose

The configuration system provides a flexible, user-friendly way to customize
Fern Shell. It supports live reloading, schema validation, and seamless
integration with both QML and Nix.

## Architecture Overview

```
~/.config/fern/config.json  →  FileWatcher  →  Config Singleton  →  QML Components
                                     ↓                ↑
                                Live Reload    Schema Validation
```

## Core Configuration Singleton

```qml
// fern/config/Config.qml
pragma Singleton
import QtQuick
import Quickshell.Io

Singleton {
    id: root

    // Configuration properties
    property alias appearance: appearance
    property alias bar: bar
    property alias modules: modules
    property alias shortcuts: shortcuts

    // Sub-configurations
    AppearanceConfig { id: appearance }
    BarConfig { id: bar }
    ModulesConfig { id: modules }
    ShortcutsConfig { id: shortcuts }

    // Configuration file handling
    FileView {
        id: configFile
        path: Paths.config + "/config.json"
        watchChanges: true

        onFileChanged: root.reload()
    }

    // JSON adapter for automatic binding
    JsonAdapter {
        id: adapter
        source: configFile.text

        onParsed: {
            root.applyConfig(data);
            root.configChanged();
        }

        onError: {
            console.error("Config parse error:", error);
            NotificationService.show("Configuration Error", error, "error");
        }
    }

    // Signals
    signal configChanged()
    signal themeChanged()

    // Methods
    function reload() {
        console.log("Reloading configuration...");
        adapter.parse();
    }

    function save() {
        const json = JSON.stringify(root.toJSON(), null, 2);
        FileIO.write(configFile.path, json);
    }

    function reset() {
        root.applyConfig(DefaultConfig.get());
        root.save();
    }

    function applyConfig(data: var) {
        appearance.apply(data.appearance || {});
        bar.apply(data.bar || {});
        modules.apply(data.modules || {});
        shortcuts.apply(data.shortcuts || {});
    }

    function toJSON(): var {
        return {
            appearance: appearance.toJSON(),
            bar: bar.toJSON(),
            modules: modules.toJSON(),
            shortcuts: shortcuts.toJSON()
        };
    }

    Component.onCompleted: {
        // Create default config if doesn't exist
        if (!FileIO.exists(configFile.path)) {
            console.log("Creating default configuration...");
            reset();
        }
        reload();
    }
}
```

## Configuration Modules

### Appearance Configuration

```qml
// fern/config/AppearanceConfig.qml
import QtQuick

QtObject {
    id: root

    // Theme
    property string theme: "dark"
    property string accentColor: "#89b4fa"

    // Typography
    property string fontFamily: "Inter"
    property string monoFontFamily: "JetBrainsMono Nerd Font"
    property string iconFontFamily: "Material Symbols Rounded"
    property int fontSize: 14

    // Spacing & Sizing
    property var spacing: {
        "tiny": 2,
        "small": 4,
        "normal": 8,
        "large": 16,
        "huge": 32
    }

    property var radius: {
        "none": 0,
        "small": 4,
        "normal": 8,
        "medium": 12,
        "large": 16,
        "full": 999
    }

    // Animation
    property var animation: {
        "instant": 0,
        "fast": 100,
        "normal": 200,
        "slow": 300,
        "verySlow": 500
    }

    // Computed palette based on theme
    readonly property var palette: theme === "dark" ? darkPalette : lightPalette

    readonly property var darkPalette: {
        "background": "#1e1e2e",
        "surface": "#313244",
        "surfaceVariant": "#45475a",
        "foreground": "#cdd6f4",
        "foregroundDim": "#a6adc8",
        "accent": accentColor,
        "error": "#f38ba8",
        "warning": "#f9e2af",
        "success": "#a6e3a1",
        "info": "#89dceb"
    }

    readonly property var lightPalette: {
        "background": "#eff1f5",
        "surface": "#e6e9ef",
        "surfaceVariant": "#dce0e8",
        "foreground": "#4c4f69",
        "foregroundDim": "#6c6f85",
        "accent": accentColor,
        "error": "#d20f39",
        "warning": "#df8e1d",
        "success": "#40a02b",
        "info": "#04a5e5"
    }

    function apply(data: var) {
        if (data.theme !== undefined) theme = data.theme;
        if (data.accentColor !== undefined) accentColor = data.accentColor;
        if (data.fontFamily !== undefined) fontFamily = data.fontFamily;
        if (data.fontSize !== undefined) fontSize = data.fontSize;
        if (data.spacing !== undefined) spacing = data.spacing;
        if (data.radius !== undefined) radius = data.radius;
        if (data.animation !== undefined) animation = data.animation;
    }

    function toJSON(): var {
        return {
            theme: theme,
            accentColor: accentColor,
            fontFamily: fontFamily,
            monoFontFamily: monoFontFamily,
            iconFontFamily: iconFontFamily,
            fontSize: fontSize,
            spacing: spacing,
            radius: radius,
            animation: animation
        };
    }
}
```

### Bar Configuration

```qml
// fern/config/BarConfig.qml
import QtQuick

QtObject {
    id: root

    // Position and sizing
    property string position: "top"  // top, bottom, left, right
    property int height: 32
    property int margin: 0
    property bool exclusive: true  // Reserve space

    // Layout
    property var layout: [
        { "module": "workspaces", "position": "left" },
        { "module": "activeWindow", "position": "left" },
        { "module": "spacer", "position": "center" },
        { "module": "clock", "position": "center" },
        { "module": "spacer", "position": "right" },
        { "module": "tray", "position": "right" },
        { "module": "statusIcons", "position": "right" }
    ]

    // Per-monitor configuration
    property bool perMonitor: true
    property var monitorOverrides: ({})  // Monitor-specific settings

    function apply(data: var) {
        if (data.position !== undefined) position = data.position;
        if (data.height !== undefined) height = data.height;
        if (data.margin !== undefined) margin = data.margin;
        if (data.exclusive !== undefined) exclusive = data.exclusive;
        if (data.layout !== undefined) layout = data.layout;
        if (data.perMonitor !== undefined) perMonitor = data.perMonitor;
        if (data.monitorOverrides !== undefined) monitorOverrides = data.monitorOverrides;
    }

    function toJSON(): var {
        return {
            position: position,
            height: height,
            margin: margin,
            exclusive: exclusive,
            layout: layout,
            perMonitor: perMonitor,
            monitorOverrides: monitorOverrides
        };
    }
}
```

### Modules Configuration

```qml
// fern/config/ModulesConfig.qml
import QtQuick

QtObject {
    id: root

    // Module-specific configurations
    property var workspaces: {
        "count": 10,
        "persistent": [1, 2, 3],
        "format": "{index}",
        "showEmpty": false,
        "scrollable": true,
        "onClick": "activate",
        "onScroll": "cycle"
    }

    property var clock: {
        "format": "HH:mm",
        "dateFormat": "ddd, MMM d",
        "showDate": false,
        "timezone": "local",
        "onClick": "toggleDate"
    }

    property var tray: {
        "iconSize": 16,
        "spacing": 4,
        "reverseOrder": false
    }

    property var statusIcons: {
        "order": ["network", "audio", "battery"],
        "showLabels": false,
        "interactive": true
    }

    property var activeWindow: {
        "maxLength": 50,
        "format": "{title}",
        "showAppName": true
    }

    // Dynamic module registry
    property var custom: ({})

    function apply(data: var) {
        if (data.workspaces !== undefined) workspaces = data.workspaces;
        if (data.clock !== undefined) clock = data.clock;
        if (data.tray !== undefined) tray = data.tray;
        if (data.statusIcons !== undefined) statusIcons = data.statusIcons;
        if (data.activeWindow !== undefined) activeWindow = data.activeWindow;
        if (data.custom !== undefined) custom = data.custom;
    }

    function get(moduleName: string): var {
        return root[moduleName] || custom[moduleName] || {};
    }

    function toJSON(): var {
        return {
            workspaces: workspaces,
            clock: clock,
            tray: tray,
            statusIcons: statusIcons,
            activeWindow: activeWindow,
            custom: custom
        };
    }
}
```

## Configuration Schema

### JSON Schema Definition

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Fern Shell Configuration",
  "type": "object",
  "properties": {
    "appearance": {
      "type": "object",
      "properties": {
        "theme": {
          "type": "string",
          "enum": ["dark", "light", "auto"],
          "default": "dark"
        },
        "accentColor": {
          "type": "string",
          "pattern": "^#[0-9a-fA-F]{6}$",
          "default": "#89b4fa"
        },
        "fontSize": {
          "type": "integer",
          "minimum": 10,
          "maximum": 24,
          "default": 14
        }
      }
    },
    "bar": {
      "type": "object",
      "properties": {
        "position": {
          "type": "string",
          "enum": ["top", "bottom", "left", "right"],
          "default": "top"
        },
        "height": {
          "type": "integer",
          "minimum": 20,
          "maximum": 100,
          "default": 32
        }
      }
    },
    "modules": {
      "type": "object",
      "additionalProperties": {
        "type": "object"
      }
    }
  }
}
```

### Schema Validation

```qml
// fern/config/SchemaValidator.qml
pragma Singleton

Singleton {
    function validate(config: var, schema: var): var {
        const errors = [];

        function validateType(value: var, schema: var, path: string) {
            // Type checking
            if (schema.type && typeof value !== schema.type) {
                errors.push(`${path}: Expected ${schema.type}, got ${typeof value}`);
            }

            // Enum validation
            if (schema.enum && !schema.enum.includes(value)) {
                errors.push(`${path}: Must be one of ${schema.enum.join(", ")}`);
            }

            // Pattern validation
            if (schema.pattern && !new RegExp(schema.pattern).test(value)) {
                errors.push(`${path}: Does not match pattern ${schema.pattern}`);
            }

            // Nested object validation
            if (schema.properties && typeof value === "object") {
                for (let key in schema.properties) {
                    if (value[key] !== undefined) {
                        validateType(value[key], schema.properties[key], `${path}.${key}`);
                    }
                }
            }
        }

        validateType(config, schema, "config");

        return {
            valid: errors.length === 0,
            errors: errors
        };
    }
}
```

## User Configuration File

### Example config.json

```json
{
  "appearance": {
    "theme": "dark",
    "accentColor": "#89b4fa",
    "fontFamily": "Inter",
    "fontSize": 14,
    "animation": {
      "normal": 200,
      "fast": 100
    }
  },
  "bar": {
    "position": "top",
    "height": 32,
    "layout": [
      { "module": "workspaces", "position": "left" },
      { "module": "clock", "position": "center" },
      { "module": "tray", "position": "right" }
    ]
  },
  "modules": {
    "workspaces": {
      "count": 10,
      "showEmpty": false
    },
    "clock": {
      "format": "HH:mm:ss",
      "showDate": true
    }
  },
  "shortcuts": {
    "terminal": "Super+Return",
    "launcher": "Super+Space",
    "reload": "Super+Shift+R"
  }
}
```

## Live Reload Implementation

```qml
// Automatic configuration reload
FileWatcher {
    path: Paths.config
    recursive: true

    onFileChanged: (file) => {
        if (file.endsWith(".json")) {
            console.log("Configuration file changed:", file);
            Config.reload();
        }
    }
}

// Graceful reload with error handling
function reload() {
    try {
        const newConfig = JSON.parse(FileIO.read(configPath));
        const validation = SchemaValidator.validate(newConfig, schema);

        if (validation.valid) {
            applyConfig(newConfig);
            NotificationService.show("Configuration Reloaded", "success");
        } else {
            console.error("Invalid configuration:", validation.errors);
            NotificationService.show("Configuration Error", validation.errors[0], "error");
        }
    } catch (e) {
        console.error("Failed to reload config:", e);
        // Keep using previous valid configuration
    }
}
```

## Nix Integration

### Home-Manager Module

```nix
{ config, lib, pkgs, ... }:
let
  cfg = config.programs.fern-shell;

  # Generate config.json from Nix
  configJson = pkgs.writeText "fern-config.json" (builtins.toJSON {
    appearance = cfg.appearance;
    bar = cfg.bar;
    modules = cfg.modules;
  });
in {
  options.programs.fern-shell = {
    appearance = {
      theme = mkOption {
        type = types.enum ["dark" "light" "auto"];
        default = "dark";
      };
      accentColor = mkOption {
        type = types.str;
        default = "#89b4fa";
      };
    };
    # ... more options
  };

  config = mkIf cfg.enable {
    # Link generated config
    xdg.configFile."fern/config.json".source = configJson;
  };
}
```

## Migration & Upgrades

```qml
// fern/config/ConfigMigrator.qml
pragma Singleton

Singleton {
    property var migrations: [
        {
            version: "0.2.0",
            migrate: function(config) {
                // Rename old property
                if (config.panel) {
                    config.bar = config.panel;
                    delete config.panel;
                }
                return config;
            }
        },
        {
            version: "0.3.0",
            migrate: function(config) {
                // Add new required field
                if (!config.appearance.animation) {
                    config.appearance.animation = DefaultConfig.appearance.animation;
                }
                return config;
            }
        }
    ]

    function migrate(config: var, fromVersion: string, toVersion: string): var {
        let migratedConfig = config;

        for (let migration of migrations) {
            if (versionCompare(migration.version, fromVersion) > 0 &&
                versionCompare(migration.version, toVersion) <= 0) {
                console.log(`Migrating config to ${migration.version}`);
                migratedConfig = migration.migrate(migratedConfig);
            }
        }

        return migratedConfig;
    }
}
```

## Performance Considerations

### Lazy Configuration Loading

```qml
// Only load module config when module is used
Loader {
    active: moduleEnabled
    sourceComponent: Module {
        config: Config.modules.get(moduleName)
    }
}
```

### Configuration Caching

```qml
QtObject {
    id: configCache
    property var cache: ({})
    property var timestamps: ({})

    function get(key: string): var {
        if (Date.now() - timestamps[key] < 1000) {
            return cache[key];
        }
        return null;
    }

    function set(key: string, value: var) {
        cache[key] = value;
        timestamps[key] = Date.now();
    }
}
```

## Debugging Configuration

```bash
# Validate configuration
fern-validate-config ~/.config/fern/config.json

# Test with different config
FERN_CONFIG=./test-config.json qs -p fern/shell.qml

# Debug config loading
QS_DEBUG=config qs -p fern/shell.qml
```

## Best Practices

### DO ✅

- Provide sensible defaults
- Validate user input
- Handle missing fields gracefully
- Support live reload
- Document all options
- Version configuration schema

### DON'T ❌

- Don't crash on invalid config
- Don't lose user customizations
- Don't require restart for changes
- Don't expose internal properties
- Don't forget migration path

## Next Steps

1. Define configuration schema
2. Implement Config singleton
3. Add schema validation
4. Create default configuration
5. Implement live reload
6. Add Nix integration
7. Test migration scenarios

Remember: Configuration should be powerful yet approachable - users should feel
confident customizing their shell.
