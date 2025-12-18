# pkgs.nix  ──  defines the fern‑shell and fern-theme derivations
#
# exports an overlay in the canonical place:  flake.overlays.default
#
# Packages:
#   - fern-shell: QML files for the QuickShell panel
#   - fern-theme: Rust CLI for theme/config management (validate, convert, query, watch)
#   - fern-core: Shared library (built as part of fern-theme)
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

      # Alias for backwards compatibility during transition
      fernctl = fernTheme;
    in
    {
      packages = {
        inherit fernShell fernTheme fernctl;
        fern-shell = fernShell;
        fern-theme = fernTheme;
        default = fernShell;
      };
    };

  flake.overlays.default = final: prev: {
    fern-shell = self.packages.${final.stdenv.hostPlatform.system}.fern-shell;
    fern-theme = self.packages.${final.stdenv.hostPlatform.system}.fern-theme;
    # Backwards compatibility alias
    fernctl = self.packages.${final.stdenv.hostPlatform.system}.fern-theme;
  };
}
