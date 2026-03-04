use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Result};

use super::models::{Entry, Feed, Group, NewEntry, NewFeed, NewGroup};
use super::schema::apply_migrations;

pub struct Repo {
    conn: Connection,
}

impl Repo {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path)?;
        apply_migrations(&conn)?;
        Ok(Self { conn })
    }

    pub fn create_feed(&self, new_feed: &NewFeed) -> Result<Feed> {
        self.conn.execute(
            "INSERT INTO feeds (title, url, created_at) VALUES (?1, ?2, ?3);",
            params![&new_feed.title, &new_feed.url, new_feed.created_at],
        )?;
        let id = self.conn.last_insert_rowid();
        self.get_feed(id)
            .and_then(|feed| feed.ok_or(rusqlite::Error::QueryReturnedNoRows))
    }

    pub fn update_feed(&self, feed: &Feed) -> Result<()> {
        self.conn.execute(
            "UPDATE feeds SET title = ?1, url = ?2, etag = ?3, last_modified = ?4, last_checked_at = ?5 WHERE id = ?6;",
            params![
                &feed.title,
                &feed.url,
                &feed.etag,
                &feed.last_modified,
                feed.last_checked_at,
                feed.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_feed(&self, feed_id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM feeds WHERE id = ?1;", params![feed_id])?;
        Ok(())
    }

    pub fn get_feed(&self, feed_id: i64) -> Result<Option<Feed>> {
        self.conn
            .query_row(
                "SELECT id, title, url, etag, last_modified, last_checked_at, group_id FROM feeds WHERE id = ?1;",
                params![feed_id],
                map_feed_row,
            )
            .optional()
    }

    pub fn list_feeds(&self) -> Result<Vec<Feed>> {
        let mut stmt = self.conn.prepare(
            "SELECT f.id, f.title, f.url, f.etag, f.last_modified, f.last_checked_at, f.group_id
             FROM feeds f
             LEFT JOIN groups g ON g.id = f.group_id
             ORDER BY COALESCE(g.position, 999999), g.name, f.created_at DESC;",
        )?;
        let rows = stmt.query_map([], map_feed_row)?;
        let mut feeds = Vec::new();
        for feed in rows {
            feeds.push(feed?);
        }
        Ok(feeds)
    }

    pub fn create_group(&self, new: &NewGroup) -> Result<Group> {
        self.conn.execute(
            "INSERT INTO groups (name, position, created_at) VALUES (?1, ?2, ?3);",
            params![&new.name, new.position, new.created_at],
        )?;
        let id = self.conn.last_insert_rowid();
        Ok(Group {
            id,
            name: new.name.clone(),
            position: new.position,
        })
    }

    pub fn list_groups(&self) -> Result<Vec<Group>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, position FROM groups ORDER BY position, name;")?;
        let rows = stmt.query_map([], |row| {
            Ok(Group {
                id: row.get(0)?,
                name: row.get(1)?,
                position: row.get(2)?,
            })
        })?;
        let mut groups = Vec::new();
        for row in rows {
            groups.push(row?);
        }
        Ok(groups)
    }

    pub fn delete_group(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM groups WHERE id = ?1;", params![id])?;
        Ok(())
    }

    pub fn rename_group(&self, id: i64, name: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE groups SET name = ?1 WHERE id = ?2;",
            params![name, id],
        )?;
        Ok(())
    }

    pub fn set_feed_group(&self, feed_id: i64, group_id: Option<i64>) -> Result<()> {
        self.conn.execute(
            "UPDATE feeds SET group_id = ?1 WHERE id = ?2;",
            params![group_id, feed_id],
        )?;
        Ok(())
    }

    pub fn update_feed_fetch_state(
        &self,
        feed_id: i64,
        etag: Option<&str>,
        last_modified: Option<&str>,
        last_checked_at: Option<i64>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE feeds SET etag = ?1, last_modified = ?2, last_checked_at = ?3 WHERE id = ?4;",
            params![etag, last_modified, last_checked_at, feed_id],
        )?;
        Ok(())
    }

    pub fn upsert_entries(&self, entries: &[NewEntry]) -> Result<usize> {
        self.conn.execute("BEGIN IMMEDIATE;", [])?;
        let result = (|| {
            let mut changed = 0;
            for entry in entries {
                let count = self.conn.execute(
                    "INSERT INTO entries (
                        feed_id, guid, title, url, author, published_at, fetched_at,
                        summary, content, content_text, hash
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                    ON CONFLICT(feed_id, guid) DO UPDATE SET
                        title = excluded.title,
                        url = excluded.url,
                        author = excluded.author,
                        published_at = excluded.published_at,
                        fetched_at = excluded.fetched_at,
                        summary = excluded.summary,
                        content = excluded.content,
                        content_text = excluded.content_text,
                        hash = excluded.hash;",
                    params![
                        entry.feed_id,
                        &entry.guid,
                        &entry.title,
                        &entry.url,
                        &entry.author,
                        entry.published_at,
                        entry.fetched_at,
                        &entry.summary,
                        &entry.content,
                        &entry.content_text,
                        &entry.hash,
                    ],
                )?;
                changed += count;
            }
            Ok(changed)
        })();

        match result {
            Ok(changed) => {
                self.conn.execute("COMMIT;", [])?;
                Ok(changed)
            }
            Err(error) => {
                let _ = self.conn.execute("ROLLBACK;", []);
                Err(error)
            }
        }
    }

    pub fn unread_count_all(&self) -> Result<i64> {
        self.conn.query_row(
            "SELECT COUNT(*) FROM entries e LEFT JOIN read_state rs ON rs.entry_id = e.id WHERE rs.entry_id IS NULL;",
            [],
            |row| row.get(0),
        )
    }

    pub fn unread_counts_by_feed(&self) -> Result<Vec<(i64, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.feed_id, COUNT(*) FROM entries e LEFT JOIN read_state rs ON rs.entry_id = e.id WHERE rs.entry_id IS NULL GROUP BY e.feed_id;",
        )?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let mut counts = Vec::new();
        for row in rows {
            counts.push(row?);
        }
        Ok(counts)
    }

    pub fn count_entries_for_feed(&self, feed_id: i64) -> Result<i64> {
        self.conn.query_row(
            "SELECT COUNT(*) FROM entries WHERE feed_id = ?1;",
            params![feed_id],
            |row| row.get(0),
        )
    }

    pub fn entries_for_feed(&self, feed_id: i64) -> Result<Vec<Entry>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.id, e.title, e.url, e.published_at, e.fetched_at, e.summary, e.content, e.saved_at, rs.read_at
             FROM entries e LEFT JOIN read_state rs ON rs.entry_id = e.id
             WHERE e.feed_id = ?1 ORDER BY e.published_at DESC, e.fetched_at DESC;",
        )?;
        let rows = stmt.query_map(params![feed_id], map_entry_row)?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        Ok(entries)
    }

    pub fn entries_for_feed_filtered(
        &self,
        feed_id: i64,
        unread_only: bool,
        saved_only: bool,
    ) -> Result<Vec<Entry>> {
        if unread_only || saved_only {
            let mut stmt = self.conn.prepare(
                "SELECT e.id, e.title, e.url, e.published_at, e.fetched_at, e.summary, e.content, e.saved_at, rs.read_at
                 FROM entries e LEFT JOIN read_state rs ON rs.entry_id = e.id
                 WHERE e.feed_id = ?1
                   AND (?2 = 0 OR rs.entry_id IS NULL)
                   AND (?3 = 0 OR e.saved_at IS NOT NULL)
                 ORDER BY e.published_at DESC, e.fetched_at DESC;",
            )?;
            let rows = stmt.query_map(params![feed_id, unread_only, saved_only], map_entry_row)?;
            return collect_entries(rows);
        }

        self.entries_for_feed(feed_id)
    }

    pub fn search_entries(
        &self,
        feed_id: Option<i64>,
        query: &str,
        unread_only: bool,
        saved_only: bool,
    ) -> Result<Vec<Entry>> {
        let pattern = format!("%{}%", query);
        let mut base = String::from(
            "SELECT e.id, e.title, e.url, e.published_at, e.fetched_at, e.summary, e.content, e.saved_at, rs.read_at
             FROM entries e LEFT JOIN read_state rs ON rs.entry_id = e.id
             WHERE (e.title LIKE ?1 OR e.content_text LIKE ?1 OR e.summary LIKE ?1)",
        );
        if feed_id.is_some() {
            base.push_str(" AND e.feed_id = ?2");
        }
        if unread_only {
            base.push_str(" AND rs.entry_id IS NULL");
        }
        if saved_only {
            base.push_str(" AND e.saved_at IS NOT NULL");
        }
        base.push_str(" ORDER BY e.published_at DESC, e.fetched_at DESC;");

        let mut stmt = self.conn.prepare(&base)?;
        let rows = match feed_id {
            Some(feed_id) => stmt.query_map(params![pattern, feed_id], map_entry_row)?,
            None => stmt.query_map(params![pattern], map_entry_row)?,
        };
        collect_entries(rows)
    }

    pub fn mark_saved(&self, entry_id: i64, saved_at: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE entries SET saved_at = ?1 WHERE id = ?2;",
            params![saved_at, entry_id],
        )?;
        Ok(())
    }

    pub fn mark_unsaved(&self, entry_id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE entries SET saved_at = NULL WHERE id = ?1;",
            params![entry_id],
        )?;
        Ok(())
    }

    pub fn mark_read(&self, entry_id: i64, read_at: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO read_state (entry_id, read_at) VALUES (?1, ?2);",
            params![entry_id, read_at],
        )?;
        Ok(())
    }

    pub fn mark_unread(&self, entry_id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM read_state WHERE entry_id = ?1;",
            params![entry_id],
        )?;
        Ok(())
    }
}

fn map_feed_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Feed> {
    Ok(Feed {
        id: row.get(0)?,
        title: row.get(1)?,
        url: row.get(2)?,
        etag: row.get(3)?,
        last_modified: row.get(4)?,
        last_checked_at: row.get(5)?,
        group_id: row.get(6)?,
    })
}

fn map_entry_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Entry> {
    Ok(Entry {
        id: row.get(0)?,
        title: row.get(1)?,
        url: row.get(2)?,
        published_at: row.get(3)?,
        fetched_at: row.get(4)?,
        summary: row.get(5)?,
        content: row.get(6)?,
        saved_at: row.get(7)?,
        read_at: row.get(8)?,
    })
}

fn collect_entries<I>(rows: I) -> Result<Vec<Entry>>
where
    I: Iterator<Item = rusqlite::Result<Entry>>,
{
    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }
    Ok(entries)
}

#[cfg(test)]
impl Repo {
    pub fn new(conn: Connection) -> Result<Self> {
        apply_migrations(&conn)?;
        Ok(Self { conn })
    }

    pub fn unread_count_for_feed(&self, feed_id: i64) -> Result<i64> {
        self.conn.query_row(
            "SELECT COUNT(*) FROM entries e LEFT JOIN read_state rs ON rs.entry_id = e.id WHERE e.feed_id = ?1 AND rs.entry_id IS NULL;",
            params![feed_id],
            |row| row.get(0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::models::{NewEntry, NewFeed, NewGroup};
    use crate::util::time::now_timestamp;

    fn test_repo() -> Repo {
        let conn = Connection::open_in_memory().expect("in memory db");
        Repo::new(conn).expect("repo")
    }

    fn sample_feed(url: &str) -> NewFeed {
        NewFeed {
            title: Some("Sample".to_string()),
            url: url.to_string(),
            created_at: now_timestamp(),
        }
    }

    fn sample_entry(feed_id: i64, guid: &str, title: &str, fetched_at: i64) -> NewEntry {
        NewEntry {
            feed_id,
            guid: guid.to_string(),
            title: Some(title.to_string()),
            url: None,
            author: None,
            published_at: None,
            fetched_at,
            summary: None,
            content: None,
            content_text: Some(title.to_string()),
            hash: None,
        }
    }

    #[test]
    fn create_and_list_feeds() {
        let repo = test_repo();
        let feed = repo
            .create_feed(&sample_feed("https://example.com/rss"))
            .unwrap();
        let feeds = repo.list_feeds().unwrap();
        assert_eq!(feeds.len(), 1);
        assert_eq!(feeds[0].id, feed.id);
        assert_eq!(feeds[0].url, "https://example.com/rss");
    }

    #[test]
    fn upsert_entries_and_unread_counts() {
        let repo = test_repo();
        let feed = repo
            .create_feed(&sample_feed("https://example.com/rss"))
            .unwrap();
        let entries = vec![
            sample_entry(feed.id, "guid-1", "First", 1),
            sample_entry(feed.id, "guid-2", "Second", 2),
        ];
        repo.upsert_entries(&entries).unwrap();

        assert_eq!(repo.unread_count_all().unwrap(), 2);
        assert_eq!(repo.unread_count_for_feed(feed.id).unwrap(), 2);

        let entry_id = repo.entries_for_feed(feed.id).unwrap()[0].id;
        repo.mark_read(entry_id, now_timestamp()).unwrap();

        assert_eq!(repo.unread_count_all().unwrap(), 1);
        assert_eq!(repo.unread_count_for_feed(feed.id).unwrap(), 1);
        repo.mark_unread(entry_id).unwrap();
        assert_eq!(repo.unread_count_all().unwrap(), 2);
    }

    #[test]
    fn create_and_list_groups() {
        let repo = test_repo();
        let group = repo
            .create_group(&NewGroup {
                name: "Tech".to_string(),
                position: 0,
                created_at: now_timestamp(),
            })
            .unwrap();
        assert_eq!(group.name, "Tech");

        let groups = repo.list_groups().unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].name, "Tech");
    }

    #[test]
    fn rename_and_delete_group() {
        let repo = test_repo();
        let group = repo
            .create_group(&NewGroup {
                name: "Old".to_string(),
                position: 0,
                created_at: now_timestamp(),
            })
            .unwrap();
        repo.rename_group(group.id, "New").unwrap();
        let groups = repo.list_groups().unwrap();
        assert_eq!(groups[0].name, "New");

        repo.delete_group(group.id).unwrap();
        let groups = repo.list_groups().unwrap();
        assert!(groups.is_empty());
    }

    #[test]
    fn set_feed_group() {
        let repo = test_repo();
        let feed = repo
            .create_feed(&sample_feed("https://example.com/rss"))
            .unwrap();
        assert!(feed.group_id.is_none());

        let group = repo
            .create_group(&NewGroup {
                name: "Tech".to_string(),
                position: 0,
                created_at: now_timestamp(),
            })
            .unwrap();
        repo.set_feed_group(feed.id, Some(group.id)).unwrap();

        let updated = repo.get_feed(feed.id).unwrap().unwrap();
        assert_eq!(updated.group_id, Some(group.id));

        repo.set_feed_group(feed.id, None).unwrap();
        let updated = repo.get_feed(feed.id).unwrap().unwrap();
        assert!(updated.group_id.is_none());
    }

    #[test]
    fn filter_and_search_entries() {
        let repo = test_repo();
        let feed = repo
            .create_feed(&sample_feed("https://example.com/rss"))
            .unwrap();
        let entries = vec![
            sample_entry(feed.id, "guid-1", "Rust News", 1),
            sample_entry(feed.id, "guid-2", "Other", 2),
        ];
        repo.upsert_entries(&entries).unwrap();

        let all = repo
            .entries_for_feed_filtered(feed.id, false, false)
            .unwrap();
        assert_eq!(all.len(), 2);

        let entry_id = repo.entries_for_feed(feed.id).unwrap()[0].id;
        repo.mark_read(entry_id, now_timestamp()).unwrap();
        let unread_only = repo
            .entries_for_feed_filtered(feed.id, true, false)
            .unwrap();
        assert_eq!(unread_only.len(), 1);

        let search = repo
            .search_entries(Some(feed.id), "Rust", false, false)
            .unwrap();
        assert_eq!(search.len(), 1);
        assert_eq!(search[0].title.as_deref(), Some("Rust News"));
    }
}
