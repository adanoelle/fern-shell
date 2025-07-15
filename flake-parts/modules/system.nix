{ self, inputs, lib, ... }:

{
  flake.nixosModules."fern-shell" = { config, pkgs, ... }:
    let
      qsPkg   = inputs.quickshell.packages.${pkgs.system}.default;
      fernPkg = self.packages.${pkgs.system}.fern-shell;
      cfg     = config.services.fern-shell;
    in
    {
      options.services.fern-shell = with lib; {
        enable = mkEnableOption "Start Fern QuickShell inside Hyprland";
      };

      config = lib.mkIf cfg.enable {
        environment.systemPackages = [ fernPkg qsPkg pkgs.ghostty ];
        programs.hyprland.enable   = true;

        # Hyprland 0.40+ : use structured `settings`
        programs.hyprland.settings."exec-once" = [
          "${qsPkg}/bin/qs -c fern"
        ];
      };
    };
}
