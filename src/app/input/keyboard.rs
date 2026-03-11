use crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;
use crate::app::actions::Action;
use crate::app::state::{FeedRow, Focus, InputMode};
use crate::util::open::open_url;
use crate::util::time::now_timestamp;

use super::dispatch_load_entries;

pub fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('q') => return true,
        KeyCode::Char('?') => {
            app.state.show_help = !app.state.show_help;
        }
        KeyCode::Tab => {
            if app.state.focus == Focus::Preview && !app.state.preview_links.is_empty() {
                let next = match app.state.selected_link_index {
                    Some(i) if i + 1 < app.state.preview_links.len() => i + 1,
                    Some(_) => 0,
                    None => 0,
                };
                app.state.selected_link_index = Some(next);
                scroll_to_selected_link(&mut app.state);
            }
        }
        KeyCode::BackTab => {
            if app.state.focus == Focus::Preview && !app.state.preview_links.is_empty() {
                let prev = match app.state.selected_link_index {
                    Some(0) => app.state.preview_links.len() - 1,
                    Some(i) => i - 1,
                    None => app.state.preview_links.len() - 1,
                };
                app.state.selected_link_index = Some(prev);
                scroll_to_selected_link(&mut app.state);
            }
        }
        KeyCode::Char('1') => {
            let _ = app.dispatch(Action::FocusFeeds);
        }
        KeyCode::Char('2') => {
            let _ = app.dispatch(Action::FocusEntries);
        }
        KeyCode::Char('3') => {
            let _ = app.dispatch(Action::FocusPreview);
        }
        KeyCode::Left | KeyCode::Char('h') => match app.state.focus {
            Focus::Preview => {
                app.state.selected_link_index = None;
                let _ = app.dispatch(Action::FocusEntries);
            }
            Focus::Entries => {
                let _ = app.dispatch(Action::FocusFeeds);
            }
            Focus::Feeds => {}
        },
        KeyCode::Right | KeyCode::Char('l') => match app.state.focus {
            Focus::Feeds => {
                let _ = app.dispatch(Action::FocusEntries);
            }
            Focus::Entries => {
                let _ = app.dispatch(Action::FocusPreview);
            }
            Focus::Preview => {}
        },
        KeyCode::Char('H') => {
            let _ = app.dispatch(Action::ResizePanel(-1));
        }
        KeyCode::Char('L') => {
            let _ = app.dispatch(Action::ResizePanel(1));
        }
        KeyCode::Char('w') => {
            let _ = app.dispatch(Action::ToggleLayout);
            crate::config::Config::save_layout(app.state.layout_mode);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let _ = app.dispatch(Action::MoveUp);
            if app.state.focus == Focus::Feeds {
                dispatch_load_entries(app);
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let _ = app.dispatch(Action::MoveDown);
            if app.state.focus == Focus::Feeds {
                dispatch_load_entries(app);
            }
        }
        KeyCode::PageUp | KeyCode::Char('K') => {
            let _ = app.dispatch(Action::PageUp);
        }
        KeyCode::PageDown | KeyCode::Char('J') => {
            let _ = app.dispatch(Action::PageDown);
        }
        KeyCode::Home => {
            let _ = app.dispatch(Action::ScrollTop);
        }
        KeyCode::End => {
            let _ = app.dispatch(Action::ScrollBottom);
        }
        KeyCode::Enter => match app.state.focus {
            Focus::Feeds => {
                dispatch_load_entries(app);
                let _ = app.dispatch(Action::FocusEntries);
            }
            Focus::Entries => {
                if app.state.selected_entry.is_some() {
                    let _ = app.dispatch(Action::FocusPreview);
                }
            }
            Focus::Preview => {
                open_selected_link_or_entry(app);
            }
        },
        KeyCode::Char(' ') => match app.state.focus {
            Focus::Feeds => {
                if let Some(row_idx) = app.state.selected_feed_row_index {
                    if let Some(FeedRow::GroupHeader { group_id, .. }) =
                        app.state.feed_rows.get(row_idx)
                    {
                        let gid = *group_id;
                        let _ = app.dispatch(Action::ToggleGroupCollapse(gid));
                    }
                }
            }
            Focus::Entries => {
                if let Some(entry_id) = app.state.selected_entry {
                    if !app.state.selected_entries.remove(&entry_id) {
                        app.state.selected_entries.insert(entry_id);
                    }
                }
            }
            Focus::Preview => {}
        },
        KeyCode::Char('r') => {
            let action = app.state.contextual_refresh_action();
            let _ = app.dispatch(action);
        }
        KeyCode::Char('f') => {
            let _ = app.dispatch(Action::ToggleUnreadFilter);
            dispatch_load_entries(app);
        }
        KeyCode::Char('g') => {
            let _ = app.dispatch(Action::ToggleSavedFilter);
            dispatch_load_entries(app);
        }
        KeyCode::Char('.') => {
            app.state.hide_read_feeds = !app.state.hide_read_feeds;
            app.state.rebuild_feed_rows();
            crate::config::Config::save_hide_read_feeds(app.state.hide_read_feeds);
            let msg = if app.state.hide_read_feeds {
                app.lang.filter_hide_read.to_string()
            } else {
                app.lang.filter_show_read.to_string()
            };
            let _ = app.dispatch(Action::SetStatus(msg));
        }
        KeyCode::Char('t') => {
            app.state.recent_only = !app.state.recent_only;
            let msg = if app.state.recent_only {
                app.lang.filter_recent_days(app.recent_days)
            } else {
                app.lang.filter_all_time.to_string()
            };
            let _ = app.dispatch(Action::SetStatus(msg));
            dispatch_load_entries(app);
        }
        KeyCode::Char('m') => {
            if !app.state.selected_entries.is_empty() {
                let ids: Vec<i64> = app.state.selected_entries.iter().copied().collect();
                let timestamp = now_timestamp();
                let any_unread = ids.iter().any(|id| {
                    app.state
                        .entry_position(*id)
                        .map(|i| app.state.entries[i].read_at.is_none())
                        .unwrap_or(false)
                });
                if any_unread {
                    for &id in &ids {
                        if let Some(idx) = app.state.entry_position(id) {
                            if app.state.entries[idx].read_at.is_none() {
                                let feed_id = app.state.entries[idx].feed_id;
                                app.state.entries[idx].read_at = Some(timestamp);
                                app.state.adjust_unread_count(feed_id, -1);
                            }
                        }
                    }
                    let _ = app.dispatch(Action::MarkAllRead(ids));
                } else {
                    for &id in &ids {
                        if let Some(idx) = app.state.entry_position(id) {
                            let feed_id = app.state.entries[idx].feed_id;
                            app.state.entries[idx].read_at = None;
                            app.state.adjust_unread_count(feed_id, 1);
                        }
                    }
                    let _ = app.dispatch(Action::MarkAllUnread(ids));
                }
                app.state.selected_entries.clear();
            } else if let Some(entry_id) = app.state.selected_entry {
                if let Some(idx) = app.state.entry_position(entry_id) {
                    let feed_id = app.state.entries[idx].feed_id;
                    if app.state.entries[idx].read_at.is_none() {
                        app.state.entries[idx].read_at = Some(now_timestamp());
                        let _ = app.dispatch(Action::MarkRead(entry_id));
                        app.state.adjust_unread_count(feed_id, -1);
                    } else {
                        app.state.entries[idx].read_at = None;
                        let _ = app.dispatch(Action::MarkUnread(entry_id));
                        app.state.adjust_unread_count(feed_id, 1);
                    }
                }
            }
        }
        KeyCode::Char('M') => {
            let unread_ids: Vec<i64> = app
                .state
                .entries
                .iter()
                .filter(|e| e.read_at.is_none())
                .map(|e| e.id)
                .collect();
            if !unread_ids.is_empty() {
                let timestamp = now_timestamp();
                for entry in &mut app.state.entries {
                    if entry.read_at.is_none() {
                        entry.read_at = Some(timestamp);
                    }
                }
                // Adjust counts once per feed instead of per entry
                let mut deltas: std::collections::HashMap<i64, i64> =
                    std::collections::HashMap::new();
                for id in &unread_ids {
                    if let Some(idx) = app.state.entry_position(*id) {
                        *deltas.entry(app.state.entries[idx].feed_id).or_default() += 1;
                    }
                }
                for (feed_id, count) in deltas {
                    app.state.adjust_unread_count(feed_id, -count);
                }
                let _ = app.dispatch(Action::MarkAllRead(unread_ids));
            }
        }
        KeyCode::Char('S') => {
            app.state.sort_mode = app.state.sort_mode.next();
            let label = match app.state.sort_mode {
                crate::app::state::SortMode::DateDesc => &app.lang.sort_date_desc,
                crate::app::state::SortMode::DateAsc => &app.lang.sort_date_asc,
                crate::app::state::SortMode::TitleAsc => &app.lang.sort_title_asc,
            };
            let _ = app.dispatch(Action::SetStatus(format!(
                "{}: {}",
                app.lang.sort_label, label
            )));
            dispatch_load_entries(app);
        }
        KeyCode::Char('R') => {
            if app.state.focus == Focus::Feeds {
                if let Some(feed_id) = app.state.selected_feed {
                    let _ = app.dispatch(Action::MarkFeedRead(feed_id));
                    let timestamp = now_timestamp();
                    for entry in &mut app.state.entries {
                        if entry.feed_id == feed_id && entry.read_at.is_none() {
                            entry.read_at = Some(timestamp);
                        }
                    }
                    let _ = app.dispatch(Action::RefreshUnreadCounts);
                    dispatch_load_entries(app);
                }
            }
        }
        KeyCode::Char('n') => {
            if app.state.focus == Focus::Preview && !app.state.preview_match_lines.is_empty() {
                let next = match app.state.preview_match_current {
                    Some(i) if i + 1 < app.state.preview_match_lines.len() => i + 1,
                    _ => 0,
                };
                app.state.preview_match_current = Some(next);
                if let Some(&line) = app.state.preview_match_lines.get(next) {
                    app.state.preview_scroll = line as u16;
                }
            }
        }
        KeyCode::Char('N') => {
            if app.state.focus == Focus::Preview && !app.state.preview_match_lines.is_empty() {
                let prev = match app.state.preview_match_current {
                    Some(0) | None => app.state.preview_match_lines.len() - 1,
                    Some(i) => i - 1,
                };
                app.state.preview_match_current = Some(prev);
                if let Some(&line) = app.state.preview_match_lines.get(prev) {
                    app.state.preview_scroll = line as u16;
                }
            }
        }
        KeyCode::Char('o') => {
            open_selected_link_or_entry(app);
        }
        KeyCode::Char('s') => {
            if !app.state.selected_entries.is_empty() {
                let ids: Vec<i64> = app.state.selected_entries.iter().copied().collect();
                let timestamp = now_timestamp();
                let any_unsaved = ids.iter().any(|id| {
                    app.state
                        .entry_position(*id)
                        .map(|i| app.state.entries[i].saved_at.is_none())
                        .unwrap_or(false)
                });
                if any_unsaved {
                    for &id in &ids {
                        if let Some(idx) = app.state.entry_position(id) {
                            if app.state.entries[idx].saved_at.is_none() {
                                app.state.entries[idx].saved_at = Some(timestamp);
                            }
                        }
                    }
                    let _ = app.dispatch(Action::MarkAllSaved(ids));
                } else {
                    for &id in &ids {
                        if let Some(idx) = app.state.entry_position(id) {
                            app.state.entries[idx].saved_at = None;
                        }
                    }
                    let _ = app.dispatch(Action::MarkAllUnsaved(ids));
                }
                app.state.selected_entries.clear();
                let _ = app.dispatch(Action::RefreshUnreadCounts);
                dispatch_load_entries(app);
            } else if let Some(entry_id) = app.state.selected_entry {
                if let Some(idx) = app.state.entry_position(entry_id) {
                    if app.state.entries[idx].saved_at.is_some() {
                        app.state.entries[idx].saved_at = None;
                        let _ = app.dispatch(Action::MarkUnsaved(entry_id));
                    } else {
                        app.state.entries[idx].saved_at = Some(now_timestamp());
                        let _ = app.dispatch(Action::MarkSaved(entry_id));
                    }
                }
                let _ = app.dispatch(Action::RefreshUnreadCounts);
                dispatch_load_entries(app);
            }
        }
        KeyCode::Char('/') => {
            app.state.input_mode = InputMode::PanelSearch;
            app.state.input_buffer.clear();
            app.state.panel_search_focus = Some(app.state.focus);
        }
        KeyCode::Char('a') => {
            app.state.input_mode = InputMode::AddFeed;
            app.state.input_buffer.clear();
            let _ = app.dispatch(Action::SetStatus(app.lang.add_feed_prompt.to_string()));
        }
        KeyCode::Char('c') => {
            if app.state.selected_feed.is_some() {
                app.state.input_mode = InputMode::AssignGroup;
                app.state.input_buffer.clear();
                let _ = app.dispatch(Action::SetStatus(app.lang.assign_group_prompt.to_string()));
            }
        }
        KeyCode::Char('C') => {
            app.state.input_mode = InputMode::ManageGroups;
            app.state.input_buffer.clear();
            let _ = app.dispatch(Action::SetStatus(app.lang.group_manage_hint.to_string()));
        }
        KeyCode::Char('i') => {
            if app.state.selected_feed.is_some() {
                app.state.input_mode = InputMode::FeedInfo;
            } else {
                let _ = app.dispatch(Action::SetStatus(app.lang.no_feed_selected.to_string()));
            }
        }
        KeyCode::Char('e') => {
            if let Some(feed_id) = app.state.selected_feed {
                if let Some(feed) = app.state.feeds.iter().find(|f| f.id == feed_id) {
                    app.state.input_mode = InputMode::RenameFeed;
                    app.state.input_buffer.clear();
                    if let Some(title) = feed.custom_title.as_deref() {
                        app.state.input_buffer.push_str(title);
                    }
                }
            } else {
                let _ = app.dispatch(Action::SetStatus(app.lang.no_feed_selected.to_string()));
            }
        }
        KeyCode::Char('d') => {
            if app.state.selected_feed.is_some() {
                app.state.input_mode = InputMode::DeleteFeed;
                app.state.input_buffer.clear();
                let _ = app.dispatch(Action::SetStatus(app.lang.delete_feed_confirm.to_string()));
            } else {
                let _ = app.dispatch(Action::SetStatus(app.lang.no_feed_selected.to_string()));
            }
        }
        KeyCode::Esc => {
            // Clear the search active on the current panel, if any
            let cleared = match app.state.focus {
                Focus::Feeds if app.state.feed_filter_query.is_some() => {
                    app.state.feed_filter_query = None;
                    app.state.rebuild_feed_rows();
                    true
                }
                Focus::Entries if app.state.search_query.is_some() => {
                    let _ = app.dispatch(Action::SetSearchQuery(String::new()));
                    true
                }
                Focus::Preview if app.state.preview_search_query.is_some() => {
                    app.state.preview_search_query = None;
                    app.state.preview_match_lines.clear();
                    app.state.preview_match_current = None;
                    true
                }
                _ => false,
            };
            if !cleared {
                match app.state.focus {
                    Focus::Preview => {
                        if app.state.selected_link_index.is_some() {
                            app.state.selected_link_index = None;
                        } else {
                            let _ = app.dispatch(Action::FocusEntries);
                        }
                    }
                    Focus::Entries => {
                        if !app.state.selected_entries.is_empty() {
                            app.state.selected_entries.clear();
                        } else {
                            let _ = app.dispatch(Action::FocusFeeds);
                        }
                    }
                    Focus::Feeds => {
                        let _ = app.dispatch(Action::ClearStatus);
                    }
                }
            }
        }
        _ => {}
    }

    false
}

fn scroll_to_selected_link(state: &mut crate::app::state::AppState) {
    let url = match state
        .selected_link_index
        .and_then(|i| state.preview_links.get(i))
    {
        Some(u) => u.clone(),
        None => return,
    };
    // Find the first link region matching the selected URL
    if let Some(region) = state.preview_link_regions.iter().find(|r| r.url == url) {
        let line = (region.line).min(u16::MAX as usize) as u16;
        let visible_height = state.preview_body_area.height;
        let scroll = state.preview_scroll;
        if line < scroll {
            state.preview_scroll = line;
        } else if line >= scroll + visible_height {
            state.preview_scroll = line.saturating_sub(visible_height / 2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::{Focus, InputMode};
    use crate::app::tests::test_app;
    use crate::store::models::{Entry, Feed};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn q_returns_quit() {
        let mut app = test_app();
        let quit = handle_key(&mut app, key(KeyCode::Char('q')));
        assert!(quit);
    }

    #[test]
    fn question_mark_toggles_help() {
        let mut app = test_app();
        assert!(!app.state.show_help);
        handle_key(&mut app, key(KeyCode::Char('?')));
        assert!(app.state.show_help);
        handle_key(&mut app, key(KeyCode::Char('?')));
        assert!(!app.state.show_help);
    }

    #[test]
    fn slash_enters_search_mode() {
        let mut app = test_app();
        handle_key(&mut app, key(KeyCode::Char('/')));
        assert_eq!(app.state.input_mode, InputMode::PanelSearch);
    }

    #[test]
    fn a_enters_add_feed_mode() {
        let mut app = test_app();
        handle_key(&mut app, key(KeyCode::Char('a')));
        assert_eq!(app.state.input_mode, InputMode::AddFeed);
    }

    #[test]
    fn d_without_feed_shows_error() {
        let mut app = test_app();
        app.state.selected_feed = None;
        handle_key(&mut app, key(KeyCode::Char('d')));
        assert_eq!(app.state.input_mode, InputMode::None);
        let status = app.state.status.as_ref().expect("status should be set");
        assert!(status.message.contains(&app.lang.no_feed_selected));
    }

    #[test]
    fn d_with_feed_enters_delete_mode() {
        let mut app = test_app();
        app.state.selected_feed = Some(1);
        handle_key(&mut app, key(KeyCode::Char('d')));
        assert_eq!(app.state.input_mode, InputMode::DeleteFeed);
    }

    #[test]
    fn esc_from_preview_to_entries() {
        let mut app = test_app();
        app.state.focus = Focus::Preview;
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.state.focus, Focus::Entries);
    }

    #[test]
    fn esc_from_entries_to_feeds() {
        let mut app = test_app();
        app.state.focus = Focus::Entries;
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.state.focus, Focus::Feeds);
    }

    #[test]
    fn f_toggles_unread_filter() {
        let mut app = test_app();
        let before = app.state.unread_only;
        handle_key(&mut app, key(KeyCode::Char('f')));
        assert_ne!(app.state.unread_only, before);
        handle_key(&mut app, key(KeyCode::Char('f')));
        assert_eq!(app.state.unread_only, before);
    }

    #[test]
    fn navigation_up_down() {
        let mut app = test_app();
        app.state.feeds = vec![
            Feed {
                id: 1,
                title: Some("A".to_string()),
                custom_title: None,
                url: "https://a.com".to_string(),
                etag: None,
                last_modified: None,
                last_checked_at: None,
                group_id: None,
            },
            Feed {
                id: 2,
                title: Some("B".to_string()),
                custom_title: None,
                url: "https://b.com".to_string(),
                etag: None,
                last_modified: None,
                last_checked_at: None,
                group_id: None,
            },
        ];
        app.state.rebuild_feed_rows();
        app.state.selected_feed_row_index = Some(0);
        app.state.focus = Focus::Feeds;

        handle_key(&mut app, key(KeyCode::Down));
        assert_eq!(app.state.selected_feed_row_index, Some(1));

        handle_key(&mut app, key(KeyCode::Up));
        assert_eq!(app.state.selected_feed_row_index, Some(0));
    }

    #[test]
    fn shift_j_scrolls_preview_down() {
        let mut app = test_app();
        app.state.preview_content_len = 20;
        app.state.preview_scroll = 0;
        handle_key(&mut app, key(KeyCode::Char('J')));
        assert!(app.state.preview_scroll > 0);
    }

    #[test]
    fn shift_k_scrolls_preview_up() {
        let mut app = test_app();
        app.state.preview_content_len = 20;
        app.state.preview_scroll = 10;
        handle_key(&mut app, key(KeyCode::Char('K')));
        assert!(app.state.preview_scroll < 10);
    }

    fn sample_entry(id: i64) -> Entry {
        Entry {
            id,
            feed_id: 1,
            title: Some(format!("Entry {id}")),
            url: None,
            author: None,
            published_at: None,
            fetched_at: 0,
            summary: None,
            content: None,
            read_at: None,
            saved_at: None,
        }
    }

    fn app_with_entries() -> crate::app::App {
        let mut app = test_app();
        app.state.entries = vec![sample_entry(10), sample_entry(11), sample_entry(12)];
        app.state.selected_entry = Some(10);
        app.state.selected_entry_index = Some(0);
        app.state.focus = Focus::Entries;
        app
    }

    #[test]
    fn space_toggles_entry_selection() {
        let mut app = app_with_entries();
        assert!(app.state.selected_entries.is_empty());

        handle_key(&mut app, key(KeyCode::Char(' ')));
        assert!(app.state.selected_entries.contains(&10));

        // Toggle off
        handle_key(&mut app, key(KeyCode::Char(' ')));
        assert!(!app.state.selected_entries.contains(&10));
    }

    #[test]
    fn space_selects_multiple_entries() {
        let mut app = app_with_entries();

        // Select first entry
        handle_key(&mut app, key(KeyCode::Char(' ')));
        assert!(app.state.selected_entries.contains(&10));

        // Move down and select second
        handle_key(&mut app, key(KeyCode::Down));
        handle_key(&mut app, key(KeyCode::Char(' ')));
        assert!(app.state.selected_entries.contains(&10));
        assert!(app.state.selected_entries.contains(&11));
        assert_eq!(app.state.selected_entries.len(), 2);
    }

    #[test]
    fn esc_clears_selection_before_changing_focus() {
        let mut app = app_with_entries();

        handle_key(&mut app, key(KeyCode::Char(' ')));
        assert!(!app.state.selected_entries.is_empty());

        // First Esc clears selection, stays in Entries
        handle_key(&mut app, key(KeyCode::Esc));
        assert!(app.state.selected_entries.is_empty());
        assert_eq!(app.state.focus, Focus::Entries);

        // Second Esc goes to Feeds
        handle_key(&mut app, key(KeyCode::Esc));
        assert_eq!(app.state.focus, Focus::Feeds);
    }

    #[test]
    fn i_with_feed_enters_feed_info_mode() {
        let mut app = test_app();
        app.state.selected_feed = Some(1);
        handle_key(&mut app, key(KeyCode::Char('i')));
        assert_eq!(app.state.input_mode, InputMode::FeedInfo);
    }

    #[test]
    fn i_without_feed_shows_error() {
        let mut app = test_app();
        app.state.selected_feed = None;
        handle_key(&mut app, key(KeyCode::Char('i')));
        assert_eq!(app.state.input_mode, InputMode::None);
        let status = app.state.status.as_ref().expect("status should be set");
        assert!(status.message.contains(&app.lang.no_feed_selected));
    }

    #[test]
    fn entries_loaded_clears_selection() {
        let mut app = app_with_entries();
        app.state.selected_entries.insert(10);
        app.state.selected_entries.insert(11);

        app.state
            .reduce(Action::EntriesLoaded(vec![sample_entry(20)]));
        assert!(app.state.selected_entries.is_empty());
    }

    #[test]
    fn slash_sets_panel_search_focus_to_current_panel() {
        let mut app = test_app();
        app.state.focus = Focus::Feeds;
        handle_key(&mut app, key(KeyCode::Char('/')));
        assert_eq!(app.state.input_mode, InputMode::PanelSearch);
        assert_eq!(app.state.panel_search_focus, Some(Focus::Feeds));

        app.state.input_mode = InputMode::None;
        app.state.focus = Focus::Entries;
        handle_key(&mut app, key(KeyCode::Char('/')));
        assert_eq!(app.state.panel_search_focus, Some(Focus::Entries));

        app.state.input_mode = InputMode::None;
        app.state.focus = Focus::Preview;
        handle_key(&mut app, key(KeyCode::Char('/')));
        assert_eq!(app.state.panel_search_focus, Some(Focus::Preview));
    }

    #[test]
    fn slash_clears_input_buffer() {
        let mut app = test_app();
        app.state.input_buffer = "old query".to_string();
        handle_key(&mut app, key(KeyCode::Char('/')));
        assert!(app.state.input_buffer.is_empty());
    }

    #[test]
    fn n_navigates_preview_matches_forward() {
        let mut app = test_app();
        app.state.focus = Focus::Preview;
        app.state.preview_match_lines = vec![5, 15, 25];
        app.state.preview_match_current = None;

        handle_key(&mut app, key(KeyCode::Char('n')));
        assert_eq!(app.state.preview_match_current, Some(0));
        assert_eq!(app.state.preview_scroll, 5);

        handle_key(&mut app, key(KeyCode::Char('n')));
        assert_eq!(app.state.preview_match_current, Some(1));
        assert_eq!(app.state.preview_scroll, 15);

        handle_key(&mut app, key(KeyCode::Char('n')));
        assert_eq!(app.state.preview_match_current, Some(2));
        assert_eq!(app.state.preview_scroll, 25);

        // Wraps around
        handle_key(&mut app, key(KeyCode::Char('n')));
        assert_eq!(app.state.preview_match_current, Some(0));
        assert_eq!(app.state.preview_scroll, 5);
    }

    #[test]
    fn n_reverse_navigates_preview_matches() {
        let mut app = test_app();
        app.state.focus = Focus::Preview;
        app.state.preview_match_lines = vec![5, 15, 25];
        app.state.preview_match_current = None;

        // N from None goes to last match
        handle_key(&mut app, key(KeyCode::Char('N')));
        assert_eq!(app.state.preview_match_current, Some(2));
        assert_eq!(app.state.preview_scroll, 25);

        handle_key(&mut app, key(KeyCode::Char('N')));
        assert_eq!(app.state.preview_match_current, Some(1));
        assert_eq!(app.state.preview_scroll, 15);
    }

    #[test]
    fn n_noop_without_matches() {
        let mut app = test_app();
        app.state.focus = Focus::Preview;
        app.state.preview_match_lines = vec![];
        handle_key(&mut app, key(KeyCode::Char('n')));
        assert_eq!(app.state.preview_match_current, None);
    }

    #[test]
    fn n_noop_outside_preview() {
        let mut app = test_app();
        app.state.focus = Focus::Entries;
        app.state.preview_match_lines = vec![5, 15];
        handle_key(&mut app, key(KeyCode::Char('n')));
        assert_eq!(app.state.preview_match_current, None);
    }

    #[test]
    fn esc_clears_feed_filter_search() {
        let mut app = test_app();
        app.state.feed_filter_query = Some("rust".to_string());
        app.state.focus = Focus::Feeds;
        handle_key(&mut app, key(KeyCode::Esc));
        assert!(app.state.feed_filter_query.is_none());
    }

    #[test]
    fn esc_clears_preview_search() {
        let mut app = test_app();
        app.state.preview_search_query = Some("test".to_string());
        app.state.preview_match_lines = vec![1, 5, 10];
        app.state.preview_match_current = Some(1);
        app.state.focus = Focus::Preview;
        handle_key(&mut app, key(KeyCode::Esc));
        assert!(app.state.preview_search_query.is_none());
        assert!(app.state.preview_match_lines.is_empty());
        assert!(app.state.preview_match_current.is_none());
    }

    #[test]
    fn esc_only_clears_search_of_current_panel() {
        let mut app = test_app();
        app.state.feed_filter_query = Some("query".to_string());
        app.state.preview_search_query = Some("test".to_string());
        app.state.focus = Focus::Preview;
        // Esc on Preview clears preview search only, not feed filter
        handle_key(&mut app, key(KeyCode::Esc));
        assert!(app.state.preview_search_query.is_none());
        assert_eq!(app.state.feed_filter_query.as_deref(), Some("query"));
        assert_eq!(app.state.focus, Focus::Preview); // still on Preview
    }

    #[test]
    fn esc_navigates_when_no_search_active_on_panel() {
        let mut app = test_app();
        // Feed filter active but we're on Preview with no preview search
        app.state.feed_filter_query = Some("query".to_string());
        app.state.focus = Focus::Preview;
        app.state.preview_search_query = None;
        handle_key(&mut app, key(KeyCode::Esc));
        // No preview search to clear → navigates back to Entries
        assert_eq!(app.state.focus, Focus::Entries);
        // Feed filter stays untouched
        assert_eq!(app.state.feed_filter_query.as_deref(), Some("query"));
    }
}

fn open_selected_link_or_entry(app: &mut App) {
    let link_url = app
        .state
        .selected_link_index
        .and_then(|i| app.state.preview_links.get(i).cloned());
    if let Some(url) = link_url {
        match open_url(&url) {
            Ok(()) => {
                let _ = app.dispatch(Action::SetStatus(app.lang.opened_in_browser.to_string()));
            }
            Err(error) => {
                let _ = app.dispatch(Action::DbError(error));
            }
        }
    } else if let Some(entry_id) = app.state.selected_entry {
        if let Some(entry) = app
            .state
            .entry_position(entry_id)
            .and_then(|i| app.state.entries.get(i))
        {
            if let Some(url) = entry.url.as_deref().filter(|v| !v.trim().is_empty()) {
                match open_url(url) {
                    Ok(()) => {
                        let _ =
                            app.dispatch(Action::SetStatus(app.lang.opened_in_browser.to_string()));
                    }
                    Err(error) => {
                        let _ = app.dispatch(Action::DbError(error));
                    }
                }
            } else {
                let _ = app.dispatch(Action::SetStatus(app.lang.entry_has_no_url.to_string()));
            }
        }
    } else {
        let _ = app.dispatch(Action::SetStatus(app.lang.no_entry_selected.to_string()));
    }
}
