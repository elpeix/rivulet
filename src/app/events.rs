use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use log::warn;

use crate::app::state::SortMode;
use crate::store::models::{Entry, Feed, Group, NewEntry, NewFeed, NewGroup};
use crate::store::repo::Repo;

pub type DbResult<T> = Result<T, String>;

#[derive(Debug, Clone)]
pub enum DbCommand {
    CreateFeed(NewFeed),
    DeleteFeed(i64),
    RenameFeed {
        id: i64,
        title: Option<String>,
    },
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
        since: Option<i64>,
        sort_mode: SortMode,
    },
    SearchEntries {
        feed_id: Option<i64>,
        query: String,
        unread_only: bool,
        saved_only: bool,
        since: Option<i64>,
    },
    MarkRead {
        entry_id: i64,
        read_at: i64,
    },
    MarkUnread(i64),
    MarkAllRead {
        entry_ids: Vec<i64>,
        read_at: i64,
    },
    MarkFeedRead {
        feed_id: i64,
        read_at: i64,
    },
    MarkSaved {
        entry_id: i64,
        saved_at: i64,
    },
    MarkUnsaved(i64),
    UnreadCountAll {
        since: Option<i64>,
    },
    UnreadCountsByFeed {
        since: Option<i64>,
    },
    CreateGroup(NewGroup),
    ListGroups,
    DeleteGroup(i64),
    RenameGroup {
        id: i64,
        name: String,
    },
    SetFeedGroup {
        feed_id: i64,
        group_id: Option<i64>,
    },
    SwapGroupPositions {
        id_a: i64,
        id_b: i64,
    },
    CountEntriesForFeed(i64),
    AllEntriesFiltered {
        unread_only: bool,
        saved_only: bool,
        since: Option<i64>,
        sort_mode: SortMode,
    },
    CountAllEntries,
    EntriesForGroupFiltered {
        group_id: Option<i64>,
        unread_only: bool,
        saved_only: bool,
        since: Option<i64>,
        sort_mode: SortMode,
    },
    CountEntriesForGroup(Option<i64>),
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
        if request.respond_to.send(response).is_err() {
            warn!("DB worker: caller disconnected before receiving response");
        }
    }
}

fn handle_command(repo: &Repo, command: DbCommand) -> DbResponse {
    match command {
        DbCommand::CreateFeed(feed) => DbResponse::Feed(map_result(repo.create_feed(&feed))),
        DbCommand::DeleteFeed(feed_id) => DbResponse::Ok(map_result(repo.delete_feed(feed_id))),
        DbCommand::RenameFeed { id, title } => {
            DbResponse::Ok(map_result(repo.rename_feed(id, title.as_deref())))
        }
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
            since,
            sort_mode,
        } => DbResponse::Entries(map_result(repo.entries_for_feed_filtered(
            feed_id,
            unread_only,
            saved_only,
            since,
            sort_mode,
        ))),
        DbCommand::SearchEntries {
            feed_id,
            query,
            unread_only,
            saved_only,
            since,
        } => DbResponse::Entries(map_result(repo.search_entries(
            feed_id,
            &query,
            unread_only,
            saved_only,
            since,
        ))),
        DbCommand::MarkRead { entry_id, read_at } => {
            DbResponse::Ok(map_result(repo.mark_read(entry_id, read_at)))
        }
        DbCommand::MarkUnread(entry_id) => DbResponse::Ok(map_result(repo.mark_unread(entry_id))),
        DbCommand::MarkAllRead { entry_ids, read_at } => {
            DbResponse::Ok(map_result(repo.mark_all_read(&entry_ids, read_at)))
        }
        DbCommand::MarkFeedRead { feed_id, read_at } => DbResponse::Ok(map_result(
            repo.mark_feed_read(feed_id, read_at).map(|_| ()),
        )),
        DbCommand::MarkSaved { entry_id, saved_at } => {
            DbResponse::Ok(map_result(repo.mark_saved(entry_id, saved_at)))
        }
        DbCommand::MarkUnsaved(entry_id) => DbResponse::Ok(map_result(repo.mark_unsaved(entry_id))),
        DbCommand::UnreadCountAll { since } => {
            DbResponse::Count(map_result(repo.unread_count_all(since)))
        }
        DbCommand::UnreadCountsByFeed { since } => {
            DbResponse::Counts(map_result(repo.unread_counts_by_feed(since)))
        }
        DbCommand::CreateGroup(group) => DbResponse::Group(map_result(repo.create_group(&group))),
        DbCommand::ListGroups => DbResponse::Groups(map_result(repo.list_groups())),
        DbCommand::DeleteGroup(id) => DbResponse::Ok(map_result(repo.delete_group(id))),
        DbCommand::RenameGroup { id, name } => {
            DbResponse::Ok(map_result(repo.rename_group(id, &name)))
        }
        DbCommand::SetFeedGroup { feed_id, group_id } => {
            DbResponse::Ok(map_result(repo.set_feed_group(feed_id, group_id)))
        }
        DbCommand::SwapGroupPositions { id_a, id_b } => {
            DbResponse::Ok(map_result(repo.swap_group_positions(id_a, id_b)))
        }
        DbCommand::CountEntriesForFeed(feed_id) => {
            DbResponse::Count(map_result(repo.count_entries_for_feed(feed_id)))
        }
        DbCommand::EntriesForGroupFiltered {
            group_id,
            unread_only,
            saved_only,
            since,
            sort_mode,
        } => DbResponse::Entries(map_result(repo.entries_for_group_filtered(
            group_id,
            unread_only,
            saved_only,
            since,
            sort_mode,
        ))),
        DbCommand::CountEntriesForGroup(group_id) => {
            DbResponse::Count(map_result(repo.count_entries_for_group(group_id)))
        }
        DbCommand::AllEntriesFiltered {
            unread_only,
            saved_only,
            since,
            sort_mode,
        } => DbResponse::Entries(map_result(repo.all_entries_filtered(
            unread_only,
            saved_only,
            since,
            sort_mode,
        ))),
        DbCommand::CountAllEntries => DbResponse::Count(map_result(repo.count_all_entries())),
    }
}

fn map_result<T>(result: rusqlite::Result<T>) -> DbResult<T> {
    result.map_err(|error| {
        warn!("SQLite error: {error}");
        error.to_string()
    })
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
