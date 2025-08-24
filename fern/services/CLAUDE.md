# Fern Services Architecture Guide

## Purpose

Services in Fern Shell are singleton QML objects that provide system
integration, state management, and business logic. They act as the bridge
between the QML UI and system APIs, ensuring clean separation of concerns.

## Service Pattern

### Basic Service Template

```qml
// fern/services/MyService.qml
pragma Singleton
import QtQuick
import Quickshell
import Quickshell.DBus  // If using D-Bus

Singleton {
    id: root

    // Public API - Properties
    readonly property bool connected: internal.connected
    readonly property var data: internal.processedData
    property real pollInterval: 5000

    // Public API - Signals
    signal dataChanged(var newData)
    signal error(string message)

    // Public API - Methods
    function refresh() {
        internal.fetchData();
    }

    function updateSetting(key: string, value: var) {
        internal.settings[key] = value;
        internal.saveSettings();
    }

    // Private implementation
    QtObject {
        id: internal

        property bool connected: false
        property var rawData: ({})
        property var processedData: ({})
        property var settings: ({})

        function fetchData() {
            // Implementation
        }

        function processData() {
            // Transform raw data
            processedData = transform(rawData);
            root.dataChanged(processedData);
        }

        function saveSettings() {
            // Persist settings
        }
    }

    // Timers, connections, etc.
    Timer {
        interval: root.pollInterval
        running: root.connected
        repeat: true
        onTriggered: internal.fetchData()
    }

    Component.onCompleted: {
        internal.fetchData();
    }
}
```

## Service Types

### 1. System Integration Service

**Purpose**: Wrap system APIs with QML-friendly interface

```qml
// fern/services/AudioService.qml
pragma Singleton
import Quickshell.DBus

Singleton {
    readonly property real volume: pulse.volume
    readonly property bool muted: pulse.muted
    readonly property string activeDevice: pulse.activeDevice

    function setVolume(value: real) {
        pulse.setVolume(Math.max(0, Math.min(1, value)));
    }

    function toggleMute() {
        pulse.setMuted(!pulse.muted);
    }

    DBusInterface {
        id: pulse
        service: "org.PulseAudio1"
        path: "/org/pulseaudio/server_lookup1"
        interface: "org.PulseAudio.Core1"

        property real volume: 0.5
        property bool muted: false
        property string activeDevice: ""

        // Implementation details...
    }
}
```

### 2. State Management Service

**Purpose**: Centralized state for UI components

```qml
// fern/services/ThemeService.qml
pragma Singleton

Singleton {
    property string currentTheme: "dark"
    property var palette: darkPalette

    readonly property var darkPalette: {
        "background": "#1e1e2e",
        "foreground": "#cdd6f4",
        "accent": "#89b4fa"
    }

    readonly property var lightPalette: {
        "background": "#eff1f5",
        "foreground": "#4c4f69",
        "accent": "#1e66f5"
    }

    function switchTheme(theme: string) {
        currentTheme = theme;
        palette = (theme === "dark") ? darkPalette : lightPalette;
        savePreference("theme", theme);
    }

    Component.onCompleted: {
        currentTheme = loadPreference("theme", "dark");
        switchTheme(currentTheme);
    }
}
```

### 3. Data Processing Service

**Purpose**: Complex data transformation and caching

```qml
// fern/services/WeatherService.qml
pragma Singleton

Singleton {
    readonly property var current: cache.current
    readonly property var forecast: cache.forecast
    readonly property bool loading: internal.loading

    function refresh() {
        internal.fetchWeather();
    }

    QtObject {
        id: internal
        property bool loading: false

        function fetchWeather() {
            loading = true;

            Http.get(weatherUrl).then(response => {
                cache.current = parseCurrentWeather(response);
                cache.forecast = parseForecast(response);
                loading = false;
            }).catch(error => {
                console.error("Weather fetch failed:", error);
                loading = false;
            });
        }
    }

    QtObject {
        id: cache
        property var current: ({})
        property var forecast: []
    }
}
```

### 4. Compositor Integration Service

**Purpose**: Hyprland-specific functionality

```qml
// fern/services/HyprlandService.qml
pragma Singleton
import Quickshell.Hyprland

Singleton {
    // Expose Hyprland data
    readonly property var workspaces: Hyprland.workspaces
    readonly property int activeWorkspace: Hyprland.activeWorkspace?.id ?? 1
    readonly property var activeWindow: Hyprland.activeWindow
    readonly property var monitors: Hyprland.monitors

    // Convenience methods
    function switchWorkspace(id: int) {
        Hyprland.dispatch(`workspace ${id}`);
    }

    function moveWindowToWorkspace(windowId: string, workspaceId: int) {
        Hyprland.dispatch(`movetoworkspace ${workspaceId},address:${windowId}`);
    }

    function toggleFloating() {
        Hyprland.dispatch("togglefloating");
    }

    // Monitor for specific workspace
    function workspaceForMonitor(monitor: var): int {
        return monitors.get(monitor)?.activeWorkspace?.id ?? 1;
    }

    // Event handling
    Connections {
        target: Hyprland

        function onWorkspaceChanged(id: int) {
            console.log("Workspace changed to:", id);
        }

        function onWindowOpened(window: var) {
            console.log("Window opened:", window.title);
        }
    }
}
```

## Inter-Service Communication

### Direct Reference

```qml
// One service using another
Singleton {
    function performAction() {
        if (NetworkService.connected) {
            WeatherService.refresh();
        }
    }
}
```

### Signal/Slot Pattern

```qml
// Service A emits
signal dataReady(var data)

// Service B connects
Connections {
    target: ServiceA
    function onDataReady(data) {
        processData(data);
    }
}
```

### Event Bus Pattern

```qml
// fern/services/EventBus.qml
pragma Singleton

Singleton {
    signal notify(string event, var data)

    function emit(event: string, data: var) {
        notify(event, data);
    }

    function on(event: string, callback: var) {
        // Register callback for event
    }
}
```

## D-Bus Integration

### Basic D-Bus Service

```qml
DBusInterface {
    service: "org.freedesktop.NetworkManager"
    path: "/org/freedesktop/NetworkManager"
    interface: "org.freedesktop.NetworkManager"

    // Properties auto-mapped
    property int connectivity: 0
    property string primaryConnection: ""

    // Methods
    function getDevices() {
        return call("GetDevices");
    }

    // Signals
    signal stateChanged(int newState)
}
```

### Complex D-Bus Handling

```qml
QtObject {
    property var dbusConnection: null

    Component.onCompleted: {
        // Manual D-Bus setup for complex cases
        dbusConnection = DBus.sessionBus();
        dbusConnection.registerObject("/org/fern/Service", this);
    }

    function handleDBusCall(method: string, args: var): var {
        // Handle incoming D-Bus calls
    }
}
```

## Error Handling

### Graceful Degradation

```qml
Singleton {
    readonly property bool available: internal.checkAvailability()
    readonly property var data: available ? internal.data : fallbackData

    QtObject {
        id: internal

        function checkAvailability(): bool {
            try {
                // Check if service is available
                return testConnection();
            } catch (e) {
                console.warn("Service unavailable:", e);
                return false;
            }
        }
    }
}
```

### Error Reporting

```qml
signal error(string code, string message, var details)

function handleError(error: var) {
    console.error("Service error:", error);

    root.error(
        error.code || "UNKNOWN",
        error.message || "An error occurred",
        { timestamp: Date.now(), stack: error.stack }
    );

    // Notify user if critical
    if (error.critical) {
        NotificationService.show("Error", error.message, "error");
    }
}
```

## Performance Optimization

### Lazy Initialization

```qml
Singleton {
    property var _instance: null

    function getInstance(): var {
        if (!_instance) {
            _instance = createExpensiveObject();
        }
        return _instance;
    }
}
```

### Debouncing

```qml
Timer {
    id: debounceTimer
    interval: 300
    repeat: false
    onTriggered: internal.performExpensiveOperation()
}

function requestUpdate() {
    debounceTimer.restart();
}
```

### Caching

```qml
QtObject {
    id: cache
    property var data: ({})
    property var timestamps: ({})

    function get(key: string): var {
        if (isValid(key)) {
            return data[key];
        }
        return null;
    }

    function set(key: string, value: var) {
        data[key] = value;
        timestamps[key] = Date.now();
    }

    function isValid(key: string): bool {
        const age = Date.now() - (timestamps[key] || 0);
        return age < 60000; // 1 minute cache
    }
}
```

## Testing Services

### Mock Service for Testing

```qml
// fern/services/MockAudioService.qml
pragma Singleton

Singleton {
    property real volume: 0.5
    property bool muted: false

    function setVolume(value: real) {
        volume = value;
        console.log("Mock: Volume set to", value);
    }

    function toggleMute() {
        muted = !muted;
        console.log("Mock: Mute toggled to", muted);
    }
}
```

### Service Testing

```bash
# Test service in isolation
qs -e 'import "fern/services"; console.log(AudioService.volume)'

# Test with mock data
QS_USE_MOCKS=1 qs -p fern/shell.qml
```

## Service Registry

### Central Service Manager

```qml
// fern/services/ServiceManager.qml
pragma Singleton

Singleton {
    readonly property var services: ({
        "audio": AudioService,
        "network": NetworkService,
        "hyprland": HyprlandService,
        "theme": ThemeService
    })

    function get(name: string): var {
        return services[name] || null;
    }

    function isAvailable(name: string): bool {
        return services[name]?.available ?? false;
    }

    Component.onCompleted: {
        // Initialize all services
        for (let name in services) {
            console.log(`Initializing ${name} service...`);
            services[name].initialize?.();
        }
    }
}
```

## Best Practices

### DO ✅

- Keep services stateless when possible
- Use readonly properties for computed values
- Implement proper cleanup in Component.onDestruction
- Document public API clearly
- Handle errors gracefully
- Cache expensive operations
- Use async operations for I/O

### DON'T ❌

- Don't expose internal implementation
- Don't create circular dependencies
- Don't block the UI thread
- Don't mutate global state directly
- Don't ignore error cases
- Don't poll when events are available

## Common Patterns

### Observer Pattern

```qml
Singleton {
    property var observers: []

    function subscribe(callback: var) {
        observers.push(callback);
    }

    function notify(data: var) {
        observers.forEach(cb => cb(data));
    }
}
```

### Factory Pattern

```qml
function createModule(type: string, config: var): var {
    switch(type) {
        case "audio": return AudioModule.create(config);
        case "network": return NetworkModule.create(config);
        default: return null;
    }
}
```

## Caelestia Reference

Study these Caelestia services:

| Service  | File                               | Key Features                         |
| -------- | ---------------------------------- | ------------------------------------ |
| Hyprland | `/caelestia/services/Hyprland.qml` | Event handling, workspace management |
| Audio    | `/caelestia/services/Audio.qml`    | PipeWire integration                 |
| Network  | `/caelestia/services/Network.qml`  | NetworkManager D-Bus                 |
| Colours  | `/caelestia/services/Colours.qml`  | Theme management                     |

## Next Steps

1. Identify system APIs you need to wrap
2. Create service singleton
3. Define public API (properties, methods, signals)
4. Implement with proper error handling
5. Add caching if needed
6. Test in isolation
7. Integrate with modules

Remember: Services are the backbone of Fern Shell - make them robust, efficient,
and well-documented.
