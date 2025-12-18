# flake‑parts/modules/home.nix
# ─────────────────────────────
#
# Home‑Manager integration for Fern QuickShell.
# Symlinks QML, installs fonts, converts TOML config to JSON,
# and exposes `programs.fern-shell` options.
#
# Provides:  flake.homeModules.fern-shell
#
#   Usage in a consumer flake:
#     imports = [ inputs.fern.homeModules.fern-shell ];
#     programs.fern-shell.enable = true;
#     programs.fern-shell.terminal = "alacritty";  # optional
#
#   Configuration options (pick one):
#     # Option 1: TOML file
#     programs.fern-shell.configFile = ./fern-config.toml;
#
#     # Option 2: Nix attribute set (merged with defaults)
#     programs.fern-shell.settings = {
#       appearance.accent = "#ff6b6b";
#       bar.height = 32;
#     };
#
{ self, inputs, lib, ... }:

{
  flake.homeModules = {
    "fern-shell" = { config, pkgs, ... }:
      let
        inherit (lib) mkEnableOption mkOption types mkIf getAttr recursiveUpdate;
        cfg = config.programs.fern-shell;

        qsPkg        = inputs.quickshell.packages.${pkgs.system}.default;
        fernPkg      = self.packages.${pkgs.system}.fern-shell;
        fernThemePkg = self.packages.${pkgs.system}.fern-theme;
        termPkg    = getAttr cfg.terminal pkgs;

        # Default configuration (mirrors fern/config.toml)
        defaultConfig = {
          appearance = {
            theme = "dark";
            accent = "#89b4fa";
            font_family = "Inter";
            font_mono = "JetBrainsMono Nerd Font";
            font_icon = "Material Symbols Rounded";
            radius = {
              none = 0;
              sm = 4;
              md = 8;
              lg = 12;
              full = 9999;
            };
          };
          bar = {
            height = 40;
            position = "top";
            margin = 0;
            modules_left = [ "workspaces" ];
            modules_center = [ "clock" ];
            modules_right = [ "tray" ];
          };
          modules = {
            clock = {
              format = "HH:mm";
              show_date = false;
              date_format = "ddd, MMM d";
            };
            workspaces = {
              show_empty = false;
              count = 10;
            };
            tray = {
              icon_size = 16;
            };
          };
        };

        # Load TOML file if provided, otherwise empty
        tomlConfig =
          if cfg.configFile != null
          then builtins.fromTOML (builtins.readFile cfg.configFile)
          else { };

        # Merge: defaults <- TOML <- Nix settings
        finalConfig = recursiveUpdate (recursiveUpdate defaultConfig tomlConfig) cfg.settings;

        # Generate JSON config file
        configJson = pkgs.writeText "fern-config.json" (builtins.toJSON finalConfig);
      in
      {
        options.programs.fern-shell = {
          enable = mkEnableOption "Enable Fern QuickShell panel";

          terminal = mkOption {
            type = types.str;
            default = "ghostty";
            description = "Terminal emulator launched by Fern.";
          };

          configFile = mkOption {
            type = types.nullOr types.path;
            default = null;
            description = ''
              Path to a TOML configuration file for Fern.
              Will be converted to JSON at build time.
            '';
            example = "./fern-config.toml";
          };

          settings = mkOption {
            type = types.attrs;
            default = { };
            description = ''
              Fern configuration as a Nix attribute set.
              Merged with defaults and any TOML config file.
              Takes precedence over TOML values.
            '';
            example = {
              appearance.accent = "#ff6b6b";
              bar.height = 32;
            };
          };
        };

        config = mkIf cfg.enable {
          home.packages = [
            fernPkg
            fernThemePkg
            qsPkg
            termPkg
            pkgs.material-symbols
            pkgs.nerdfonts.jetbrains-mono
          ];

          # Symlink QML files
          home.file.".config/quickshell/fern".source = "${fernPkg}/share/fern";

          # Install generated JSON config
          xdg.configFile."fern/config.json".source = configJson;
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
