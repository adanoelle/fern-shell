{
  description = "Fern – a QuickShell‑based Hyprland shell";

  inputs = {
    nixpkgs.url     = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    quickshell.url  = "git+https://git.outfoxxed.me/outfoxxed/quickshell";
    quickshell.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];

      imports = [
        ./flake-parts/dev.nix
        ./flake-parts/pkgs.nix
        ./flake-parts/checks.nix
        ./flake-parts/modules/home.nix
        ./flake-parts/modules/system.nix
      ];
    };
}

