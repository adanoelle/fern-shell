# Architecture

Fern Shell uses a hybrid QML + Rust architecture with clear separation of concerns.

## High-Level Overview

```
┌─────────────────────────────────────────────┐
│           User's NixOS/Home-Manager         │
│      programs.fern-shell.enable = true      │
└────────────────────┬────────────────────────┘
                     │
┌────────────────────▼────────────────────────┐
│              Fern Shell                     │
│  ┌────────────────────────────────────────┐ │
│  │        QML/QuickShell Frontend         │ │
│  │  - Panel with modules                  │ │
│  │  - Live-reloadable                     │ │
│  │  - Wayland layer-shell                 │ │
│  └────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────┐ │
│  │        Rust Backend (fernctl)          │ │
│  │  - Configuration management            │ │
│  │  - Theme token generation              │ │
│  │  - File watching                       │ │
│  └────────────────────────────────────────┘ │
└─────────────────────────────────────────────┘
```

## Dual-Window System

Each screen has two Wayland windows:

| Window | Purpose | Exclusive Zone |
|--------|---------|----------------|
| `panelWindow` | Visual rendering | None (Ignore) |
| `exclusionWindow` | Space reservation | Dynamic (bar width) |

**Why two windows?**
- Visual effects (curved border) need full-screen coverage
- Exclusive zones are rectangular (just the bar area)
- Separating them avoids compositor edge cases

## Hexagonal Architecture (Rust)

The fernctl crate uses ports & adapters:

```
┌─────────────────────────────────────────────┐
│                  Domain                     │
│  (pure business logic, zero dependencies)   │
│  - Theme composition                        │
│  - Design tokens                            │
│  - User config mapping                      │
└──────────────────┬──────────────────────────┘
                   │
     ┌─────────────┴─────────────┐
     │                           │
┌────▼────┐                 ┌────▼────┐
│ Inbound │                 │ Outbound│
│  Ports  │                 │  Ports  │
│─────────│                 │─────────│
│ Config  │                 │ Persist │
│ Query   │                 │ Notify  │
│Validate │                 │   IPC   │
└────┬────┘                 └────┬────┘
     │                           │
┌────▼───────────────────────────▼────┐
│              Adapters               │
│  - TOML parser                      │
│  - JSON serializer                  │
│  - File system                      │
│  - D-Bus (future)                   │
└─────────────────────────────────────┘
```

## State Flow

```
User hovers left edge
  → BarInteractions.containsMouse = true
    → bar.isHovered = true
      → BarWrapper state: "" → "visible"
        → implicitWidth: 4 → 48
          → Border mask updates
          → Backgrounds width updates
          → exclusionWindow.exclusiveZone updates
            → Compositor pushes windows right
```

All updates happen through QML property bindings - no imperative code.

## Key Design Decisions

### QML + QuickShell over GTK
- Live reload during development
- Declarative UI with reactive bindings
- Strong animation support

### Hyprland-Specific
- Direct workspace integration via IPC
- Layer-shell protocol for panel windows
- No abstraction layers for other compositors

### Nix Flakes
- Reproducible builds
- Easy distribution via flake inputs
- Home-Manager and NixOS modules

## Files

- `/fern/shell.qml` - Window setup, screen variants
- `/fernctl/src/lib.rs` - Rust crate entry point
- `/fernctl/src/ports/` - Hexagonal interfaces
- `/flake.nix` - Nix flake entry point
