# pkgs.nix  ──  defines the fern‑shell derivation
#
# exports an overlay in the canonical place:  flake.overlays.default
#
{ self, lib, ... }:

{
  perSystem = { system, pkgs, ... }:
    let
      fernShell = pkgs.stdenv.mkDerivation {
        pname   = "fern-shell";
        version = "0.0.1";
        src     = self;
        nativeBuildInputs = [ pkgs.makeWrapper ];
        installPhase = ''
          mkdir -p $out/share/fern $out/bin

          # Copy QML files
          cp -r fern/* $out/share/fern/

          # Install helper scripts
          install -Dm755 scripts/fern-config-watch $out/bin/fern-config-watch
        '';
      };
    in
    {
      packages.fern-shell = fernShell;
    };

  flake.overlays.default = final: prev: {
    fern-shell = self.packages.${final.stdenv.hostPlatform.system}.fern-shell;
  };
}

