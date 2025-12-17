# Testing

Test patterns for Fern Shell.

## QML Testing

### Manual Testing

The primary testing method is visual verification with live reload:

```bash
qs -p fern/shell.qml
```

Edit files and observe changes immediately.

### QML Lint

Check for syntax errors and type issues:

```bash
nix develop -c qmllint fern/**/*.qml
```

This runs automatically in CI.

## Rust Testing

### Unit Tests

```bash
cargo test -p fernctl
```

Tests live alongside code in `#[cfg(test)]` modules:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_composition() {
        let config = UserConfig::default();
        let theme = Theme::from_config(&config);
        assert_eq!(theme.accent, "#89b4fa");
    }
}
```

### Integration Tests

Located in `/fernctl/tests/`:

```bash
cargo test --test integration
```

### Doc Tests

Examples in documentation are tested:

```rust
/// Convert user config to theme.
///
/// # Example
///
/// ```
/// use fernctl::domain::UserConfig;
/// let config = UserConfig::default();
/// assert!(config.appearance.theme == "dark");
/// ```
pub fn example() {}
```

## Nix Checks

Run all CI checks locally:

```bash
nix flake check
```

This includes:
- `qmllint` - QML syntax/type checking
- `cargo test` - Rust tests
- `cargo clippy` - Rust lints
- `nix build` - Package builds successfully

## CI/CD

GitHub Actions runs on every push:

```yaml
# .github/workflows/ci.yml
- name: Run checks
  run: nix flake check
```

## Test Patterns

### Mock Services

Use trait objects for testing without real system services:

```rust
trait HyprlandPort {
    fn get_workspaces(&self) -> Vec<Workspace>;
}

struct MockHyprland {
    workspaces: Vec<Workspace>,
}

impl HyprlandPort for MockHyprland {
    fn get_workspaces(&self) -> Vec<Workspace> {
        self.workspaces.clone()
    }
}
```

### Snapshot Testing

For configuration output, use `insta`:

```rust
#[test]
fn test_config_serialization() {
    let config = UserConfig::default();
    let json = serde_json::to_string_pretty(&config).unwrap();
    insta::assert_snapshot!(json);
}
```

## Coverage

Generate coverage report:

```bash
cargo tarpaulin --out Html
open tarpaulin-report.html
```
