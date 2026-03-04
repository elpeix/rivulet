#[derive(Debug, Clone)]
pub struct Group {
    pub id: i64,
    pub name: String,
    pub position: i64,
}

#[derive(Debug, Clone)]
pub struct NewGroup {
    pub name: String,
    pub position: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct Feed {
    pub id: i64,
    pub title: Option<String>,
    pub url: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub last_checked_at: Option<i64>,
    pub group_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct NewFeed {
    pub title: Option<String>,
    pub url: String,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub id: i64,
    pub title: Option<String>,
    pub url: Option<String>,
    pub published_at: Option<i64>,
    pub fetched_at: i64,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub saved_at: Option<i64>,
    pub read_at: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct NewEntry {
    pub feed_id: i64,
    pub guid: String,
    pub title: Option<String>,
    pub url: Option<String>,
    pub author: Option<String>,
    pub published_at: Option<i64>,
    pub fetched_at: i64,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub content_text: Option<String>,
    pub hash: Option<String>,
}

impl NewEntry {
    pub fn normalized(mut self) -> Self {
        self.guid = normalize_guid(
            Some(self.guid.as_str()),
            self.url.as_deref(),
            self.title.as_deref(),
        );
        self.content_text =
            normalize_content_text(self.content.as_deref(), self.summary.as_deref());
        self
    }
}

pub fn normalize_guid(guid: Option<&str>, url: Option<&str>, title: Option<&str>) -> String {
    for value in [guid, url, title].into_iter().flatten() {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    String::new()
}

pub fn normalize_content_text(content: Option<&str>, summary: Option<&str>) -> Option<String> {
    let source = content.or(summary)?;
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return None;
    }
    let text = crate::util::html::to_text(trimmed);
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}
