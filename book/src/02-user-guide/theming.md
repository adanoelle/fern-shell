# Theming

Fern Shell uses a three-tier design token system for consistent theming.

## Token Architecture

```
Tier 1: Primitives (built-in palettes)
  ↓
Tier 2: Semantic Tokens (configurable)
  ↓
Tier 3: Component Tokens (internal)
```

### Tier 1: Primitives

Built-in color palettes that you don't configure directly:

**Dark Palette (Catppuccin Mocha)**

- Base: `#1e1e2e`
- Surface: `#313244`
- Text: `#cdd6f4`

**Light Palette (Catppuccin Latte)**

- Base: `#eff1f5`
- Surface: `#e6e9ef`
- Text: `#4c4f69`

### Tier 2: Semantic Tokens

These are what you configure in `config.toml`:

```toml
[appearance]
theme = "dark"      # Selects the palette
accent = "#89b4fa"  # Override the accent color
```

### Tier 3: Component Tokens

Internal mappings that components use. You don't configure these directly:

- `barBackground` → `background`
- `moduleBackground` → `surface`
- `textPrimary` → `foreground`

## Color Reference

| Token        | Dark      | Light     | Usage              |
| ------------ | --------- | --------- | ------------------ |
| `background` | `#1e1e2e` | `#eff1f5` | Bar background     |
| `surface`    | `#313244` | `#e6e9ef` | Module backgrounds |
| `foreground` | `#cdd6f4` | `#4c4f69` | Primary text       |
| `accent`     | `#89b4fa` | `#1e66f5` | Active states      |

## Fonts

```toml
[appearance]
font_family = "Inter"                    # UI text
font_mono = "JetBrainsMono Nerd Font"    # Monospace
font_icon = "Material Symbols Rounded"   # Icons
```

## Custom Accents

Popular accent colors from Catppuccin:

| Name  | Hex       | Preview    |
| ----- | --------- | ---------- |
| Blue  | `#89b4fa` | Default    |
| Mauve | `#cba6f7` | Purple     |
| Pink  | `#f5c2e7` | Light pink |
| Teal  | `#94e2d5` | Cyan-ish   |
| Green | `#a6e3a1` | Soft green |
