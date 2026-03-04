use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use crate::store::models::{Entry, Feed, Group, NewEntry, NewFeed, NewGroup};
use crate::store::repo::Repo;

pub type DbResult<T> = Result<T, String>;

#[derive(Debug, Clone)]
pub enum DbCommand {
    CreateFeed(NewFeed),
    DeleteFeed(i64),
    ListFeeds,
    UpdateFeed(Feed),
    UpdateFeedFetchState {
        feed_id: i64,
        etag: Option<String>,
        last_modified: Option<String>,
        last_checked_at: Option<i64>,
    },
    UpsertEntries(Vec<NewEntry>),
    EntriesForFeedFiltered {
        feed_id: i64,
        unread_only: bool,
        saved_only: bool,
    },
    SearchEntries {
        feed_id: Option<i64>,
        query: String,
        unread_only: bool,
        saved_only: bool,
    },
    MarkRead {
        entry_id: i64,
        read_at: i64,
    },
    MarkUnread(i64),
    MarkSaved {
        entry_id: i64,
        saved_at: i64,
    },
    MarkUnsaved(i64),
    UnreadCountAll,
    UnreadCountsByFeed,
    CreateGroup(NewGroup),
    ListGroups,
    DeleteGroup(i64),
    RenameGroup { id: i64, name: String },
    SetFeedGroup { feed_id: i64, group_id: Option<i64> },
    CountEntriesForFeed(i64),
}

#[derive(Debug, Clone)]
pub enum DbResponse {
    Feed(DbResult<Feed>),
    Feeds(DbResult<Vec<Feed>>),
    Entries(DbResult<Vec<Entry>>),
    Count(DbResult<i64>),
    Counts(DbResult<Vec<(i64, i64)>>),
    Updated(DbResult<usize>),
    Ok(DbResult<()>),
    Group(DbResult<Group>),
    Groups(DbResult<Vec<Group>>),
}

#[derive(Debug)]
pub enum DbWorkerError {
    Send,
    Recv,
    Init,
}

struct DbRequest {
    command: DbCommand,
    respond_to: Sender<DbResponse>,
}

#[derive(Clone)]
pub struct DbWorker {
    sender: Sender<DbRequest>,
}

impl DbWorker {
    pub fn start(path: impl AsRef<Path>) -> Result<Self, DbWorkerError> {
        let repo = Repo::open(path).map_err(|_| DbWorkerError::Init)?;
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || run_worker(repo, rx));
        Ok(Self { sender: tx })
    }

    pub fn send(&self, command: DbCommand) -> Result<DbResponse, DbWorkerError> {
        let (tx, rx) = mpsc::channel();
        self.sender
            .send(DbRequest {
                command,
                respond_to: tx,
            })
            .map_err(|_| DbWorkerError::Send)?;
        rx.recv().map_err(|_| DbWorkerError::Recv)
    }
}

fn run_worker(repo: Repo, rx: Receiver<DbRequest>) {
    for request in rx {
        let response = handle_command(&repo, request.command);
        let _ = request.respond_to.send(response);
    }
}

fn handle_command(repo: &Repo, command: DbCommand) -> DbResponse {
    match command {
        DbCommand::CreateFeed(feed) => DbResponse::Feed(map_result(repo.create_feed(&feed))),
        DbCommand::DeleteFeed(feed_id) => DbResponse::Ok(map_result(repo.delete_feed(feed_id))),
        DbCommand::ListFeeds => DbResponse::Feeds(map_result(repo.list_feeds())),
        DbCommand::UpdateFeed(feed) => DbResponse::Ok(map_result(repo.update_feed(&feed))),
        DbCommand::UpdateFeedFetchState {
            feed_id,
            etag,
            last_modified,
            last_checked_at,
        } => DbResponse::Ok(map_result(repo.update_feed_fetch_state(
            feed_id,
            etag.as_deref(),
            last_modified.as_deref(),
            last_checked_at,
        ))),
        DbCommand::UpsertEntries(entries) => {
            DbResponse::Updated(map_result(repo.upsert_entries(&entries)))
        }
        DbCommand::EntriesForFeedFiltered {
            feed_id,
            unread_only,
            saved_only,
        } => DbResponse::Entries(map_result(repo.entries_for_feed_filtered(
            feed_id,
            unread_only,
            saved_only,
        ))),
        DbCommand::SearchEntries {
            feed_id,
            query,
            unread_only,
            saved_only,
        } => DbResponse::Entries(map_result(repo.search_entries(
            feed_id,
            &query,
            unread_only,
            saved_only,
        ))),
        DbCommand::MarkRead { entry_id, read_at } => {
            DbResponse::Ok(map_result(repo.mark_read(entry_id, read_at)))
        }
        DbCommand::MarkUnread(entry_id) => DbResponse::Ok(map_result(repo.mark_unread(entry_id))),
        DbCommand::MarkSaved { entry_id, saved_at } => {
            DbResponse::Ok(map_result(repo.mark_saved(entry_id, saved_at)))
        }
        DbCommand::MarkUnsaved(entry_id) => DbResponse::Ok(map_result(repo.mark_unsaved(entry_id))),
        DbCommand::UnreadCountAll => DbResponse::Count(map_result(repo.unread_count_all())),
        DbCommand::UnreadCountsByFeed => {
            DbResponse::Counts(map_result(repo.unread_counts_by_feed()))
        }
        DbCommand::CreateGroup(group) => {
            DbResponse::Group(map_result(repo.create_group(&group)))
        }
        DbCommand::ListGroups => DbResponse::Groups(map_result(repo.list_groups())),
        DbCommand::DeleteGroup(id) => DbResponse::Ok(map_result(repo.delete_group(id))),
        DbCommand::RenameGroup { id, name } => {
            DbResponse::Ok(map_result(repo.rename_group(id, &name)))
        }
        DbCommand::SetFeedGroup { feed_id, group_id } => {
            DbResponse::Ok(map_result(repo.set_feed_group(feed_id, group_id)))
        }
        DbCommand::CountEntriesForFeed(feed_id) => {
            DbResponse::Count(map_result(repo.count_entries_for_feed(feed_id)))
        }
    }
}

fn map_result<T>(result: rusqlite::Result<T>) -> DbResult<T> {
    result.map_err(|error| error.to_string())
}

#[cfg(test)]
impl DbWorker {
    pub fn start_in_memory() -> Result<Self, DbWorkerError> {
        let conn = rusqlite::Connection::open_in_memory().map_err(|_| DbWorkerError::Init)?;
        let repo = Repo::new(conn).map_err(|_| DbWorkerError::Init)?;
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || run_worker(repo, rx));
        Ok(Self { sender: tx })
    }
}
