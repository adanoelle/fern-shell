# Fern Shell - AI Development Guide

## Project Vision

Fern is a next-generation desktop shell for Hyprland, designed to replace
traditional status bars like waybar with a modern, QML-based approach that
prioritizes developer experience and rapid iteration.

### Core Goals

- **Replace waybar** with a more maintainable, live-reloadable alternative
- **Rapid iteration** through QML hot-reload without compilation
- **Nix-first** for reproducible builds and easy distribution
- **Minimal dependencies** while providing full desktop shell functionality
- **Modern stack**: QML UI, Rust services, Nix packaging

## Architecture Overview

```
┌─────────────────────────────────────────┐
│           User's NixOS/HM               │
│  programs.fern-shell.enable = true      │
└────────────────┬────────────────────────┘
                 │
┌────────────────▼────────────────────────┐
│          Fern Shell Package             │
│  ┌──────────────────────────────────┐   │
│  │     QML/QuickShell Frontend      │   │
│  │  • Panel with modules            │   │
│  │  • Live-reloadable               │   │
│  │  • System integration via D-Bus  │   │
│  └──────────────────────────────────┘   │
│  ┌──────────────────────────────────┐   │
│  │   Rust Service Daemons (Future)  │   │
│  │  • Audio (PipeWire)              │   │
│  │  • Power (UPower)                │   │
│  │  • Network monitoring            │   │
│  └──────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

## Key Design Decisions

### 1. QML + QuickShell over GTK/C++

- **Rationale**: Live reload during development, declarative UI, modern reactive
  patterns
- **Trade-off**: Requires Qt dependencies but gains rapid development cycle
- **Implementation**: Pure QML files in `fern/` directory, loaded by QuickShell

### 2. Hyprland-Specific vs Compositor-Agnostic

- **Rationale**: Can leverage Hyprland's specific features without abstraction
  layers
- **Benefits**: Direct workspace integration, native animations, layer-shell
  support
- **Future**: Could add other compositors but not a priority

### 3. Nix Flakes for Everything

- **Rationale**: Reproducible builds, easy user installation, proper dependency
  management
- **Structure**: flake-parts for modular organization
- **Modules**: Both Home-Manager and NixOS modules provided

### 4. Rust for System Services (Planned)

- **Rationale**: Memory safety, performance, good async story for system
  monitoring
- **Integration**: IPC with QML frontend via D-Bus or custom protocol
- **Examples**: fernctl CLI, audio daemon, power monitoring

## Development Workflow

### Quick Start

```bash
# Enter development shell
nix develop

# Live preview with hot-reload
qs -p fern/shell.qml

# Format Nix files
nix fmt

# Run CI checks locally
nix flake check
```

### Testing Changes

1. Make QML changes in `fern/` directory
2. QuickShell auto-reloads on file save
3. Test in live Hyprland session
4. Package and test full installation

### Common Tasks

- **Add new QML module**: Create in `fern/modules/`, import in `shell.qml`
- **Update dependencies**: Modify `flake.nix` inputs, run `nix flake update`
- **Test installation**: Use `nix run .#fern-shell` without installing

## Roadmap: Waybar Feature Parity

### Phase 1: Core Modules (Current Priority)

- [ ] **Workspaces** - Hyprland workspace indicator/switcher
- [ ] **Clock** - Time/date display with formatting
- [ ] **System Tray** - StatusNotifierItem protocol support

### Phase 2: System Monitoring

- [ ] **Audio** - PipeWire volume control
- [ ] **Network** - NetworkManager integration
- [ ] **Battery** - UPower integration (laptops)
- [ ] **CPU/Memory** - Resource monitoring

### Phase 3: Enhanced Features

- [ ] **Window Title** - Active window display
- [ ] **Media Player** - MPRIS2 support
- [ ] **Custom Modules** - User script integration
- [ ] **Configuration System** - User preferences

### Phase 4: Polish

- [ ] **Animations** - Smooth transitions
- [ ] **Themes** - Color scheme support
- [ ] **fernctl** - CLI management tool
- [ ] **Documentation** - User guide

## Module Development Pattern

Each module should follow this structure:

```qml
// fern/modules/ModuleName.qml
import Quickshell
import QtQuick

Rectangle {
    // Module properties
    property var config: ({})

    // System integration (D-Bus, processes, etc.)
    // UI components
    // Update logic
}
```

## Testing & Validation

### Before Commits

1. Run `nix flake check` - Ensures package builds
2. Test with `qs -p fern/shell.qml` - Visual verification
3. Run `nix fmt` - Code formatting

### Integration Testing

1. Install via Home-Manager module in test config
2. Launch Hyprland session
3. Verify all modules load and update correctly
4. Check resource usage (CPU, memory)

## Conventions

### Code Style

- **QML**: 4-space indentation, clear property grouping
- **Nix**: nixfmt-rfc-style formatting (via `nix fmt`)
- **Rust** (future): rustfmt with default settings
- **Comments**: Explain "why" not "what"

### Git Commits

- Prefix with module: `qml:`, `nix:`, `rust:`, `docs:`
- Present tense: "Add workspace module" not "Added"
- Reference issues when applicable

### File Organization

```
fern/
├── shell.qml           # Main panel entry point
├── modules/           # Individual panel modules
│   ├── Clock.qml
│   ├── Workspaces.qml
│   └── ...
└── themes/           # Color schemes (future)

flake-parts/
├── dev.nix          # Development shell
├── pkgs.nix         # Package definitions
├── checks.nix       # CI checks
└── modules/         # Nix modules
```

## Common Pitfalls & Solutions

### QML Hot-Reload Not Working

- Ensure `qs -p` is running (not just `qs`)
- Check for syntax errors in QML files
- Restart QuickShell if it crashes

### Nix Build Failures

- Run `nix flake check` for detailed errors
- Ensure all dependencies are in flake inputs
- Check that `src` paths are correct

### Hyprland Integration Issues

- Verify Hyprland IPC socket is accessible
- Check layer-shell protocol support
- Ensure running under Wayland session

## Resources

- [QuickShell Documentation](https://quickshell.outfoxxed.me/docs/)
- [Hyprland IPC](https://wiki.hyprland.org/IPC/)
- [QML Reference](https://doc.qt.io/qt-6/qmlreference.html)
- [Nix Flakes](https://nixos.wiki/wiki/Flakes)
