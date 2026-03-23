# Changelog

## [1.3.0] - 2026-03-16

### Added

- **Copy preview to clipboard**: Press `y` in the Preview panel to copy the rendered text (title + body) to the system clipboard
- **Clear read entries**: Press `x` in the Entries panel (when unread filter is active) to remove already-read entries from the visible list
- **Reload entries**: Press `F5` to force a full reload of entries from the database
- **Mouse scroll in help modal**: Scroll up/down with the mouse wheel while the help dialog is open

### Changed

- **Merge-based refresh**: After a background refresh, new entries are merged into the current list instead of replacing it, preserving read entries and scroll position
- **Help scroll clamping**: Help modal scroll is now bounded to the actual content height instead of allowing unbounded scrolling
- New dependency: `arboard` for cross-platform clipboard support

### Internal

- New `Action::EntriesMerged` for merge-based entry loading
- Extracted `fixup_entry_selection()` and `resort_entries()` from reducers to reduce duplication
- `handle_help_scroll()` shared between keyboard and mouse input
- `help_max_scroll` field in `AppState` computed during render
- New i18n keys: `copied_to_clipboard`, `copy_failed`, `help_copy_preview`, `help_clear_read`, `help_reload_entries` (en + ca)
- New tests: clipboard copy, clear read entries, merge entries, resort, help scroll clamping, help mouse scroll

## [1.2.0] - 2026-03-11

### Added

- **Context-aware panel search**: `/` now searches differently depending on the focused panel:
  - **Feeds**: Filter feeds by name in real-time (case-insensitive)
  - **Entries**: Full-text search against the database
  - **Preview**: Highlight matching text with `n`/`N` navigation between matches
- **Refresh selected feed from entries**: Press `r` in the Entries panel to refresh only the current feed
- **Inline search bar**: Search input is now rendered at the bottom of each panel instead of a modal dialog
- **Match counter in preview**: Shows current/total matches (e.g. `[2/5]`) during preview search

### Changed

- `Esc` behavior is now layered: first clears the active search on the current panel, then navigates back
- Search is automatically cleared when switching feeds
- Help modal updated with `n`/`N` keybinding for next/previous match
- **Status bar layout**: Refreshing spinner and notifications now appear before the unread count on the right side, with consistent separators

### Internal

- `InputMode::Search` replaced by `InputMode::PanelSearch` with `panel_search_focus` tracking which panel initiated the search
- New state fields: `feed_filter_query`, `preview_search_query`, `preview_match_lines`, `preview_match_current`
- `adjust_unread_count` optimises read/unread toggling by updating counters locally instead of dispatching `RefreshUnreadCounts` to the database
- `highlight_spans` utility handles case-insensitive matching with correct multi-byte Unicode offset mapping
- `element_link_url` extracted as shared helper, removing duplication between `extract_links` and `convert_line`
- Removed unused i18n keys: `search_prompt`, `search_title`, `query_label`
- New tests covering panel search, match navigation, Esc layering, highlight spans, and unread count adjustment

## [1.1.0] - 2026-03-10

### Added

- **Feed selection and visibility toggle**: Select/deselect entries with `Space` in the entries panel, with checkbox indicators
- **Batch operations on selected entries**: Mark read/unread (`m`) and save/unsave (`s`) multiple selected entries at once
- **Hide read feeds**: Toggle with `.` to hide feeds with no unread entries; groups with no unread feeds are also hidden. Preference is persisted in config
- **Feed info modal**: Press `i` to show feed title and URL in a modal dialog
- **Auto-scroll to selected link**: Tab/Shift+Tab link navigation now scrolls the preview to keep the selected link visible

### Changed

- Default theme set to `terminal`
- Increased fetch concurrency for faster feed refresh
- `Esc` in entries panel now clears selection before switching focus to feeds
- `flush_feed_rows` returns whether the selected feed changed, allowing automatic entry reload

### Fixed

- Preview link navigation (`Tab`/`Shift+Tab`) now scrolls to keep the selected link visible

### Internal

- New DB commands: `MarkAllUnread`, `MarkAllSaved`, `MarkAllUnsaved` with corresponding repo methods
- `AppState` tracks `selected_entries: HashSet<i64>` and `hide_read_feeds: bool`
- `rebuild_feed_rows` respects `hide_read_feeds` filter, hiding empty feeds and groups
- `update_feed_row_counts` optimises unread counter updates without full row rebuild
- New `InputMode::FeedInfo` variant and `Modal::FeedInfo` for the feed info dialog
- Added i18n keys for feed info, hide/show read feeds, and select entry help text (en + ca)
- Theme support for `selected_entry` style across all theme files
- New tests: `hide_read_feeds_filters_rows`, `hide_read_feeds_hides_empty_groups`, `feed_info_esc_closes`, `feed_info_ignores_other_keys`

## [1.0.1] - 2026-03-09

### Added

- **Simple theme**: New minimal theme option

### Changed

- Package name changed to `rivulet-reader`

### Fixed

- Remove unknown name on Linux release workflow

## [1.0.0] - 2026-03-09

### Added

- **3-panel TUI layout**: Feeds, Entries, and Preview panels with keyboard and mouse navigation
- **Feed management**: Add, rename, and delete RSS/Atom feeds
- **Feed auto-discovery**: Automatically detect feed URLs from website URLs
- **Feed categories/groups**: Organize feeds by topic with collapsible sections
- **OPML import/export**: Import and export feed subscriptions in OPML format
- **Rich preview**: HTML-to-text rendering with bold, italic, links, and code styles
- **Incremental search**: Real-time filtering while typing
- **Entry filters**: Filter by unread, saved, and time period
- **Sort modes**: Multiple sort options for entries
- **Entry management**: Mark read/unread, save/unsave, mark all read, mark feed read
- **Open in browser**: Open entries and links in the default browser
- **Resizable panels**: `H`/`L` keys to grow/shrink the focused panel
- **Toggle layout**: Switch between different panel layout configurations
- **Mouse support**: Click to select feeds/entries, scroll preview
- **Refresh animation**: Braille spinner in status bar during feed refresh
- **Entry counter**: "ENTRIES (12/45)" showing filtered/total count
- **Help modal**: Two-column help with sections (Navigation/Feeds/Entries/General)
- **Theming system**: Customizable themes via TOML files (catppuccin-mocha, catppuccin-latte, gruvbox, gruvbox-light, kanagawa, solarized-light)
- **Internationalization (i18n)**: English and Catalan support via TOML locale files
- **Configuration**: `~/.config/rivulet/config.toml` for language, refresh interval, theme, and layout
- **SQLite storage**: Local database with robust migrations
- **Background scheduler**: Automatic feed refresh at configurable intervals
- **Release workflow**: GitHub Actions CI/CD, Homebrew formula, and AUR package
- **License**: GNU GPL v3
