# Development Setup

Get your development environment ready for contributing to Fern Shell.

## Prerequisites

- [Nix](https://nixos.org/download.html) with flakes enabled
- [Hyprland](https://hyprland.org/) compositor
- A code editor with QML support (recommended: VS Code with QML extension)

## Clone and Enter Dev Shell

```bash
git clone https://github.com/adanoelle/fern-shell
cd fern-shell
nix develop
```

The dev shell provides:
- QuickShell for running QML
- Qt6 development tools
- Rust toolchain for fernctl
- mdbook for documentation
- Formatting tools (nixfmt, rustfmt)

## Running the Shell

```bash
# Live preview with hot-reload
qs -p fern/shell.qml
```

Edit any QML file and save - changes appear immediately.

## Building

```bash
# Build fernctl (Rust CLI)
cargo build -p fernctl

# Build everything via Nix
nix build

# Run CI checks locally
nix flake check
```

## Project Structure

```
fern-shell/
├── fern/           # QML frontend
├── fernctl/        # Rust backend (CLI)
├── book/           # Documentation (mdbook)
├── flake-parts/    # Nix modules
└── tests/          # Test infrastructure
```

## Next Steps

- **[Architecture](architecture.md)** - Understand the system design
- **[Adding Modules](adding-modules.md)** - Create a new bar module
- **[Rust Services](rust-services.md)** - Build backend integrations
- **[Testing](testing.md)** - Test patterns and CI
