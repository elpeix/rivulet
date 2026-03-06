use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::app::actions::Action;
use crate::app::state::FeedRow;
use crate::app::App;
use crate::ui;
use crate::util::open::open_url;


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
                            FeedRow::AllFeeds => {
                                app.state.selected_feed_row_index = Some(row_idx);
                                let _ = app.dispatch(Action::LoadAllEntries {
                                    unread_only: app.state.unread_only,
                                    saved_only: app.state.saved_only,
                                    since: app.since_cutoff(),
                                });
                            }
                            FeedRow::GroupHeader { group_id, .. } => {
                                let gid = *group_id;
                                app.state.selected_feed_row_index = Some(row_idx);
                                let _ = app.dispatch(Action::LoadEntriesForGroup {
                                    group_id: Some(gid),
                                    unread_only: app.state.unread_only,
                                    saved_only: app.state.saved_only,
                                    since: app.since_cutoff(),
                                });
                            }
                            FeedRow::UngroupedHeader { .. } => {
                                app.state.selected_feed_row_index = Some(row_idx);
                                let _ = app.dispatch(Action::LoadEntriesForGroup {
                                    group_id: None,
                                    unread_only: app.state.unread_only,
                                    saved_only: app.state.saved_only,
                                    since: app.since_cutoff(),
                                });
                            }
                            FeedRow::FeedItem { feed_index } => {
                                if let Some(feed_id) =
                                    app.state.feeds.get(*feed_index).map(|f| f.id)
                                {
                                    app.state.selected_feed_row_index = Some(row_idx);
                                    app.state.reduce(Action::SelectFeed(Some(feed_id)));
                                    let _ = app.dispatch(Action::LoadEntriesFiltered {
                                        feed_id,
                                        unread_only: app.state.unread_only,
                                        saved_only: app.state.saved_only,
                                        since: app.since_cutoff(),
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
                    && let Some(entry_id) = app.state.entries.get(index).map(|entry| entry.id)
                {
                    app.state.reduce(Action::SelectEntry(Some(entry_id)));
                }
                return;
            }

            if contains(layout.columns[2], x, y) {
                let _ = app.dispatch(Action::FocusPreview);
                if let Some(url) = hit_test_link(app, x, y) {
                    match open_url(&url) {
                        Ok(()) => {
                            let _ = app.dispatch(
                                Action::SetStatus(app.lang.opened_in_browser.to_string()),
                            );
                        }
                        Err(error) => {
                            let _ = app.dispatch(Action::DbError(error));
                        }
                    }
                }
            }
        }
        MouseEventKind::ScrollUp => {
            if contains(layout.columns[1], x, y) {
                let _ = app.dispatch(Action::FocusEntries);
                for _ in 0..3 {
                    let _ = app.dispatch(Action::MoveUp);
                }
            } else if contains(layout.columns[2], x, y) {
                let _ = app.dispatch(Action::FocusPreview);
                for _ in 0..3 {
                    let _ = app.dispatch(Action::MoveUp);
                }
            }
        }
        MouseEventKind::ScrollDown => {
            if contains(layout.columns[1], x, y) {
                let _ = app.dispatch(Action::FocusEntries);
                for _ in 0..3 {
                    let _ = app.dispatch(Action::MoveDown);
                }
            } else if contains(layout.columns[2], x, y) {
                let _ = app.dispatch(Action::FocusPreview);
                for _ in 0..3 {
                    let _ = app.dispatch(Action::MoveDown);
                }
            }
        }
        _ => {}
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

fn hit_test_link(app: &App, x: u16, y: u16) -> Option<String> {
    let area = app.state.preview_body_area;
    if !contains(area, x, y) {
        return None;
    }
    let rel_line = (y - area.y) as usize + app.state.preview_scroll as usize;
    let rel_col = x - area.x;
    app.state
        .preview_link_regions
        .iter()
        .find(|r| r.line == rel_line && rel_col >= r.col_start && rel_col < r.col_end)
        .map(|r| r.url.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn contains_inside() {
        let area = Rect::new(5, 5, 10, 10);
        assert!(contains(area, 5, 5));
        assert!(contains(area, 10, 10));
        assert!(contains(area, 14, 14));
    }

    #[test]
    fn contains_outside() {
        let area = Rect::new(5, 5, 10, 10);
        assert!(!contains(area, 4, 5));
        assert!(!contains(area, 5, 4));
        assert!(!contains(area, 15, 5));
        assert!(!contains(area, 5, 15));
    }

    #[test]
    fn contains_zero_size() {
        let area = Rect::new(5, 5, 0, 0);
        assert!(!contains(area, 5, 5));
    }

    #[test]
    fn list_index_within_list_area() {
        // Panel at (0,0) 20x20. Inner = (1,1) 18x18. List area = (1,3) 18x16.
        let panel = Rect::new(0, 0, 20, 20);
        // Click at row 5, which is offset 2 from list_area.y=3
        assert_eq!(list_index(5, 5, panel, 1), Some(2));
    }

    #[test]
    fn list_index_first_row() {
        let panel = Rect::new(0, 0, 20, 20);
        // First row of list area is y=3 (panel.y+1+2)
        assert_eq!(list_index(5, 3, panel, 1), Some(0));
    }

    #[test]
    fn list_index_outside_panel() {
        let panel = Rect::new(10, 10, 20, 20);
        // Click outside panel
        assert_eq!(list_index(5, 5, panel, 1), None);
    }

    #[test]
    fn list_index_in_header_area() {
        let panel = Rect::new(0, 0, 20, 20);
        // y=1 is in the inner area but above the list area (header/separator)
        assert_eq!(list_index(5, 1, panel, 1), None);
        assert_eq!(list_index(5, 2, panel, 1), None);
    }

    #[test]
    fn list_index_with_row_height_2() {
        let panel = Rect::new(0, 0, 20, 20);
        // list_area.y = 3, row_height = 2
        assert_eq!(list_index(5, 3, panel, 2), Some(0));
        assert_eq!(list_index(5, 4, panel, 2), Some(0));
        assert_eq!(list_index(5, 5, panel, 2), Some(1));
    }

    #[test]
    fn hit_test_link_returns_none_outside_area() {
        use crate::app::tests::test_app;
        let app = test_app();
        // preview_body_area defaults to Rect::default() (0,0,0,0)
        assert!(hit_test_link(&app, 10, 10).is_none());
    }

    #[test]
    fn hit_test_link_finds_matching_region() {
        use crate::app::tests::test_app;
        use crate::ui::rich_text::LinkRegion;

        let mut app = test_app();
        app.state.preview_body_area = Rect::new(5, 5, 40, 20);
        app.state.preview_scroll = 0;
        app.state.preview_link_regions = vec![LinkRegion {
            line: 2,
            col_start: 3,
            col_end: 10,
            url: "https://example.com".to_string(),
        }];

        // line = (y - area.y) + scroll = (7 - 5) + 0 = 2, col = x - area.x = 8 - 5 = 3
        assert_eq!(
            hit_test_link(&app, 8, 7),
            Some("https://example.com".to_string())
        );
        // col_end is exclusive: col 15-5=10 should miss
        assert!(hit_test_link(&app, 15, 7).is_none());
        // wrong line
        assert!(hit_test_link(&app, 8, 8).is_none());
    }

    #[test]
    fn hit_test_link_accounts_for_scroll() {
        use crate::app::tests::test_app;
        use crate::ui::rich_text::LinkRegion;

        let mut app = test_app();
        app.state.preview_body_area = Rect::new(0, 0, 40, 20);
        app.state.preview_scroll = 5;
        app.state.preview_link_regions = vec![LinkRegion {
            line: 7,
            col_start: 0,
            col_end: 10,
            url: "https://scroll.test".to_string(),
        }];

        // line = (y - 0) + 5 = 2 + 5 = 7
        assert_eq!(
            hit_test_link(&app, 3, 2),
            Some("https://scroll.test".to_string())
        );
    }
}
