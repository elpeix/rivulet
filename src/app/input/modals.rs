use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;
use crate::app::actions::Action;
use crate::app::state::{AppState, InputMode};
use crate::i18n::Lang;
use crate::ui;

const MAX_GROUP_NAME_LEN: usize = 64;

pub fn handle_help_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('?' | 'q') => {
            app.state.show_help = false;
            app.state.help_scroll = 0;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.state.help_scroll = app.state.help_scroll.saturating_add(1);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.state.help_scroll = app.state.help_scroll.saturating_sub(1);
        }
        KeyCode::PageDown => {
            app.state.help_scroll = app.state.help_scroll.saturating_add(10);
        }
        KeyCode::PageUp => {
            app.state.help_scroll = app.state.help_scroll.saturating_sub(10);
        }
        KeyCode::Home => {
            app.state.help_scroll = 0;
        }
        KeyCode::End => {
            app.state.help_scroll = u16::MAX;
        }
        _ => {}
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
        _ => {}
    }

    match key.code {
        KeyCode::Esc => {
            if mode == InputMode::Search {
                let _ = app.dispatch(Action::SetSearchQuery(String::new()));
            }
            let _ = app.dispatch(Action::ClearStatus);
            return true;
        }
        KeyCode::Char('y' | 'Y') if mode == InputMode::DeleteFeed => {
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
                InputMode::Search => {
                    // Already applied incrementally, just close modal
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
                InputMode::AddFeed => {
                    if !value.is_empty() {
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
        KeyCode::Backspace => {
            app.state.input_buffer.pop();
        }
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                return false;
            }
            app.state.input_buffer.push(c);
        }
        _ => {}
    }

    match &mode {
        InputMode::Search => {
            let prompt = format!("{}{}", app.lang.search_prompt, app.state.input_buffer);
            let _ = app.dispatch(Action::SetStatus(prompt));
            let query = app.state.input_buffer.trim().to_string();
            let _ = app.dispatch(Action::SetSearchQuery(query));
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
        KeyCode::Up | KeyCode::Char('k') => {
            if app.state.modal_selection > 0 {
                app.state.modal_selection -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.state.modal_selection + 1 < total_options {
                app.state.modal_selection += 1;
            }
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
        KeyCode::Up | KeyCode::Char('k') => {
            if app.state.modal_selection > 0 {
                app.state.modal_selection -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.state.modal_selection + 1 < total_options {
                app.state.modal_selection += 1;
            }
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
        KeyCode::Up | KeyCode::Char('k') => {
            if app.state.modal_selection > 0 {
                app.state.modal_selection -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if group_count > 0 && app.state.modal_selection + 1 < group_count {
                app.state.modal_selection += 1;
            }
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
                app.state.input_buffer.clear();
                app.state.input_buffer.push_str(&group.name);
                let prompt = format!("{}{}", app.lang.rename_prompt, app.state.input_buffer);
                let _ = app.dispatch(Action::SetStatus(prompt));
            }
        }
        KeyCode::Char('K') => {
            if app.state.modal_selection > 0 {
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
        }
        KeyCode::Char('J') => {
            if app.state.modal_selection + 1 < group_count {
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
        }
        _ => {}
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
        KeyCode::Backspace => {
            app.state.input_buffer.pop();
            update_text_status(app, &mode);
        }
        KeyCode::Char(c) => {
            if !key.modifiers.contains(KeyModifiers::CONTROL)
                && app.state.input_buffer.len() < MAX_GROUP_NAME_LEN
            {
                app.state.input_buffer.push(c);
                update_text_status(app, &mode);
            }
        }
        _ => {}
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
        InputMode::Search => Some(ui::Modal::Input {
            title: lang.search_title.to_string(),
            prompt: lang.query_label.to_string(),
            value: state.input_buffer.clone(),
            hint: None,
        }),
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
        InputMode::DeleteFeed => Some(ui::Modal::Confirm {
            title: lang.delete_feed_title.to_string(),
            prompt: if state.selected_feed.is_some() {
                lang.delete_feed_confirm.to_string()
            } else {
                lang.no_feed_selected.to_string()
            },
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
        InputMode::None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tests::test_app;
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

        handle_help_key(&mut app, key(KeyCode::Char('j')));
        assert_eq!(app.state.help_scroll, 6);

        handle_help_key(&mut app, key(KeyCode::Char('k')));
        assert_eq!(app.state.help_scroll, 5);
    }

    #[test]
    fn search_esc_cancels() {
        let mut app = test_app();
        app.state.input_mode = InputMode::Search;
        app.state.input_buffer = "test".to_string();
        let closed = handle_input_mode(&mut app, key(KeyCode::Esc));
        assert!(closed);
    }

    #[test]
    fn search_typing_updates_buffer() {
        let mut app = test_app();
        app.state.input_mode = InputMode::Search;
        app.state.input_buffer.clear();

        handle_input_mode(&mut app, key(KeyCode::Char('h')));
        handle_input_mode(&mut app, key(KeyCode::Char('i')));
        assert_eq!(app.state.input_buffer, "hi");
    }

    #[test]
    fn search_backspace_removes_char() {
        let mut app = test_app();
        app.state.input_mode = InputMode::Search;
        app.state.input_buffer = "abc".to_string();

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
    fn delete_feed_n_cancels() {
        let mut app = test_app();
        app.state.input_mode = InputMode::DeleteFeed;
        let closed = handle_input_mode(&mut app, key(KeyCode::Char('n')));
        assert!(closed);
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
    fn add_feed_group_out_of_range_sends_none() {
        let mut app = test_app();
        app.state.input_mode = InputMode::AddFeedGroup {
            url: "https://example.com/feed".to_string(),
        };
        app.state.modal_selection = 99; // way out of range
        let closed = handle_input_mode(&mut app, key(KeyCode::Enter));
        assert!(closed); // should handle gracefully
    }
}
