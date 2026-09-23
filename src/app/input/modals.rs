use crossterm::event::{KeyCode, KeyEvent};

use crate::app::App;
use crate::app::actions::Action;
use crate::app::state::{AppState, Focus, InputMode, StatusKind};
use crate::i18n::Lang;
use crate::ui;

use super::keyboard::mark_all_visible_read;

const MAX_GROUP_NAME_LEN: usize = 64;

pub fn handle_help_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('?' | 'q') => {
            app.state.show_help = false;
            app.state.help_scroll = 0;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.state.help_scroll = app
                .state
                .help_scroll
                .saturating_add(1)
                .min(app.state.help_max_scroll);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.state.help_scroll = app.state.help_scroll.saturating_sub(1);
        }
        KeyCode::PageDown | KeyCode::Char('J') => {
            app.state.help_scroll = app
                .state
                .help_scroll
                .saturating_add(10)
                .min(app.state.help_max_scroll);
        }
        KeyCode::PageUp | KeyCode::Char('K') => {
            app.state.help_scroll = app.state.help_scroll.saturating_sub(10);
        }
        KeyCode::Home | KeyCode::Char('g') => {
            app.state.help_scroll = 0;
        }
        KeyCode::End | KeyCode::Char('G') => {
            app.state.help_scroll = app.state.help_max_scroll;
        }
        _ => {}
    }
}

pub fn handle_help_scroll(app: &mut App, delta: i16) {
    if delta > 0 {
        app.state.help_scroll = app
            .state
            .help_scroll
            .saturating_add(delta as u16)
            .min(app.state.help_max_scroll);
    } else {
        app.state.help_scroll = app.state.help_scroll.saturating_sub(delta.unsigned_abs());
    }
}

pub fn handle_input_mode(app: &mut App, key: KeyEvent) -> bool {
    let mode = app.state.input_mode.clone();

    // Handle modal-based input modes first
    match mode {
        InputMode::AddFeedGroup { .. } => {
            return handle_add_feed_group(app, key);
        }
        InputMode::AssignGroup => {
            return handle_assign_group(app, key);
        }
        InputMode::ManageGroups => {
            return handle_manage_groups(app, key);
        }
        InputMode::AddGroup | InputMode::RenameGroup => {
            return handle_group_text_input(app, key);
        }
        InputMode::DeleteGroup { group_id } => {
            return handle_delete_group(app, key, group_id);
        }
        InputMode::SelectDiscoveredFeed { feeds, group_id } => {
            return handle_select_discovered_feed(app, key, &feeds, group_id);
        }
        InputMode::FeedInfo => {
            match key.code {
                KeyCode::Esc => {
                    app.state.input_mode = InputMode::None;
                    let _ = app.dispatch(Action::ClearStatus);
                }
                KeyCode::Char('u') => {
                    if let Some(url) = app.state.selected_feed_ref().map(|f| f.url.clone()) {
                        app.state.input_mode = InputMode::EditFeedUrl;
                        app.state.input_buffer.set(url);
                    }
                }
                _ => {}
            }
            return false;
        }
        InputMode::EditFeedUrl => {
            return handle_edit_feed_url(app, key);
        }
        InputMode::MarkAllRead { .. } => {
            if matches!(key.code, KeyCode::Char('y' | 'Y' | 's' | 'S')) {
                mark_all_visible_read(app);
            }
            let _ = app.dispatch(Action::ClearStatus);
            return true;
        }
        InputMode::Discovering => {
            if key.code == KeyCode::Esc {
                app.cancel_discovery();
                let _ = app.dispatch(Action::ClearStatus);
                return true;
            }
            return false;
        }
        _ => {}
    }

    match key.code {
        KeyCode::Esc => {
            if mode == InputMode::PanelSearch {
                match app.state.panel_search_focus {
                    Some(Focus::Feeds) => {
                        app.state.feed_filter_query = None;
                        app.state.rebuild_feed_rows();
                    }
                    Some(Focus::Entries) => {
                        let _ = app.dispatch(Action::SetSearchQuery(String::new()));
                    }
                    Some(Focus::Preview) => {
                        app.state.preview_search_query = None;
                        app.state.preview_match_lines.clear();
                        app.state.preview_match_current = None;
                    }
                    None => {}
                }
                app.state.panel_search_focus = None;
            }
            let _ = app.dispatch(Action::ClearStatus);
            return true;
        }
        KeyCode::Char('y' | 'Y' | 's' | 'S') if mode == InputMode::DeleteFeed => {
            if let Some(feed_id) = app.state.selected_feed {
                let _ = app.dispatch(Action::DeleteFeed(feed_id));
            }
            let _ = app.dispatch(Action::ClearStatus);
            return true;
        }
        KeyCode::Char('n' | 'N') if mode == InputMode::DeleteFeed => {
            let _ = app.dispatch(Action::ClearStatus);
            return true;
        }
        KeyCode::Enter => {
            let value = app.state.input_buffer.trim().to_string();
            match &mode {
                InputMode::PanelSearch => {
                    // Keep filter active, just close the search bar
                    app.state.panel_search_focus = None;
                }
                InputMode::RenameFeed => {
                    if let Some(feed_id) = app.state.selected_feed {
                        let custom = if value.is_empty() { None } else { Some(value) };
                        let _ = app.dispatch(Action::RenameFeed {
                            id: feed_id,
                            title: custom,
                        });
                    }
                }
                InputMode::AddFeed if !value.is_empty() => {
                    if app.state.groups.is_empty() {
                        let _ = app.dispatch(Action::AddFeed {
                            title: None,
                            url: value,
                            group_id: None,
                        });
                    } else {
                        app.state.input_mode = InputMode::AddFeedGroup { url: value };
                        app.state.modal_selection = 0;
                        return false;
                    }
                }
                InputMode::DeleteFeed => {
                    let _ = app.dispatch(Action::ClearStatus);
                    return true;
                }
                _ => {}
            }
            let _ = app.dispatch(Action::ClearStatus);
            return true;
        }
        _ => {
            if !app.state.input_buffer.handle_key(key) {
                return false;
            }
        }
    }

    match &mode {
        InputMode::PanelSearch => {
            let query = app.state.input_buffer.trim().to_string();
            match app.state.panel_search_focus {
                Some(Focus::Feeds) => {
                    app.state.feed_filter_query = if query.is_empty() { None } else { Some(query) };
                    app.state.rebuild_feed_rows();
                }
                Some(Focus::Entries) => {
                    let _ = app.dispatch(Action::SetSearchQuery(query));
                }
                Some(Focus::Preview) => {
                    let new_query = if query.is_empty() { None } else { Some(query) };
                    if app.state.preview_search_query != new_query {
                        app.state.preview_match_current = None;
                    }
                    app.state.preview_search_query = new_query;
                }
                None => {}
            }
        }
        InputMode::AddFeed => {
            let prompt = format!("{}{}", app.lang.add_feed_prompt, app.state.input_buffer);
            let _ = app.dispatch(Action::SetStatus(prompt));
        }
        InputMode::DeleteFeed => {
            let _ = app.dispatch(Action::SetStatus(app.lang.delete_feed_confirm.to_string()));
        }
        _ => {}
    }
    false
}

fn handle_add_feed_group(app: &mut App, key: KeyEvent) -> bool {
    let group_count = app.state.groups.len();
    let total_options = group_count + 1; // +1 "No category"

    match key.code {
        KeyCode::Esc => {
            let _ = app.dispatch(Action::ClearStatus);
            return true;
        }
        KeyCode::Up | KeyCode::Char('k') if app.state.modal_selection > 0 => {
            app.state.modal_selection -= 1;
        }
        KeyCode::Down | KeyCode::Char('j') if app.state.modal_selection + 1 < total_options => {
            app.state.modal_selection += 1;
        }
        KeyCode::Enter => {
            let url = if let InputMode::AddFeedGroup { ref url } = app.state.input_mode {
                url.clone()
            } else {
                return true;
            };
            let group_id = app
                .state
                .groups
                .get(app.state.modal_selection)
                .map(|g| g.id);
            let _ = app.dispatch(Action::AddFeed {
                title: None,
                url,
                group_id,
            });
            let _ = app.dispatch(Action::ClearStatus);
            return true;
        }
        _ => {}
    }
    false
}

fn handle_assign_group(app: &mut App, key: KeyEvent) -> bool {
    let group_count = app.state.groups.len();
    let total_options = group_count + 2; // +1 ungrouped, +1 new

    match key.code {
        KeyCode::Esc => {
            let _ = app.dispatch(Action::ClearStatus);
            return true;
        }
        KeyCode::Up | KeyCode::Char('k') if app.state.modal_selection > 0 => {
            app.state.modal_selection -= 1;
        }
        KeyCode::Down | KeyCode::Char('j') if app.state.modal_selection + 1 < total_options => {
            app.state.modal_selection += 1;
        }
        KeyCode::Enter => {
            if let Some(feed_id) = app.state.selected_feed {
                if let Some(group) = app.state.groups.get(app.state.modal_selection) {
                    let group_id = group.id;
                    let _ = app.dispatch(Action::AssignFeedToGroup {
                        feed_id,
                        group_id: Some(group_id),
                    });
                } else if app.state.modal_selection == group_count {
                    // "No category"
                    let _ = app.dispatch(Action::AssignFeedToGroup {
                        feed_id,
                        group_id: None,
                    });
                } else {
                    // "New category..." - switch to AddGroup mode
                    app.state.input_mode = InputMode::AddGroup;
                    app.state.modal_selection = 0;
                    let _ = app.dispatch(Action::SetStatus(app.lang.new_group_name.to_string()));
                    return false;
                }
            }
            let _ = app.dispatch(Action::ClearStatus);
            return true;
        }
        _ => {}
    }
    false
}

fn handle_manage_groups(app: &mut App, key: KeyEvent) -> bool {
    let group_count = app.state.groups.len();

    match key.code {
        KeyCode::Esc => {
            let _ = app.dispatch(Action::ClearStatus);
            return true;
        }
        KeyCode::Up | KeyCode::Char('k') if app.state.modal_selection > 0 => {
            app.state.modal_selection -= 1;
        }
        KeyCode::Down | KeyCode::Char('j')
            if group_count > 0 && app.state.modal_selection + 1 < group_count =>
        {
            app.state.modal_selection += 1;
        }
        KeyCode::Char('a') => {
            app.state.input_mode = InputMode::AddGroup;
            app.state.input_buffer.clear();
            let _ = app.dispatch(Action::SetStatus(app.lang.new_group_name.to_string()));
        }
        KeyCode::Char('d') => {
            if let Some(group) = app.state.groups.get(app.state.modal_selection) {
                let group_id = group.id;
                app.state.input_mode = InputMode::DeleteGroup { group_id };
                let _ = app.dispatch(Action::SetStatus(app.lang.delete_group_confirm.to_string()));
            }
        }
        KeyCode::Char('r') => {
            if let Some(group) = app.state.groups.get(app.state.modal_selection) {
                app.state.input_mode = InputMode::RenameGroup;
                app.state.input_buffer.set(group.name.as_str());
                let prompt = format!("{}{}", app.lang.rename_prompt, app.state.input_buffer);
                let _ = app.dispatch(Action::SetStatus(prompt));
            }
        }
        KeyCode::Char('K') if app.state.modal_selection > 0 => {
            if let (Some(a), Some(b)) = (
                app.state.groups.get(app.state.modal_selection),
                app.state.groups.get(app.state.modal_selection - 1),
            ) {
                let id_a = a.id;
                let id_b = b.id;
                let _ = app.dispatch(Action::SwapGroupOrder { id_a, id_b });
                app.state.modal_selection -= 1;
            }
        }
        KeyCode::Char('J') if app.state.modal_selection + 1 < group_count => {
            if let (Some(a), Some(b)) = (
                app.state.groups.get(app.state.modal_selection),
                app.state.groups.get(app.state.modal_selection + 1),
            ) {
                let id_a = a.id;
                let id_b = b.id;
                let _ = app.dispatch(Action::SwapGroupOrder { id_a, id_b });
                app.state.modal_selection += 1;
            }
        }
        _ => {}
    }
    false
}

fn handle_edit_feed_url(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.state.input_mode = InputMode::FeedInfo;
        }
        KeyCode::Enter => {
            let value = app.state.input_buffer.trim().to_string();
            let target = app
                .state
                .selected_feed_ref()
                .filter(|f| !value.is_empty() && value != f.url)
                .map(|f| f.id);
            match target {
                Some(feed_id) => {
                    let _ = app.dispatch(Action::SetFeedUrl {
                        id: feed_id,
                        url: value,
                    });
                    if app.state.status.as_ref().map(|s| s.kind) != Some(StatusKind::Error) {
                        app.state.input_mode = InputMode::FeedInfo;
                    }
                }
                None => {
                    app.state.input_mode = InputMode::FeedInfo;
                }
            }
        }
        _ => {
            app.state.input_buffer.handle_key(key);
        }
    }
    false
}

fn handle_group_text_input(app: &mut App, key: KeyEvent) -> bool {
    let mode = app.state.input_mode.clone();

    match key.code {
        KeyCode::Esc => {
            app.state.input_mode = InputMode::ManageGroups;
            let _ = app.dispatch(Action::SetStatus(app.lang.group_manage_hint.to_string()));
        }
        KeyCode::Enter => {
            let value = app.state.input_buffer.trim().to_string();
            if !value.is_empty() {
                match mode {
                    InputMode::AddGroup => {
                        let _ = app.dispatch(Action::AddGroup { name: value });
                    }
                    InputMode::RenameGroup => {
                        if let Some(group) = app.state.groups.get(app.state.modal_selection) {
                            let id = group.id;
                            let _ = app.dispatch(Action::RenameGroup { id, name: value });
                        }
                    }
                    _ => {}
                }
            }
            app.state.input_mode = InputMode::ManageGroups;
            let _ = app.dispatch(Action::SetStatus(app.lang.group_manage_hint.to_string()));
        }
        KeyCode::Char(_) if app.state.input_buffer.len() >= MAX_GROUP_NAME_LEN => {}
        _ => {
            if app.state.input_buffer.handle_key(key) {
                update_text_status(app, &mode);
            }
        }
    }
    false
}

fn update_text_status(app: &mut App, mode: &InputMode) {
    let prompt = match mode {
        InputMode::AddGroup => {
            format!("{}{}", app.lang.new_group_name, app.state.input_buffer)
        }
        InputMode::RenameGroup => {
            format!("{}{}", app.lang.rename_prompt, app.state.input_buffer)
        }
        _ => return,
    };
    let _ = app.dispatch(Action::SetStatus(prompt));
}

fn handle_select_discovered_feed(
    app: &mut App,
    key: KeyEvent,
    feeds: &[crate::fetch::discovery::DiscoveredFeed],
    group_id: Option<i64>,
) -> bool {
    match key.code {
        KeyCode::Esc => {
            let _ = app.dispatch(Action::ClearStatus);
            return true;
        }
        KeyCode::Up | KeyCode::Char('k') if app.state.modal_selection > 0 => {
            app.state.modal_selection -= 1;
        }
        KeyCode::Down | KeyCode::Char('j') if app.state.modal_selection + 1 < feeds.len() => {
            app.state.modal_selection += 1;
        }
        KeyCode::Enter => {
            if let Some(feed) = feeds.get(app.state.modal_selection) {
                let _ = app.dispatch(Action::AddDiscoveredFeed {
                    url: feed.url.clone(),
                    group_id,
                });
            }
            let _ = app.dispatch(Action::ClearStatus);
            return true;
        }
        _ => {}
    }
    false
}

fn handle_delete_group(app: &mut App, key: KeyEvent, group_id: i64) -> bool {
    match key.code {
        KeyCode::Char('y' | 'Y') => {
            let _ = app.dispatch(Action::DeleteGroup(group_id));
            if app.state.modal_selection > 0 {
                app.state.modal_selection -= 1;
            }
            app.state.input_mode = InputMode::ManageGroups;
            let _ = app.dispatch(Action::SetStatus(app.lang.group_manage_hint.to_string()));
        }
        _ => {
            app.state.input_mode = InputMode::ManageGroups;
            let _ = app.dispatch(Action::SetStatus(app.lang.group_manage_hint.to_string()));
        }
    }
    false
}

pub fn current_modal(state: &AppState, lang: &Lang) -> Option<ui::Modal> {
    if state.show_help {
        return Some(ui::Modal::Help {
            scroll: state.help_scroll,
        });
    }

    match &state.input_mode {
        InputMode::PanelSearch => None, // Rendered inline, not as modal
        InputMode::RenameFeed => Some(ui::Modal::Input {
            title: lang.rename_feed_title.to_string(),
            prompt: lang.name_label.to_string(),
            value: state.input_buffer.clone(),
            hint: Some(lang.rename_feed_hint.to_string()),
        }),
        InputMode::AddFeed => Some(ui::Modal::Input {
            title: lang.add_feed_title.to_string(),
            prompt: lang.url_label.to_string(),
            value: state.input_buffer.clone(),
            hint: None,
        }),
        InputMode::EditFeedUrl => Some(ui::Modal::Input {
            title: lang.edit_feed_url_title.to_string(),
            prompt: lang.url_label.to_string(),
            value: state.input_buffer.clone(),
            hint: None,
        }),
        InputMode::DeleteFeed => Some(ui::Modal::Confirm {
            title: lang.delete_feed_title.to_string(),
            prompt: if state.selected_feed.is_some() {
                lang.delete_feed_confirm.to_string()
            } else {
                lang.no_feed_selected.to_string()
            },
        }),
        InputMode::MarkAllRead { unread_count } => Some(ui::Modal::Confirm {
            title: lang.mark_all_read_title.to_string(),
            prompt: lang.mark_all_read_confirm(*unread_count),
        }),
        InputMode::AddFeedGroup { .. } | InputMode::AssignGroup => Some(ui::Modal::AssignGroup {
            selection: state.modal_selection,
        }),
        InputMode::ManageGroups | InputMode::DeleteGroup { .. } => Some(ui::Modal::ManageGroups {
            selection: state.modal_selection,
        }),
        InputMode::AddGroup => Some(ui::Modal::GroupInput {
            title: lang.new_category.to_string(),
            value: state.input_buffer.clone(),
        }),
        InputMode::RenameGroup => Some(ui::Modal::GroupInput {
            title: lang.rename_category.to_string(),
            value: state.input_buffer.clone(),
        }),
        InputMode::FeedInfo => {
            if let Some(feed) = state.selected_feed_ref() {
                let title = feed.display_title().unwrap_or(&lang.no_title).to_string();
                Some(ui::Modal::FeedInfo {
                    title,
                    url: feed.url.clone(),
                    bypass_cache: feed.bypass_cache,
                })
            } else {
                None
            }
        }
        InputMode::Discovering => Some(ui::Modal::Discovering),
        InputMode::SelectDiscoveredFeed { feeds, .. } => Some(ui::Modal::SelectDiscoveredFeed {
            feeds: feeds.clone(),
            selection: state.modal_selection,
        }),
        InputMode::None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tests::test_app;
    use crate::store::models::Feed;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn help_esc_closes() {
        let mut app = test_app();
        app.state.show_help = true;
        handle_help_key(&mut app, key(KeyCode::Esc));
        assert!(!app.state.show_help);
    }

    #[test]
    fn help_scroll_up_down() {
        let mut app = test_app();
        app.state.show_help = true;
        app.state.help_scroll = 5;
        app.state.help_max_scroll = 20;

        handle_help_key(&mut app, key(KeyCode::Char('j')));
        assert_eq!(app.state.help_scroll, 6);

        handle_help_key(&mut app, key(KeyCode::Char('k')));
        assert_eq!(app.state.help_scroll, 5);
    }

    #[test]
    fn help_scroll_clamped_to_max() {
        let mut app = test_app();
        app.state.show_help = true;
        app.state.help_scroll = 9;
        app.state.help_max_scroll = 10;

        handle_help_key(&mut app, key(KeyCode::Char('j')));
        assert_eq!(app.state.help_scroll, 10);

        // Can't go past max
        handle_help_key(&mut app, key(KeyCode::Char('j')));
        assert_eq!(app.state.help_scroll, 10);

        // End goes to max, not u16::MAX
        handle_help_key(&mut app, key(KeyCode::End));
        assert_eq!(app.state.help_scroll, 10);
    }

    #[test]
    fn help_g_and_shift_g_jump_to_top_and_bottom() {
        let mut app = test_app();
        app.state.show_help = true;
        app.state.help_scroll = 3;
        app.state.help_max_scroll = 15;

        handle_help_key(&mut app, key(KeyCode::Char('G')));
        assert_eq!(app.state.help_scroll, 15);

        handle_help_key(&mut app, key(KeyCode::Char('g')));
        assert_eq!(app.state.help_scroll, 0);
    }

    #[test]
    fn help_shift_j_and_k_page_scroll() {
        let mut app = test_app();
        app.state.show_help = true;
        app.state.help_scroll = 0;
        app.state.help_max_scroll = 30;

        handle_help_key(&mut app, key(KeyCode::Char('J')));
        assert_eq!(app.state.help_scroll, 10);

        handle_help_key(&mut app, key(KeyCode::Char('K')));
        assert_eq!(app.state.help_scroll, 0);
    }

    #[test]
    fn help_mouse_scroll() {
        let mut app = test_app();
        app.state.show_help = true;
        app.state.help_scroll = 5;
        app.state.help_max_scroll = 20;

        handle_help_scroll(&mut app, 3);
        assert_eq!(app.state.help_scroll, 8);

        handle_help_scroll(&mut app, -3);
        assert_eq!(app.state.help_scroll, 5);
    }

    #[test]
    fn search_esc_cancels() {
        let mut app = test_app();
        app.state.input_mode = InputMode::PanelSearch;
        app.state.input_buffer.set("test");
        let closed = handle_input_mode(&mut app, key(KeyCode::Esc));
        assert!(closed);
    }

    #[test]
    fn search_typing_updates_buffer() {
        let mut app = test_app();
        app.state.input_mode = InputMode::PanelSearch;
        app.state.input_buffer.clear();

        handle_input_mode(&mut app, key(KeyCode::Char('h')));
        handle_input_mode(&mut app, key(KeyCode::Char('i')));
        assert_eq!(app.state.input_buffer, "hi");
    }

    #[test]
    fn search_backspace_removes_char() {
        let mut app = test_app();
        app.state.input_mode = InputMode::PanelSearch;
        app.state.input_buffer.set("abc");

        handle_input_mode(&mut app, key(KeyCode::Backspace));
        assert_eq!(app.state.input_buffer, "ab");
    }

    #[test]
    fn delete_feed_y_confirms() {
        let mut app = test_app();
        app.state.input_mode = InputMode::DeleteFeed;
        app.state.selected_feed = Some(1);
        let closed = handle_input_mode(&mut app, key(KeyCode::Char('y')));
        assert!(closed);
    }

    #[test]
    fn delete_feed_s_confirms_catalan() {
        let mut app = test_app();
        app.state.input_mode = InputMode::DeleteFeed;
        app.state.selected_feed = Some(1);
        let closed = handle_input_mode(&mut app, key(KeyCode::Char('s')));
        assert!(closed);
    }

    #[test]
    fn delete_feed_n_cancels() {
        let mut app = test_app();
        app.state.input_mode = InputMode::DeleteFeed;
        let closed = handle_input_mode(&mut app, key(KeyCode::Char('n')));
        assert!(closed);
    }

    fn app_with_unread_entries() -> App {
        let mut app = test_app();
        app.state.entries = (10..13)
            .map(|id| crate::store::models::Entry {
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
            })
            .collect();
        app.state.input_mode = InputMode::MarkAllRead { unread_count: 3 };
        app
    }

    #[test]
    fn mark_all_read_y_confirms_and_marks_entries() {
        let mut app = app_with_unread_entries();
        let closed = handle_input_mode(&mut app, key(KeyCode::Char('y')));
        assert!(closed);
        assert!(app.state.entries.iter().all(|e| e.read_at.is_some()));
    }

    #[test]
    fn mark_all_read_s_confirms_catalan() {
        let mut app = app_with_unread_entries();
        let closed = handle_input_mode(&mut app, key(KeyCode::Char('s')));
        assert!(closed);
        assert!(app.state.entries.iter().all(|e| e.read_at.is_some()));
    }

    #[test]
    fn mark_all_read_n_cancels() {
        let mut app = app_with_unread_entries();
        let closed = handle_input_mode(&mut app, key(KeyCode::Char('n')));
        assert!(closed);
        assert!(app.state.entries.iter().all(|e| e.read_at.is_none()));
    }

    #[test]
    fn mark_all_read_esc_cancels() {
        let mut app = app_with_unread_entries();
        let closed = handle_input_mode(&mut app, key(KeyCode::Esc));
        assert!(closed);
        assert!(app.state.entries.iter().all(|e| e.read_at.is_none()));
    }

    #[test]
    fn mark_all_read_enter_cancels() {
        let mut app = app_with_unread_entries();
        let closed = handle_input_mode(&mut app, key(KeyCode::Enter));
        assert!(closed);
        assert!(app.state.entries.iter().all(|e| e.read_at.is_none()));
    }

    #[test]
    fn mark_all_read_modal_shows_count() {
        let app = app_with_unread_entries();
        match current_modal(&app.state, &app.lang) {
            Some(ui::Modal::Confirm { title, prompt }) => {
                assert_eq!(title, app.lang.mark_all_read_title);
                assert_eq!(prompt, app.lang.mark_all_read_confirm(3));
                assert!(prompt.contains('3'));
            }
            other => panic!("expected confirm modal, got {}", other.is_some()),
        }
    }

    #[test]
    fn manage_groups_selection_out_of_range() {
        let mut app = test_app();
        // No groups exist, modal_selection is 0
        app.state.input_mode = InputMode::ManageGroups;
        app.state.modal_selection = 5;
        // Pressing 'd' with out-of-range selection should not crash
        handle_input_mode(&mut app, key(KeyCode::Char('d')));
        // Should stay in ManageGroups since .get() returns None
        assert_eq!(app.state.input_mode, InputMode::ManageGroups);
    }

    #[test]
    fn manage_groups_rename_out_of_range() {
        let mut app = test_app();
        app.state.input_mode = InputMode::ManageGroups;
        app.state.modal_selection = 10;
        // Pressing 'r' with out-of-range selection should not crash
        handle_input_mode(&mut app, key(KeyCode::Char('r')));
        assert_eq!(app.state.input_mode, InputMode::ManageGroups);
    }

    #[test]
    fn delete_group_confirms_with_stored_id() {
        let mut app = test_app();
        // Store a specific group_id in the variant
        app.state.input_mode = InputMode::DeleteGroup { group_id: 42 };
        // Confirming should not panic even if groups list is empty
        let closed = handle_input_mode(&mut app, key(KeyCode::Char('y')));
        assert!(!closed); // returns to ManageGroups, not fully closed
        assert_eq!(app.state.input_mode, InputMode::ManageGroups);
    }

    #[test]
    fn delete_group_cancel_returns_to_manage() {
        let mut app = test_app();
        app.state.input_mode = InputMode::DeleteGroup { group_id: 1 };
        let closed = handle_input_mode(&mut app, key(KeyCode::Char('n')));
        assert!(!closed);
        assert_eq!(app.state.input_mode, InputMode::ManageGroups);
    }

    #[test]
    fn assign_group_with_no_groups_selects_none() {
        let mut app = test_app();
        app.state.input_mode = InputMode::AssignGroup;
        app.state.selected_feed = Some(1);
        app.state.modal_selection = 0; // first option = "No category" when groups is empty
        let closed = handle_input_mode(&mut app, key(KeyCode::Enter));
        assert!(closed);
    }

    #[test]
    fn feed_info_esc_closes() {
        let mut app = test_app();
        app.state.input_mode = InputMode::FeedInfo;
        handle_input_mode(&mut app, key(KeyCode::Esc));
        assert_eq!(app.state.input_mode, InputMode::None);
    }

    #[test]
    fn feed_info_ignores_other_keys() {
        let mut app = test_app();
        app.state.input_mode = InputMode::FeedInfo;
        handle_input_mode(&mut app, key(KeyCode::Char('a')));
        assert_eq!(app.state.input_mode, InputMode::FeedInfo);
    }

    fn app_with_selected_feed() -> App {
        let mut app = test_app();
        app.state.feeds = vec![Feed {
            id: 1,
            title: Some("A".to_string()),
            custom_title: None,
            url: "https://a.com/feed".to_string(),
            etag: None,
            last_modified: None,
            last_checked_at: None,
            group_id: None,
            bypass_cache: false,
        }];
        app.state.rebuild_feed_rows();
        app.state.selected_feed = Some(1);
        app
    }

    #[test]
    fn feed_info_u_opens_url_editor_prefilled() {
        let mut app = app_with_selected_feed();
        app.state.input_mode = InputMode::FeedInfo;
        let closed = handle_input_mode(&mut app, key(KeyCode::Char('u')));
        assert!(!closed);
        assert_eq!(app.state.input_mode, InputMode::EditFeedUrl);
        assert_eq!(app.state.input_buffer, "https://a.com/feed");
    }

    #[test]
    fn feed_info_u_without_feed_does_nothing() {
        let mut app = test_app();
        app.state.input_mode = InputMode::FeedInfo;
        handle_input_mode(&mut app, key(KeyCode::Char('u')));
        assert_eq!(app.state.input_mode, InputMode::FeedInfo);
    }

    #[test]
    fn edit_feed_url_esc_returns_to_feed_info() {
        let mut app = app_with_selected_feed();
        app.state.input_mode = InputMode::EditFeedUrl;
        app.state.input_buffer.set("https://a.com/other");
        let closed = handle_input_mode(&mut app, key(KeyCode::Esc));
        assert!(!closed);
        assert_eq!(app.state.input_mode, InputMode::FeedInfo);
        assert_eq!(app.state.feeds[0].url, "https://a.com/feed");
    }

    #[test]
    fn edit_feed_url_typing_edits_buffer() {
        let mut app = app_with_selected_feed();
        app.state.input_mode = InputMode::EditFeedUrl;
        app.state.input_buffer.set("https://a.com/fee");
        handle_input_mode(&mut app, key(KeyCode::Char('d')));
        assert_eq!(app.state.input_buffer, "https://a.com/feed");
        handle_input_mode(&mut app, key(KeyCode::Backspace));
        assert_eq!(app.state.input_buffer, "https://a.com/fee");
        assert_eq!(app.state.input_mode, InputMode::EditFeedUrl);
    }

    #[test]
    fn edit_feed_url_cursor_moves_and_edits_in_the_middle() {
        let mut app = app_with_selected_feed();
        app.state.input_mode = InputMode::FeedInfo;
        handle_input_mode(&mut app, key(KeyCode::Char('u')));
        assert_eq!(app.state.input_buffer.cursor(), "https://a.com/feed".len());

        handle_input_mode(&mut app, key(KeyCode::Home));
        for _ in 0.."https://".len() {
            handle_input_mode(&mut app, key(KeyCode::Right));
        }
        handle_input_mode(&mut app, key(KeyCode::Delete));
        handle_input_mode(&mut app, key(KeyCode::Char('b')));
        handle_input_mode(&mut app, key(KeyCode::End));
        handle_input_mode(&mut app, key(KeyCode::Left));
        handle_input_mode(&mut app, key(KeyCode::Backspace));

        assert_eq!(app.state.input_buffer, "https://b.com/fed");
        assert_eq!(app.state.input_mode, InputMode::EditFeedUrl);
    }

    #[test]
    fn edit_feed_url_modal_carries_cursor_position() {
        let mut app = app_with_selected_feed();
        app.state.input_mode = InputMode::EditFeedUrl;
        app.state.input_buffer.set("https://a.com/feed");
        app.state.input_buffer.move_home();
        match current_modal(&app.state, &app.lang) {
            Some(ui::Modal::Input { value, .. }) => assert_eq!(value.cursor(), 0),
            other => panic!("expected input modal, got {}", other.is_some()),
        }
    }

    #[test]
    fn edit_feed_url_enter_with_invalid_url_shows_error_and_stays_open() {
        let mut app = app_with_selected_feed();
        app.state.input_mode = InputMode::EditFeedUrl;
        app.state.input_buffer.set("not-a-url");
        let closed = handle_input_mode(&mut app, key(KeyCode::Enter));
        assert!(!closed);
        assert_eq!(app.state.input_mode, InputMode::EditFeedUrl);
        let status = app.state.status.as_ref().expect("status");
        assert_eq!(status.kind, StatusKind::Error);
    }

    #[test]
    fn edit_feed_url_enter_unchanged_returns_to_feed_info() {
        let mut app = app_with_selected_feed();
        app.state.input_mode = InputMode::EditFeedUrl;
        app.state.input_buffer.set("https://a.com/feed");
        let closed = handle_input_mode(&mut app, key(KeyCode::Enter));
        assert!(!closed);
        assert_eq!(app.state.input_mode, InputMode::FeedInfo);
    }

    #[test]
    fn edit_feed_url_modal_shows_input_with_url() {
        let mut app = app_with_selected_feed();
        app.state.input_mode = InputMode::EditFeedUrl;
        app.state.input_buffer.set("https://a.com/feed");
        match current_modal(&app.state, &app.lang) {
            Some(ui::Modal::Input { title, value, .. }) => {
                assert_eq!(title, app.lang.edit_feed_url_title);
                assert_eq!(value, "https://a.com/feed");
            }
            other => panic!("expected input modal, got {}", other.is_some()),
        }
    }

    #[test]
    fn add_feed_group_out_of_range_sends_none() {
        let mut app = test_app();
        app.state.input_mode = InputMode::AddFeedGroup {
            url: "https://example.com/feed".to_string(),
        };
        app.state.modal_selection = 99; // way out of range
        let closed = handle_input_mode(&mut app, key(KeyCode::Enter));
        assert!(closed); // should handle gracefully
    }

    #[test]
    fn panel_search_esc_clears_feed_filter() {
        let mut app = test_app();
        app.state.input_mode = InputMode::PanelSearch;
        app.state.panel_search_focus = Some(Focus::Feeds);
        app.state.feed_filter_query = Some("rust".to_string());
        let closed = handle_input_mode(&mut app, key(KeyCode::Esc));
        assert!(closed);
        assert!(app.state.feed_filter_query.is_none());
        assert!(app.state.panel_search_focus.is_none());
    }

    #[test]
    fn panel_search_esc_clears_preview_search() {
        let mut app = test_app();
        app.state.input_mode = InputMode::PanelSearch;
        app.state.panel_search_focus = Some(Focus::Preview);
        app.state.preview_search_query = Some("test".to_string());
        app.state.preview_match_lines = vec![1, 5];
        app.state.preview_match_current = Some(0);

        let closed = handle_input_mode(&mut app, key(KeyCode::Esc));
        assert!(closed);
        assert!(app.state.preview_search_query.is_none());
        assert!(app.state.preview_match_lines.is_empty());
        assert!(app.state.preview_match_current.is_none());
    }

    #[test]
    fn panel_search_enter_keeps_filter_active() {
        let mut app = test_app();
        app.state.input_mode = InputMode::PanelSearch;
        app.state.panel_search_focus = Some(Focus::Feeds);
        app.state.feed_filter_query = Some("rust".to_string());
        app.state.input_buffer.set("rust");

        let closed = handle_input_mode(&mut app, key(KeyCode::Enter));
        assert!(closed);
        // Filter stays active, only search bar closes
        assert_eq!(app.state.feed_filter_query.as_deref(), Some("rust"));
        assert!(app.state.panel_search_focus.is_none());
    }

    #[test]
    fn panel_search_typing_updates_feed_filter() {
        let mut app = test_app();
        app.state.input_mode = InputMode::PanelSearch;
        app.state.panel_search_focus = Some(Focus::Feeds);
        app.state.input_buffer.clear();

        handle_input_mode(&mut app, key(KeyCode::Char('r')));
        handle_input_mode(&mut app, key(KeyCode::Char('u')));
        assert_eq!(app.state.feed_filter_query.as_deref(), Some("ru"));
    }

    #[test]
    fn panel_search_typing_updates_preview_search() {
        let mut app = test_app();
        app.state.input_mode = InputMode::PanelSearch;
        app.state.panel_search_focus = Some(Focus::Preview);
        app.state.input_buffer.clear();

        handle_input_mode(&mut app, key(KeyCode::Char('a')));
        handle_input_mode(&mut app, key(KeyCode::Char('b')));
        assert_eq!(app.state.preview_search_query.as_deref(), Some("ab"));
    }

    #[test]
    fn panel_search_inserts_at_cursor_and_updates_filter() {
        let mut app = test_app();
        app.state.input_mode = InputMode::PanelSearch;
        app.state.panel_search_focus = Some(Focus::Feeds);
        app.state.input_buffer.set("rst");

        handle_input_mode(&mut app, key(KeyCode::Left));
        handle_input_mode(&mut app, key(KeyCode::Left));
        handle_input_mode(&mut app, key(KeyCode::Char('u')));
        assert_eq!(app.state.feed_filter_query.as_deref(), Some("rust"));
    }

    #[test]
    fn rename_feed_cursor_edits_in_the_middle() {
        let mut app = app_with_selected_feed();
        app.state.input_mode = InputMode::RenameFeed;
        app.state.input_buffer.set("Feed");

        handle_input_mode(&mut app, key(KeyCode::Home));
        handle_input_mode(&mut app, key(KeyCode::Char('M')));
        handle_input_mode(&mut app, key(KeyCode::Char('y')));
        handle_input_mode(&mut app, key(KeyCode::Char(' ')));
        assert_eq!(app.state.input_buffer, "My Feed");
    }

    #[test]
    fn group_input_cursor_edits_and_respects_max_length() {
        let mut app = test_app();
        app.state.input_mode = InputMode::AddGroup;
        app.state.input_buffer.set("Nws");

        handle_input_mode(&mut app, key(KeyCode::Left));
        handle_input_mode(&mut app, key(KeyCode::Left));
        handle_input_mode(&mut app, key(KeyCode::Char('e')));
        handle_input_mode(&mut app, key(KeyCode::Delete));
        assert_eq!(app.state.input_buffer, "Nes");

        app.state.input_buffer.set("x".repeat(MAX_GROUP_NAME_LEN));
        handle_input_mode(&mut app, key(KeyCode::Home));
        handle_input_mode(&mut app, key(KeyCode::Char('y')));
        assert_eq!(app.state.input_buffer.len(), MAX_GROUP_NAME_LEN);
        assert_eq!(app.state.input_buffer.cursor(), 0);
    }

    #[test]
    fn panel_search_backspace_clears_filter_when_empty() {
        let mut app = test_app();
        app.state.input_mode = InputMode::PanelSearch;
        app.state.panel_search_focus = Some(Focus::Feeds);
        app.state.input_buffer.set("r");
        app.state.feed_filter_query = Some("r".to_string());

        handle_input_mode(&mut app, key(KeyCode::Backspace));
        assert!(app.state.feed_filter_query.is_none());
    }

    #[test]
    fn panel_search_not_rendered_as_modal() {
        let lang = Lang::from_code("en");
        let mut state = AppState::default();
        state.input_mode = InputMode::PanelSearch;
        assert!(current_modal(&state, &lang).is_none());
    }
}
