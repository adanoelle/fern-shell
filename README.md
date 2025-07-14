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

    flake-parts/dev.nix – Nushell‑first dev env (nix develop).

    flake-parts/pkgs.nix – builds the fern-shell package & exports an overlay.

    flake-parts/modules/ – Home‑Manager and NixOS modules; one option (programs.fern-shell.enable) is all users need.

    flake-parts/checks.nix – CI jobs (qmllint, package build).

    fern/ – QML sources; live‑reload via qs -p fern/shell.qml.

### Quick start

# Option A: Home‑Manager
```nix
{
  inputs.fern.url = "github:adanoelle/fern-shell";
  imports = [ inputs.fern.homeModules.fern-shell ];
  programs.fern-shell.enable = true;
}
```

# Option B: NixOS system module

```nix
{
  imports = [ inputs.fern.nixosModules.fern-shell ];
  services.fern-shell.enable = true;
}
```

Rebuild, choose Hyprland + Fern at login, press Super+Return → Ghostty pops up.

### Roadmap

1. Ghostty launcher & keybind (v0.0.2)
2. Full‑screen app launcher (v0.1.0)
3. Workspace & status widgets (v0.2.0)
4. Multi‑monitor, fernctl CLI, notification pop‑ups.
References

The guidance above draws from:

    NixOS option declaration docs 
    nlewo.github.io

    nix.dev module tutorial 
    Nix

    flake‑parts official site & documentation generator 
    flake.parts

    mkdocs‑flake integration guide 
    flake.parts

    QuickShell live‑reload tutorial 
    NixOS Wiki

    Hyprland + layer‑shell availability discussions 
    NixOS Discourse

    Overlay placement advice from flake‑parts manual 
    flake.parts

    Practical flake explanations (“Flakes aren’t real…”) 
    jade.fyi

    High‑level flake intro for newcomers (“Explain to me like I’m 5”) 
    Reddit
