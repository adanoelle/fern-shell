# flake‑parts/modules/home.nix
# ─────────────────────────────
# 
# Home‑Manager integration for Fern QuickShell.
# Symlinks QML, installs fonts, and exposes a single `programs.fern-shell`
# option documented via mkOption below.
# 
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
    "fern-shell" = { config, pkgs, ... }:
      let
        inherit (lib) mkEnableOption mkOption types mkIf getAttr;
        cfg = config.programs.fern-shell;

        qsPkg   = inputs.quickshell.packages.${pkgs.system}.default;
        fernPkg = self.packages.${pkgs.system}.fern-shell;
        termPkg = getAttr cfg.terminal pkgs;
      in
      {
        options.programs.fern-shell = {
          enable   = mkEnableOption "Enable Fern QuickShell panel";
          terminal = mkOption {
            type = types.str; default = "ghostty";
            description = "Terminal emulator launched by Fern.";
          };
        };

        config = mkIf cfg.enable {
          home.packages = [
            fernPkg qsPkg termPkg
            pkgs.material-symbols
            pkgs.nerdfonts.jetbrains-mono
          ];

          home.file.".config/quickshell/fern".source =
            "${fernPkg}/share/fern";
        };
      };

    "fern-fonts" = { config, pkgs, ... }:
      let
        inherit (lib) mkEnableOption mkOption types mkIf literalExpression getAttr;
        cfg = config.programs.fern-fonts;
      in
      {
        options.programs.fern-fonts = {
          enable = mkEnableOption "Install Fern’s recommended fonts";
          extraFonts = mkOption {
            type = types.listOf types.str; default = [ ];
            description = "Extra font packages to install for this user.";
            example = literalExpression ''[ "noto-fonts" ]'';
          };
        };

        config = mkIf cfg.enable {
          home.packages = with pkgs;
            [ nerdfonts.jetbrains-mono material-symbols ]
            ++ map (name: getAttr name pkgs) cfg.extraFonts;

          fonts.fontconfig.enable = true;
        };
      };
  };
}
