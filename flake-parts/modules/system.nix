{ self, inputs, lib, ... }:

{
  flake.nixosModules = {
    fern-shell = { config, pkgs, ... }:
      let
        cfg     = config.services.fern-shell;
        qsPkg   = inputs.quickshell.packages.${pkgs.system}.default;
        fernPkg = self.packages.${pkgs.system}.fern-shell;
      in
      {
        options.services.fern-shell = with lib; {
          enable = mkEnableOption "Start Fern QuickShell inside Hyprland";
        };

        config = lib.mkIf cfg.enable {
          # Install runtime deps
          environment.systemPackages = [
            fernPkg
            qsPkg
            pkgs.ghostty              # default terminal
          ];

          # Rely on upstream Hyprland module for portals/polkit
          programs.hyprland.enable = true;   # simple and canonical :contentReference[oaicite:7]{index=7}

          # Auto‑start QuickShell once per Hyprland session
          programs.hyprland.extraConfig = ''
            exec-once = ${qsPkg}/bin/qs -c fern
          '';
        };
      };
  };
}

