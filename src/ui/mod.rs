pub mod layout;
pub mod rich_text;
pub mod theme;
pub mod widgets;

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Clear, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};
use ratatui::Frame;

use crate::app::state::AppState;
use crate::i18n::Lang;
use crate::ui::layout::{centered_rect, layout_chunks};
use crate::ui::theme::Theme;
use crate::ui::widgets::{
    assign_group_modal_text, entries_list, feeds_list, header_bar, manage_groups_modal_text, modal,
    panel_block, preview_block, preview_parts, selected_entry, status_bar,
};

#[derive(Debug, Clone)]
pub enum Modal {
    Input {
        title: String,
        prompt: String,
        value: String,
    },
    Confirm {
        title: String,
        prompt: String,
    },
    Help,
    AssignGroup {
        selection: usize,
    },
    ManageGroups {
        selection: usize,
    },
    GroupInput {
        title: String,
        value: String,
    },
}

pub fn draw(frame: &mut Frame<'_>, state: &mut AppState, theme: &Theme, modal_state: Option<Modal>, lang: &Lang) {
    let layout = layout_chunks(frame.area(), state.panel_ratios);

    let header = header_bar(state, theme, lang);
    frame.render_widget(header, layout.header);

    let mut feed_state = ListState::default();
    feed_state.select(state.selected_feed_row_index);
    let feeds_focused = state.focus == crate::app::state::Focus::Feeds;
    let feeds_block = panel_block(theme, feeds_focused, Some(theme.feeds_bg));
    let feeds_inner = feeds_block.inner(layout.columns[0]);
    frame.render_widget(feeds_block, layout.columns[0]);
    let feeds_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(feeds_inner);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            lang.feeds,
            theme.section_title_style(feeds_focused),
        )))
        .alignment(Alignment::Center),
        feeds_split[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "-".repeat(feeds_split[1].width.saturating_sub(1) as usize),
            theme.dim_style(),
        )))
        .alignment(Alignment::Center),
        feeds_split[1],
    );
    let feeds = feeds_list(state, theme, feeds_split[2].width, lang);
    frame.render_stateful_widget(feeds, feeds_split[2], &mut feed_state);

    let mut entry_state = ListState::default();
    entry_state.select(state.selected_entry_index);
    let entries_focused = state.focus == crate::app::state::Focus::Entries;
    let entries_block = panel_block(theme, entries_focused, None);
    let entries_inner = entries_block.inner(layout.columns[1]);
    frame.render_widget(entries_block, layout.columns[1]);
    let entries_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(entries_inner);
    let entries_title = lang.entries_panel_title(state.entries.len(), state.total_entry_count as usize);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            entries_title,
            theme.section_title_style(entries_focused),
        )))
        .alignment(Alignment::Center),
        entries_split[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "-".repeat(entries_split[1].width.saturating_sub(1) as usize),
            theme.dim_style(),
        )))
        .alignment(Alignment::Center),
        entries_split[1],
    );
    let entries = entries_list(&state.entries, theme, entries_split[2].width, lang);
    frame.render_stateful_widget(entries, entries_split[2], &mut entry_state);

    let focused_preview = state.focus == crate::app::state::Focus::Preview;
    let block = preview_block(theme, focused_preview);
    let inner = block.inner(layout.columns[2]);
    frame.render_widget(block, layout.columns[2]);
    let preview_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(inner);
    let preview_inner = preview_split[2];
    let parts = preview_parts(
        selected_entry(&state.entries, state.selected_entry),
        theme,
        preview_inner.width,
        lang,
    );
    state.preview_content_len = parts.body_len;
    let visible_height = preview_inner.height as usize;
    let preview_title = if parts.body_len > visible_height {
        let current_line = state.preview_scroll as usize + 1;
        lang.preview_panel_title(current_line, parts.body_len)
    } else {
        lang.preview.to_string()
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            preview_title,
            theme.section_title_style(focused_preview),
        )))
        .alignment(Alignment::Center),
        preview_split[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "-".repeat(preview_split[1].width.saturating_sub(1) as usize),
            theme.dim_style(),
        )))
        .alignment(Alignment::Center),
        preview_split[1],
    );
    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(preview_inner);
    frame.render_widget(Paragraph::new(parts.title), split[0]);
    frame.render_widget(Paragraph::new(parts.meta), split[1]);
    frame.render_widget(Paragraph::new(""), split[2]);
    let body_text = ratatui::text::Text::from(parts.body_lines);
    frame.render_widget(
        Paragraph::new(body_text)
            .wrap(Wrap { trim: false })
            .scroll((state.preview_scroll, 0)),
        split[3],
    );

    if parts.body_len > split[3].height as usize {
        let scrollbar_area = preview_scrollbar_area(split[3]);
        let mut scrollbar_state =
            ScrollbarState::new(parts.body_len).position(state.preview_scroll as usize);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            scrollbar_area,
            &mut scrollbar_state,
        );
    }

    let status = status_bar(state, theme, lang);
    frame.render_widget(status, layout.status);

    if let Some(modal_state) = modal_state {
        let area = centered_rect(70, 60, frame.area());
        frame.render_widget(Clear, area);
        match modal_state {
            Modal::Input {
                title,
                prompt,
                value,
            } => {
                let cursor = Span::styled("_", theme.highlight_style());
                let input_line = Line::from(vec![Span::raw(value), cursor]);
                let text = Text::from(vec![
                    Line::from(prompt),
                    input_line,
                    Line::from(""),
                    Line::from(lang.enter_confirm_esc_cancel),
                ]);
                frame.render_widget(modal(&title, text, theme), area);
            }
            Modal::Confirm { title, prompt } => {
                let text = Text::from(vec![
                    Line::from(prompt),
                    Line::from(""),
                    Line::from(lang.y_confirm_n_cancel),
                ]);
                frame.render_widget(modal(&title, text, theme), area);
            }
            Modal::AssignGroup { selection } => {
                let text = assign_group_modal_text(&state.groups, selection, theme, lang);
                frame.render_widget(modal(lang.category_title, text, theme), area);
            }
            Modal::ManageGroups { selection } => {
                let text = manage_groups_modal_text(&state.groups, selection, theme, lang);
                frame.render_widget(modal(lang.categories_title, text, theme), area);
            }
            Modal::GroupInput { title, value } => {
                let cursor = Span::styled("_", theme.highlight_style());
                let input_line = Line::from(vec![Span::raw(value), cursor]);
                let text = Text::from(vec![
                    Line::from(lang.name_label),
                    input_line,
                    Line::from(""),
                    Line::from(lang.enter_confirm_esc_cancel),
                ]);
                frame.render_widget(modal(&title, text, theme), area);
            }
            Modal::Help => {
                let inner_width = area.width.saturating_sub(2) as usize;
                let separator = Line::from(Span::styled(
                    "\u{2500}".repeat(inner_width),
                    theme.dim_style(),
                ));
                let heading = |label: &str| {
                    Line::from(Span::styled(
                        format!(" {}", label),
                        theme.section_title_style(true),
                    ))
                };
                let col_width = inner_width / 2;
                let item = |key: &str, label: &str| {
                    let key_text = format!("  {:<12}", key);
                    Line::from(vec![
                        Span::styled(key_text, Style::default().fg(theme.accent_alt)),
                        Span::styled(label.to_string(), Style::default().fg(theme.text)),
                    ])
                };
                let row = |left_key: &str, left_label: &str, right_key: &str, right_label: &str| {
                    Line::from(vec![
                        Span::styled(format!("  {:<12}", left_key), Style::default().fg(theme.accent_alt)),
                        Span::styled(
                            format!("{:<width$}", left_label, width = col_width.saturating_sub(14)),
                            Style::default().fg(theme.text),
                        ),
                        Span::styled(format!("{:<12}", right_key), Style::default().fg(theme.accent_alt)),
                        Span::styled(right_label.to_string(), Style::default().fg(theme.text)),
                    ])
                };
                let text = Text::from(vec![
                    Line::from(""),
                    heading("Navigation"),
                    separator.clone(),
                    row("Tab", "Next panel", "Shift-Tab", "Previous panel"),
                    row("Left/Right", "Move panel", "Up/Down", "Move selection"),
                    row("PgUp/PgDn", "Scroll preview", "Home/End", "Top / bottom"),
                    row("H / L", "Resize panel", "Enter", "Select / open"),
                    item("Esc", "Back"),
                    Line::from(""),
                    heading("Feeds"),
                    separator.clone(),
                    row("a", "Add feed", "d", "Delete feed"),
                    row("r", "Refresh all", "f", "Toggle unread"),
                    row("g", "Toggle saved", "c", "Assign group"),
                    item("C", "Manage groups"),
                    Line::from(""),
                    heading("Entries"),
                    separator.clone(),
                    row("m", "Toggle read", "s", "Save for later"),
                    row("o", "Open in browser", "/", "Search"),
                    Line::from(""),
                    heading("General"),
                    separator,
                    row("?", "Toggle help", "q", "Quit"),
                ]);
                frame.render_widget(modal(lang.help_title, text, theme), area);
            }
        }
    }
}

fn preview_scrollbar_area(area: Rect) -> Rect {
    let width = area.width.saturating_sub(1).min(1);
    Rect {
        x: area.x + area.width.saturating_sub(1),
        y: area.y,
        width: width.max(1),
        height: area.height,
    }
}
