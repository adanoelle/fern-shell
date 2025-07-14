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

      # --- Builds the package (catches missing deps early)
      build = self'.packages.fern-shell;
    };
  };
}

