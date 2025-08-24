# Fern Shell Troubleshooting Guide

## Common Issues & Solutions

### Installation Issues

#### Problem: Nix flake fails to build

```
error: attribute 'fern-shell' missing
```

**Solution:**

```bash
# Update flake inputs
nix flake update

# Check flake outputs
nix flake show

# Try building directly
nix build .#fern-shell --print-build-logs
```

#### Problem: Home-Manager module not found

```
error: The option `programs.fern-shell` does not exist
```

**Solution:**

```nix
# In your flake.nix
{
  inputs.fern.url = "github:yourusername/fern-shell";

  # In your home-manager config
  imports = [ inputs.fern.homeModules.fern-shell ];
}
```

### QuickShell Issues

#### Problem: QuickShell not starting

```
Failed to create shell surface
```

**Solution:**

```bash
# Ensure Wayland session
echo $WAYLAND_DISPLAY  # Should not be empty

# Check QuickShell installation
which qs
qs --version

# Test QuickShell directly
qs -c "print('QuickShell works')"
```

#### Problem: QML module not found

```
module "Quickshell.Hyprland" is not installed
```

**Solution:**

```bash
# Ensure QuickShell is built with Hyprland support
nix build github:outfoxxed/quickshell#quickshell

# Set QML import path
export QML2_IMPORT_PATH=$(qs --qml-import-path):$QML2_IMPORT_PATH
```

### Hyprland Integration

#### Problem: Bar not appearing

```
No output from qs command
```

**Solution:**

```bash
# Check Hyprland IPC
hyprctl version
ls -la /tmp/hypr/$HYPRLAND_INSTANCE_SIGNATURE/

# Test Hyprland integration
qs -e 'import Quickshell.Hyprland; print(Hyprland.workspaces)'

# Check layer shell
hyprctl layers
```

#### Problem: Bar appears in wrong position

```
Bar overlaps windows or appears off-screen
```

**Solution:**

```qml
// In fern/shell.qml, ensure anchors are set
PanelWindow {
    anchors {
        top: true    // or bottom: true
        left: true
        right: true
    }
    exclusiveZone: height  // Reserve space
}
```

### Module Issues

#### Problem: Module not loading

```
Module 'Clock' is undefined
```

**Solution:**

```bash
# Check module file exists
ls fern/modules/Clock.qml

# Test module in isolation
qs -p fern/modules/Clock.qml

# Check for syntax errors
qmllint fern/modules/Clock.qml
```

#### Problem: Module crashes on startup

```
TypeError: Cannot read property 'x' of undefined
```

**Solution:**

```qml
// Add safety checks
property var config: moduleConfig || {}
property string value: config?.setting ?? "default"

// Check service availability
Component.onCompleted: {
    if (!ServiceName.available) {
        console.warn("Service not available");
        return;
    }
}
```

### Configuration Issues

#### Problem: Config changes not applying

```
Modified config.json but no changes visible
```

**Solution:**

```bash
# Check config file location
ls ~/.config/fern/config.json

# Validate JSON syntax
jq . ~/.config/fern/config.json

# Watch for reload messages
journalctl -f | grep fern

# Force reload
pkill -USR1 quickshell
```

#### Problem: Invalid configuration causes crash

```
JSON parse error at line X
```

**Solution:**

```javascript
// Add to Config.qml
function loadConfig() {
  try {
    const data = JSON.parse(fileContent)
    applyConfig(data)
  } catch (e) {
    console.error('Config error:', e)
    // Use defaults
    applyConfig(defaultConfig)
  }
}
```

### Performance Issues

#### Problem: High CPU usage

```
quickshell using 100% CPU
```

**Solution:**

```qml
// Check for infinite loops
Timer {
    interval: 100  // Not 0!
    repeat: true
}

// Optimize bindings
// Bad:
property string text: expensiveFunction()

// Good:
property string text: ""
Timer {
    interval: 1000
    onTriggered: text = expensiveFunction()
}
```

#### Problem: Memory leak

```
Memory usage grows over time
```

**Solution:**

```qml
// Destroy unused components
Component.onDestruction: {
    timer.stop();
    connections.destroy();
}

// Clear caches periodically
Timer {
    interval: 3600000  // 1 hour
    onTriggered: {
        imageCache.clear();
        dataCache = {};
    }
}
```

### Visual Issues

#### Problem: Text not visible

```
Components appear but text is missing
```

**Solution:**

```bash
# Check fonts are installed
fc-list | grep "JetBrains"
fc-list | grep "Inter"

# Install missing fonts
nix-shell -p jetbrains-mono inter

# In QML, fallback fonts
font.family: "Inter, sans-serif"
```

#### Problem: Icons not showing

```
Square boxes instead of icons
```

**Solution:**

```bash
# Install icon fonts
nix-shell -p material-symbols

# Check font is loaded
fc-list | grep "Material Symbols"

# In QML, ensure correct font
font.family: "Material Symbols Rounded"
```

### Service Issues

#### Problem: D-Bus service not available

```
DBus error: Service not found
```

**Solution:**

```bash
# Check service is running
busctl --user list | grep ServiceName

# Start service manually
systemctl --user start servicename

# Check D-Bus permissions
dbus-monitor --session
```

#### Problem: Audio service not working

```
Volume changes have no effect
```

**Solution:**

```bash
# Check PipeWire/PulseAudio
pactl info
wpctl status

# Test audio service
qs -e 'import "fern/services"; AudioService.setVolume(0.5)'
```

### Development Issues

#### Problem: Hot reload not working

```
Changes to QML files not reflected
```

**Solution:**

```bash
# Use -p flag for hot reload
qs -p fern/shell.qml  # Not just 'qs fern/shell.qml'

# Check file watcher
inotifywait -m fern/

# Restart if needed
pkill quickshell && qs -p fern/shell.qml
```

#### Problem: Cannot debug QML

```
console.log not appearing
```

**Solution:**

```bash
# Enable QML debugging
export QT_LOGGING_RULES="qt.qml.debug=true"

# Use stderr for output
console.error("Debug:", variable);

# Check journal
journalctl -f | grep quickshell
```

## Debugging Tools

### QML Debugging

```bash
# Enable verbose logging
QT_LOGGING_RULES="*.debug=true" qs -p fern/shell.qml

# Visual debugging
QSG_VISUALIZE=batches qs -p fern/shell.qml
QSG_VISUALIZE=clip qs -p fern/shell.qml
QSG_VISUALIZE=overdraw qs -p fern/shell.qml

# Performance profiling
QSG_RENDER_TIMING=1 qs -p fern/shell.qml
```

### System Information

```bash
# Check environment
env | grep -E "(WAYLAND|QT|QML|HYPR)"

# Graphics info
glxinfo | grep "OpenGL renderer"
vainfo  # For hardware acceleration

# Process info
ps aux | grep quickshell
pmap $(pgrep quickshell)  # Memory map
```

### Hyprland Debugging

```bash
# Monitor Hyprland events
socat - UNIX-CONNECT:/tmp/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket2.sock

# Check window properties
hyprctl clients

# Layer shell info
hyprctl layers
```

## Error Messages Reference

| Error                          | Cause               | Solution                    |
| ------------------------------ | ------------------- | --------------------------- |
| `Cannot anchor to null item`   | Parent not set      | Ensure component has parent |
| `Unable to assign undefined`   | Missing property    | Add null checks             |
| `Maximum call stack exceeded`  | Infinite recursion  | Check binding loops         |
| `Cannot read property of null` | Null reference      | Add safety checks           |
| `Module not found`             | Import path issue   | Check QML2_IMPORT_PATH      |
| `Signal is not defined`        | Typo in signal name | Check signal spelling       |
| `Component is not ready`       | Async loading       | Use Component.onCompleted   |

## Getting Help

### Logs to Collect

```bash
# System info
nix-info
hyprctl version
qs --version

# Fern logs
journalctl --user -u quickshell -n 100

# Configuration
cat ~/.config/fern/config.json

# Build log
nix build .#fern-shell --print-build-logs 2>&1
```

### Where to Get Help

1. Check this troubleshooting guide
2. Search existing GitHub issues
3. Ask in QuickShell Discord/Matrix
4. Create GitHub issue with:
   - System information
   - Error messages
   - Minimal reproduction
   - Logs collected above

## Prevention Tips

### Best Practices

1. **Test modules in isolation** before integration
2. **Use defensive programming** - check for null/undefined
3. **Keep backups** of working configurations
4. **Update incrementally** - one module at a time
5. **Monitor performance** during development
6. **Document issues** as you solve them

### Regular Maintenance

```bash
# Weekly: Update dependencies
nix flake update

# Monthly: Clean cache
rm -rf ~/.cache/quickshell

# Check for memory leaks
valgrind qs -p fern/shell.qml

# Profile performance
perf record qs -p fern/shell.qml
perf report
```

Remember: Most issues are configuration or environment related. Start with the
simplest solution first!
