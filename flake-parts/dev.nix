# flake-parts/dev.nix
{ inputs, lib, ... }:

{
  perSystem = { system, pkgs, ... }:
    let
      qsPkg    = inputs.quickshell.packages.${system}.default;   # ← scoped
      rustPkgs = [ pkgs.rustc pkgs.cargo ];                      # deterministic, no overlay
    in
    {
      formatter = pkgs.nixfmt-rfc-style;

      devShells.default = pkgs.mkShell {
        QT_QPA_PLATFORM = "wayland";

        packages = with pkgs; [
          qsPkg           # QuickShell CLI + QML libs
          hyprland
          qt6.full        # designer, qmllint, etc.
          ghostty         # default terminal for live tests
          nushell
        ] ++ rustPkgs;

        shellHook = ''
          echo ---- Fern dev‑shell ready ----
          echo "Live preview:  qs -p fern/shell.qml"
        '';
      };
    };
}
