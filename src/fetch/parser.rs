use std::io::Cursor;

use feed_rs::model::{Entry as FeedEntry, Feed};

use crate::store::models::NewEntry;

#[derive(Debug)]
pub enum ParseError {
    Feed,
}

impl From<feed_rs::parser::ParseFeedError> for ParseError {
    fn from(_: feed_rs::parser::ParseFeedError) -> Self {
        Self::Feed
    }
}

pub fn parse_feed(bytes: &[u8]) -> Result<Feed, ParseError> {
    let cursor = Cursor::new(bytes);
    Ok(feed_rs::parser::parse(cursor)?)
}

pub fn map_entries(feed_id: i64, feed: &Feed, fetched_at: i64) -> Vec<NewEntry> {
    feed.entries
        .iter()
        .map(|entry| map_entry(feed_id, entry, fetched_at).normalized())
        .collect()
}

pub fn map_entry(feed_id: i64, entry: &FeedEntry, fetched_at: i64) -> NewEntry {
    let guid = entry.id.trim().to_string();

    let title = entry
        .title
        .as_ref()
        .map(|text| text.content.trim().to_string())
        .filter(|value| !value.is_empty());

    let summary = entry
        .summary
        .as_ref()
        .map(|text| text.content.trim().to_string())
        .filter(|value| !value.is_empty());

    let content = entry
        .content
        .as_ref()
        .and_then(|content| content.body.as_ref())
        .map(|text| text.trim().to_string())
        .filter(|value| !value.is_empty());

    let url = entry
        .links
        .iter()
        .find(|link| matches!(link.rel.as_deref(), Some("alternate") | None))
        .or_else(|| entry.links.first())
        .map(|link| link.href.trim().to_string())
        .filter(|value| !value.is_empty());

    let author = entry
        .authors
        .first()
        .map(|author| author.name.trim().to_string())
        .filter(|value| !value.is_empty());

    let published_at = entry.published.or(entry.updated).map(|dt| dt.timestamp());

    let hash = None; // computed later by NewEntry::normalized()

    NewEntry {
        feed_id,
        guid,
        title,
        url,
        author,
        published_at,
        fetched_at,
        summary,
        content,
        content_text: None,
        hash,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RSS_SAMPLE: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Test Feed</title>
    <link>https://example.com</link>
    <item>
      <title>First Post</title>
      <link>https://example.com/1</link>
      <guid>entry-1</guid>
      <description>Summary of first post</description>
    </item>
    <item>
      <title>Second Post</title>
      <link>https://example.com/2</link>
      <guid>entry-2</guid>
      <description>Summary of second post</description>
    </item>
  </channel>
</rss>"#;

    const ATOM_SAMPLE: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Atom Feed</title>
  <entry>
    <title>Atom Entry</title>
    <id>atom-1</id>
    <summary>Atom summary</summary>
    <link href="https://example.com/atom/1"/>
  </entry>
</feed>"#;

    #[test]
    fn parse_rss_feed() {
        let feed = parse_feed(RSS_SAMPLE).expect("should parse RSS");
        assert_eq!(feed.title.as_ref().unwrap().content, "Test Feed");
        assert_eq!(feed.entries.len(), 2);
    }

    #[test]
    fn parse_atom_feed() {
        let feed = parse_feed(ATOM_SAMPLE).expect("should parse Atom");
        assert_eq!(feed.title.as_ref().unwrap().content, "Atom Feed");
        assert_eq!(feed.entries.len(), 1);
    }

    #[test]
    fn parse_invalid_feed() {
        let result = parse_feed(b"this is not xml");
        assert!(result.is_err());
    }

    #[test]
    fn map_entries_from_rss() {
        let feed = parse_feed(RSS_SAMPLE).unwrap();
        let entries = map_entries(42, &feed, 1000);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].feed_id, 42);
        assert_eq!(entries[0].title.as_deref(), Some("First Post"));
        assert_eq!(entries[0].url.as_deref(), Some("https://example.com/1"));
        assert_eq!(entries[1].title.as_deref(), Some("Second Post"));
    }

    #[test]
    fn map_entry_preserves_guid() {
        let feed = parse_feed(RSS_SAMPLE).unwrap();
        let entry = map_entry(1, &feed.entries[0], 500);
        assert_eq!(entry.guid, "entry-1");
        assert_eq!(entry.fetched_at, 500);
    }

    #[test]
    fn map_entries_from_atom() {
        let feed = parse_feed(ATOM_SAMPLE).unwrap();
        let entries = map_entries(10, &feed, 2000);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].guid, "atom-1");
        assert_eq!(entries[0].title.as_deref(), Some("Atom Entry"));
        assert_eq!(
            entries[0].url.as_deref(),
            Some("https://example.com/atom/1")
        );
    }
}
