use html2text::render::{RichAnnotation, TaggedLine, TaggedLineElement};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::ui::theme::Theme;

pub fn rich_lines_to_ratatui(
    tagged_lines: Vec<TaggedLine<Vec<RichAnnotation>>>,
    theme: &Theme,
) -> Vec<Line<'static>> {
    tagged_lines
        .into_iter()
        .map(|tl| convert_line(tl, theme))
        .collect()
}

fn convert_line(tagged_line: TaggedLine<Vec<RichAnnotation>>, theme: &Theme) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();

    for element in tagged_line.iter() {
        if let TaggedLineElement::Str(ts) = element {
            if ts.s.is_empty() {
                continue;
            }
            let text = post_process_text(&ts.s);
            let style = annotations_to_style(&ts.tag, theme);
            spans.push(Span::styled(text, style));
        }
    }

    if spans.is_empty() {
        spans.push(Span::raw(""));
    }

    Line::from(spans)
}

fn annotations_to_style(annotations: &[RichAnnotation], theme: &Theme) -> Style {
    let mut style = Style::default();

    for ann in annotations {
        match ann {
            RichAnnotation::Strong => {
                style = style.add_modifier(Modifier::BOLD);
            }
            RichAnnotation::Emphasis => {
                style = style.add_modifier(Modifier::ITALIC);
            }
            RichAnnotation::Link(_) => {
                style = style.fg(theme.accent).add_modifier(Modifier::UNDERLINED);
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
