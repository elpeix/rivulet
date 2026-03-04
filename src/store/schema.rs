use rusqlite::{Connection, Result};

const SCHEMA_VERSION: i64 = 3;

const MIGRATION_1: &str = r#"
CREATE TABLE IF NOT EXISTS feeds (
  id INTEGER PRIMARY KEY,
  title TEXT,
  url TEXT NOT NULL UNIQUE,
  etag TEXT,
  last_modified TEXT,
  last_checked_at INTEGER,
  created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS entries (
  id INTEGER PRIMARY KEY,
  feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
  guid TEXT NOT NULL,
  title TEXT,
  url TEXT,
  author TEXT,
  published_at INTEGER,
  fetched_at INTEGER NOT NULL,
  summary TEXT,
  content TEXT,
  content_text TEXT,
  hash TEXT,
  UNIQUE(feed_id, guid)
);

CREATE TABLE IF NOT EXISTS read_state (
  entry_id INTEGER PRIMARY KEY REFERENCES entries(id) ON DELETE CASCADE,
  read_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_entries_feed_pub
  ON entries(feed_id, published_at DESC);
CREATE INDEX IF NOT EXISTS idx_entries_feed_fetch
  ON entries(feed_id, fetched_at DESC);
CREATE INDEX IF NOT EXISTS idx_read_state_read_at
  ON read_state(read_at DESC);
"#;

const MIGRATION_2: &str = r#"
ALTER TABLE entries ADD COLUMN saved_at INTEGER;
"#;

const MIGRATION_3: &str = r#"
CREATE TABLE IF NOT EXISTS groups (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  position INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL
);

ALTER TABLE feeds ADD COLUMN group_id INTEGER REFERENCES groups(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_feeds_group ON feeds(group_id);
"#;

pub fn apply_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL);")?;

    let version: i64 = conn
        .query_row(
            "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1;",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if version < 1 {
        conn.execute_batch(MIGRATION_1)?;
        conn.execute_batch("INSERT INTO schema_version (version) VALUES (1);")?;
    }

    if version < 2 {
        conn.execute_batch(MIGRATION_2)?;
        conn.execute_batch("INSERT INTO schema_version (version) VALUES (2);")?;
    }

    if version < 3 {
        conn.execute_batch(MIGRATION_3)?;
        conn.execute_batch("INSERT INTO schema_version (version) VALUES (3);")?;
    }

    if version > SCHEMA_VERSION {
        return Err(rusqlite::Error::ExecuteReturnedResults);
    }

    Ok(())
}
