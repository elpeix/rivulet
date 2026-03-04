use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use crate::app::actions::Action;
use crate::app::state::Focus;
use crate::app::App;
use crate::i18n::Lang;
use crate::ui;
use crate::util::open::open_url;
use crate::util::time::now_timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    None,
    Search,
    AddFeed,
    DeleteFeed,
    AssignGroup,
    ManageGroups,
    AddGroup,
    RenameGroup,
    DeleteGroup,
}

pub fn handle_key(
    app: &mut App,
    key: KeyEvent,
    input_mode: &mut InputMode,
    input_buffer: &mut String,
    show_help: &mut bool,
) -> bool {
    match key.code {
        KeyCode::Char('q') => return true,
        KeyCode::Char('?') => {
            *show_help = !*show_help;
        }
        KeyCode::Tab => {
            let _ = app.dispatch(Action::FocusNext);
        }
        KeyCode::BackTab => {
            let _ = app.dispatch(Action::FocusPrev);
        }
        KeyCode::Left | KeyCode::Char('h') => match app.state.focus {
            Focus::Preview => {
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
            load_entries_if_on_feeds(app);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let _ = app.dispatch(Action::MoveDown);
            load_entries_if_on_feeds(app);
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
                // Check if current row is a header → toggle collapse
                if let Some(row_idx) = app.state.selected_feed_row_index {
                    match &app.state.feed_rows[row_idx] {
                        crate::app::state::FeedRow::GroupHeader { group_id, .. } => {
                            let gid = *group_id;
                            let _ = app.dispatch(Action::ToggleGroupCollapse(gid));
                        }
                        crate::app::state::FeedRow::UngroupedHeader { .. } => {
                            // No collapse for ungrouped
                        }
                        crate::app::state::FeedRow::FeedItem { .. } => {
                            if let Some(feed_id) = app.state.selected_feed {
                                let _ = app.dispatch(Action::LoadEntriesFiltered {
                                    feed_id,
                                    unread_only: app.state.unread_only,
                                    saved_only: app.state.saved_only,
                                });
                                let _ = app.dispatch(Action::FocusEntries);
                            }
                        }
                    }
                } else if let Some(feed_id) = app.state.selected_feed {
                    let _ = app.dispatch(Action::LoadEntriesFiltered {
                        feed_id,
                        unread_only: app.state.unread_only,
                        saved_only: app.state.saved_only,
                    });
                    let _ = app.dispatch(Action::FocusEntries);
                }
            }
            Focus::Entries => {
                if app.state.selected_entry.is_some() {
                    let _ = app.dispatch(Action::FocusPreview);
                }
            }
            Focus::Preview => {}
        },
        KeyCode::Char('r') => {
            let _ = app.dispatch(Action::RefreshFeeds);
        }
        KeyCode::Char('f') => {
            let _ = app.dispatch(Action::ToggleUnreadFilter);
        }
        KeyCode::Char('g') => {
            let _ = app.dispatch(Action::ToggleSavedFilter);
        }
        KeyCode::Char('m') => {
            if let Some(entry_id) = app.state.selected_entry {
                if let Some(entry) = app.state.entries.iter_mut().find(|entry| entry.id == entry_id)
                {
                    if entry.read_at.is_none() {
                        let timestamp = now_timestamp();
                        entry.read_at = Some(timestamp);
                        let _ = app.dispatch(Action::MarkRead(entry_id));
                    } else {
                        entry.read_at = None;
                        let _ = app.dispatch(Action::MarkUnread(entry_id));
                    }
                }
                let _ = app.dispatch(Action::RefreshUnreadCounts);
            }
        }
        KeyCode::Char('o') => {
            if let Some(entry_id) = app.state.selected_entry {
                if let Some(entry) = app.state.entries.iter().find(|entry| entry.id == entry_id) {
                    if let Some(url) = entry.url.as_deref().filter(|value| !value.trim().is_empty())
                    {
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
        KeyCode::Char('s') => {
            if let Some(entry_id) = app.state.selected_entry {
                if let Some(entry) = app.state.entries.iter().find(|entry| entry.id == entry_id) {
                    if entry.saved_at.is_some() {
                        let _ = app.dispatch(Action::MarkUnsaved(entry_id));
                    } else {
                        let _ = app.dispatch(Action::MarkSaved(entry_id));
                    }
                }
                refresh_entries(app);
            }
        }
        KeyCode::Char('/') => {
            *input_mode = InputMode::Search;
            input_buffer.clear();
            let _ = app.dispatch(Action::SetStatus(app.lang.search_prompt.to_string()));
        }
        KeyCode::Char('a') => {
            *input_mode = InputMode::AddFeed;
            input_buffer.clear();
            let _ = app.dispatch(Action::SetStatus(app.lang.add_feed_prompt.to_string()));
        }
        KeyCode::Char('c') => {
            if app.state.selected_feed.is_some() {
                *input_mode = InputMode::AssignGroup;
                input_buffer.clear();
                let _ = app.dispatch(Action::SetStatus("Assign group...".to_string()));
            }
        }
        KeyCode::Char('C') => {
            *input_mode = InputMode::ManageGroups;
            input_buffer.clear();
            let _ = app.dispatch(Action::SetStatus(app.lang.group_manage_hint.to_string()));
        }
        KeyCode::Char('d') => {
            if app.state.selected_feed.is_some() {
                *input_mode = InputMode::DeleteFeed;
                input_buffer.clear();
                let _ = app.dispatch(Action::SetStatus(
                    app.lang.delete_feed_confirm.to_string(),
                ));
            } else {
                let _ = app.dispatch(Action::SetStatus(app.lang.no_feed_selected.to_string()));
            }
        }
        KeyCode::Esc => match app.state.focus {
            Focus::Preview => {
                let _ = app.dispatch(Action::FocusEntries);
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

pub fn handle_input_mode(
    app: &mut App,
    key: KeyEvent,
    input_mode: &mut InputMode,
    buffer: &mut String,
    modal_selection: &mut usize,
) -> bool {
    // Handle modal-based input modes first
    match *input_mode {
        InputMode::AssignGroup => {
            return handle_assign_group(app, key, input_mode, modal_selection);
        }
        InputMode::ManageGroups => {
            return handle_manage_groups(app, key, input_mode, buffer, modal_selection);
        }
        InputMode::AddGroup | InputMode::RenameGroup => {
            return handle_group_text_input(app, key, input_mode, buffer, modal_selection);
        }
        InputMode::DeleteGroup => {
            return handle_delete_group(app, key, input_mode, modal_selection);
        }
        _ => {}
    }

    match key.code {
        KeyCode::Esc => {
            if *input_mode == InputMode::Search {
                // Cancel search: clear query and reload unfiltered
                let _ = app.dispatch(Action::SetSearchQuery(String::new()));
            }
            let _ = app.dispatch(Action::ClearStatus);
            return true;
        }
        KeyCode::Char('y') | KeyCode::Char('Y') if *input_mode == InputMode::DeleteFeed => {
            if let Some(feed_id) = app.state.selected_feed {
                let _ = app.dispatch(Action::DeleteFeed(feed_id));
            }
            let _ = app.dispatch(Action::ClearStatus);
            return true;
        }
        KeyCode::Char('n') | KeyCode::Char('N') if *input_mode == InputMode::DeleteFeed => {
            let _ = app.dispatch(Action::ClearStatus);
            return true;
        }
        KeyCode::Enter => {
            let value = buffer.trim().to_string();
            match input_mode {
                InputMode::Search => {
                    // Already applied incrementally, just close modal
                }
                InputMode::AddFeed => {
                    if !value.is_empty() {
                        let _ = app.dispatch(Action::AddFeed {
                            title: None,
                            url: value,
                        });
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
            buffer.pop();
        }
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                return false;
            }
            buffer.push(c);
        }
        _ => {}
    }

    match input_mode {
        InputMode::Search => {
            let _ = app.dispatch(Action::SetStatus(format!("{}{}", app.lang.search_prompt, buffer)));
            // Incremental search: filter as the user types
            let query = buffer.trim().to_string();
            let _ = app.dispatch(Action::SetSearchQuery(query));
        }
        InputMode::AddFeed => {
            let _ = app.dispatch(Action::SetStatus(format!("{}{}", app.lang.add_feed_prompt, buffer)));
        }
        InputMode::DeleteFeed => {
            let _ = app.dispatch(Action::SetStatus(
                app.lang.delete_feed_confirm.to_string(),
            ));
        }
        _ => {}
    }
    false
}

fn handle_assign_group(
    app: &mut App,
    key: KeyEvent,
    input_mode: &mut InputMode,
    selection: &mut usize,
) -> bool {
    // Options: groups[0..n], "No category", "New category..."
    let group_count = app.state.groups.len();
    let total_options = group_count + 2; // +1 ungrouped, +1 new

    match key.code {
        KeyCode::Esc => {
            let _ = app.dispatch(Action::ClearStatus);
            return true;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if *selection > 0 {
                *selection -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if *selection + 1 < total_options {
                *selection += 1;
            }
        }
        KeyCode::Enter => {
            if let Some(feed_id) = app.state.selected_feed {
                if *selection < group_count {
                    let group_id = app.state.groups[*selection].id;
                    let _ = app.dispatch(Action::AssignFeedToGroup {
                        feed_id,
                        group_id: Some(group_id),
                    });
                } else if *selection == group_count {
                    // "No category"
                    let _ = app.dispatch(Action::AssignFeedToGroup {
                        feed_id,
                        group_id: None,
                    });
                } else {
                    // "New category..." - switch to AddGroup mode
                    *input_mode = InputMode::AddGroup;
                    *selection = 0;
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

fn handle_manage_groups(
    app: &mut App,
    key: KeyEvent,
    input_mode: &mut InputMode,
    buffer: &mut String,
    selection: &mut usize,
) -> bool {
    let group_count = app.state.groups.len();

    match key.code {
        KeyCode::Esc => {
            let _ = app.dispatch(Action::ClearStatus);
            return true;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if *selection > 0 {
                *selection -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if group_count > 0 && *selection + 1 < group_count {
                *selection += 1;
            }
        }
        KeyCode::Char('a') => {
            *input_mode = InputMode::AddGroup;
            buffer.clear();
            let _ = app.dispatch(Action::SetStatus(app.lang.new_group_name.to_string()));
        }
        KeyCode::Char('d') => {
            if group_count > 0 && *selection < group_count {
                *input_mode = InputMode::DeleteGroup;
                let _ = app.dispatch(Action::SetStatus(app.lang.delete_group_confirm.to_string()));
            }
        }
        KeyCode::Char('r') => {
            if group_count > 0 && *selection < group_count {
                *input_mode = InputMode::RenameGroup;
                buffer.clear();
                buffer.push_str(&app.state.groups[*selection].name);
                let _ = app.dispatch(Action::SetStatus(format!("{}{}", app.lang.rename_prompt, buffer)));
            }
        }
        _ => {}
    }
    false
}

fn handle_group_text_input(
    app: &mut App,
    key: KeyEvent,
    input_mode: &mut InputMode,
    buffer: &mut String,
    selection: &mut usize,
) -> bool {
    match key.code {
        KeyCode::Esc => {
            *input_mode = InputMode::ManageGroups;
            let _ = app.dispatch(Action::SetStatus(app.lang.group_manage_hint.to_string()));
        }
        KeyCode::Enter => {
            let value = buffer.trim().to_string();
            if !value.is_empty() {
                match *input_mode {
                    InputMode::AddGroup => {
                        let _ = app.dispatch(Action::AddGroup { name: value });
                    }
                    InputMode::RenameGroup => {
                        if *selection < app.state.groups.len() {
                            let id = app.state.groups[*selection].id;
                            let _ = app.dispatch(Action::RenameGroup { id, name: value });
                        }
                    }
                    _ => {}
                }
            }
            *input_mode = InputMode::ManageGroups;
            let _ = app.dispatch(Action::SetStatus(app.lang.group_manage_hint.to_string()));
        }
        KeyCode::Backspace => {
            buffer.pop();
            match *input_mode {
                InputMode::AddGroup => {
                    let _ = app.dispatch(Action::SetStatus(format!("{}{}", app.lang.new_group_name, buffer)));
                }
                InputMode::RenameGroup => {
                    let _ = app.dispatch(Action::SetStatus(format!("{}{}", app.lang.rename_prompt, buffer)));
                }
                _ => {}
            }
        }
        KeyCode::Char(c) => {
            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                buffer.push(c);
                match *input_mode {
                    InputMode::AddGroup => {
                        let _ = app.dispatch(Action::SetStatus(format!("{}{}", app.lang.new_group_name, buffer)));
                    }
                    InputMode::RenameGroup => {
                        let _ = app.dispatch(Action::SetStatus(format!("{}{}", app.lang.rename_prompt, buffer)));
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    false
}

fn handle_delete_group(
    app: &mut App,
    key: KeyEvent,
    input_mode: &mut InputMode,
    selection: &mut usize,
) -> bool {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if *selection < app.state.groups.len() {
                let id = app.state.groups[*selection].id;
                let _ = app.dispatch(Action::DeleteGroup(id));
                if *selection > 0 {
                    *selection -= 1;
                }
            }
            *input_mode = InputMode::ManageGroups;
            let _ = app.dispatch(Action::SetStatus(app.lang.group_manage_hint.to_string()));
        }
        _ => {
            *input_mode = InputMode::ManageGroups;
            let _ = app.dispatch(Action::SetStatus(app.lang.group_manage_hint.to_string()));
        }
    }
    false
}

pub fn handle_mouse(app: &mut App, event: MouseEvent, area: ratatui::layout::Rect) {
    let layout = ui::layout::layout_chunks(area, app.state.panel_ratios);
    let (x, y) = (event.column, event.row);

    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if contains(layout.columns[0], x, y) {
                let _ = app.dispatch(Action::FocusFeeds);
                if let Some(row_idx) = list_index(x, y, layout.columns[0], 1) {
                    if let Some(row) = app.state.feed_rows.get(row_idx) {
                        match row {
                            crate::app::state::FeedRow::GroupHeader { group_id, .. } => {
                                let gid = *group_id;
                                app.state.selected_feed_row_index = Some(row_idx);
                                let _ = app.dispatch(Action::ToggleGroupCollapse(gid));
                            }
                            crate::app::state::FeedRow::UngroupedHeader { .. } => {
                                app.state.selected_feed_row_index = Some(row_idx);
                            }
                            crate::app::state::FeedRow::FeedItem { feed_index } => {
                                if let Some(feed_id) = app.state.feeds.get(*feed_index).map(|f| f.id) {
                                    app.state.selected_feed_row_index = Some(row_idx);
                                    app.state.reduce(Action::SelectFeed(Some(feed_id)));
                                    let _ = app.dispatch(Action::LoadEntriesFiltered {
                                        feed_id,
                                        unread_only: app.state.unread_only,
                                        saved_only: app.state.saved_only,
                                    });
                                }
                            }
                        }
                    }
                }
                return;
            }

            if contains(layout.columns[1], x, y) {
                let _ = app.dispatch(Action::FocusEntries);
                if let Some(index) = list_index(x, y, layout.columns[1], 1)
                    && let Some(entry_id) = app.state.entries.get(index).map(|entry| entry.id) {
                        app.state.reduce(Action::SelectEntry(Some(entry_id)));
                    }
                return;
            }

            if contains(layout.columns[2], x, y) {
                let _ = app.dispatch(Action::FocusPreview);
            }
        }
        MouseEventKind::ScrollUp => {
            if contains(layout.columns[2], x, y) {
                let _ = app.dispatch(Action::FocusPreview);
                for _ in 0..3 {
                    let _ = app.dispatch(Action::MoveUp);
                }
            }
        }
        MouseEventKind::ScrollDown => {
            if contains(layout.columns[2], x, y) {
                let _ = app.dispatch(Action::FocusPreview);
                for _ in 0..3 {
                    let _ = app.dispatch(Action::MoveDown);
                }
            }
        }
        _ => {}
    }
}

pub fn current_modal(
    input_mode: &InputMode,
    input_buffer: &str,
    show_help: bool,
    has_feed: bool,
    modal_selection: usize,
    _state: &crate::app::state::AppState,
    lang: &Lang,
) -> Option<ui::Modal> {
    if show_help {
        return Some(ui::Modal::Help);
    }

    match input_mode {
        InputMode::Search => Some(ui::Modal::Input {
            title: lang.search_title.to_string(),
            prompt: lang.query_label.to_string(),
            value: input_buffer.to_string(),
        }),
        InputMode::AddFeed => Some(ui::Modal::Input {
            title: lang.add_feed_title.to_string(),
            prompt: lang.url_label.to_string(),
            value: input_buffer.to_string(),
        }),
        InputMode::DeleteFeed => Some(ui::Modal::Confirm {
            title: lang.delete_feed_title.to_string(),
            prompt: if has_feed {
                lang.delete_feed_confirm.to_string()
            } else {
                lang.no_feed_selected.to_string()
            },
        }),
        InputMode::AssignGroup => Some(ui::Modal::AssignGroup {
            selection: modal_selection,
        }),
        InputMode::ManageGroups | InputMode::DeleteGroup => Some(ui::Modal::ManageGroups {
            selection: modal_selection,
        }),
        InputMode::AddGroup => Some(ui::Modal::GroupInput {
            title: lang.new_category.to_string(),
            value: input_buffer.to_string(),
        }),
        InputMode::RenameGroup => Some(ui::Modal::GroupInput {
            title: lang.rename_category.to_string(),
            value: input_buffer.to_string(),
        }),
        InputMode::None => None,
    }
}

fn contains(area: ratatui::layout::Rect, x: u16, y: u16) -> bool {
    x >= area.x
        && x < area.x.saturating_add(area.width)
        && y >= area.y
        && y < area.y.saturating_add(area.height)
}

fn list_index(
    x: u16,
    y: u16,
    panel: ratatui::layout::Rect,
    row_height: u16,
) -> Option<usize> {
    let inner = ratatui::layout::Rect {
        x: panel.x.saturating_add(1),
        y: panel.y.saturating_add(1),
        width: panel.width.saturating_sub(2),
        height: panel.height.saturating_sub(2),
    };

    let list_area = ratatui::layout::Rect {
        x: inner.x,
        y: inner.y.saturating_add(2),
        width: inner.width,
        height: inner.height.saturating_sub(2),
    };

    if !contains(list_area, x, y) {
        return None;
    }

    let offset = y.saturating_sub(list_area.y);
    Some((offset / row_height) as usize)
}

fn refresh_entries(app: &mut App) {
    let _ = app.dispatch(Action::RefreshUnreadCounts);
    if let Some(feed_id) = app.state.selected_feed {
        if let Some(query) = app.state.search_query.clone() {
            let _ = app.dispatch(Action::SearchEntries {
                feed_id: Some(feed_id),
                query,
            });
        } else {
            let _ = app.dispatch(Action::LoadEntriesFiltered {
                feed_id,
                unread_only: app.state.unread_only,
                saved_only: app.state.saved_only,
            });
        }
    }
}

fn load_entries_if_on_feeds(app: &mut App) {
    if app.state.focus != Focus::Feeds {
        return;
    }
    // Don't load entries when navigating over group headers
    if let Some(row_idx) = app.state.selected_feed_row_index {
        if let Some(row) = app.state.feed_rows.get(row_idx) {
            if !matches!(row, crate::app::state::FeedRow::FeedItem { .. }) {
                return;
            }
        }
    }
    if let Some(feed_id) = app.state.selected_feed {
        let _ = app.dispatch(Action::LoadEntriesFiltered {
            feed_id,
            unread_only: app.state.unread_only,
            saved_only: app.state.saved_only,
        });
    }
}
