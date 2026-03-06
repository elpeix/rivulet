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
            }
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
        KeyCode::PageUp => {
            let _ = app.dispatch(Action::PageUp);
        }
        KeyCode::PageDown => {
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
        KeyCode::Char(' ') => {
            if app.state.focus == Focus::Feeds {
                if let Some(row_idx) = app.state.selected_feed_row_index {
                    if let Some(FeedRow::GroupHeader { group_id, .. }) =
                        app.state.feed_rows.get(row_idx)
                    {
                        let gid = *group_id;
                        let _ = app.dispatch(Action::ToggleGroupCollapse(gid));
                    }
                }
            }
        }
        KeyCode::Char('r') => {
            let _ = app.dispatch(Action::RefreshFeeds);
        }
        KeyCode::Char('f') => {
            let _ = app.dispatch(Action::ToggleUnreadFilter);
            dispatch_load_entries(app);
        }
        KeyCode::Char('g') => {
            let _ = app.dispatch(Action::ToggleSavedFilter);
            dispatch_load_entries(app);
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
            if let Some(entry_id) = app.state.selected_entry {
                if let Some(idx) = app.state.entry_position(entry_id) {
                    if app.state.entries[idx].read_at.is_none() {
                        let timestamp = now_timestamp();
                        app.state.entries[idx].read_at = Some(timestamp);
                        let _ = app.dispatch(Action::MarkRead(entry_id));
                    } else {
                        app.state.entries[idx].read_at = None;
                        let _ = app.dispatch(Action::MarkUnread(entry_id));
                    }
                }
                let _ = app.dispatch(Action::RefreshUnreadCounts);
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
                let _ = app.dispatch(Action::MarkAllRead(unread_ids));
                let _ = app.dispatch(Action::RefreshUnreadCounts);
            }
        }
        KeyCode::Char('S') => {
            app.state.sort_mode = app.state.sort_mode.next();
            let label = match app.state.sort_mode {
                crate::app::state::SortMode::DateDesc => app.lang.sort_date_desc,
                crate::app::state::SortMode::DateAsc => app.lang.sort_date_asc,
                crate::app::state::SortMode::TitleAsc => app.lang.sort_title_asc,
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
        KeyCode::Char('o') => {
            open_selected_link_or_entry(app);
        }
        KeyCode::Char('s') => {
            if let Some(entry_id) = app.state.selected_entry {
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
            app.state.input_mode = InputMode::Search;
            app.state.input_buffer.clear();
            let _ = app.dispatch(Action::SetStatus(app.lang.search_prompt.to_string()));
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
        KeyCode::Esc => match app.state.focus {
            Focus::Preview => {
                if app.state.selected_link_index.is_some() {
                    app.state.selected_link_index = None;
                } else {
                    let _ = app.dispatch(Action::FocusEntries);
                }
            }
            Focus::Entries => {
                let _ = app.dispatch(Action::FocusFeeds);
            }
            Focus::Feeds => {
                let _ = app.dispatch(Action::ClearStatus);
            }
        },
        _ => {}
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::{Focus, InputMode};
    use crate::app::tests::test_app;
    use crate::store::models::Feed;
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
        assert_eq!(app.state.input_mode, InputMode::Search);
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
