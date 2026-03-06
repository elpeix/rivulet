use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredFeed {
    pub url: String,
    pub title: Option<String>,
    pub feed_type: FeedType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedType {
    Rss,
    Atom,
}

impl std::fmt::Display for FeedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rss => write!(f, "RSS"),
            Self::Atom => write!(f, "Atom"),
        }
    }
}

/// Discover RSS/Atom feeds from HTML by looking for `<link rel="alternate">` tags in the head.
pub fn discover_feeds(html: &str, base_url: &str) -> Vec<DiscoveredFeed> {
    let base = Url::parse(base_url).ok();
    let mut feeds = Vec::new();

    // Only search within <head> to avoid false positives
    let head = extract_head(html);

    for link in LinkTagIter::new(head) {
        if let Some(feed) = parse_link_tag(&link, base.as_ref()) {
            if !feeds.iter().any(|f: &DiscoveredFeed| f.url == feed.url) {
                feeds.push(feed);
            }
        }
    }

    feeds
}

fn extract_head(html: &str) -> &str {
    let lower = html.to_ascii_lowercase();
    let start = lower.find("<head").unwrap_or(0);
    let end = lower.find("</head>").unwrap_or(html.len());
    &html[start..end.min(html.len())]
}

fn parse_link_tag(tag: &str, base: Option<&Url>) -> Option<DiscoveredFeed> {
    let rel = extract_attr(tag, "rel")?;
    if !rel.eq_ignore_ascii_case("alternate") {
        return None;
    }

    let type_attr = extract_attr(tag, "type")?;
    let feed_type = match type_attr.to_ascii_lowercase().as_str() {
        "application/rss+xml" => FeedType::Rss,
        "application/atom+xml" => FeedType::Atom,
        _ => return None,
    };

    let href = extract_attr(tag, "href")?;
    if href.is_empty() {
        return None;
    }

    let url = resolve_url(&href, base)?;
    let title = extract_attr(tag, "title");

    Some(DiscoveredFeed {
        url,
        title,
        feed_type,
    })
}

fn resolve_url(href: &str, base: Option<&Url>) -> Option<String> {
    if href.starts_with("http://") || href.starts_with("https://") {
        Some(href.to_string())
    } else if let Some(base) = base {
        base.join(href).ok().map(|u| u.to_string())
    } else {
        None
    }
}

fn extract_attr(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let patterns = [
        format!(" {name}=\""),
        format!(" {name}='"),
        format!(" {name} = \""),
        format!(" {name} = '"),
        format!(" {name}= \""),
        format!(" {name}= '"),
        format!(" {name} =\""),
        format!(" {name} ='"),
    ];
    for pattern in &patterns {
        if let Some(start) = lower.find(pattern.as_str()) {
            let quote = if pattern.ends_with('"') { '"' } else { '\'' };
            let value_start = start + pattern.len();
            if let Some(end) = tag[value_start..].find(quote) {
                let value = tag[value_start..value_start + end].trim().to_string();
                if !value.is_empty() {
                    return Some(value);
                }
            }
        }
    }
    None
}

/// Iterator over `<link ...>` tags in HTML.
struct LinkTagIter<'a> {
    remaining: &'a str,
}

impl<'a> LinkTagIter<'a> {
    fn new(html: &'a str) -> Self {
        Self { remaining: html }
    }
}

impl<'a> Iterator for LinkTagIter<'a> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        let lower = self.remaining.to_ascii_lowercase();
        let start = lower.find("<link ")?;
        let after_start = start + 1;
        let end = self.remaining[after_start..]
            .find('>')
            .map(|i| after_start + i + 1)?;
        let tag = self.remaining[start..end].to_string();
        self.remaining = &self.remaining[end..];
        Some(tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_rss_link() {
        let html = r#"<html><head>
            <link rel="alternate" type="application/rss+xml" title="My Blog" href="/feed.xml">
        </head><body></body></html>"#;
        let feeds = discover_feeds(html, "https://example.com");
        assert_eq!(feeds.len(), 1);
        assert_eq!(feeds[0].url, "https://example.com/feed.xml");
        assert_eq!(feeds[0].title.as_deref(), Some("My Blog"));
        assert_eq!(feeds[0].feed_type, FeedType::Rss);
    }

    #[test]
    fn discover_atom_link() {
        let html = r#"<html><head>
            <link rel="alternate" type="application/atom+xml" href="https://example.com/atom.xml">
        </head></html>"#;
        let feeds = discover_feeds(html, "https://example.com");
        assert_eq!(feeds.len(), 1);
        assert_eq!(feeds[0].feed_type, FeedType::Atom);
        assert_eq!(feeds[0].title, None);
    }

    #[test]
    fn discover_multiple_feeds() {
        let html = r#"<head>
            <link rel="alternate" type="application/rss+xml" title="RSS" href="/rss">
            <link rel="alternate" type="application/atom+xml" title="Atom" href="/atom">
        </head>"#;
        let feeds = discover_feeds(html, "https://example.com");
        assert_eq!(feeds.len(), 2);
    }

    #[test]
    fn ignores_non_feed_links() {
        let html = r#"<head>
            <link rel="stylesheet" href="/style.css">
            <link rel="alternate" type="text/html" href="/page">
            <link rel="alternate" type="application/rss+xml" href="/feed">
        </head>"#;
        let feeds = discover_feeds(html, "https://example.com");
        assert_eq!(feeds.len(), 1);
    }

    #[test]
    fn ignores_links_outside_head() {
        let html = r#"<html><head></head><body>
            <link rel="alternate" type="application/rss+xml" href="/feed">
        </body></html>"#;
        let feeds = discover_feeds(html, "https://example.com");
        assert_eq!(feeds.is_empty(), true);
    }

    #[test]
    fn resolves_relative_urls() {
        let html = r#"<head>
            <link rel="alternate" type="application/rss+xml" href="../feed.xml">
        </head>"#;
        let feeds = discover_feeds(html, "https://example.com/blog/");
        assert_eq!(feeds.len(), 1);
        assert_eq!(feeds[0].url, "https://example.com/feed.xml");
    }

    #[test]
    fn deduplicates_feeds() {
        let html = r#"<head>
            <link rel="alternate" type="application/rss+xml" href="/feed">
            <link rel="alternate" type="application/rss+xml" href="/feed">
        </head>"#;
        let feeds = discover_feeds(html, "https://example.com");
        assert_eq!(feeds.len(), 1);
    }

    #[test]
    fn handles_single_quotes() {
        let html = r#"<head>
            <link rel='alternate' type='application/rss+xml' href='/feed.xml'>
        </head>"#;
        let feeds = discover_feeds(html, "https://example.com");
        assert_eq!(feeds.len(), 1);
    }

    #[test]
    fn no_partial_attr_match() {
        let html = r#"<head>
            <link data-rel="alternate" type="application/rss+xml" href="/feed">
        </head>"#;
        let feeds = discover_feeds(html, "https://example.com");
        assert!(feeds.is_empty());
    }

    #[test]
    fn no_head_searches_whole_document() {
        let html = r#"<link rel="alternate" type="application/rss+xml" href="/feed">"#;
        let feeds = discover_feeds(html, "https://example.com");
        assert_eq!(feeds.len(), 1);
    }
}
