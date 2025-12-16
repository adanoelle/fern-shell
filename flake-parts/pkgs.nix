# pkgs.nix  ──  defines the fern‑shell and fernctl derivations
#
# exports an overlay in the canonical place:  flake.overlays.default
#
# Packages:
#   - fern-shell: QML files for the QuickShell panel
#   - fernctl: Rust CLI for config management (validate, convert, query, watch)
#
{ self, lib, ... }:

{
  perSystem = { system, pkgs, ... }:
    let
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

      # Rust CLI for configuration management
      fernctl = pkgs.rustPlatform.buildRustPackage {
        pname = "fernctl";
        version = "0.1.0";
        src = "${self}/fernctl";
        cargoLock.lockFile = "${self}/fernctl/Cargo.lock";

        # Build with all default features (cli, fancy-errors, watch)
        buildFeatures = [ "cli" "fancy-errors" "watch" ];

        meta = with lib; {
          description = "Configuration manager for Fern Shell";
          homepage = "https://github.com/adanoelle/fern-shell";
          license = licenses.mit;
          maintainers = [ ];
          mainProgram = "fernctl";
        };
      };
    in
    {
      packages = {
        inherit fernShell fernctl;
        fern-shell = fernShell;
        default = fernShell;
      };
    };

  flake.overlays.default = final: prev: {
    fern-shell = self.packages.${final.stdenv.hostPlatform.system}.fern-shell;
    fernctl = self.packages.${final.stdenv.hostPlatform.system}.fernctl;
  };
}

