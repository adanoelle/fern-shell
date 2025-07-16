# flake‑parts/modules/fonts.nix
# ────────────────────────────
# This file now defines **only** the NixOS system module.
# The Home‑Manager variant lives in modules/home.nix alongside fern‑shell.
{ self, lib, ... }:

{
  flake.nixosModules."fern-fonts" = { config, pkgs, ... }:
    let
      inherit (lib) mkEnableOption mkOption types mkIf getAttr;
      cfg = config.services.fern-fonts;
    in
    {
      options.services.fern-fonts = {
        enable = mkEnableOption "System‑wide install of Fern fonts";

        extraFonts = mkOption {
          type        = types.listOf types.str;
          default     = [ ];
          description = "Extra system‑wide font packages (e.g. CJK or emoji).";
        };
      };

      config = lib.mkIf cfg.enable {
        fonts = {
          enableDefaultPackages = true;
          packages = with pkgs;
            [ nerd-fonts.jetbrains-mono material-symbols ]
            ++ map (name: getAttr name pkgs) cfg.extraFonts;
        };
      };
    };
}

