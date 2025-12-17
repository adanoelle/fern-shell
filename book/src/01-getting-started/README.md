# Introduction

**Fern Shell** is a modern, QML-based desktop shell for
[Hyprland](https://hyprland.org/). It provides a sleek status bar with
live-reloadable components, designed for rapid iteration and easy customization.

## Why Fern Shell?

| Traditional Bars            | Fern Shell                     |
| --------------------------- | ------------------------------ |
| Recompile to change styling | Live reload on file save       |
| Limited theming options     | Full design token system       |
| Separate config languages   | QML + TOML configuration       |
| Complex build systems       | Nix flakes for reproducibility |

## Key Features

- **Live Reload**: Change QML files and see updates instantly
- **Hyprland Native**: Built specifically for Hyprland's layer-shell protocol
- **Design Tokens**: Consistent theming with a three-tier token system
- **Minimal Dependencies**: Qt6/QML frontend, optional Rust backend
- **Nix-First**: Easy installation via Home-Manager or NixOS modules

## Architecture at a Glance

```
┌─────────────────────────────────────┐
│         Fern Shell Package          │
├─────────────────────────────────────┤
│  QML/QuickShell Frontend            │
│  - Panel with modules               │
│  - Live-reloadable                  │
│  - System integration via D-Bus     │
├─────────────────────────────────────┤
│  Rust Service (fernctl)             │
│  - Configuration management         │
│  - Theme token generation           │
│  - File watching                    │
└─────────────────────────────────────┘
```

## What's Next?

- **[Installation](installation.md)** - Get Fern Shell running on your system
- **[Quick Tour](quick-tour.md)** - A 5-minute walkthrough of the basics
