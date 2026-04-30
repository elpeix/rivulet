pub mod layout;
pub mod rich_text;
pub mod theme;
pub mod widgets;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Clear, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};

use crate::app::state::AppState;
use crate::fetch::discovery::DiscoveredFeed;
use crate::i18n::Lang;
use crate::ui::layout::{build_layout, centered_rect};
use crate::ui::theme::Theme;
use crate::ui::widgets::{
    assign_group_modal_text, entries_list, feeds_list, manage_groups_modal_text, modal,
    panel_block, preview_block, preview_parts, selected_entry, status_bar, status_bar_height,
};

#[derive(Debug, Clone)]
pub enum Modal {
    Input {
        title: String,
        prompt: String,
        value: String,
        hint: Option<String>,
    },
    Confirm {
        title: String,
        prompt: String,
    },
    Help {
        scroll: u16,
    },
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
    FeedInfo {
        title: String,
        url: String,
    },
    Discovering,
    SelectDiscoveredFeed {
        feeds: Vec<DiscoveredFeed>,
        selection: usize,
    },
}

pub fn draw(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    theme: &Theme,
    modal_state: Option<Modal>,
    recent_days: i64,
    lang: &Lang,
) {
    let area = frame.area();
    let sh = status_bar_height(state, recent_days, lang, area.width);
    let layout = build_layout(
        area,
        state.layout_mode,
        state.panel_ratios,
        state.split_ratio,
        sh,
    );

    draw_feeds_panel(frame, state, theme, layout.feeds, lang);
    draw_entries_panel(frame, state, theme, layout.entries, lang);
    draw_preview_panel(frame, state, theme, layout.preview, lang);

    let status = status_bar(state, theme, recent_days, lang, area.width);
    frame.render_widget(status, layout.status);

    if let Some(modal_state) = modal_state {
        draw_modal(frame, state, theme, modal_state, lang);
    }
}

fn draw_feeds_panel(
    frame: &mut Frame<'_>,
    state: &AppState,
    theme: &Theme,
    area: Rect,
    lang: &Lang,
) {
    let mut feed_state = ListState::default();
    feed_state.select(state.selected_feed_row_index);
    let focused = state.focus == crate::app::state::Focus::Feeds;
    let searching = state.input_mode == crate::app::state::InputMode::PanelSearch
        && state.panel_search_focus == Some(crate::app::state::Focus::Feeds);
    let has_filter = state.feed_filter_query.is_some();
    let search_height = if searching || has_filter { 1 } else { 0 };
    let block = panel_block(theme, focused, Some(theme.feeds_bg));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(search_height),
        ])
        .split(inner);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw(" "),
            Span::styled(lang.feeds.as_str(), theme.section_title_style(focused)),
        ])),
        split[0],
    );
    render_separator(frame, theme, split[1]);
    let feeds = feeds_list(state, theme, split[2].width, lang);
    frame.render_stateful_widget(feeds, split[2], &mut feed_state);
    if searching || has_filter {
        let query = state.feed_filter_query.as_deref().unwrap_or("");
        render_search_bar(frame, theme, split[3], query, searching, None);
    }
}

fn draw_entries_panel(
    frame: &mut Frame<'_>,
    state: &AppState,
    theme: &Theme,
    area: Rect,
    lang: &Lang,
) {
    let mut entry_state = ListState::default();
    entry_state.select(state.selected_entry_index);
    let focused = state.focus == crate::app::state::Focus::Entries;
    let searching = state.input_mode == crate::app::state::InputMode::PanelSearch
        && state.panel_search_focus == Some(crate::app::state::Focus::Entries);
    let has_filter = state.search_query.is_some();
    let search_height = if searching || has_filter { 1 } else { 0 };
    let block = panel_block(theme, focused, None);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(search_height),
        ])
        .split(inner);

    let sort_label = match state.sort_mode {
        crate::app::state::SortMode::DateDesc => lang.sort_date_desc.as_str(),
        crate::app::state::SortMode::DateAsc => lang.sort_date_asc.as_str(),
        crate::app::state::SortMode::TitleAsc => lang.sort_title_asc.as_str(),
    };
    let title_style = theme.section_title_style(focused);
    let available = split[0].width.saturating_sub(1) as usize;
    let title_len = lang.entries.chars().count();
    let sort_len = sort_label.chars().count();
    let padding = available.saturating_sub(title_len + sort_len);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw(" "),
            Span::styled(lang.entries.as_str(), title_style),
            Span::raw(" ".repeat(padding)),
            Span::styled(sort_label, theme.dim_style()),
        ])),
        split[0],
    );
    render_separator(frame, theme, split[1]);

    let list_area = split[2];
    let has_scrollbar = state.entries.len() > list_area.height as usize;
    let entry_width = if has_scrollbar {
        list_area.width.saturating_sub(1)
    } else {
        list_area.width
    };
    let entries = entries_list(
        &state.entries,
        &state.feeds,
        state.viewing_group,
        theme,
        entry_width,
        lang,
        &state.selected_entries,
        state.search_query.as_deref(),
    );
    frame.render_stateful_widget(entries, list_area, &mut entry_state);

    if has_scrollbar {
        let scrollbar_area = Rect {
            x: list_area.x + list_area.width.saturating_sub(1),
            y: list_area.y,
            width: 1,
            height: list_area.height,
        };
        let mut scrollbar_state = ScrollbarState::new(state.entries.len())
            .position(state.selected_entry_index.unwrap_or(0));
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            scrollbar_area,
            &mut scrollbar_state,
        );
    }
    if searching || has_filter {
        let query = state.search_query.as_deref().unwrap_or("");
        render_search_bar(frame, theme, split[3], query, searching, None);
    }
}

fn draw_preview_panel(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    theme: &Theme,
    area: Rect,
    lang: &Lang,
) {
    let focused = state.focus == crate::app::state::Focus::Preview;
    let searching = state.input_mode == crate::app::state::InputMode::PanelSearch
        && state.panel_search_focus == Some(crate::app::state::Focus::Preview);
    let has_filter = state.preview_search_query.is_some();
    let search_height = if searching || has_filter { 1 } else { 0 };
    let block = preview_block(theme, focused);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(search_height),
        ])
        .split(inner);
    let preview_inner = split[2];
    let selected_link_url = state
        .selected_link_index
        .and_then(|i| state.preview_links.get(i))
        .cloned();
    let parts = preview_parts(
        selected_entry(&state.entries, state.selected_entry),
        theme,
        preview_inner.width,
        lang,
        selected_link_url.as_deref(),
    );
    let body_len = parts.body_len;
    state.preview_content_len = body_len;
    state.preview_links = parts.links;
    state.preview_link_regions = parts.link_regions;
    let visible_height = preview_inner.height as usize;
    let preview_title = if body_len > visible_height {
        let current_line = state.preview_scroll as usize + 1;
        lang.preview_panel_title(current_line, body_len)
    } else {
        lang.preview.to_string()
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw(" "),
            Span::styled(preview_title, theme.section_title_style(focused)),
        ])),
        split[0],
    );
    render_separator(frame, theme, split[1]);

    let title_height = parts.title_lines.len() as u16;
    let content_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(title_height),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(preview_inner);
    frame.render_widget(
        Paragraph::new(Text::from(parts.title_lines)),
        content_split[0],
    );
    frame.render_widget(Paragraph::new(parts.meta), content_split[1]);
    frame.render_widget(Paragraph::new(""), content_split[2]);
    state.preview_body_area = content_split[3];

    let mut body_lines = parts.body_lines;
    let highlight_style = Style::default().fg(theme.highlight_fg).bg(theme.accent);
    let active_style = Style::default().fg(theme.highlight_fg).bg(theme.accent_alt);
    if let Some(ref q) = state.preview_search_query {
        let matches = rich_text::highlight_search(
            &mut body_lines,
            q,
            highlight_style,
            active_style,
            state.preview_match_current,
        );
        state.update_preview_matches(matches);
    } else {
        state.update_preview_matches(Vec::new());
    }

    let body_text = Text::from(body_lines);
    frame.render_widget(
        Paragraph::new(body_text)
            .wrap(Wrap { trim: false })
            .scroll((state.preview_scroll, 0)),
        content_split[3],
    );

    if body_len > content_split[3].height as usize {
        let scrollbar_area = scrollbar_rect(content_split[3]);
        let mut scrollbar_state =
            ScrollbarState::new(body_len).position(state.preview_scroll as usize);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            scrollbar_area,
            &mut scrollbar_state,
        );
    }

    if searching || has_filter {
        let query = state.preview_search_query.as_deref().unwrap_or("");
        let match_info = if !state.preview_match_lines.is_empty() {
            let cur = state.preview_match_current.map(|i| i + 1).unwrap_or(0);
            Some((cur, state.preview_match_lines.len()))
        } else if !query.is_empty() {
            Some((0, 0))
        } else {
            None
        };
        render_search_bar(frame, theme, split[3], query, searching, match_info);
    }
}

fn draw_modal(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    theme: &Theme,
    modal_state: Modal,
    lang: &Lang,
) {
    let area = centered_rect(70, 60, frame.area());
    frame.render_widget(Clear, area);
    match modal_state {
        Modal::Input {
            title,
            prompt,
            value,
            hint,
        } => {
            let cursor = Span::styled("_", theme.highlight_style());
            let input_line = Line::from(vec![Span::raw(value), cursor]);
            let mut lines = vec![
                Line::from(""),
                Line::from(prompt),
                input_line,
                Line::from(""),
            ];
            if let Some(hint) = hint {
                lines.push(Line::from(Span::styled(hint, theme.dim_style())));
                lines.push(Line::from(""));
            }
            lines.push(Line::from(lang.enter_confirm_esc_cancel.as_str()));
            let text = Text::from(lines);
            frame.render_widget(modal(&title, text, theme), area);
        }
        Modal::Confirm { title, prompt } => {
            let text = Text::from(vec![
                Line::from(prompt),
                Line::from(""),
                Line::from(lang.y_confirm_n_cancel.as_str()),
            ]);
            frame.render_widget(modal(&title, text, theme), area);
        }
        Modal::AssignGroup { selection } => {
            let text = assign_group_modal_text(&state.groups, selection, theme, lang);
            frame.render_widget(modal(&lang.category_title, text, theme), area);
        }
        Modal::ManageGroups { selection } => {
            let text = manage_groups_modal_text(&state.groups, selection, theme, lang);
            frame.render_widget(modal(&lang.categories_title, text, theme), area);
        }
        Modal::GroupInput { title, value } => {
            let cursor = Span::styled("_", theme.highlight_style());
            let input_line = Line::from(vec![Span::raw(value), cursor]);
            let text = Text::from(vec![
                Line::from(lang.name_label.as_str()),
                input_line,
                Line::from(""),
                Line::from(lang.enter_confirm_esc_cancel.as_str()),
            ]);
            frame.render_widget(modal(&title, text, theme), area);
        }
        Modal::Help { scroll } => {
            draw_help_modal(frame, state, theme, area, scroll, lang);
        }
        Modal::FeedInfo { title, url } => {
            let text = Text::from(vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled(format!("{}: ", lang.name_label), theme.dim_style()),
                    Span::raw(title),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(format!("{}: ", lang.url_label), theme.dim_style()),
                    Span::raw(url),
                ]),
                Line::from(""),
                Line::from(Span::styled("Esc", theme.dim_style())),
            ]);
            frame.render_widget(modal(&lang.feed_info_title, text, theme), area);
        }
        Modal::Discovering => {
            let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let frame_char = spinner[state.tick % spinner.len()];
            let text = Text::from(vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        format!("{frame_char} "),
                        Style::default().fg(theme.accent_alt),
                    ),
                    Span::raw(lang.discovering.as_str()),
                ]),
            ]);
            frame.render_widget(modal(&lang.add_feed_title, text, theme), area);
        }
        Modal::SelectDiscoveredFeed { feeds, selection } => {
            let mut lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    lang.select_feed_prompt.to_string(),
                    theme.dim_style(),
                )),
                Line::from(""),
            ];
            for (i, feed) in feeds.iter().enumerate() {
                let marker = if i == selection { " \u{25b6} " } else { "   " };
                let style = if i == selection {
                    theme.highlight_style()
                } else {
                    Style::default()
                };
                let label = if let Some(title) = &feed.title {
                    format!("{title} ({}) - {}", feed.feed_type, feed.url)
                } else {
                    format!("({}) {}", feed.feed_type, feed.url)
                };
                lines.push(Line::from(vec![
                    Span::styled(marker.to_string(), style),
                    Span::styled(label, style),
                ]));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(lang.enter_confirm_esc_cancel.as_str()));
            let text = Text::from(lines);
            frame.render_widget(modal(&lang.select_feed_title, text, theme), area);
        }
    }
}

fn draw_help_modal(
    frame: &mut Frame<'_>,
    state: &mut AppState,
    theme: &Theme,
    area: Rect,
    scroll: u16,
    lang: &Lang,
) {
    let inner_width = area.width.saturating_sub(2) as usize;
    let separator = Line::from(Span::styled(
        "\u{2500}".repeat(inner_width),
        theme.dim_style(),
    ));
    let heading = |label: &str| {
        Line::from(Span::styled(
            format!(" {label}"),
            theme.section_title_style(true),
        ))
    };
    let col_width = inner_width / 2;
    let row = |left_key: &str, left_label: &str, right_key: &str, right_label: &str| {
        Line::from(vec![
            Span::styled(
                format!("  {left_key:<14}"),
                Style::default().fg(theme.accent_alt),
            ),
            Span::styled(
                format!(
                    "{:<width$}",
                    left_label,
                    width = col_width.saturating_sub(16)
                ),
                Style::default().fg(theme.text),
            ),
            Span::styled(
                format!("{right_key:<14}"),
                Style::default().fg(theme.accent_alt),
            ),
            Span::styled(right_label.to_string(), Style::default().fg(theme.text)),
        ])
    };
    let text = Text::from(vec![
        Line::from(""),
        heading(&lang.help_navigation),
        separator.clone(),
        row(
            "Left/Right",
            &lang.help_move_panel,
            "Up/Down",
            &lang.help_move_selection,
        ),
        row(
            "PgUp/PgDn",
            &lang.help_scroll_preview,
            "Home/End g/G",
            &lang.help_top_bottom,
        ),
        row(
            "H / L",
            &lang.help_resize_panel,
            "Enter",
            &lang.help_select_open,
        ),
        row(
            "Space",
            &lang.help_collapse_category,
            "w",
            &lang.help_toggle_layout,
        ),
        row("1 / 2 / 3", &lang.help_jump_panel, "Esc", &lang.help_back),
        Line::from(""),
        heading(&lang.help_feeds),
        separator.clone(),
        row("a", &lang.help_add_feed, "e", &lang.help_rename_feed),
        row("d", &lang.help_delete_feed, "r", &lang.help_refresh_all),
        row("f", &lang.help_toggle_unread, "b", &lang.help_toggle_saved),
        row(
            "c",
            &lang.help_assign_category,
            "C",
            &lang.help_manage_categories,
        ),
        row("R", &lang.help_mark_feed_read, "S", &lang.help_cycle_sort),
        row("i", &lang.help_feed_info, ".", &lang.help_toggle_read_feeds),
        row("t", &lang.help_toggle_time, "", ""),
        Line::from(""),
        heading(&lang.help_entries),
        separator.clone(),
        row(
            "Space",
            &lang.help_select_entry,
            "m",
            &lang.help_toggle_read,
        ),
        row("M", &lang.help_mark_all_read, "s", &lang.help_save_later),
        row("x", &lang.help_clear_read, "", ""),
        row("/", &lang.help_search, "n / N", &lang.help_search_next),
        row("o", &lang.help_open_browser, "y", &lang.help_copy_preview),
        row(
            "Tab",
            &lang.help_next_link,
            "Shift-Tab",
            &lang.help_prev_link,
        ),
        row(
            "r",
            &lang.help_refresh_feed,
            "F5",
            &lang.help_reload_entries,
        ),
        Line::from(""),
        heading(&lang.help_general),
        separator,
        row("?", &lang.help_toggle_help, "q", &lang.help_quit),
    ]);
    let content_height = text.lines.len();
    let inner_height = area.height.saturating_sub(2) as usize;
    let max_scroll = u16::try_from(content_height.saturating_sub(inner_height)).unwrap_or(u16::MAX);
    state.help_max_scroll = max_scroll;
    let clamped = scroll.min(max_scroll);
    let title = if content_height > inner_height {
        let has_up = clamped > 0;
        let has_down = clamped < max_scroll;
        match (has_up, has_down) {
            (true, true) => format!("{} ▲▼", lang.help_title),
            (true, false) => format!("{} ▲", lang.help_title),
            (false, true) => format!("{} ▼", lang.help_title),
            (false, false) => lang.help_title.to_string(),
        }
    } else {
        lang.help_title.to_string()
    };
    frame.render_widget(modal(&title, text, theme).scroll((clamped, 0)), area);
}

fn render_separator(frame: &mut Frame<'_>, theme: &Theme, area: Rect) {
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "-".repeat(area.width.saturating_sub(1) as usize),
            theme.dim_style(),
        )))
        .alignment(Alignment::Center),
        area,
    );
}

fn scrollbar_rect(area: Rect) -> Rect {
    Rect {
        x: area.x + area.width.saturating_sub(1),
        y: area.y,
        width: 1,
        height: area.height,
    }
}

fn render_search_bar(
    frame: &mut Frame<'_>,
    theme: &Theme,
    area: Rect,
    query: &str,
    is_typing: bool,
    match_info: Option<(usize, usize)>,
) {
    let mut spans = vec![Span::styled("/", Style::default().fg(theme.accent))];
    spans.push(Span::raw(query.to_string()));
    if is_typing {
        spans.push(Span::styled("_", theme.highlight_style()));
    }
    if let Some((current, total)) = match_info {
        let info = if total == 0 {
            " [0]".to_string()
        } else {
            format!(" [{}/{}]", current, total)
        };
        spans.push(Span::styled(info, Style::default().fg(theme.dim)));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
