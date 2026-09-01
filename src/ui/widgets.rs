use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap};

use std::collections::HashSet;

use crate::app::state::{AppState, FeedRow, StatusKind};
use crate::i18n::Lang;
use crate::store::models::{Entry, Feed, Group};
use crate::ui::rich_text::{LinkRegion, rich_lines_to_ratatui};
use crate::ui::theme::Theme;
use crate::util::html::{extract_links, to_rich_lines};
use crate::util::time::{format_timestamp, format_timestamp_relative};
use unicode_width::UnicodeWidthStr;

fn str_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// Split text into spans, highlighting case-insensitive matches of `query`.
/// Works correctly with multi-byte characters where `to_lowercase()` may change
/// byte length (e.g. Turkish İ) by building a mapping from lowercase byte offsets
/// back to original string byte offsets.
pub fn highlight_spans(
    text: &str,
    query: &str,
    normal: Style,
    highlight: Style,
) -> Vec<Span<'static>> {
    if query.is_empty() {
        return vec![Span::styled(text.to_string(), normal)];
    }
    let lower_query = query.to_lowercase();

    // Build lowercase text and a mapping: for each byte in lower_text,
    // record which byte offset in `text` the corresponding char starts at.
    let mut lower_text = String::new();
    // Maps byte offset in lower_text → byte offset in text (one entry per char).
    let mut lower_to_orig: Vec<(usize, usize)> = Vec::new();
    for (orig_offset, ch) in text.char_indices() {
        let lower_start = lower_text.len();
        let orig_len = ch.len_utf8();
        for lc in ch.to_lowercase() {
            lower_text.push(lc);
        }
        let lower_len = lower_text.len() - lower_start;
        lower_to_orig.push((lower_start, orig_offset));
        // Also store end sentinel for the last char
        lower_to_orig.push((lower_start + lower_len, orig_offset + orig_len));
    }
    // Deduplicate sentinel entries and ensure we can look up any lower offset
    lower_to_orig.sort_unstable();
    lower_to_orig.dedup();

    let map_offset = |lower_off: usize| -> usize {
        match lower_to_orig.binary_search_by_key(&lower_off, |&(lo, _)| lo) {
            Ok(i) => lower_to_orig[i].1,
            Err(i) if i > 0 => lower_to_orig[i - 1].1,
            _ => 0,
        }
    };

    let mut spans = Vec::new();
    let mut last_orig = 0;
    for (lower_start, _) in lower_text.match_indices(&lower_query) {
        let lower_end = lower_start + lower_query.len();
        let orig_start = map_offset(lower_start);
        let orig_end = map_offset(lower_end);
        if orig_start > last_orig {
            spans.push(Span::styled(
                text[last_orig..orig_start].to_string(),
                normal,
            ));
        }
        spans.push(Span::styled(
            text[orig_start..orig_end].to_string(),
            highlight,
        ));
        last_orig = orig_end;
    }
    if last_orig < text.len() {
        spans.push(Span::styled(text[last_orig..].to_string(), normal));
    }
    if spans.is_empty() {
        spans.push(Span::styled(text.to_string(), normal));
    }
    spans
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_width = 0;

    for word in text.split_whitespace() {
        let word_width = str_width(word);
        if current.is_empty() {
            current = word.to_string();
            current_width = word_width;
        } else if current_width + 1 + word_width <= max_width {
            current.push(' ');
            current.push_str(word);
            current_width += 1 + word_width;
        } else {
            lines.push(current);
            current = word.to_string();
            current_width = word_width;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

pub fn feeds_list<'a>(
    state: &AppState,
    theme: &Theme,
    area_width: u16,
    lang: &'a Lang,
) -> List<'a> {
    let highlight_symbol = " ";
    let max_width = (area_width as usize).saturating_sub(str_width(highlight_symbol));
    let items: Vec<ListItem> = state
        .feed_rows
        .iter()
        .filter_map(|row| match row {
            FeedRow::AllFeeds => {
                let all_unread = state.all_feeds_unread();
                let counter = if all_unread > 0 {
                    format!("{all_unread}")
                } else {
                    String::new()
                };
                let available = max_width;
                let prefix = "\u{2605} ";
                let title_max = available
                    .saturating_sub(str_width(prefix))
                    .saturating_sub(str_width(&counter));
                let truncated = truncate_with_ellipsis(&lang.all_feeds, title_max);
                let used = str_width(prefix) + str_width(&truncated) + str_width(&counter);
                let gap = available.saturating_sub(used);
                let line = Line::from(vec![
                    Span::styled(
                        format!("{prefix}{truncated}"),
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" ".repeat(gap)),
                    Span::styled(counter, theme.dim_style()),
                ]);
                Some(ListItem::new(line))
            }
            FeedRow::GroupHeader {
                name,
                unread,
                group_id,
            } => {
                let collapsed = state.collapsed_groups.contains(group_id);
                let arrow = if collapsed { "\u{25b6}" } else { "\u{25bc}" };
                let counter = if *unread > 0 {
                    format!("{unread}")
                } else {
                    String::new()
                };
                let available = max_width;
                let prefix = format!("{arrow} ");
                let title_max = available
                    .saturating_sub(str_width(&prefix))
                    .saturating_sub(str_width(&counter));
                let truncated = truncate_with_ellipsis(name, title_max);
                let used = str_width(&prefix) + str_width(&truncated) + str_width(&counter);
                let gap = available.saturating_sub(used);
                let line = Line::from(vec![
                    Span::styled(
                        format!("{prefix}{truncated}"),
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" ".repeat(gap)),
                    Span::styled(counter, theme.dim_style()),
                ]);
                Some(ListItem::new(line))
            }
            FeedRow::UngroupedHeader { unread } => {
                let counter = if *unread > 0 {
                    format!("{unread}")
                } else {
                    String::new()
                };
                let available = max_width;
                let prefix = "\u{25bc} ";
                let title_max = available
                    .saturating_sub(str_width(prefix))
                    .saturating_sub(str_width(&counter));
                let truncated = truncate_with_ellipsis(&lang.uncategorized, title_max);
                let used = str_width(prefix) + str_width(&truncated) + str_width(&counter);
                let gap = available.saturating_sub(used);
                let line = Line::from(vec![
                    Span::styled(
                        format!("{prefix}{truncated}"),
                        Style::default().fg(theme.dim).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" ".repeat(gap)),
                    Span::styled(counter, theme.dim_style()),
                ]);
                Some(ListItem::new(line))
            }
            FeedRow::FeedItem { feed_index } => {
                let feed = state.feeds.get(*feed_index)?;
                let unread = state.unread_counts.get(&feed.id).copied().unwrap_or(0);
                let title = feed
                    .display_title()
                    .filter(|value| !value.is_empty())
                    .unwrap_or(feed.url.as_str());
                let base_style = if unread > 0 {
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text)
                };
                let counter = if unread > 0 {
                    format!("{unread}")
                } else {
                    String::new()
                };
                let has_groups = !state.groups.is_empty();
                let indent = if has_groups { "  " } else { "" };
                let available = max_width;
                let title_max = available
                    .saturating_sub(str_width(&counter))
                    .saturating_sub(str_width(indent));
                let truncated = truncate_with_ellipsis(title, title_max);
                let used = str_width(indent) + str_width(&truncated) + str_width(&counter);
                let gap = available.saturating_sub(used);
                let match_style = Style::default().fg(theme.highlight_fg).bg(theme.accent);
                let mut spans = vec![Span::raw(indent.to_string())];
                if let Some(ref q) = state.feed_filter_query {
                    spans.extend(highlight_spans(&truncated, q, base_style, match_style));
                } else {
                    spans.push(Span::styled(truncated, base_style));
                }
                spans.push(Span::raw(" ".repeat(gap)));
                spans.push(Span::styled(counter, theme.dim_style()));
                let line = Line::from(spans);
                Some(ListItem::new(line))
            }
        })
        .collect();

    List::new(items)
        .highlight_style(theme.highlight_style())
        .highlight_symbol(highlight_symbol)
}

#[allow(clippy::too_many_arguments)]
pub fn entries_list<'a>(
    entries: &'a [Entry],
    feeds: &'a [Feed],
    show_feed: bool,
    theme: &Theme,
    max_width: u16,
    lang: &'a Lang,
    selected_entries: &HashSet<i64>,
    search_query: Option<&str>,
) -> List<'a> {
    // Pre-compute feed name column width when showing feeds
    let feed_col_width = if show_feed {
        let max_name = entries
            .iter()
            .filter_map(|e| {
                feeds
                    .iter()
                    .find(|f| f.id == e.feed_id)
                    .and_then(|f| f.display_title())
                    .map(|t| t.chars().count())
            })
            .max()
            .unwrap_or(0)
            .min(16);
        max_name + 1 // +1 for separator space
    } else {
        0
    };

    let items: Vec<ListItem> = entries
        .iter()
        .map(|entry| {
            let title = entry
                .title
                .as_deref()
                .filter(|value| !value.is_empty())
                .unwrap_or(lang.no_title.as_str());
            let unread = entry.read_at.is_none();
            let saved = entry.saved_at.is_some();
            let is_selected = selected_entries.contains(&entry.id);
            let date = entry
                .published_at
                .or(Some(entry.fetched_at))
                .map(|ts| format_timestamp_relative(ts, lang))
                .unwrap_or_default();
            let title_style = if unread {
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };
            let mut title_style = title_style;
            if saved {
                title_style = title_style.fg(theme.status_ok);
            }
            let prefix = if saved {
                lang.saved_marker.as_str()
            } else {
                ""
            };
            let select_marker = if is_selected { "\u{258c}" } else { " " };
            let select_width = str_width(select_marker);
            let available = (max_width as usize).saturating_sub(1);
            let date_len = date.len();

            let mut spans = Vec::new();

            spans.push(Span::styled(
                select_marker,
                if is_selected {
                    Style::default().fg(theme.accent)
                } else {
                    Style::default()
                },
            ));

            if show_feed && feed_col_width > 0 {
                let feed_name = feeds
                    .iter()
                    .find(|f| f.id == entry.feed_id)
                    .and_then(|f| f.display_title())
                    .unwrap_or("?");
                let truncated_feed = truncate_with_ellipsis(feed_name, feed_col_width - 1);
                let pad = feed_col_width.saturating_sub(truncated_feed.chars().count());
                spans.push(Span::styled(
                    format!("{}{}", truncated_feed, " ".repeat(pad)),
                    Style::default().fg(theme.accent_alt),
                ));
            }

            if saved {
                spans.push(Span::styled(
                    lang.saved_marker.as_str(),
                    Style::default().fg(theme.status_ok),
                ));
            }
            let prefix_width = prefix.chars().count();
            let title_max = available
                .saturating_sub(select_width)
                .saturating_sub(feed_col_width)
                .saturating_sub(prefix_width)
                .saturating_sub(date_len)
                .saturating_sub(1);
            let truncated = truncate_with_ellipsis(title, title_max);
            let padding = available
                .saturating_sub(select_width)
                .saturating_sub(feed_col_width)
                .saturating_sub(prefix_width)
                .saturating_sub(truncated.chars().count())
                .saturating_sub(date_len);
            if let Some(q) = search_query {
                let match_style = Style::default().fg(theme.highlight_fg).bg(theme.accent);
                let padded = format!(
                    "{:<width$}",
                    truncated,
                    width = truncated.chars().count() + padding
                );
                spans.extend(highlight_spans(&padded, q, title_style, match_style));
            } else {
                spans.push(Span::styled(truncated, title_style));
                spans.push(Span::raw(" ".repeat(padding)));
            }
            spans.push(Span::styled(date, theme.dim_style()));
            let lines = vec![Line::from(spans)];
            let mut item = ListItem::new(lines);
            if is_selected {
                item = item.style(Style::default().bg(theme.selection_bg));
            }
            item
        })
        .collect();

    List::new(items)
        .highlight_style(theme.highlight_style())
        .highlight_symbol(" ")
}

pub struct PreviewParts<'a> {
    pub title_lines: Vec<Line<'a>>,
    pub meta: Line<'a>,
    pub body_lines: Vec<Line<'static>>,
    pub body_len: usize,
    pub links: Vec<String>,
    pub link_regions: Vec<LinkRegion>,
}

pub fn preview_parts<'a>(
    entry: Option<&'a Entry>,
    theme: &'a Theme,
    width: u16,
    lang: &'a Lang,
    selected_link_url: Option<&str>,
) -> PreviewParts<'a> {
    if let Some(entry) = entry {
        let title = entry
            .title
            .clone()
            .unwrap_or_else(|| lang.no_title.to_string());
        let title_style = Style::default().fg(theme.text).add_modifier(Modifier::BOLD);
        let title_width = width.saturating_sub(2).max(20) as usize;
        let title_lines = wrap_text(&title, title_width)
            .into_iter()
            .map(|s| Line::from(Span::styled(s, title_style)))
            .collect::<Vec<_>>();

        let date = entry
            .published_at
            .or(Some(entry.fetched_at))
            .map(format_timestamp)
            .unwrap_or_default();
        let author = entry.author.clone().unwrap_or_default();
        let url = entry.url.clone().unwrap_or_default();
        let other_len = [date.as_str(), author.as_str()]
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| str_width(s) + 3) // " | " separator
            .sum::<usize>();
        let url_max = title_width.saturating_sub(other_len);
        let url_truncated = truncate_with_ellipsis(&url, url_max);
        let meta_parts: Vec<&str> = [date.as_str(), author.as_str(), url_truncated.as_str()]
            .iter()
            .filter(|s| !s.is_empty())
            .copied()
            .collect();
        let meta_line = Line::from(Span::styled(meta_parts.join(" | "), theme.dim_style()));

        let body_html = entry
            .content
            .as_deref()
            .or(entry.summary.as_deref())
            .unwrap_or("");
        let render_width = width.saturating_sub(2).max(20) as usize;
        let tagged = to_rich_lines(body_html, render_width);
        let links = extract_links(&tagged);
        let rich = rich_lines_to_ratatui(tagged, theme, selected_link_url);
        let body_len = rich.lines.len();

        PreviewParts {
            title_lines,
            meta: meta_line,
            body_lines: rich.lines,
            body_len,
            links,
            link_regions: rich.link_regions,
        }
    } else {
        PreviewParts {
            title_lines: vec![Line::from(lang.no_entry_selected.as_str())],
            meta: Line::from(""),
            body_lines: vec![Line::from("")],
            body_len: 1,
            links: Vec::new(),
            link_regions: Vec::new(),
        }
    }
}

pub fn panel_block(theme: &Theme, focused: bool, bg: Option<ratatui::style::Color>) -> Block<'_> {
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

pub fn preview_block(theme: &Theme, focused: bool) -> Block<'_> {
    panel_block(theme, focused, Some(theme.preview_bg))
}

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn status_bar<'a>(
    state: &'a AppState,
    theme: &Theme,
    recent_days: i64,
    lang: &'a Lang,
    width: u16,
) -> Paragraph<'a> {
    // Left spans: app name + version | feed | filter | search
    let version = env!("CARGO_PKG_VERSION");
    let feed_title = state
        .selected_feed
        .and_then(|id| state.feeds.iter().find(|feed| feed.id == id))
        .and_then(|feed| {
            feed.display_title()
                .filter(|value| !value.is_empty())
                .or(Some(feed.url.as_str()))
        })
        .unwrap_or(lang.no_feed_selected.as_str());
    let mut filters: Vec<String> = Vec::new();
    if state.unread_only {
        filters.push(lang.filter_unread.to_string());
    }
    if state.saved_only {
        filters.push(lang.filter_saved.to_string());
    }
    if state.recent_only {
        filters.push(lang.filter_recent_days(recent_days));
    }
    if state.hide_read_feeds {
        filters.push(lang.filter_hide_read.to_string());
    }
    let filter = if filters.is_empty() {
        lang.filter_all.to_string()
    } else {
        filters.join(" + ")
    };
    let mut left_spans = vec![
        Span::styled(
            format!(" {} v{}", lang.app_name, version),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  |  ", theme.dim_style()),
        Span::styled(lang.feed_label.as_str(), theme.dim_style()),
        Span::styled(feed_title, Style::default().fg(theme.text)),
        Span::styled("  |  ", theme.dim_style()),
        Span::styled(lang.filter_label.as_str(), theme.dim_style()),
        Span::styled(filter, Style::default().fg(theme.accent_alt)),
    ];
    if let Some(query) = state.search_query.as_deref()
        && !query.is_empty()
    {
        left_spans.push(Span::styled("  |  ", theme.dim_style()));
        left_spans.push(Span::styled(lang.search_label.as_str(), theme.dim_style()));
        left_spans.push(Span::styled(query, Style::default().fg(theme.accent)));
    }

    // Right spans: refreshing | unread | status | help hint
    let mut right_spans = Vec::new();

    if state.refreshing {
        let frame = SPINNER_FRAMES[state.tick % SPINNER_FRAMES.len()];
        right_spans.push(Span::styled(
            format!("{} {}", frame, lang.refreshing),
            Style::default().fg(theme.accent_alt),
        ));
        right_spans.push(Span::styled("  |  ", theme.dim_style()));
    }

    if let Some(status) = state.status.as_ref() {
        let color = if status.kind == StatusKind::Error {
            theme.status_err
        } else {
            theme.status_ok
        };
        right_spans.push(Span::styled(
            status.message.clone(),
            Style::default().fg(color),
        ));
        right_spans.push(Span::styled("  |  ", theme.dim_style()));
    }

    let total = format!("{}: {}", lang.unread_label, state.total_unread);
    right_spans.push(Span::styled(total, Style::default().fg(theme.text)));

    right_spans.push(Span::raw("  |  "));
    right_spans.push(Span::styled(
        lang.status_bar_hint.as_str(),
        theme.dim_style(),
    ));

    let left_width: usize = left_spans.iter().map(|s| s.width()).sum();
    let right_width: usize = right_spans.iter().map(|s| s.width()).sum();
    let available = width.saturating_sub(2) as usize;
    let single_line = fits_single_line(left_width, right_width, available);

    let lines = if single_line {
        let padding = available.saturating_sub(left_width + right_width);
        left_spans.push(Span::raw(" ".repeat(padding)));
        left_spans.extend(right_spans);
        vec![Line::from(left_spans)]
    } else {
        vec![Line::from(left_spans), Line::from(right_spans)]
    };

    Paragraph::new(lines).style(Style::default().fg(theme.text).bg(theme.header_bg))
}

fn fits_single_line(left_width: usize, right_width: usize, available: usize) -> bool {
    left_width + right_width + 2 <= available
}

pub fn status_bar_height(state: &AppState, recent_days: i64, lang: &Lang, width: u16) -> u16 {
    let version = env!("CARGO_PKG_VERSION");
    let feed_title = state
        .selected_feed
        .and_then(|id| state.feeds.iter().find(|feed| feed.id == id))
        .and_then(|feed| {
            feed.display_title()
                .filter(|value| !value.is_empty())
                .or(Some(feed.url.as_str()))
        })
        .unwrap_or(lang.no_feed_selected.as_str());
    let mut left_w = format!(" {} v{}", lang.app_name, version).len()
        + "  |  ".len()
        + lang.feed_label.len()
        + feed_title.len()
        + "  |  ".len()
        + lang.filter_label.len();
    let mut filter_parts: Vec<usize> = Vec::new();
    if state.unread_only {
        filter_parts.push(lang.filter_unread.len());
    }
    if state.saved_only {
        filter_parts.push(lang.filter_saved.len());
    }
    if state.recent_only {
        filter_parts.push(lang.filter_recent_days(recent_days).len());
    }
    if state.hide_read_feeds {
        filter_parts.push(lang.filter_hide_read.len());
    }
    if filter_parts.is_empty() {
        left_w += lang.filter_all.len();
    } else {
        let separators = " + ".len() * filter_parts.len().saturating_sub(1);
        left_w += filter_parts.iter().sum::<usize>() + separators;
    }
    if let Some(query) = state.search_query.as_deref()
        && !query.is_empty()
    {
        left_w += "  |  ".len() + lang.search_label.len() + query.len();
    }

    let mut right_w = format!("{}: {}", lang.unread_label, state.total_unread).len();
    if state.refreshing {
        right_w += 2 + 2 + lang.refreshing.len(); // "  " + spinner + " " + text
    }
    if let Some(status) = &state.status {
        right_w += "  |  ".len() + status.message.len();
    }
    right_w += "  |  ".len() + lang.status_bar_hint.len();

    let available = width.saturating_sub(2) as usize;
    if fits_single_line(left_w, right_w, available) {
        1
    } else {
        2
    }
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

pub fn assign_group_modal_text(
    groups: &[Group],
    selection: usize,
    theme: &Theme,
    lang: &Lang,
) -> Text<'static> {
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            lang.select_category.to_string(),
            theme.dim_style(),
        )),
        Line::from(""),
    ];
    for (i, group) in groups.iter().enumerate() {
        let marker = if i == selection { " \u{25b6} " } else { "   " };
        let style = if i == selection {
            theme.highlight_style()
        } else {
            Style::default().fg(theme.text)
        };
        lines.push(Line::from(vec![
            Span::styled(marker.to_string(), style),
            Span::styled(group.name.clone(), style),
        ]));
    }
    // Separator
    if !groups.is_empty() {
        lines.push(Line::from(""));
    }
    // "No category" option
    let idx = groups.len();
    let marker = if selection == idx {
        " \u{25b6} "
    } else {
        "   "
    };
    let style = if selection == idx {
        theme.highlight_style()
    } else {
        theme.dim_style()
    };
    lines.push(Line::from(vec![
        Span::styled(marker.to_string(), style),
        Span::styled(lang.no_category.to_string(), style),
    ]));
    // "New category..." option
    let idx = groups.len() + 1;
    let marker = if selection == idx {
        " \u{25b6} "
    } else {
        "   "
    };
    let style = if selection == idx {
        theme.highlight_style()
    } else {
        Style::default().fg(theme.accent_alt)
    };
    lines.push(Line::from(vec![
        Span::styled(marker.to_string(), style),
        Span::styled(lang.new_category_option.to_string(), style),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        lang.enter_confirm_esc_cancel.to_string(),
        theme.dim_style(),
    )));
    Text::from(lines)
}

pub fn manage_groups_modal_text(
    groups: &[Group],
    selection: usize,
    theme: &Theme,
    lang: &Lang,
) -> Text<'static> {
    let mut lines = vec![Line::from("")];
    if groups.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("  {}", lang.no_categories),
            theme.dim_style(),
        )));
    } else {
        for (i, group) in groups.iter().enumerate() {
            let marker = if i == selection { " \u{25b6} " } else { "   " };
            let style = if i == selection {
                theme.highlight_style()
            } else {
                Style::default().fg(theme.text)
            };
            let pos = format!("{}. ", i + 1);
            lines.push(Line::from(vec![
                Span::styled(marker.to_string(), style),
                Span::styled(pos, theme.dim_style()),
                Span::styled(group.name.clone(), style),
            ]));
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
    use unicode_width::UnicodeWidthChar;
    if max == 0 {
        return String::new();
    }
    if str_width(s) <= max {
        return s.to_string();
    }
    if max <= 1 {
        return "\u{2026}".to_string();
    }
    let mut result = String::new();
    let mut width = 0;
    for ch in s.chars() {
        let cw = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + cw + 1 > max {
            break;
        }
        result.push(ch);
        width += cw;
    }
    result.push('\u{2026}');
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_ascii_within_limit() {
        assert_eq!(truncate_with_ellipsis("hello", 10), "hello");
    }

    #[test]
    fn truncate_ascii_exact_limit() {
        assert_eq!(truncate_with_ellipsis("hello", 5), "hello");
    }

    #[test]
    fn truncate_ascii_over_limit() {
        let result = truncate_with_ellipsis("hello world", 6);
        assert_eq!(str_width(&result), 6);
        assert!(result.ends_with('\u{2026}'));
    }

    #[test]
    fn truncate_empty_max() {
        assert_eq!(truncate_with_ellipsis("hello", 0), "");
    }

    #[test]
    fn truncate_max_one() {
        assert_eq!(truncate_with_ellipsis("hello", 1), "\u{2026}");
    }

    #[test]
    fn truncate_unicode_width() {
        // Star (★) is 1 column wide, but 3 bytes
        let s = "★ Feed Name";
        let result = truncate_with_ellipsis(s, 8);
        assert!(str_width(&result) <= 8);
        assert!(result.ends_with('\u{2026}'));
    }

    #[test]
    fn str_width_unicode() {
        // ★ = 1 column, ▶ = 1 column
        assert_eq!(str_width("\u{2605} "), 2);
        assert_eq!(str_width("\u{25b6} "), 2);
        assert_eq!(str_width("abc"), 3);
    }

    #[test]
    fn highlight_spans_no_match() {
        let style = Style::default();
        let hl = Style::default().fg(ratatui::style::Color::Red);
        let spans = highlight_spans("hello world", "xyz", style, hl);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content.as_ref(), "hello world");
    }

    #[test]
    fn highlight_spans_single_match() {
        let style = Style::default();
        let hl = Style::default().fg(ratatui::style::Color::Red);
        let spans = highlight_spans("hello world", "world", style, hl);
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].content.as_ref(), "hello ");
        assert_eq!(spans[0].style, style);
        assert_eq!(spans[1].content.as_ref(), "world");
        assert_eq!(spans[1].style, hl);
    }

    #[test]
    fn highlight_spans_multiple_matches() {
        let style = Style::default();
        let hl = Style::default().fg(ratatui::style::Color::Red);
        let spans = highlight_spans("abcabc", "abc", style, hl);
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].content.as_ref(), "abc");
        assert_eq!(spans[0].style, hl);
        assert_eq!(spans[1].content.as_ref(), "abc");
        assert_eq!(spans[1].style, hl);
    }

    #[test]
    fn highlight_spans_case_insensitive() {
        let style = Style::default();
        let hl = Style::default().fg(ratatui::style::Color::Red);
        let spans = highlight_spans("Hello HELLO", "hello", style, hl);
        // "Hello" (hl) + " " (normal) + "HELLO" (hl)
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0].content.as_ref(), "Hello");
        assert_eq!(spans[0].style, hl);
        assert_eq!(spans[1].content.as_ref(), " ");
        assert_eq!(spans[1].style, style);
        assert_eq!(spans[2].content.as_ref(), "HELLO");
        assert_eq!(spans[2].style, hl);
    }

    #[test]
    fn highlight_spans_empty_query() {
        let style = Style::default();
        let hl = Style::default().fg(ratatui::style::Color::Red);
        let spans = highlight_spans("hello", "", style, hl);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content.as_ref(), "hello");
    }

    #[test]
    fn highlight_spans_multibyte_lowercase_length_change() {
        // Turkish İ (U+0130, 2 bytes) lowercases to "i\u{307}" (3 bytes).
        // This must not panic or produce wrong slicing.
        let style = Style::default();
        let hl = Style::default().fg(ratatui::style::Color::Red);
        let spans = highlight_spans("İstanbul", "i\u{307}stanbul", style, hl);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content.as_ref(), "İstanbul");
        assert_eq!(spans[0].style, hl);
    }

    #[test]
    fn highlight_spans_accented_chars() {
        let style = Style::default();
        let hl = Style::default().fg(ratatui::style::Color::Red);
        let spans = highlight_spans("café latte", "café", style, hl);
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].content.as_ref(), "café");
        assert_eq!(spans[0].style, hl);
        assert_eq!(spans[1].content.as_ref(), " latte");
    }
}
