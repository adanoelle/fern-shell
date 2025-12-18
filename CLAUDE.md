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

### 4. Rust Crate Workspace

- **Rationale**: Memory safety, performance, good async story for system
  monitoring
- **Architecture**: Cargo workspace with specialized crates
- **Integration**: IPC with QML frontend via D-Bus or state files

#### Crate Structure

```
crates/
├── fern-core/       # Shared types, traits, utilities (paths, state, config)
├── fern-theme/      # Theme & config management CLI (formerly fernctl)
├── fern-obs/        # OBS WebSocket bridge (planned)
├── fernctl/         # Control plane & orchestrator (planned)
└── fernctl-tui/     # TUI dashboard (planned)
```

#### Key Crates

- **fern-core**: Common infrastructure (XDG paths, service state types, error handling)
- **fern-theme**: Type-safe design tokens, TOML→JSON conversion, validation
- **fern-obs**: OBS Studio integration via WebSocket (future)
- **fernctl**: Unified control plane for service orchestration (future)

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
- [x] **fern-theme** - Theme/config CLI (formerly fernctl)
- [ ] **fernctl** - Unified control plane (planned)
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
fern-shell/
├── Cargo.toml          # Workspace root
├── Cargo.lock          # Workspace lockfile
├── flake.nix           # Nix flake entry point
│
├── crates/
│   ├── fern-core/      # Shared library
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── paths.rs    # XDG paths
│   │       ├── state.rs    # Service state types
│   │       ├── config.rs   # Config utilities
│   │       └── error.rs    # Shared errors
│   │
│   └── fern-theme/     # Theme CLI (formerly fernctl)
│       └── src/
│           ├── main.rs
│           ├── lib.rs
│           ├── domain/     # Design tokens, themes
│           ├── ports/      # Hexagonal architecture
│           ├── adapters/   # File I/O, TOML parsing
│           └── commands/   # CLI commands
│
├── fern/
│   ├── shell.qml       # Main panel entry point
│   ├── modules/        # Individual panel modules
│   │   ├── Clock.qml
│   │   ├── Workspaces.qml
│   │   └── ...
│   ├── services/       # QML service singletons
│   ├── components/     # Reusable QML components
│   └── config/         # Configuration loading
│
└── flake-parts/
    ├── dev.nix         # Development shell
    ├── pkgs.nix        # Package definitions
    ├── checks.nix      # CI checks
    └── modules/        # Nix modules
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

## Reference Implementation: Caelestia

We have a local copy of Soramanew's Caelestia shell at
`/home/ada/src/nix/caelestia/` for reference.

### Key Files to Study

| Feature          | Caelestia Location                              | Description                       |
| ---------------- | ----------------------------------------------- | --------------------------------- |
| Bar Architecture | `/caelestia/modules/bar/Bar.qml`                | Dynamic module loading system     |
| Workspaces       | `/caelestia/modules/bar/components/workspaces/` | Complete workspace implementation |
| Configuration    | `/caelestia/config/Config.qml`                  | Live-reloading JSON config        |
| Services         | `/caelestia/services/`                          | System integration patterns       |
| Components       | `/caelestia/components/`                        | Reusable styled components        |
| System Tray      | `/caelestia/modules/bar/components/Tray.qml`    | SystemTray API usage              |

### Learning Path from Caelestia

1. **Start with components:** Study `StyledRect.qml` for base component patterns
2. **Understand services:** Review `Hyprland.qml` for service singleton pattern
3. **Learn bar structure:** Analyze `Bar.qml` for module architecture
4. **Configuration system:** See `Config.qml` for live reload implementation

### Patterns to Adopt from Caelestia

```qml
// Component pattern from Caelestia
import qs.config  // Their config import
import qs.components  // Their component library

StyledClippingRect {  // Base component with theming
    required property ShellScreen screen  // Explicit dependencies

    // Consistent styling
    color: Colours.palette.surface
    radius: Appearance.rounding.normal
}
```

### What to Reference When

- **Building a new module?** Check `/caelestia/modules/bar/components/`
- **Adding system integration?** Look at `/caelestia/services/`
- **Implementing animations?** See `/caelestia/components/StyledRect.qml`
- **Setting up config?** Study `/caelestia/config/`

See `/home/ada/src/nix/caelestia/CLAUDE.md` for detailed analysis.
