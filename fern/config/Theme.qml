pragma Singleton
import QtQuick

// Theme singleton providing design tokens for all Fern components.
// Uses a three-tier architecture:
//   Tier 1: Primitives (internal color palettes)
//   Tier 2: Semantic tokens (configurable via config.toml)
//   Tier 3: Component tokens (internal mappings)

QtObject {
    id: root

    // === CONFIGURATION INPUT ===
    // Set by ConfigLoader when config changes
    property var config: ({})

    // Signal emitted when theme values change
    signal themeChanged()

    // === TIER 1: PRIMITIVES (Internal) ===
    // Color palettes - not directly configurable

    readonly property var _darkPalette: ({
        base: "#1e1e2e",
        surface: "#313244",
        surfaceHover: "#45475a",
        surfaceActive: "#585b70",
        overlay: "#6c7086",
        text: "#cdd6f4",
        textDim: "#a6adc8",
        // Semantic colors
        red: "#f38ba8",
        yellow: "#f9e2af",
        green: "#a6e3a1",
        blue: "#89b4fa",
        purple: "#cba6f7"
    })

    readonly property var _lightPalette: ({
        base: "#eff1f5",
        surface: "#e6e9ef",
        surfaceHover: "#dce0e8",
        surfaceActive: "#ccd0da",
        overlay: "#9ca0b0",
        text: "#4c4f69",
        textDim: "#6c6f85",
        // Semantic colors
        red: "#d20f39",
        yellow: "#df8e1d",
        green: "#40a02b",
        blue: "#1e66f5",
        purple: "#8839ef"
    })

    // Active palette based on theme setting
    readonly property var _palette: _resolveTheme() === "dark" ? _darkPalette : _lightPalette

    function _resolveTheme(): string {
        const configTheme = config.appearance?.theme ?? "dark";
        if (configTheme === "auto") {
            // Future: detect system preference
            return "dark";
        }
        return configTheme;
    }

    // === TIER 2: SEMANTIC TOKENS (Configurable) ===

    // -- Colors --
    readonly property color background: _palette.base
    readonly property color surface: _palette.surface
    readonly property color surfaceHover: _palette.surfaceHover
    readonly property color surfaceActive: _palette.surfaceActive
    readonly property color overlay: _palette.overlay
    readonly property color foreground: _palette.text
    readonly property color foregroundDim: _palette.textDim

    // Accent color - user configurable
    readonly property color accent: config.appearance?.accent ?? _palette.blue

    // Semantic status colors
    readonly property color error: _palette.red
    readonly property color warning: _palette.yellow
    readonly property color success: _palette.green
    readonly property color info: _palette.blue

    // -- Typography --
    readonly property string fontFamily: config.appearance?.font_family ?? "Inter"
    readonly property string fontMono: config.appearance?.font_mono ?? "JetBrainsMono Nerd Font"
    readonly property string fontIcon: config.appearance?.font_icon ?? "Material Symbols Rounded"

    // Font size scale (not configurable - breaks layouts)
    readonly property var fontSize: ({
        xs: 10,
        sm: 12,
        md: 14,
        lg: 16,
        xl: 20,
        xxl: 24
    })

    // -- Radius --
    readonly property var radius: ({
        none: config.appearance?.radius?.none ?? 0,
        sm: config.appearance?.radius?.sm ?? 4,
        md: config.appearance?.radius?.md ?? 8,
        lg: config.appearance?.radius?.lg ?? 12,
        full: config.appearance?.radius?.full ?? 9999
    })

    // -- Spacing (not configurable - breaks layouts) --
    readonly property var spacing: ({
        xs: 2,
        sm: 4,
        md: 8,
        lg: 12,
        xl: 16,
        xxl: 24
    })

    // -- Animation (not configurable) --
    readonly property var duration: ({
        instant: 0,
        fast: 150,
        normal: 300,
        slow: 400,
        popout: 350  // Organic popout animation
    })

    readonly property var easing: ({
        standard: Easing.OutCubic,
        enter: Easing.OutCubic,
        exit: Easing.InCubic
    })

    // Material Design 3 bezier curves for organic animations
    readonly property var curves: ({
        // Standard transitions
        standard: [0.2, 0, 0, 1],
        // Emphasized entry (snappy, organic)
        emphasized: [0.05, 0.7, 0.1, 1],
        // Expressive spatial (slight overshoot for "bloom" effect)
        expressive: [0.38, 1.21, 0.22, 1]
    })

    // === TIER 3: COMPONENT TOKENS (Internal) ===
    // These map semantic tokens to specific component uses.
    // Components should use these, not semantic tokens directly.

    // -- Bar --
    readonly property int barHeight: config.bar?.height ?? 40
    readonly property string barPosition: config.bar?.position ?? "left"
    readonly property int barMargin: config.bar?.margin ?? 0
    readonly property color barBackground: background
    readonly property int barPadding: spacing.sm
    readonly property int barRadius: radius.none

    // -- Bar (Vertical/Left) --
    readonly property int barWidth: config.bar?.width ?? 48

    // -- Border (Screen frame with curved corners) --
    readonly property int borderThickness: config.border?.thickness ?? 4
    readonly property int borderRounding: config.border?.rounding ?? 6

    // -- Gaps (Hyprland window gaps, controlled by Fern) --
    readonly property int gapsIn: config.gaps?.inner ?? 5
    readonly property int gapsOut: config.gaps?.outer ?? 5

    // -- Module (generic container) --
    readonly property int moduleRadius: radius.md
    readonly property int moduleSpacing: spacing.lg
    readonly property int modulePadding: spacing.md
    readonly property color moduleBackground: surface
    readonly property color moduleBackgroundHover: surfaceHover

    // -- Button / Interactive --
    readonly property int buttonRadius: radius.sm
    readonly property int buttonPadding: spacing.sm
    readonly property color buttonBackground: Qt.rgba(0, 0, 0, 0)
    readonly property color buttonBackgroundHover: surfaceHover
    readonly property color buttonBackgroundActive: overlay

    // -- Icon --
    readonly property int iconSize: 16
    readonly property int iconSizeSm: 14
    readonly property int iconSizeLg: 20

    // -- Text --
    readonly property color textPrimary: foreground
    readonly property color textSecondary: foregroundDim
    readonly property color textAccent: accent

    // === HELPER FUNCTIONS ===

    // Apply config and notify listeners
    function applyConfig(newConfig: var) {
        config = newConfig;
        themeChanged();
    }

    // Get module-specific config
    function moduleConfig(moduleName: string): var {
        return config.modules?.[moduleName] ?? {};
    }
}
