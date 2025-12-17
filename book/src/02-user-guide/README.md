# Configuration

Fern Shell is configured through a combination of TOML files and
Home-Manager/NixOS options.

## Configuration Sources

1. **Home-Manager/NixOS options** - Primary configuration method
2. **`~/.config/fern/config.toml`** - Runtime configuration file
3. **Environment variables** - Override specific settings

## Quick Example

```toml
[appearance]
theme = "dark"
accent = "#89b4fa"
font_family = "Inter"

[bar]
position = "left"
width = 48
persistent = false
modules = ["workspaces", "spacer", "clock"]

[modules.workspaces]
count = 10
show_empty = true

[modules.clock]
format_time = "%H\n%M"
format_date = "%a\n%d"
```

## Sections

- **[Config File Format](config-file.md)** - Complete TOML schema
- **[Theming](theming.md)** - Colors, fonts, and design tokens
- **[Modules](modules.md)** - Configure individual modules
