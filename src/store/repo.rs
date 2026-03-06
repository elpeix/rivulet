use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Result};
use rusqlite::types::Value;

use super::models::{Entry, Feed, Group, NewEntry, NewFeed, NewGroup};
use super::schema::apply_migrations;
use crate::app::state::SortMode;

fn order_clause(mode: SortMode) -> &'static str {
    match mode {
        SortMode::DateDesc => "ORDER BY e.published_at DESC, e.fetched_at DESC",
        SortMode::DateAsc => "ORDER BY e.published_at ASC, e.fetched_at ASC",
        SortMode::TitleAsc => "ORDER BY LOWER(e.title) ASC, e.published_at DESC",
    }
}

enum GroupScope {
    All,
    Group(i64),
    Ungrouped,
}

struct EntryFilter {
    feed_id: Option<i64>,
    group_scope: GroupScope,
    search: Option<String>,
    unread_only: bool,
    saved_only: bool,
    since: Option<i64>,
    sort_mode: SortMode,
}

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

    pub fn rename_feed(&self, feed_id: i64, custom_title: Option<&str>) -> Result<()> {
        self.conn.execute(
            "UPDATE feeds SET custom_title = ?1 WHERE id = ?2;",
            params![custom_title, feed_id],
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
                "SELECT id, title, custom_title, url, etag, last_modified, last_checked_at, group_id FROM feeds WHERE id = ?1;",
                params![feed_id],
                map_feed_row,
            )
            .optional()
    }

    pub fn list_feeds(&self) -> Result<Vec<Feed>> {
        let mut stmt = self.conn.prepare(
            "SELECT f.id, f.title, f.custom_title, f.url, f.etag, f.last_modified, f.last_checked_at, f.group_id
             FROM feeds f
             LEFT JOIN groups g ON g.id = f.group_id
             ORDER BY COALESCE(g.position, 999999), g.name, LOWER(COALESCE(f.custom_title, f.title));",
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

    pub fn swap_group_positions(&self, id_a: i64, id_b: i64) -> Result<()> {
        self.conn.execute_batch("BEGIN IMMEDIATE;")?;
        let result = (|| {
            let pos_a: i64 = self.conn.query_row(
                "SELECT position FROM groups WHERE id = ?1;",
                params![id_a],
                |row| row.get(0),
            )?;
            let pos_b: i64 = self.conn.query_row(
                "SELECT position FROM groups WHERE id = ?1;",
                params![id_b],
                |row| row.get(0),
            )?;
            self.conn.execute(
                "UPDATE groups SET position = ?1 WHERE id = ?2;",
                params![pos_b, id_a],
            )?;
            self.conn.execute(
                "UPDATE groups SET position = ?1 WHERE id = ?2;",
                params![pos_a, id_b],
            )?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.conn.execute_batch("COMMIT;")?;
                Ok(())
            }
            Err(e) => {
                let _ = self.conn.execute_batch("ROLLBACK;");
                Err(e)
            }
        }
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
                        hash = excluded.hash
                    WHERE entries.hash IS NULL OR entries.hash != excluded.hash;",
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

    pub fn unread_count_all(&self, since: Option<i64>) -> Result<i64> {
        let mut sql = String::from(
            "SELECT COUNT(*) FROM entries e LEFT JOIN read_state rs ON rs.entry_id = e.id WHERE rs.entry_id IS NULL",
        );
        if since.is_some() {
            sql.push_str(" AND COALESCE(e.published_at, e.fetched_at) >= ?1");
        }
        match since {
            Some(ts) => self.conn.query_row(&sql, params![ts], |row| row.get(0)),
            None => self.conn.query_row(&sql, [], |row| row.get(0)),
        }
    }

    pub fn unread_counts_by_feed(&self, since: Option<i64>) -> Result<Vec<(i64, i64)>> {
        let mut sql = String::from(
            "SELECT e.feed_id, COUNT(*) FROM entries e LEFT JOIN read_state rs ON rs.entry_id = e.id WHERE rs.entry_id IS NULL",
        );
        if since.is_some() {
            sql.push_str(" AND COALESCE(e.published_at, e.fetched_at) >= ?1");
        }
        sql.push_str(" GROUP BY e.feed_id");
        let mapper = |row: &rusqlite::Row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?));
        let mut stmt = self.conn.prepare(&sql)?;
        let mut counts = Vec::new();
        if let Some(ts) = since {
            let rows = stmt.query_map(params![ts], mapper)?;
            for row in rows { counts.push(row?); }
        } else {
            let rows = stmt.query_map([], mapper)?;
            for row in rows { counts.push(row?); }
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

    fn query_entries(&self, filter: &EntryFilter) -> Result<Vec<Entry>> {
        let mut sql = String::from(
            "SELECT e.id, e.feed_id, e.title, e.url, e.author, e.published_at, e.fetched_at, \
             e.summary, e.content, e.saved_at, rs.read_at \
             FROM entries e LEFT JOIN read_state rs ON rs.entry_id = e.id",
        );
        if !matches!(filter.group_scope, GroupScope::All) {
            sql.push_str(" JOIN feeds f ON f.id = e.feed_id");
        }

        let mut conditions: Vec<String> = Vec::new();
        let mut values: Vec<Value> = Vec::new();

        if let Some(feed_id) = filter.feed_id {
            values.push(Value::Integer(feed_id));
            conditions.push(format!("e.feed_id = ?{}", values.len()));
        }
        match filter.group_scope {
            GroupScope::All => {}
            GroupScope::Group(gid) => {
                values.push(Value::Integer(gid));
                conditions.push(format!("f.group_id = ?{}", values.len()));
            }
            GroupScope::Ungrouped => {
                conditions.push("f.group_id IS NULL".to_string());
            }
        }
        if let Some(ref query) = filter.search {
            values.push(Value::Text(format!("%{query}%")));
            let idx = values.len();
            conditions.push(format!(
                "(e.title LIKE ?{idx} OR e.content_text LIKE ?{idx} OR e.summary LIKE ?{idx})"
            ));
        }
        if let Some(ts) = filter.since {
            values.push(Value::Integer(ts));
            conditions.push(format!(
                "COALESCE(e.published_at, e.fetched_at) >= ?{}",
                values.len()
            ));
        }
        if filter.unread_only {
            conditions.push("rs.entry_id IS NULL".to_string());
        }
        if filter.saved_only {
            conditions.push("e.saved_at IS NOT NULL".to_string());
        }

        if conditions.is_empty() {
            sql.push_str(" WHERE 1=1");
        } else {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push(' ');
        sql.push_str(order_clause(filter.sort_mode));
        sql.push(';');

        let mut stmt = self.conn.prepare(&sql)?;
        let refs: Vec<&dyn rusqlite::types::ToSql> =
            values.iter().map(|v| v as &dyn rusqlite::types::ToSql).collect();
        let rows = stmt.query_map(refs.as_slice(), map_entry_row)?;
        collect_entries(rows)
    }

    pub fn all_entries_filtered(
        &self,
        unread_only: bool,
        saved_only: bool,
        since: Option<i64>,
        sort_mode: SortMode,
    ) -> Result<Vec<Entry>> {
        self.query_entries(&EntryFilter {
            feed_id: None,
            group_scope: GroupScope::All,
            search: None,
            unread_only,
            saved_only,
            since,
            sort_mode,
        })
    }

    pub fn count_all_entries(&self) -> Result<i64> {
        self.conn.query_row(
            "SELECT COUNT(*) FROM entries;",
            [],
            |row| row.get(0),
        )
    }

    pub fn entries_for_group_filtered(
        &self,
        group_id: Option<i64>,
        unread_only: bool,
        saved_only: bool,
        since: Option<i64>,
        sort_mode: SortMode,
    ) -> Result<Vec<Entry>> {
        let group_scope = match group_id {
            Some(gid) => GroupScope::Group(gid),
            None => GroupScope::Ungrouped,
        };
        self.query_entries(&EntryFilter {
            feed_id: None,
            group_scope,
            search: None,
            unread_only,
            saved_only,
            since,
            sort_mode,
        })
    }

    pub fn count_entries_for_group(&self, group_id: Option<i64>) -> Result<i64> {
        match group_id {
            Some(gid) => self.conn.query_row(
                "SELECT COUNT(*) FROM entries e JOIN feeds f ON f.id = e.feed_id WHERE f.group_id = ?1;",
                params![gid],
                |row| row.get(0),
            ),
            None => self.conn.query_row(
                "SELECT COUNT(*) FROM entries e JOIN feeds f ON f.id = e.feed_id WHERE f.group_id IS NULL;",
                [],
                |row| row.get(0),
            ),
        }
    }

    pub fn entries_for_feed_filtered(
        &self,
        feed_id: i64,
        unread_only: bool,
        saved_only: bool,
        since: Option<i64>,
        sort_mode: SortMode,
    ) -> Result<Vec<Entry>> {
        self.query_entries(&EntryFilter {
            feed_id: Some(feed_id),
            group_scope: GroupScope::All,
            search: None,
            unread_only,
            saved_only,
            since,
            sort_mode,
        })
    }

    pub fn search_entries(
        &self,
        feed_id: Option<i64>,
        query: &str,
        unread_only: bool,
        saved_only: bool,
        since: Option<i64>,
    ) -> Result<Vec<Entry>> {
        self.query_entries(&EntryFilter {
            feed_id,
            group_scope: GroupScope::All,
            search: Some(query.to_string()),
            unread_only,
            saved_only,
            since,
            sort_mode: SortMode::DateDesc,
        })
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

    pub fn mark_all_read(&self, entry_ids: &[i64], read_at: i64) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        for &id in entry_ids {
            tx.execute(
                "INSERT OR REPLACE INTO read_state (entry_id, read_at) VALUES (?1, ?2);",
                params![id, read_at],
            )?;
        }
        tx.commit()
    }

    pub fn mark_feed_read(&self, feed_id: i64, read_at: i64) -> Result<usize> {
        let count = self.conn.execute(
            "INSERT OR REPLACE INTO read_state (entry_id, read_at)
             SELECT id, ?2 FROM entries WHERE feed_id = ?1
             AND id NOT IN (SELECT entry_id FROM read_state);",
            params![feed_id, read_at],
        )?;
        Ok(count)
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
        custom_title: row.get(2)?,
        url: row.get(3)?,
        etag: row.get(4)?,
        last_modified: row.get(5)?,
        last_checked_at: row.get(6)?,
        group_id: row.get(7)?,
    })
}

fn map_entry_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Entry> {
    Ok(Entry {
        id: row.get(0)?,
        feed_id: row.get(1)?,
        title: row.get(2)?,
        url: row.get(3)?,
        author: row.get(4)?,
        published_at: row.get(5)?,
        fetched_at: row.get(6)?,
        summary: row.get(7)?,
        content: row.get(8)?,
        saved_at: row.get(9)?,
        read_at: row.get(10)?,
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

    pub fn entries_for_feed(&self, feed_id: i64) -> Result<Vec<Entry>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.id, e.feed_id, e.title, e.url, e.author, e.published_at, e.fetched_at, e.summary, e.content, e.saved_at, rs.read_at
             FROM entries e LEFT JOIN read_state rs ON rs.entry_id = e.id
             WHERE e.feed_id = ?1 ORDER BY e.published_at DESC, e.fetched_at DESC;",
        )?;
        let rows = stmt.query_map(params![feed_id], map_entry_row)?;
        collect_entries(rows)
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
    use crate::app::state::SortMode;
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

        assert_eq!(repo.unread_count_all(None).unwrap(), 2);
        assert_eq!(repo.unread_count_for_feed(feed.id).unwrap(), 2);

        let entry_id = repo.entries_for_feed(feed.id).unwrap()[0].id;
        repo.mark_read(entry_id, now_timestamp()).unwrap();

        assert_eq!(repo.unread_count_all(None).unwrap(), 1);
        assert_eq!(repo.unread_count_for_feed(feed.id).unwrap(), 1);
        repo.mark_unread(entry_id).unwrap();
        assert_eq!(repo.unread_count_all(None).unwrap(), 2);
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
            .entries_for_feed_filtered(feed.id, false, false, None, SortMode::DateDesc)
            .unwrap();
        assert_eq!(all.len(), 2);

        let entry_id = repo.entries_for_feed(feed.id).unwrap()[0].id;
        repo.mark_read(entry_id, now_timestamp()).unwrap();
        let unread_only = repo
            .entries_for_feed_filtered(feed.id, true, false, None, SortMode::DateDesc)
            .unwrap();
        assert_eq!(unread_only.len(), 1);

        let search = repo
            .search_entries(Some(feed.id), "Rust", false, false, None)
            .unwrap();
        assert_eq!(search.len(), 1);
        assert_eq!(search[0].title.as_deref(), Some("Rust News"));
    }

    #[test]
    fn rename_feed_custom_title() {
        let repo = test_repo();
        let feed = repo
            .create_feed(&sample_feed("https://example.com/rss"))
            .unwrap();
        assert!(feed.custom_title.is_none());

        repo.rename_feed(feed.id, Some("My Feed")).unwrap();
        let updated = repo.get_feed(feed.id).unwrap().unwrap();
        assert_eq!(updated.custom_title.as_deref(), Some("My Feed"));
        assert_eq!(updated.display_title(), Some("My Feed"));

        repo.rename_feed(feed.id, None).unwrap();
        let restored = repo.get_feed(feed.id).unwrap().unwrap();
        assert!(restored.custom_title.is_none());
        assert_eq!(restored.display_title(), Some("Sample"));
    }

    #[test]
    fn mark_feed_read() {
        let repo = test_repo();
        let feed = repo
            .create_feed(&sample_feed("https://example.com/rss"))
            .unwrap();
        let entries = vec![
            sample_entry(feed.id, "g1", "A", 1),
            sample_entry(feed.id, "g2", "B", 2),
            sample_entry(feed.id, "g3", "C", 3),
        ];
        repo.upsert_entries(&entries).unwrap();
        assert_eq!(repo.unread_count_for_feed(feed.id).unwrap(), 3);

        let count = repo.mark_feed_read(feed.id, now_timestamp()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(repo.unread_count_for_feed(feed.id).unwrap(), 0);
    }

    #[test]
    fn mark_saved_and_unsaved() {
        let repo = test_repo();
        let feed = repo
            .create_feed(&sample_feed("https://example.com/rss"))
            .unwrap();
        repo.upsert_entries(&[sample_entry(feed.id, "g1", "A", 1)])
            .unwrap();
        let entry_id = repo.entries_for_feed(feed.id).unwrap()[0].id;

        repo.mark_saved(entry_id, now_timestamp()).unwrap();
        let saved = repo
            .entries_for_feed_filtered(feed.id, false, true, None, SortMode::DateDesc)
            .unwrap();
        assert_eq!(saved.len(), 1);

        repo.mark_unsaved(entry_id).unwrap();
        let saved = repo
            .entries_for_feed_filtered(feed.id, false, true, None, SortMode::DateDesc)
            .unwrap();
        assert!(saved.is_empty());
    }

    #[test]
    fn swap_group_positions() {
        let repo = test_repo();
        let g1 = repo
            .create_group(&NewGroup {
                name: "First".to_string(),
                position: 0,
                created_at: now_timestamp(),
            })
            .unwrap();
        let g2 = repo
            .create_group(&NewGroup {
                name: "Second".to_string(),
                position: 1,
                created_at: now_timestamp(),
            })
            .unwrap();

        repo.swap_group_positions(g1.id, g2.id).unwrap();
        let groups = repo.list_groups().unwrap();
        assert_eq!(groups[0].name, "Second");
        assert_eq!(groups[1].name, "First");
    }

    #[test]
    fn entries_sort_order() {
        let repo = test_repo();
        let feed = repo
            .create_feed(&sample_feed("https://example.com/rss"))
            .unwrap();
        let entries = vec![
            sample_entry(feed.id, "g1", "Banana", 1),
            sample_entry(feed.id, "g2", "Apple", 3),
            sample_entry(feed.id, "g3", "Cherry", 2),
        ];
        repo.upsert_entries(&entries).unwrap();

        let desc = repo
            .entries_for_feed_filtered(
                feed.id, false, false, None,
                SortMode::DateDesc,
            )
            .unwrap();
        assert_eq!(desc[0].title.as_deref(), Some("Apple"));

        let asc = repo
            .entries_for_feed_filtered(
                feed.id, false, false, None,
                SortMode::DateAsc,
            )
            .unwrap();
        assert_eq!(asc[0].title.as_deref(), Some("Banana"));

        let by_title = repo
            .entries_for_feed_filtered(
                feed.id, false, false, None,
                SortMode::TitleAsc,
            )
            .unwrap();
        assert_eq!(by_title[0].title.as_deref(), Some("Apple"));
        assert_eq!(by_title[1].title.as_deref(), Some("Banana"));
        assert_eq!(by_title[2].title.as_deref(), Some("Cherry"));
    }

    #[test]
    fn entries_for_group_filtered() {
        let repo = test_repo();
        let feed1 = repo
            .create_feed(&sample_feed("https://a.com/rss"))
            .unwrap();
        let feed2 = repo
            .create_feed(&sample_feed("https://b.com/rss"))
            .unwrap();
        let group = repo
            .create_group(&NewGroup {
                name: "G".to_string(),
                position: 0,
                created_at: now_timestamp(),
            })
            .unwrap();
        repo.set_feed_group(feed1.id, Some(group.id)).unwrap();

        repo.upsert_entries(&[sample_entry(feed1.id, "g1", "In group", 1)])
            .unwrap();
        repo.upsert_entries(&[sample_entry(feed2.id, "g2", "Ungrouped", 2)])
            .unwrap();

        let grouped = repo
            .entries_for_group_filtered(Some(group.id), false, false, None, SortMode::DateDesc)
            .unwrap();
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[0].title.as_deref(), Some("In group"));

        let ungrouped = repo
            .entries_for_group_filtered(None, false, false, None, SortMode::DateDesc)
            .unwrap();
        assert_eq!(ungrouped.len(), 1);
        assert_eq!(ungrouped[0].title.as_deref(), Some("Ungrouped"));

        assert_eq!(repo.count_entries_for_group(Some(group.id)).unwrap(), 1);
        assert_eq!(repo.count_entries_for_group(None).unwrap(), 1);
    }

    #[test]
    fn unread_counts_with_since_filter() {
        let repo = test_repo();
        let feed = repo
            .create_feed(&sample_feed("https://example.com/rss"))
            .unwrap();
        let now = now_timestamp();
        let old = now - 90 * 86400; // 90 days ago
        let entries = vec![
            sample_entry(feed.id, "g1", "Old", old),
            sample_entry(feed.id, "g2", "Recent", now),
        ];
        repo.upsert_entries(&entries).unwrap();

        assert_eq!(repo.unread_count_all(None).unwrap(), 2);

        let since = now - 30 * 86400;
        assert_eq!(repo.unread_count_all(Some(since)).unwrap(), 1);

        let by_feed = repo.unread_counts_by_feed(Some(since)).unwrap();
        assert_eq!(by_feed.len(), 1);
        assert_eq!(by_feed[0], (feed.id, 1));
    }

    #[test]
    fn all_entries_filtered() {
        let repo = test_repo();
        let feed1 = repo
            .create_feed(&sample_feed("https://a.com/rss"))
            .unwrap();
        let feed2 = repo
            .create_feed(&sample_feed("https://b.com/rss"))
            .unwrap();
        repo.upsert_entries(&[sample_entry(feed1.id, "g1", "A", 1)])
            .unwrap();
        repo.upsert_entries(&[sample_entry(feed2.id, "g2", "B", 2)])
            .unwrap();

        let all = repo
            .all_entries_filtered(false, false, None, SortMode::DateDesc)
            .unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(repo.count_all_entries().unwrap(), 2);
    }
}
