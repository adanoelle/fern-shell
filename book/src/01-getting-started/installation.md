# Installation

Fern Shell is distributed as a Nix flake with Home-Manager and NixOS modules.

## Prerequisites

- [Nix](https://nixos.org/download.html) with flakes enabled
- [Hyprland](https://hyprland.org/) compositor
- [QuickShell](https://quickshell.outfoxxed.me/) (provided by flake)

## Home-Manager Installation

Add the flake to your `flake.nix` inputs:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager.url = "github:nix-community/home-manager";
    fern-shell.url = "github:adanoelle/fern-shell";
  };
}
```

Enable the module in your Home-Manager configuration:

```nix
{ inputs, ... }:
{
  imports = [ inputs.fern-shell.homeManagerModules.default ];

  programs.fern-shell = {
    enable = true;
    # Optional: customize settings
    settings = {
      appearance = {
        theme = "dark";
        accent = "#89b4fa";
      };
      bar = {
        position = "left";
        width = 48;
      };
    };
  };
}
```

## NixOS Installation

For system-wide installation, use the NixOS module:

```nix
{ inputs, ... }:
{
  imports = [ inputs.fern-shell.nixosModules.default ];

  programs.fern-shell.enable = true;
}
```

## Manual / Development

Clone and run directly:

```bash
git clone https://github.com/adanoelle/fern-shell
cd fern-shell
nix develop

# Run with live reload
qs -p fern/shell.qml
```

## Verifying Installation

After installation, start Hyprland. Fern Shell should appear as a bar on the
left edge of your screen.

If you don't see the bar:

1. Check that Hyprland is running
2. Look for errors: `journalctl --user -u fern-shell`
3. See [Troubleshooting](../TROUBLESHOOTING.md)
