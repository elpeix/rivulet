use html2text::render::{RichAnnotation, TaggedLine};

pub fn to_text(html: &str) -> String {
    let text = html2text::from_read(html.as_bytes(), 80).unwrap_or_default();
    normalize_whitespace(&text)
}

pub fn to_rich_lines(html: &str, width: usize) -> Vec<TaggedLine<Vec<RichAnnotation>>> {
    let width = width.max(20);
    html2text::from_read_rich(html.as_bytes(), width).unwrap_or_default()
}

fn normalize_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}
