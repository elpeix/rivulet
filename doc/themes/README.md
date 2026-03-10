# Themes

Community theme files for Rivulet. Copy any `.toml` file to `~/.config/rivulet/themes/` and set `theme = "<name>"` in your `config.toml`.

## Available themes

### Dark

| Theme | Description |
|---|---|
| [kanagawa](kanagawa.toml) | Inspired by Kanagawa Wave — muted blue-grey tones (built-in default) |
| [gruvbox](gruvbox.toml) | Retro groove with warm browns and greens |
| [catppuccin-mocha](catppuccin-mocha.toml) | Soothing pastel dark with lavender tones |

### Light

| Theme | Description |
|---|---|
| [solarized-light](solarized-light.toml) | Ethan Schoonover's precision light palette |
| [catppuccin-latte](catppuccin-latte.toml) | Warm pastel light with blue accents |
| [gruvbox-light](gruvbox-light.toml) | Light complement of the Gruvbox palette |

## Creating your own theme

Create a `.toml` file with the required color fields. Colors can be:

- **Hex**: `"#RRGGBB"` (e.g. `"#282828"`)
- **Named**: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `gray`, `dark_gray`, `light_red`, `light_green`, `light_yellow`, `light_blue`, `light_magenta`, `light_cyan`, `reset`

Required fields (16):

```
header_bg, border, focus_border, focus_title,
highlight_bg, highlight_fg, focus_bg, block_bg,
feeds_bg, preview_bg, text, dim,
status_ok, status_err, accent, accent_alt
```

Optional fields:

| Field | Description | Default |
|---|---|---|
| `selection_bg` | Background for multi-selected entries | `#323246` (dark) / `dark_gray` (terminal) |

Use `reset` for colors that should inherit from the terminal (useful for the `terminal` built-in theme style).
