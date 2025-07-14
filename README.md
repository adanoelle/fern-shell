# fern-shell

[![nix-flake](https://github.com/adanoelle/fern-shell/actions/workflows/ci.yml/badge.svg)](https://github.com/adanoelle/fern-shell/actions/workflows/ci.yml)


> Fern is a QML‑powered desktop shell that runs on the Hyprland compositor, built for NixOS first.
> It marries Qt Quick for fluid UI, Rust for safe background services, and Nix flakes for reproducible packaging.
> Clone the repo, enable one Home‑Manager option, rebuild – you’re looking at a custom panel in under two minutes.

### Why we built Fern

Motivation | Design Choice
---        | ---
Rapid iteration without recompiling C++ |	UI in pure QML with live‑reload
Wayland‑native tiling | Target Hyprland’s layer‑shell + effects
Reproducible installs for contributors | Everything packaged via a flake; CI runs nix flake check
Minimal runtime deps | Rust daemons + PipeWire/UPower bindings only when needed
Modern CLI ergonomics | Nushell wrappers & fernctl Rust binary

### What’s inside the flake

* [dev.nix](flake-parts/dev.nix)       – Nushell‑first dev env (nix develop).
* [pkgs.nix](flake-parts/pkgs.nix)     – builds the fern-shell package & exports an overlay.
* [modules/](flake-parts/modules/)     – Home‑Manager and NixOS modules; one option (programs.fern-shell.enable) is all users need.
* [checks.nix](flake-parts/checks.nix) – CI jobs (qmllint, package build).
* [fern/](fern/) – QML sources; live‑reload via qs -p fern/shell.qml.

## Quick start

### Option A: Home‑Manager
```nix
{
  inputs.fern.url = "github:adanoelle/fern-shell";
  imports = [ inputs.fern.homeModules.fern-shell ];
  programs.fern-shell.enable = true;
}
```

### Option B: NixOS system module

```nix
{
  imports = [ inputs.fern.nixosModules.fern-shell ];
  services.fern-shell.enable = true;
}
```

Rebuild, choose Hyprland + Fern at login, press Super+Return → Ghostty pops up.

## Contributing

The flake provides a development environment in [flake-parts/dev.nix](flake-parts/dev.nix).

```bash
nix develop
```

### What’s inside the flake

* [dev.nix](flake-parts/dev.nix)       – Nushell‑first dev env (nix develop).
* [pkgs.nix](flake-parts/pkgs.nix)     – builds the fern-shell package & exports an overlay.
* [modules/](flake-parts/modules/)     – Home‑Manager and NixOS modules; one option (programs.fern-shell.enable) is all users need.
* [checks.nix](flake-parts/checks.nix) – CI jobs (qmllint, package build).
* [fern/](fern/) – QML sources; live‑reload via qs -p fern/shell.qml.

