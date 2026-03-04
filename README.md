# Rivulet

A terminal RSS reader built with Rust and [ratatui](https://github.com/ratatui-org/ratatui).

![TUI](https://img.shields.io/badge/interface-TUI-blue)
![Rust](https://img.shields.io/badge/lang-Rust-orange)

## Features

- 3-panel layout: Feeds | Entries | Preview
- Feed categories with collapsible groups
- Rich HTML preview (bold, italic, links, code blocks)
- Incremental search, read/unread and saved filters
- Mouse support, resizable panels (H/L keys)
- i18n: English and Catalan (`~/.config/rivulet/config.toml`)
- Local SQLite storage

## Install

```sh
cargo install --path .
```

## Usage

```sh
rivulet
```

### Key bindings

| Key | Action |
|-----|--------|
| `a` | Add feed |
| `d` | Delete feed |
| `r` | Refresh all |
| `f` / `g` | Toggle unread / saved filter |
| `s` | Save entry |
| `o` | Open in browser |
| `/` | Search |
| `c` / `C` | Assign / manage categories |
| `H` / `L` | Resize panels |
| `?` | Help |
| `q` | Quit |

## Config

```toml
# ~/.config/rivulet/config.toml
language = "en"  # or "ca" for Catalan
```

## License

GPLv3
