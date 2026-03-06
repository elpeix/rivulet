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
    pub custom_title: Option<String>,
    pub url: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub last_checked_at: Option<i64>,
    pub group_id: Option<i64>,
}

impl Feed {
    pub fn display_title(&self) -> Option<&str> {
        self.custom_title.as_deref().or(self.title.as_deref())
    }
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
    pub feed_id: i64,
    pub title: Option<String>,
    pub url: Option<String>,
    pub author: Option<String>,
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
        self.hash = Some(self.compute_hash());
        self
    }

    fn compute_hash(&self) -> String {
        // FNV-1a 64-bit — deterministic across Rust versions (unlike DefaultHasher)
        const FNV_OFFSET: u64 = 0xcbf29ce484222325;
        const FNV_PRIME: u64 = 0x00000100000001B3;

        let mut h = FNV_OFFSET;
        for part in [
            self.title.as_deref().unwrap_or(""),
            self.url.as_deref().unwrap_or(""),
            self.summary.as_deref().unwrap_or(""),
            self.content.as_deref().unwrap_or(""),
        ] {
            for byte in part.as_bytes() {
                h ^= u64::from(*byte);
                h = h.wrapping_mul(FNV_PRIME);
            }
            // separator between fields
            h ^= 0xFF;
            h = h.wrapping_mul(FNV_PRIME);
        }
        // Hash published_at
        if let Some(ts) = self.published_at {
            for byte in ts.to_le_bytes() {
                h ^= u64::from(byte);
                h = h.wrapping_mul(FNV_PRIME);
            }
        }
        format!("{h:016x}")
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
    if text.is_empty() { None } else { Some(text) }
}
