use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap};

use crate::app::state::{AppState, FeedRow, StatusKind};
use crate::i18n::Lang;
use crate::store::models::{Entry, Group};
use crate::ui::rich_text::rich_lines_to_ratatui;
use crate::ui::theme::Theme;
use crate::util::html::to_rich_lines;
use crate::util::time::{format_timestamp, format_timestamp_relative};

pub fn feeds_list<'a>(state: &AppState, theme: &Theme, max_width: u16, lang: &Lang) -> List<'a> {
    let items: Vec<ListItem> = state
        .feed_rows
        .iter()
        .map(|row| match row {
            FeedRow::GroupHeader { name, unread, group_id } => {
                let collapsed = state.collapsed_groups.contains(group_id);
                let arrow = if collapsed { "\u{25b6}" } else { "\u{25bc}" };
                let counter = format!("  {}", unread);
                let available = max_width as usize;
                let title_max = available
                    .saturating_sub(counter.len())
                    .saturating_sub(3); // arrow + space
                let truncated = truncate_with_ellipsis(name, title_max);
                let line = Line::from(vec![
                    Span::styled(
                        format!("{} {}", arrow, truncated),
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(counter, theme.dim_style()),
                ]);
                ListItem::new(line)
            }
            FeedRow::UngroupedHeader { unread } => {
                let counter = format!("  {}", unread);
                let label = lang.uncategorized;
                let available = max_width as usize;
                let title_max = available
                    .saturating_sub(counter.len())
                    .saturating_sub(3);
                let truncated = truncate_with_ellipsis(label, title_max);
                let line = Line::from(vec![
                    Span::styled(
                        format!("\u{25bc} {}", truncated),
                        Style::default()
                            .fg(theme.dim)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(counter, theme.dim_style()),
                ]);
                ListItem::new(line)
            }
            FeedRow::FeedItem { feed_index } => {
                let feed = &state.feeds[*feed_index];
                let unread = state.unread_counts.get(&feed.id).copied().unwrap_or(0);
                let title = feed
                    .title
                    .as_deref()
                    .filter(|value| !value.is_empty())
                    .unwrap_or(feed.url.as_str());
                let base_style = if unread > 0 {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let counter = format!("  {}", unread);
                let has_groups = !state.groups.is_empty();
                let indent = if has_groups { "  " } else { "" };
                let available = max_width as usize;
                let title_max = available
                    .saturating_sub(counter.len())
                    .saturating_sub(indent.len())
                    .saturating_sub(1);
                let truncated = truncate_with_ellipsis(title, title_max);
                let line = Line::from(vec![
                    Span::raw(indent.to_string()),
                    Span::styled(truncated, base_style),
                    Span::styled(counter, theme.dim_style()),
                ]);
                ListItem::new(line)
            }
        })
        .collect();

    List::new(items)
        .highlight_style(theme.highlight_style())
        .highlight_symbol(" ")
}

pub fn entries_list<'a>(entries: &'a [Entry], theme: &Theme, max_width: u16, lang: &Lang) -> List<'a> {
    let items: Vec<ListItem> = entries
        .iter()
        .map(|entry| {
            let title = entry
                .title
                .as_deref()
                .filter(|value| !value.is_empty())
                .unwrap_or(lang.no_title);
            let unread = entry.read_at.is_none();
            let saved = entry.saved_at.is_some();
            let date = entry
                .published_at
                .or(Some(entry.fetched_at))
                .map(|ts| format_timestamp_relative(ts, lang))
                .unwrap_or_default();
            let title_style = if unread {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let mut title_style = title_style;
            if saved {
                title_style = title_style.fg(theme.status_ok);
            }
            let prefix = if saved { lang.saved_marker } else { "" };
            let available = (max_width as usize).saturating_sub(1);
            let date_len = date.len();
            let title_max = available.saturating_sub(prefix.len()).saturating_sub(date_len).saturating_sub(1);
            let truncated = truncate_with_ellipsis(title, title_max);
            let padding = available.saturating_sub(prefix.len()).saturating_sub(truncated.chars().count()).saturating_sub(date_len);
            let mut spans = Vec::new();
            if saved {
                spans.push(Span::styled(lang.saved_marker, Style::default().fg(theme.status_ok)));
            }
            spans.push(Span::styled(truncated, title_style));
            spans.push(Span::raw(" ".repeat(padding)));
            spans.push(Span::styled(date, theme.dim_style()));
            let lines = vec![Line::from(spans)];
            ListItem::new(lines)
        })
        .collect();

    List::new(items)
        .highlight_style(theme.highlight_style())
        .highlight_symbol(" ")
}

pub struct PreviewParts<'a> {
    pub title: Line<'a>,
    pub meta: Line<'a>,
    pub body_lines: Vec<Line<'static>>,
    pub body_len: usize,
}

pub fn preview_parts<'a>(
    entry: Option<&'a Entry>,
    theme: &'a Theme,
    width: u16,
    lang: &Lang,
) -> PreviewParts<'a> {
    if let Some(entry) = entry {
        let title = entry
            .title
            .clone()
            .unwrap_or_else(|| lang.no_title.to_string());
        let title_line = Line::from(Span::styled(
            title,
            Style::default().add_modifier(Modifier::BOLD),
        ));

        let date = entry
            .published_at
            .or(Some(entry.fetched_at))
            .map(format_timestamp)
            .unwrap_or_else(|| "".to_string());
        let url = entry.url.clone().unwrap_or_default();
        let meta_text = match (date.is_empty(), url.is_empty()) {
            (false, false) => format!("{} | {}", date, url),
            (false, true) => date,
            (true, false) => url,
            (true, true) => String::new(),
        };
        let meta_line = Line::from(Span::styled(meta_text, theme.dim_style()));

        let body_html = entry
            .content
            .as_deref()
            .or(entry.summary.as_deref())
            .unwrap_or("");
        let render_width = width.saturating_sub(2).max(20) as usize;
        let tagged = to_rich_lines(body_html, render_width);
        let body_lines = rich_lines_to_ratatui(tagged, theme);
        let body_len = body_lines.len();

        PreviewParts {
            title: title_line,
            meta: meta_line,
            body_lines,
            body_len,
        }
    } else {
        PreviewParts {
            title: Line::from(lang.no_entry_selected),
            meta: Line::from(""),
            body_lines: vec![Line::from("")],
            body_len: 1,
        }
    }
}

pub fn panel_block<'a>(
    theme: &'a Theme,
    focused: bool,
    bg: Option<ratatui::style::Color>,
) -> Block<'a> {
    let base_style = if focused {
        theme.focus_block_style()
    } else if let Some(color) = bg {
        Style::default().bg(color)
    } else {
        theme.block_style()
    };

    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if focused {
            theme.focus_border_style()
        } else {
            theme.border_style()
        })
        .style(base_style)
}

pub fn preview_block<'a>(theme: &'a Theme, focused: bool) -> Block<'a> {
    panel_block(theme, focused, Some(theme.preview_bg))
}

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn status_bar<'a>(state: &AppState, theme: &Theme, lang: &Lang) -> Paragraph<'a> {
    let mut spans = Vec::new();
    let total = format!("{}: {}", lang.unread_label, state.total_unread);
    spans.push(Span::styled(total, Style::default().fg(theme.text)));

    if state.refreshing {
        let frame = SPINNER_FRAMES[state.tick % SPINNER_FRAMES.len()];
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("{} {}", frame, lang.refreshing),
            Style::default().fg(theme.accent_alt),
        ));
    }

    if let Some(status) = state.status.as_ref() {
        let color = if status.kind == StatusKind::Error {
            theme.status_err
        } else {
            theme.status_ok
        };
        spans.push(Span::raw("  |  "));
        spans.push(Span::styled(
            status.message.clone(),
            Style::default().fg(color),
        ));
    }

    spans.push(Span::raw("  |  "));
    spans.push(Span::styled(
        lang.status_bar_hint,
        theme.dim_style(),
    ));

    Paragraph::new(Line::from(spans))
        .block(Block::default().borders(Borders::TOP))
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(theme.text).bg(theme.header_bg))
}

pub fn modal<'a>(title: &'a str, text: Text<'a>, theme: &'a Theme) -> Paragraph<'a> {
    Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(title)
                .border_style(theme.focus_border_style())
                .title_style(theme.focus_title_style()),
        )
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(theme.text).bg(theme.block_bg))
}

pub fn header_bar<'a>(state: &'a AppState, theme: &Theme, lang: &Lang) -> Paragraph<'a> {
    let focus = match state.focus {
        crate::app::state::Focus::Feeds => lang.feeds,
        crate::app::state::Focus::Entries => lang.entries,
        crate::app::state::Focus::Preview => lang.preview,
    };
    let feed_title = state
        .selected_feed
        .and_then(|id| state.feeds.iter().find(|feed| feed.id == id))
        .and_then(|feed| {
            feed.title
                .as_deref()
                .filter(|value| !value.is_empty())
                .or(Some(feed.url.as_str()))
        })
        .unwrap_or(lang.no_feed_selected);
    let mut filters = Vec::new();
    if state.unread_only {
        filters.push(lang.filter_unread);
    }
    if state.saved_only {
        filters.push(lang.filter_saved);
    }
    let filter = if filters.is_empty() {
        lang.filter_all.to_string()
    } else {
        filters.join(" + ")
    };
    let mut spans = vec![
        Span::styled(
            lang.app_name,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  |  ", theme.dim_style()),
        Span::styled(lang.feed_label, theme.dim_style()),
        Span::styled(feed_title, Style::default().fg(theme.text)),
        Span::styled("  |  ", theme.dim_style()),
        Span::styled(lang.focus_label, theme.dim_style()),
        Span::styled(focus, Style::default().fg(theme.accent_alt)),
        Span::styled("  |  ", theme.dim_style()),
        Span::styled(lang.filter_label, theme.dim_style()),
        Span::styled(filter, Style::default().fg(theme.accent_alt)),
    ];
    if let Some(query) = state.search_query.as_deref()
        && !query.is_empty() {
            spans.push(Span::styled("  |  ", theme.dim_style()));
            spans.push(Span::styled(lang.search_label, theme.dim_style()));
            spans.push(Span::styled(query, Style::default().fg(theme.accent)));
        }

    Paragraph::new(Line::from(spans)).style(theme.header_style())
}

pub fn assign_group_modal_text(groups: &[Group], selection: usize, theme: &Theme, lang: &Lang) -> Text<'static> {
    let mut lines = vec![
        Line::from(lang.select_category.to_string()),
        Line::from(""),
    ];
    for (i, group) in groups.iter().enumerate() {
        let marker = if i == selection { "> " } else { "  " };
        let style = if i == selection {
            theme.highlight_style()
        } else {
            Style::default()
        };
        lines.push(Line::from(Span::styled(
            format!("{}{}", marker, group.name),
            style,
        )));
    }
    // "No category" option
    let idx = groups.len();
    let marker = if selection == idx { "> " } else { "  " };
    let style = if selection == idx {
        theme.highlight_style()
    } else {
        theme.dim_style()
    };
    lines.push(Line::from(Span::styled(
        format!("{}{}", marker, lang.no_category),
        style,
    )));
    // "New category..." option
    let idx = groups.len() + 1;
    let marker = if selection == idx { "> " } else { "  " };
    let style = if selection == idx {
        theme.highlight_style()
    } else {
        Style::default().fg(theme.accent_alt)
    };
    lines.push(Line::from(Span::styled(
        format!("{}{}", marker, lang.new_category_option),
        style,
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        lang.enter_confirm_esc_cancel.to_string(),
        theme.dim_style(),
    )));
    Text::from(lines)
}

pub fn manage_groups_modal_text(groups: &[Group], selection: usize, theme: &Theme, lang: &Lang) -> Text<'static> {
    let mut lines = vec![
        Line::from(lang.categories_title.trim().to_string()),
        Line::from(""),
    ];
    if groups.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("  {}", lang.no_categories),
            theme.dim_style(),
        )));
    } else {
        for (i, group) in groups.iter().enumerate() {
            let marker = if i == selection { "> " } else { "  " };
            let style = if i == selection {
                theme.highlight_style()
            } else {
                Style::default()
            };
            lines.push(Line::from(Span::styled(
                format!("{}{}", marker, group.name),
                style,
            )));
        }
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        lang.group_manage_hint.to_string(),
        theme.dim_style(),
    )));
    Text::from(lines)
}

pub fn selected_entry(entries: &[Entry], selected: Option<i64>) -> Option<&Entry> {
    selected.and_then(|id| entries.iter().find(|entry| entry.id == id))
}

fn truncate_with_ellipsis(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else if max <= 1 {
        "\u{2026}".to_string()
    } else {
        let mut result: String = chars[..max - 1].iter().collect();
        result.push('\u{2026}');
        result
    }
}

