{ self, lib, ... }:

{
  perSystem = { self', pkgs, system, ... }: {
    checks = {
      # --- QML syntax / import validation
      qmllint = pkgs.runCommand "qmllint" { } ''
        ${pkgs.qt6.qtdeclarative}/bin/qmllint \
          ${self'.packages.fern-shell}/share/fern/shell.qml
        touch $out
      '';

      # --- Builds the packages (catches missing deps early)
      build = self'.packages.fern-shell;
      build-fern-theme = self'.packages.fern-theme;
    };
  };
}

