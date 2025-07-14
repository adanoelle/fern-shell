# flake‑parts/modules/home.nix
# Provides:  flake.homeModules.fern-shell
#
#   Usage in a consumer flake:
#     imports = [ inputs.fern.homeModules.fern-shell ];
#     programs.fern-shell.enable  = true;
#     programs.fern-shell.terminal = "alacritty";  # optional override
# 
{ self, inputs, lib, ... }:

{
  flake.homeModules = {
    fern-shell = { config, pkgs, ... }:
      let
        inherit (lib) mkEnableOption mkOption types mkIf getAttr;
        cfg      = config.programs.fern-shell;

        # default to Ghostty; user can override with any pkgs attr‑name
        terminalPkg = getAttr cfg.terminal pkgs;
        qsPkg       = inputs.quickshell.packages.${pkgs.system}.default;
        fernPkg     = self.packages.${pkgs.system}.fern-shell;
      in
      {
        options.programs.fern-shell = {
          enable = mkEnableOption "Enable the Fern QuickShell environment";

          terminal = mkOption {
            type        = types.str;
            default     = "ghostty";
            description = "Terminal emulator launched by Fern.";
          };
        };

        config = mkIf cfg.enable {
          home.packages = [
            fernPkg
            qsPkg
            terminalPkg               # ghostty by default
            pkgs.material-symbols
            pkgs.nerdfonts.jetbrains-mono
          ];

          # Symlink QML sources
          home.file.".config/quickshell/fern".source =
            "${fernPkg}/share/fern";
        };
      };
  };
}

