use html2text::render::{RichAnnotation, TaggedLine, TaggedLineElement};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

use crate::ui::theme::Theme;

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

            // Check if this span is a link
            for ann in &ts.tag {
                if let RichAnnotation::Link(url) = ann {
                    regions.push(LinkRegion {
                        line: line_idx,
                        col_start: col,
                        col_end: col + width,
                        url: url.clone(),
                    });
                    break;
                }
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
    let mut style = Style::default();

    for ann in annotations {
        match ann {
            RichAnnotation::Strong => {
                style = style.add_modifier(Modifier::BOLD);
            }
            RichAnnotation::Emphasis => {
                style = style.add_modifier(Modifier::ITALIC);
            }
            RichAnnotation::Link(url) => {
                let is_selected =
                    selected_link_url.is_some_and(|sel| sel == url.as_str());
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

fn post_process_text(text: &str) -> String {
    // Replace ASCII HR lines (-----) with box-drawing chars
    let trimmed = text.trim();
    if trimmed.len() >= 3 && trimmed.chars().all(|c| c == '-' || c == '─') {
        return "─".repeat(trimmed.len());
    }
    text.to_string()
}
