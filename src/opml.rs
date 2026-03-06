use std::io::{Read, Write};

use crate::store::models::{Feed, Group};

pub struct OpmlOutline {
    pub title: String,
    pub url: String,
    pub group: Option<String>,
}

pub fn export_opml(
    writer: &mut impl Write,
    feeds: &[Feed],
    groups: &[Group],
) -> std::io::Result<()> {
    let group_map: std::collections::HashMap<i64, &str> = groups
        .iter()
        .map(|g| (g.id, g.name.as_str()))
        .collect();

    // Group feeds by category
    let mut categorized: std::collections::BTreeMap<String, Vec<&Feed>> =
        std::collections::BTreeMap::new();
    let mut ungrouped: Vec<&Feed> = Vec::new();

    for feed in feeds {
        if let Some(gid) = feed.group_id {
            if let Some(name) = group_map.get(&gid) {
                categorized
                    .entry(name.to_string())
                    .or_default()
                    .push(feed);
            } else {
                ungrouped.push(feed);
            }
        } else {
            ungrouped.push(feed);
        }
    }

    writeln!(writer, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(writer, r#"<opml version="2.0">"#)?;
    writeln!(writer, r"  <head><title>Rivulet Subscriptions</title></head>")?;
    writeln!(writer, r"  <body>")?;

    for (group_name, group_feeds) in &categorized {
        writeln!(
            writer,
            r#"    <outline text="{}">"#,
            escape_xml(group_name)
        )?;
        for feed in group_feeds {
            write_feed_outline(writer, feed, "      ")?;
        }
        writeln!(writer, r"    </outline>")?;
    }

    for feed in &ungrouped {
        write_feed_outline(writer, feed, "    ")?;
    }

    writeln!(writer, r"  </body>")?;
    writeln!(writer, r"</opml>")?;

    Ok(())
}

fn write_feed_outline(writer: &mut impl Write, feed: &Feed, indent: &str) -> std::io::Result<()> {
    let title = feed
        .display_title()
        .unwrap_or(&feed.url);
    writeln!(
        writer,
        r#"{}<outline type="rss" text="{}" xmlUrl="{}" />"#,
        indent,
        escape_xml(title),
        escape_xml(&feed.url),
    )
}

pub fn parse_opml(reader: &mut impl Read) -> Result<Vec<OpmlOutline>, String> {
    let mut xml = String::new();
    reader
        .read_to_string(&mut xml)
        .map_err(|e| format!("Failed to read OPML: {e}"))?;

    let doc = roxmltree::Document::parse(&xml)
        .map_err(|e| format!("Failed to parse OPML XML: {e}"))?;

    let body = doc
        .descendants()
        .find(|n| n.tag_name().name().eq_ignore_ascii_case("body"))
        .ok_or_else(|| "Missing <body> element in OPML".to_string())?;

    let mut outlines = Vec::new();

    for node in body.children().filter(|n| n.tag_name().name().eq_ignore_ascii_case("outline")) {
        let xml_url = node.attribute("xmlUrl").or_else(|| node.attribute("xmlurl"));
        let text = node.attribute("text");

        if let Some(url) = xml_url {
            // Top-level feed (no group)
            outlines.push(OpmlOutline {
                title: text.unwrap_or(url).to_string(),
                url: url.to_string(),
                group: None,
            });
        } else {
            // Category outline — collect child feeds
            let group_name = text.map(std::string::ToString::to_string);
            for child in node.children().filter(|n| n.tag_name().name().eq_ignore_ascii_case("outline")) {
                let child_url = child.attribute("xmlUrl").or_else(|| child.attribute("xmlurl"));
                let child_text = child.attribute("text");
                if let Some(url) = child_url {
                    outlines.push(OpmlOutline {
                        title: child_text.unwrap_or(url).to_string(),
                        url: url.to_string(),
                        group: group_name.clone(),
                    });
                }
            }
        }
    }

    Ok(outlines)
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_flat_opml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<opml version="2.0">
  <head><title>Test</title></head>
  <body>
    <outline type="rss" text="Blog A" xmlUrl="https://a.com/feed" />
    <outline type="rss" text="Blog B" xmlUrl="https://b.com/rss" />
  </body>
</opml>"#;
        let outlines = parse_opml(&mut xml.as_bytes()).unwrap();
        assert_eq!(outlines.len(), 2);
        assert_eq!(outlines[0].title, "Blog A");
        assert_eq!(outlines[0].url, "https://a.com/feed");
        assert!(outlines[0].group.is_none());
    }

    #[test]
    fn parse_categorized_opml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<opml version="2.0">
  <body>
    <outline text="Tech">
      <outline type="rss" text="Hacker News" xmlUrl="https://hn.com/rss" />
    </outline>
    <outline type="rss" text="Ungrouped" xmlUrl="https://other.com/feed" />
  </body>
</opml>"#;
        let outlines = parse_opml(&mut xml.as_bytes()).unwrap();
        assert_eq!(outlines.len(), 2);
        assert_eq!(outlines[0].group.as_deref(), Some("Tech"));
        assert!(outlines[1].group.is_none());
    }

    #[test]
    fn parse_escaped_attrs() {
        let xml = r#"<opml version="2.0"><body><outline type="rss" text="A &amp; B" xmlUrl="https://a.com/feed?x=1&amp;y=2" /></body></opml>"#;
        let outlines = parse_opml(&mut xml.as_bytes()).unwrap();
        assert_eq!(outlines[0].title, "A & B");
        assert_eq!(outlines[0].url, "https://a.com/feed?x=1&y=2");
    }

    #[test]
    fn export_roundtrip() {
        let groups = vec![Group {
            id: 1,
            name: "Tech".to_string(),
            position: 0,
        }];
        let feeds = vec![
            Feed {
                id: 1,
                title: Some("Blog A".to_string()),
                custom_title: None,
                url: "https://a.com/feed".to_string(),
                etag: None,
                last_modified: None,
                last_checked_at: None,
                group_id: Some(1),
            },
            Feed {
                id: 2,
                title: Some("Blog B".to_string()),
                custom_title: None,
                url: "https://b.com/rss".to_string(),
                etag: None,
                last_modified: None,
                last_checked_at: None,
                group_id: None,
            },
        ];

        let mut buf = Vec::new();
        export_opml(&mut buf, &feeds, &groups).unwrap();
        let xml = String::from_utf8(buf).unwrap();

        let outlines = parse_opml(&mut xml.as_bytes()).unwrap();
        assert_eq!(outlines.len(), 2);
        assert_eq!(outlines[0].title, "Blog A");
        assert_eq!(outlines[0].group.as_deref(), Some("Tech"));
        assert_eq!(outlines[1].title, "Blog B");
        assert!(outlines[1].group.is_none());
    }

    #[test]
    fn parse_malformed_xml_returns_error() {
        let xml = r#"<opml><body><outline text="broken">"#;
        let result = parse_opml(&mut xml.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn parse_missing_body_returns_error() {
        let xml = r#"<?xml version="1.0"?><opml version="2.0"><head/></opml>"#;
        let result = parse_opml(&mut xml.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn parse_multiline_outline() {
        let xml = r#"<?xml version="1.0"?>
<opml version="2.0">
  <body>
    <outline
      type="rss"
      text="Multi Line"
      xmlUrl="https://multi.com/feed" />
  </body>
</opml>"#;
        let outlines = parse_opml(&mut xml.as_bytes()).unwrap();
        assert_eq!(outlines.len(), 1);
        assert_eq!(outlines[0].title, "Multi Line");
        assert_eq!(outlines[0].url, "https://multi.com/feed");
    }
}
