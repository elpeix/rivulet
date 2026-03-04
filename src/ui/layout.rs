use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct UiLayout {
    pub header: Rect,
    pub columns: [Rect; 3],
    pub status: Rect,
}

pub fn layout_chunks(area: Rect, ratios: [u16; 3]) -> UiLayout {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(area);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(ratios[0]),
            Constraint::Percentage(ratios[1]),
            Constraint::Percentage(ratios[2]),
        ])
        .split(main[1]);

    UiLayout {
        header: main[0],
        columns: [columns[0], columns[1], columns[2]],
        status: main[2],
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
