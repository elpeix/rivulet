# Rivulet

A terminal RSS reader built with Rust and [ratatui](https://github.com/ratatui-org/ratatui).

![TUI](https://img.shields.io/badge/interface-TUI-blue)
![Rust](https://img.shields.io/badge/lang-Rust-orange)
![License](https://img.shields.io/badge/license-GPLv3-green)

## Features

- **Dual layout** — 3-column (Feeds | Entries | Preview) or 2-column split mode, toggle with `w`
- **Feed categories** — Group feeds by topic with collapsible sections
- **Rich HTML preview** — Bold, italic, links, code blocks, lists
- **Smart filtering** — Unread, saved, configurable time filter, incremental search
- **Entry sorting** — By date (newest/oldest first) or title A-Z
- **Mouse support** — Click to select, scroll to navigate
- **Mark read** — Mark individual entries, all visible, or entire feed as read
- **Feed renaming** — Custom names with ability to restore the original
- **Save for later** — Bookmark entries with `s`, filter saved with `g`
- **Feed auto-discovery** — Paste a website URL and Rivulet finds the RSS/Atom feed automatically
- **OPML import/export** — Migrate feeds from/to other RSS readers
- **Auto-refresh** — Configurable periodic refresh (default: 30 min)
- **i18n** — English and Catalan
- **Local SQLite storage** — No external services required

## Install

```sh
cargo install --path .
```

Requires Rust 2024 edition (1.85+).

## Usage

```sh
rivulet
```

Press `a` to add a feed. You can paste either a direct feed URL or a website URL — Rivulet will auto-discover the feed. Press `r` to refresh.

### OPML import/export

Import feeds from another RSS reader:

```sh
rivulet import subscriptions.opml
```

Export your feeds:

```sh
rivulet export subscriptions.opml   # to file
rivulet export                      # to stdout
```

Categories are preserved during import and export.

### Key bindings

| Key | Action |
|---|---|
| **Navigation** | |
| `Left` / `Right` | Previous / next panel |
| `1` / `2` / `3` | Jump to Feeds / Entries / Preview |
| `Up` / `Down` | Move selection |
| `PgUp` / `PgDn` | Scroll preview |
| `Home` / `End` | Top / bottom |
| `H` / `L` | Resize focused panel |
| `w` | Toggle layout (columns / split) |
| `Enter` | Select feed / open entry |
| `Space` | Collapse/expand category |
| `Esc` | Back |
| **Feeds** | |
| `a` | Add feed |
| `e` | Rename feed |
| `d` | Delete feed |
| `r` | Refresh all feeds |
| `f` | Toggle unread filter |
| `g` | Toggle saved filter |
| `c` | Assign category |
| `C` | Manage categories |
| `R` | Mark feed as read |
| `S` | Cycle sort mode |
| `t` | Toggle time filter |
| **Entries** | |
| `m` | Toggle read/unread |
| `M` | Mark all visible as read |
| `s` | Save for later |
| `/` | Search |
| `o` | Open in browser |
| `Tab` / `Shift+Tab` | Next / previous link in preview |
| **General** | |
| `?` | Help |
| `q` | Quit |

## Config

```toml
# ~/.config/rivulet/config.toml
language = "en"         # or "ca" for Catalan
refresh_minutes = 30    # auto-refresh interval (0 to disable)
recent_days = 30        # time filter window in days
layout = "columns"      # or "split" for 2-column stacked layout
```

The config file is created automatically on first run.

## License

[GNU General Public License v3.0](LICENSE)
