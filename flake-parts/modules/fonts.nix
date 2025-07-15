{ self, lib, ... }:

{
  flake = {
    ### 1. Home‑Manager ##############################################
    homeModules.fern-fonts = { config, pkgs, ... }:
      let
        cfg = config.programs.fern-fonts;
      in {
        options.programs.fern-fonts = with lib; {
          enable = mkEnableOption "Install & register Fern’s recommended fonts";

          extraFonts = mkOption {
            type = with types; listOf str;
            default = [ ];
            description = ''
              Extra font package attribute names to install in addition to
              JetBrainsMono Nerd Font and Material Symbols.
            '';
            example = literalExpression ''[ "noto-fonts" ]'';
          };
        };

        config = lib.mkIf cfg.enable {
          home.packages = with pkgs;
            [ nerdfonts.jetbrains-mono material-symbols ] ++
            (map (name: getAttr name pkgs) cfg.extraFonts);

          fonts.fontconfig.enable = true;
        };
      };

    ### 2. NixOS system ################################################
    nixosModules.fern-fonts = { config, pkgs, ... }:
      let
        cfg = config.services.fern-fonts;
      in {
        options.services.fern-fonts = with lib; {
          enable = mkEnableOption "System‑wide install of Fern fonts";

          extraFonts = mkOption {
            type = with types; listOf str;
            default = [ ];
            description = ''
              Extra font packages (system scope).  Useful for additional CJK
              or emoji fonts.
            '';
          };
        };

        config = lib.mkIf cfg.enable {
          fonts = {
            enableDefaultPackages = true;
            packages = with pkgs;
              [ nerdfonts.jetbrains-mono material-symbols ]
              ++ (map (name: getAttr name pkgs) cfg.extraFonts);
          };
        };
      };
  };
}

