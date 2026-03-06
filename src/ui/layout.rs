use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::app::state::LayoutMode;

pub struct UiLayout {
    pub feeds: Rect,
    pub entries: Rect,
    pub preview: Rect,
    pub status: Rect,
}

pub fn build_layout(
    area: Rect,
    mode: LayoutMode,
    panel_ratios: [u16; 3],
    split_ratio: u16,
    status_height: u16,
) -> UiLayout {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(status_height)])
        .split(area);

    match mode {
        LayoutMode::Columns => {
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(panel_ratios[0]),
                    Constraint::Percentage(panel_ratios[1]),
                    Constraint::Percentage(panel_ratios[2]),
                ])
                .split(main[0]);
            UiLayout {
                feeds: columns[0],
                entries: columns[1],
                preview: columns[2],
                status: main[1],
            }
        }
        LayoutMode::Split => {
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(panel_ratios[0]),
                    Constraint::Percentage(100 - panel_ratios[0]),
                ])
                .split(main[0]);
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(split_ratio),
                    Constraint::Percentage(100 - split_ratio),
                ])
                .split(columns[1]);
            UiLayout {
                feeds: columns[0],
                entries: rows[0],
                preview: rows[1],
                status: main[1],
            }
        }
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1]);

    horizontal[1]
}
