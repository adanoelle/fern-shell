# pkgs.nix  ──  defines the fern‑shell and fern-theme derivations
#
# exports an overlay in the canonical place:  flake.overlays.default
#
# Packages:
#   - fern-shell: QML files for the QuickShell panel
#   - fern-theme: Rust CLI for theme/config management (validate, convert, query, watch)
#   - fern-obs: Rust daemon for OBS WebSocket integration
#   - fern-core: Shared library (built as part of fern-theme and fern-obs)
#
{ self, lib, inputs, ... }:

{
  perSystem = { system, pkgs, ... }:
    let
      # Re-export quickshell for users
      quickshell = inputs.quickshell.packages.${system}.default;
      # QML panel files
      fernShell = pkgs.stdenv.mkDerivation {
        pname   = "fern-shell";
        version = "0.1.0";
        src     = self;
        installPhase = ''
          mkdir -p $out/share/fern
          cp -r fern/* $out/share/fern/
        '';
      };

      # Rust CLI for theme/configuration management
      # Built from the Cargo workspace
      fernTheme = pkgs.rustPlatform.buildRustPackage {
        pname = "fern-theme";
        version = "0.1.0";
        src = self;
        cargoLock.lockFile = "${self}/Cargo.lock";

        # Build only the fern-theme package from the workspace
        buildAndTestSubdir = "crates/fern-theme";

        # Build with all default features (cli, fancy-errors, watch)
        buildFeatures = [ "cli" "fancy-errors" "watch" ];

        meta = with lib; {
          description = "Theme and configuration manager for Fern Shell";
          homepage = "https://github.com/adanoelle/fern-shell";
          license = licenses.mit;
          maintainers = [ ];
          mainProgram = "fern-theme";
        };
      };

      # Rust daemon for OBS WebSocket integration
      # Built from the Cargo workspace
      fernObs = pkgs.rustPlatform.buildRustPackage {
        pname = "fern-obs";
        version = "0.1.0";
        src = self;
        cargoLock.lockFile = "${self}/Cargo.lock";

        # Build only the fern-obs package from the workspace
        buildAndTestSubdir = "crates/fern-obs";

        # Build with CLI feature
        buildFeatures = [ "cli" ];

        meta = with lib; {
          description = "OBS WebSocket bridge daemon for Fern Shell";
          homepage = "https://github.com/adanoelle/fern-shell";
          license = licenses.mit;
          maintainers = [ ];
          mainProgram = "fern-obs";
        };
      };

      # Control plane CLI + TUI for managing Fern Shell
      # Built from the Cargo workspace
      fernctl = pkgs.rustPlatform.buildRustPackage {
        pname = "fernctl";
        version = "0.1.0";
        src = self;
        cargoLock.lockFile = "${self}/Cargo.lock";

        # Build only the fernctl package from the workspace
        buildAndTestSubdir = "crates/fernctl";

        # Build with CLI and TUI features
        buildFeatures = [ "cli" "tui" ];

        meta = with lib; {
          description = "Control plane for Fern Shell (CLI + TUI dashboard)";
          homepage = "https://github.com/adanoelle/fern-shell";
          license = licenses.mit;
          maintainers = [ ];
          mainProgram = "fernctl";
        };
      };
    in
    {
      packages = {
        inherit fernShell fernTheme fernObs fernctl quickshell;
        fern-shell = fernShell;
        fern-theme = fernTheme;
        fern-obs = fernObs;
        default = fernShell;
      };
    };

  flake.overlays.default = final: prev: {
    fern-shell = self.packages.${final.stdenv.hostPlatform.system}.fern-shell;
    fern-theme = self.packages.${final.stdenv.hostPlatform.system}.fern-theme;
    fern-obs = self.packages.${final.stdenv.hostPlatform.system}.fern-obs;
    fernctl = self.packages.${final.stdenv.hostPlatform.system}.fernctl;
  };
}
