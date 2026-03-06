use html2text::render::{RichAnnotation, TaggedLine, TaggedLineElement};

pub fn to_text(html: &str) -> String {
    let text = html2text::from_read(html.as_bytes(), 80).unwrap_or_default();
    normalize_whitespace(&text)
}

pub fn to_rich_lines(html: &str, width: usize) -> Vec<TaggedLine<Vec<RichAnnotation>>> {
    let width = width.max(20);
    html2text::from_read_rich(html.as_bytes(), width).unwrap_or_default()
}

/// Extract unique link URLs from tagged lines, preserving order of first appearance.
pub fn extract_links(tagged_lines: &[TaggedLine<Vec<RichAnnotation>>]) -> Vec<String> {
    let mut links = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for tl in tagged_lines {
        for element in tl.iter() {
            if let TaggedLineElement::Str(ts) = element {
                for ann in &ts.tag {
                    if let RichAnnotation::Link(url) = ann {
                        if seen.insert(url.clone()) {
                            links.push(url.clone());
                        }
                    }
                }
            }
        }
    }
    links
}

fn normalize_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}
