use html2text::render::{RichAnnotation, TaggedLine, TaggedLineElement};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

use crate::ui::theme::Theme;
use crate::ui::widgets::highlight_spans;
use crate::util::html::element_link_url;

#[derive(Debug, Clone)]
pub struct LinkRegion {
    pub line: usize,
    pub col_start: u16,
    pub col_end: u16,
    pub url: String,
}

pub struct RichResult {
    pub lines: Vec<Line<'static>>,
    pub link_regions: Vec<LinkRegion>,
}

pub fn rich_lines_to_ratatui(
    tagged_lines: Vec<TaggedLine<Vec<RichAnnotation>>>,
    theme: &Theme,
    selected_link_url: Option<&str>,
) -> RichResult {
    let mut lines = Vec::new();
    let mut link_regions = Vec::new();

    for (line_idx, tl) in tagged_lines.into_iter().enumerate() {
        let (line, regions) = convert_line(tl, theme, selected_link_url, line_idx);
        lines.push(line);
        link_regions.extend(regions);
    }

    RichResult {
        lines,
        link_regions,
    }
}

fn convert_line(
    tagged_line: TaggedLine<Vec<RichAnnotation>>,
    theme: &Theme,
    selected_link_url: Option<&str>,
    line_idx: usize,
) -> (Line<'static>, Vec<LinkRegion>) {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut regions = Vec::new();
    let mut col: u16 = 0;

    for element in tagged_line.iter() {
        if let TaggedLineElement::Str(ts) = element {
            if ts.s.is_empty() {
                continue;
            }
            let text = post_process_text(&ts.s);
            let width = UnicodeWidthStr::width(text.as_str()) as u16;
            let style = annotations_to_style(&ts.tag, theme, selected_link_url);

            if let Some(url) = element_link_url(element) {
                regions.push(LinkRegion {
                    line: line_idx,
                    col_start: col,
                    col_end: col + width,
                    url: url.clone(),
                });
            }

            col += width;
            spans.push(Span::styled(text, style));
        }
    }

    if spans.is_empty() {
        spans.push(Span::raw(""));
    }

    (Line::from(spans), regions)
}

fn annotations_to_style(
    annotations: &[RichAnnotation],
    theme: &Theme,
    selected_link_url: Option<&str>,
) -> Style {
    let mut style = Style::default().fg(theme.text);

    for ann in annotations {
        match ann {
            RichAnnotation::Strong => {
                style = style.add_modifier(Modifier::BOLD);
            }
            RichAnnotation::Emphasis => {
                style = style.add_modifier(Modifier::ITALIC);
            }
            RichAnnotation::Link(url) => {
                let is_selected = selected_link_url.is_some_and(|sel| sel == url.as_str());
                if is_selected {
                    style = style
                        .fg(theme.highlight_fg)
                        .bg(theme.accent)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
                } else {
                    style = style.fg(theme.accent).add_modifier(Modifier::UNDERLINED);
                }
            }
            RichAnnotation::Code => {
                style = style.fg(theme.accent_alt);
            }
            RichAnnotation::Preformat(_) => {
                style = style.fg(theme.dim);
            }
            RichAnnotation::Strikeout => {
                style = style.add_modifier(Modifier::CROSSED_OUT);
            }
            RichAnnotation::Image(_) => {
                style = style.fg(theme.dim).add_modifier(Modifier::ITALIC);
            }
            _ => {}
        }
    }

    style
}

/// Highlight search matches in already-rendered lines. Returns indices of lines with matches.
/// `active_match` is the index into the returned Vec to style with `active_style`.
pub fn highlight_search(
    lines: &mut Vec<Line<'static>>,
    query: &str,
    highlight: Style,
    active_style: Style,
    active_match: Option<usize>,
) -> Vec<usize> {
    if query.is_empty() {
        return Vec::new();
    }
    let mut match_lines = Vec::new();

    for (line_idx, line) in lines.iter_mut().enumerate() {
        let full_text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        if !full_text.to_lowercase().contains(&query.to_lowercase()) {
            continue;
        }
        let is_active = active_match == Some(match_lines.len());
        match_lines.push(line_idx);
        let style = if is_active { active_style } else { highlight };
        let new_spans: Vec<Span<'static>> = line
            .spans
            .drain(..)
            .flat_map(|span| {
                let text = span.content.into_owned();
                let base = span.style;
                highlight_spans(&text, query, base, style)
            })
            .collect();
        *line = Line::from(new_spans);
    }

    match_lines
}

#[cfg(test)]
mod tests {
    use super::*;

    use ratatui::style::Color;

    const HL: Style = Style::new().fg(Color::Yellow);
    const ACTIVE: Style = Style::new().fg(Color::Red);

    #[test]
    fn highlight_search_empty_query() {
        let mut lines = vec![Line::from("hello world")];
        let matches = highlight_search(&mut lines, "", HL, ACTIVE, None);
        assert!(matches.is_empty());
    }

    #[test]
    fn highlight_search_no_match() {
        let mut lines = vec![Line::from("hello world")];
        let matches = highlight_search(&mut lines, "xyz", HL, ACTIVE, None);
        assert!(matches.is_empty());
    }

    #[test]
    fn highlight_search_single_line_match() {
        let mut lines = vec![
            Line::from("no match here"),
            Line::from("find the word"),
            Line::from("nothing"),
        ];
        let matches = highlight_search(&mut lines, "word", HL, ACTIVE, None);
        assert_eq!(matches, vec![1]);
        assert!(lines[1].spans.len() > 1);
    }

    #[test]
    fn highlight_search_multiple_lines() {
        let mut lines = vec![
            Line::from("rust is great"),
            Line::from("go is fine"),
            Line::from("rust again"),
        ];
        let matches = highlight_search(&mut lines, "rust", HL, ACTIVE, None);
        assert_eq!(matches, vec![0, 2]);
    }

    #[test]
    fn highlight_search_case_insensitive() {
        let mut lines = vec![Line::from("Hello HELLO hello")];
        let matches = highlight_search(&mut lines, "hello", HL, ACTIVE, None);
        assert_eq!(matches, vec![0]);
        assert!(lines[0].spans.len() >= 3);
    }

    #[test]
    fn highlight_search_preserves_existing_styles() {
        let bold = Style::default().add_modifier(Modifier::BOLD);
        let mut lines = vec![Line::from(vec![
            Span::styled("bold text", bold),
            Span::raw(" normal"),
        ])];
        let matches = highlight_search(&mut lines, "text", HL, ACTIVE, None);
        assert_eq!(matches, vec![0]);
        let spans = &lines[0].spans;
        assert_eq!(spans[0].content.as_ref(), "bold ");
        assert_eq!(spans[0].style, bold);
        assert_eq!(spans[1].content.as_ref(), "text");
        assert_eq!(spans[1].style, HL);
    }

    #[test]
    fn highlight_search_active_match_uses_active_style() {
        let mut lines = vec![
            Line::from("rust is great"),
            Line::from("go is fine"),
            Line::from("rust again"),
        ];
        let matches = highlight_search(&mut lines, "rust", HL, ACTIVE, Some(1));
        assert_eq!(matches, vec![0, 2]);
        // First match (index 0) uses normal highlight
        assert_eq!(lines[0].spans[0].style, HL);
        // Second match (index 1) uses active style
        assert_eq!(lines[2].spans[0].style, ACTIVE);
    }
}

fn post_process_text(text: &str) -> String {
    // Replace ASCII HR lines (-----) with box-drawing chars
    let trimmed = text.trim();
    if trimmed.len() >= 3 && trimmed.chars().all(|c| c == '-' || c == '─') {
        return "─".repeat(trimmed.len());
    }
    text.to_string()
}
