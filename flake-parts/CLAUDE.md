# Flake-Parts Infrastructure Guide

## Overview

This directory contains the modular Nix infrastructure for Fern Shell, organized
using the flake-parts framework. Each file serves a specific purpose in the
build, development, and distribution pipeline.

## File Structure & Responsibilities

```
flake-parts/
├── dev.nix       # Development shell environment
├── pkgs.nix      # Package definitions and overlay
├── checks.nix    # CI/CD checks and validations
└── modules/      # Nix module definitions
    ├── home.nix    # Home-Manager module
    ├── system.nix  # NixOS system module
    └── fonts.nix   # Font configuration module
```

## Design Principles

### 1. Separation of Concerns

Each file has a single, clear responsibility. Don't mix package definitions with
module logic or development tools.

### 2. Minimal Overlays

We avoid overlays where possible, preferring explicit package references. The
only overlay we export is for `fern-shell` itself.

### 3. Input Following

All flake inputs follow `nixpkgs` to ensure consistency and reduce closure size:

```nix
quickshell.inputs.nixpkgs.follows = "nixpkgs";
```

### 4. Cross-Platform Awareness

While currently Linux-only, structure allows for future platform support:

```nix
systems = [ "x86_64-linux" "aarch64-linux" ];  # Future
```

## Package Building (pkgs.nix)

### Current Pattern

```nix
perSystem = { system, pkgs, ... }:
  let
    fernShell = pkgs.stdenv.mkDerivation {
      pname = "fern-shell";
      version = "0.0.1";  # TODO: Use git revision
      src = self;

      installPhase = ''
        mkdir -p $out/share/fern
        cp -r fern $out/share/
      '';
    };
  in {
    packages.fern-shell = fernShell;
  };
```

### Future Enhancements

```nix
# Version from git
version = self.shortRev or "dev";

# Rust components
nativeBuildInputs = with pkgs; [ rustc cargo ];

buildPhase = ''
  cargo build --release -p fernctl
'';

# Runtime dependencies
propagatedBuildInputs = with pkgs; [
  quickshell
  qt6.qtbase
];

# Wrapper for environment
postInstall = ''
  wrapProgram $out/bin/fernctl \
    --prefix PATH : ${lib.makeBinPath [ quickshell ]}
'';
```

## Development Shell (dev.nix)

### Environment Variables

```nix
devShells.default = pkgs.mkShell {
  # Wayland/Qt configuration
  QT_QPA_PLATFORM = "wayland";
  QT_SCALE_FACTOR = "1";

  # Development paths
  FERN_DEV_MODE = "1";
  XDG_DATA_DIRS = "${fernShell}/share:$XDG_DATA_DIRS";

  # Rust development (future)
  RUST_BACKTRACE = "1";
  RUST_LOG = "fern=debug";
};
```

### Tool Categories

```nix
packages = with pkgs; [
  # Core tools
  qsPkg
  hyprland

  # QML development
  qt6.qttools      # Designer, qmllint
  qt6.qtdeclarative # QML runtime

  # Rust development (future)
  rustc
  cargo
  rust-analyzer

  # System integration testing
  dbus
  pipewire

  # Utilities
  jq               # JSON processing
  ripgrep          # Fast searching
  watchexec        # File watching
];
```

### Shell Hooks

```nix
shellHook = ''
  echo "Fern Dev Shell Activated"
  echo "Commands:"
  echo "  qs -p fern/shell.qml  - Live preview"
  echo "  nix flake check       - Run CI checks"
  echo "  nix fmt              - Format code"

  # Set up QML import path
  export QML2_IMPORT_PATH="${qsPkg}/lib/qt-6/qml:$QML2_IMPORT_PATH"

  # Create dev symlink for faster iteration
  ln -sf $PWD/fern ~/.config/quickshell/fern-dev 2>/dev/null || true
'';
```

## Module System (modules/)

### Home-Manager Module Pattern

```nix
# modules/home.nix
flake.homeModules.fern-shell = { config, lib, pkgs, ... }:
  let
    cfg = config.programs.fern-shell;
    inherit (lib) mkEnableOption mkOption types mkIf;
  in {
    options.programs.fern-shell = {
      enable = mkEnableOption "Fern Shell";

      package = mkOption {
        type = types.package;
        default = self.packages.${pkgs.system}.fern-shell;
        description = "Fern Shell package to use";
      };

      config = mkOption {
        type = types.attrs;
        default = {};
        description = "Configuration to pass to Fern";
        example = literalExpression ''
          {
            position = "top";
            height = 32;
            modules = [ "workspaces" "clock" "tray" ];
          }
        '';
      };

      terminal = mkOption {
        type = types.str;
        default = "ghostty";
        description = "Terminal emulator to launch";
      };

      extraPackages = mkOption {
        type = types.listOf types.package;
        default = [];
        description = "Extra packages to install with Fern";
      };
    };

    config = mkIf cfg.enable {
      # Install packages
      home.packages = [ cfg.package ] ++ cfg.extraPackages;

      # Set up configuration
      xdg.configFile."quickshell/fern".source =
        "${cfg.package}/share/fern";

      xdg.configFile."fern/config.json".text =
        builtins.toJSON cfg.config;

      # Systemd service (future)
      systemd.user.services.fern-shell = {
        Unit = {
          Description = "Fern Shell";
          PartOf = [ "graphical-session.target" ];
        };
        Service = {
          ExecStart = "${cfg.package}/bin/fern-launcher";
          Restart = "on-failure";
        };
        Install.WantedBy = [ "hyprland-session.target" ];
      };
    };
  };
```

### NixOS System Module Pattern

```nix
# modules/system.nix
flake.nixosModules.fern-shell = { config, lib, pkgs, ... }:
  let
    cfg = config.services.fern-shell;
  in {
    options.services.fern-shell = {
      enable = mkEnableOption "Fern Shell system-wide";

      users = mkOption {
        type = types.listOf types.str;
        default = [];
        description = "Users to enable Fern Shell for";
      };
    };

    config = mkIf cfg.enable {
      # System-wide packages
      environment.systemPackages = [ fernPkg qsPkg ];

      # D-Bus services
      services.dbus.packages = [ fernPkg ];

      # User configuration
      users.users = lib.genAttrs cfg.users (user: {
        packages = [ fernPkg ];
      });

      # Session configuration
      programs.hyprland.enable = mkDefault true;
    };
  };
```

## CI/CD Checks (checks.nix)

### Check Categories

```nix
perSystem = { self', pkgs, ... }: {
  checks = {
    # Package builds
    package = self'.packages.fern-shell;

    # QML validation
    qml-lint = pkgs.runCommand "qml-lint" {} ''
      ${pkgs.qt6.qtdeclarative}/bin/qmllint \
        ${self}/fern/**/*.qml
      touch $out
    '';

    # Nix formatting
    formatting = pkgs.runCommand "nix-fmt-check" {} ''
      ${pkgs.nixfmt-rfc-style}/bin/nixfmt --check \
        ${self}/**/*.nix
      touch $out
    '';

    # Future: Rust tests
    rust-tests = pkgs.runCommand "rust-tests" {} ''
      cd ${self}
      ${pkgs.cargo}/bin/cargo test
      touch $out
    '';

    # Module evaluation
    module-eval = pkgs.runCommand "module-eval" {} ''
      ${pkgs.nix}/bin/nix eval --impure --expr '
        (import <nixpkgs/nixos> {
          configuration = {
            imports = [ ${self.nixosModules.fern-shell} ];
            services.fern-shell.enable = true;
          };
        }).config
      '
      touch $out
    '';
  };
};
```

## Dependency Management

### Current Dependencies

```nix
inputs = {
  nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  flake-parts.url = "github:hercules-ci/flake-parts";
  quickshell.url = "git+https://git.outfoxxed.me/outfoxxed/quickshell";
};
```

### Adding New Dependencies

```nix
# 1. Add to flake.nix inputs
inputs.rust-overlay.url = "github:oxalica/rust-overlay";

# 2. Follow nixpkgs when possible
inputs.rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

# 3. Use in appropriate file
# dev.nix:
{ inputs, ... }: {
  perSystem = { system, ... }: {
    _module.args.pkgs = import inputs.nixpkgs {
      inherit system;
      overlays = [ inputs.rust-overlay.overlays.default ];
    };
  };
}
```

## Configuration Patterns

### User Configuration Flow

```
~/.config/fern/config.json → Nix module → Package wrapper → QML runtime
```

### Configuration Schema (Future)

```nix
# Generate JSON schema from Nix options
configSchema = pkgs.writeText "config-schema.json" (
  builtins.toJSON (lib.evalModules {
    modules = [{
      options = fernConfigOptions;
    }];
  }).options
);
```

## Testing Strategies

### Local Testing

```bash
# Test package build
nix build .#fern-shell

# Test module in VM
nix run nixpkgs#nixos-rebuild -- build-vm \
  --flake .#test-vm

# Test Home-Manager module
home-manager switch --flake .#test-home
```

### Integration Tests

```nix
# tests/default.nix
{
  # Hyprland session test
  hyprland-integration = nixosTest {
    nodes.machine = {
      imports = [ self.nixosModules.fern-shell ];
      services.fern-shell.enable = true;
      services.xserver.displayManager.autoLogin = {
        enable = true;
        user = "test";
      };
    };

    testScript = ''
      machine.wait_for_unit("graphical.target")
      machine.wait_for_file("/run/user/1000/hyprland.sock")
      machine.succeed("pgrep quickshell")
    '';
  };
}
```

## Performance Considerations

### Build Performance

- Use `callPackage` pattern for better caching
- Minimize import-from-derivation
- Prefer `runCommand` over `mkDerivation` for simple tasks

### Runtime Performance

- Lazy-load optional features
- Use systemd socket activation where applicable
- Minimize closure size with `nix path-info -S`

## Common Patterns

### Optional Features

```nix
mkOption {
  type = types.submodule {
    options = {
      enable = mkEnableOption "feature";
      package = mkOption {
        type = types.package;
        default = pkgs.feature;
      };
    };
  };
  default = {};
}
```

### Package Variants

```nix
packages = {
  fern-shell = fernShell;
  fern-shell-minimal = fernShell.override {
    withTray = false;
    withBluetooth = false;
  };
  fern-shell-full = fernShell.override {
    withAllModules = true;
  };
};
```

### Cross-Compilation (Future)

```nix
packages = lib.genAttrs systems (system:
  let
    pkgs = nixpkgsFor.${system};
  in {
    fern-shell = pkgs.callPackage ./package.nix {};
  }
);
```

## Debugging Tips

### Nix Debugging

```bash
# Show derivation
nix show-derivation .#fern-shell

# Eval module options
nix eval .#homeModules.fern-shell.options

# Build with debug
nix build .#fern-shell --print-build-logs

# Enter build environment
nix develop .#fern-shell
```

### Module Debugging

```nix
# Add debug output
config = mkIf cfg.enable (lib.traceVal {
  # config here
});

# Check final config
nix eval --json .#nixosConfigurations.example.config.services.fern-shell
```

## Future Enhancements

### Plugin System

```nix
# Allow third-party modules
extraModules = mkOption {
  type = types.listOf types.path;
  default = [];
  description = "Extra QML modules to load";
};
```

### Theme Management

```nix
themes = {
  nord = ./themes/nord.nix;
  dracula = ./themes/dracula.nix;
  catppuccin = ./themes/catppuccin.nix;
};
```

### Update Automation

```nix
# Update flake inputs weekly
.github/workflows/update.yml:
  - uses: DeterminateSystems/update-flake-lock@v20
    with:
      pr-title: "Update flake.lock"
      pr-labels: dependencies
```
