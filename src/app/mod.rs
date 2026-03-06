pub mod actions;
pub mod events;
pub mod input;
pub mod state;

use std::sync::mpsc::Receiver;
use std::sync::Arc;

use log::{error, info, warn};

use crate::fetch::client::{FetchOptions, HttpClient};
use crate::fetch::parser::{map_entries, parse_feed};
use crate::fetch::scheduler::{FetchJob, Scheduler};
use crate::i18n::Lang;
use crate::store::models::{Feed, NewFeed, NewGroup};
use crate::util::time::now_timestamp;
use actions::Action;
use events::{DbCommand, DbResponse, DbWorker, DbWorkerError};
use state::AppState;

use tokio::runtime::Runtime;

pub struct RefreshComplete {
    pub refreshed: usize,
    pub updated_entries: i64,
    pub errors: usize,
    pub last_error: Option<String>,
}

pub struct App {
    pub state: AppState,
    pub lang: Lang,
    pub recent_days: i64,
    db: DbWorker,
    client: HttpClient,
    max_concurrency: usize,
    refresh_rx: Option<Receiver<RefreshComplete>>,
    runtime: Arc<Runtime>,
}

impl App {
    pub fn new(db: DbWorker, lang: Lang, recent_days: i64) -> Result<Self, String> {
        let client =
            HttpClient::new(FetchOptions::default()).map_err(|error| format!("{error}"))?;

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .map_err(|e| format!("Failed to create async runtime: {e}"))?;

        Ok(Self {
            state: AppState::default(),
            lang,
            recent_days,
            db,
            client,
            max_concurrency: 6,
            refresh_rx: None,
            runtime: Arc::new(runtime),
        })
    }

    pub fn since_cutoff(&self) -> Option<i64> {
        if self.state.recent_only {
            Some(now_timestamp() - self.recent_days * 86400)
        } else {
            None
        }
    }

    pub fn dispatch(&mut self, action: Action) -> Result<(), DbWorkerError> {
        match action {
            // Feeds
            Action::LoadFeeds => {
                let response = self.db.send(DbCommand::ListFeeds)?;
                self.handle_db_response(response);
            }
            Action::AddFeed { title, url, group_id } => self.add_feed(title, url, group_id)?,
            Action::RenameFeed { id, title } => {
                self.send_and_handle(DbCommand::RenameFeed { id, title })?;
                self.send_and_handle(DbCommand::ListFeeds)?;
            }
            Action::DeleteFeed(feed_id) => {
                self.send_and_handle(DbCommand::DeleteFeed(feed_id))?;
                self.send_and_handle(DbCommand::ListFeeds)?;
            }
            Action::RefreshFeeds => self.refresh_feeds()?,

            // Entries
            Action::LoadEntriesFiltered { feed_id, unread_only, saved_only, since } => {
                self.load_entries_for_feed(feed_id, unread_only, saved_only, since)?;
            }
            Action::LoadAllEntries { unread_only, saved_only, since } => {
                self.load_all_entries(unread_only, saved_only, since)?;
            }
            Action::LoadEntriesForGroup { group_id, unread_only, saved_only, since } => {
                self.load_entries_for_group(group_id, unread_only, saved_only, since)?;
            }
            Action::SetSearchQuery(query) => self.set_search_query(query)?,
            Action::ToggleUnreadFilter => self.state.reduce(Action::ToggleUnreadFilter),
            Action::ToggleSavedFilter => self.state.reduce(Action::ToggleSavedFilter),

            // Read/save state
            Action::RefreshUnreadCounts => {
                let since = self.since_cutoff();
                self.send_and_handle(DbCommand::UnreadCountsByFeed { since })?;
                self.send_and_handle(DbCommand::UnreadCountAll { since })?;
            }
            Action::MarkRead(id) => {
                self.send_and_handle(DbCommand::MarkRead { entry_id: id, read_at: now_timestamp() })?;
            }
            Action::MarkUnread(id) => self.send_and_handle(DbCommand::MarkUnread(id))?,
            Action::MarkAllRead(ids) => {
                self.send_and_handle(DbCommand::MarkAllRead { entry_ids: ids, read_at: now_timestamp() })?;
            }
            Action::MarkFeedRead(id) => {
                self.send_and_handle(DbCommand::MarkFeedRead { feed_id: id, read_at: now_timestamp() })?;
            }
            Action::MarkSaved(id) => {
                self.send_and_handle(DbCommand::MarkSaved { entry_id: id, saved_at: now_timestamp() })?;
            }
            Action::MarkUnsaved(id) => self.send_and_handle(DbCommand::MarkUnsaved(id))?,

            // Groups
            Action::LoadGroups => self.send_and_handle(DbCommand::ListGroups)?,
            Action::AddGroup { name } => {
                let max_pos = self.state.groups.iter().map(|g| g.position).max().unwrap_or(-1);
                let new = NewGroup { name, position: max_pos + 1, created_at: now_timestamp() };
                self.send_and_handle(DbCommand::CreateGroup(new))?;
                self.reload_groups_and_feeds()?;
            }
            Action::DeleteGroup(id) => {
                self.send_and_handle(DbCommand::DeleteGroup(id))?;
                self.reload_groups_and_feeds()?;
            }
            Action::RenameGroup { id, name } => {
                self.send_and_handle(DbCommand::RenameGroup { id, name })?;
                self.reload_groups_and_feeds()?;
            }
            Action::SwapGroupOrder { id_a, id_b } => {
                self.send_and_handle(DbCommand::SwapGroupPositions { id_a, id_b })?;
                self.reload_groups_and_feeds()?;
            }
            Action::AssignFeedToGroup { feed_id, group_id } => {
                self.send_and_handle(DbCommand::SetFeedGroup { feed_id, group_id })?;
                self.reload_groups_and_feeds()?;
            }

            // Pure state
            other => self.state.reduce(other),
        }

        Ok(())
    }

    fn send_and_handle(&mut self, cmd: DbCommand) -> Result<(), DbWorkerError> {
        let response = self.db.send(cmd)?;
        self.handle_db_response(response);
        Ok(())
    }

    fn add_feed(&mut self, title: Option<String>, url: String, group_id: Option<i64>) -> Result<(), DbWorkerError> {
        if !is_valid_feed_url(&url) {
            self.state.reduce(Action::DbError(self.lang.invalid_url(&url)));
            return Ok(());
        }
        let feed = NewFeed { title, url: url.clone(), created_at: now_timestamp() };
        let response = self.db.send(DbCommand::CreateFeed(feed))?;
        let new_feed_id = if let DbResponse::Feed(Ok(ref f)) = response { Some(f.id) } else { None };
        self.handle_db_response(response);
        if let (Some(fid), Some(gid)) = (new_feed_id, group_id) {
            self.send_and_handle(DbCommand::SetFeedGroup { feed_id: fid, group_id: Some(gid) })?;
        }
        self.send_and_handle(DbCommand::ListFeeds)?;
        self.refresh_feeds()
    }

    fn load_entries_for_feed(&mut self, feed_id: i64, unread_only: bool, saved_only: bool, since: Option<i64>) -> Result<(), DbWorkerError> {
        self.state.viewing_group = false;
        if self.state.selected_feed != Some(feed_id) {
            self.state.reduce(Action::SelectFeed(Some(feed_id)));
        }
        let response = self.db.send(DbCommand::EntriesForFeedFiltered {
            feed_id, unread_only, saved_only, since, sort_mode: self.state.sort_mode,
        })?;
        self.handle_db_response(response);
        self.refresh_entry_count(feed_id)
    }

    fn load_all_entries(&mut self, unread_only: bool, saved_only: bool, since: Option<i64>) -> Result<(), DbWorkerError> {
        self.state.viewing_group = true;
        self.state.selected_feed = None;
        self.state.selected_feed_index = None;
        let response = self.db.send(DbCommand::AllEntriesFiltered {
            unread_only, saved_only, since, sort_mode: self.state.sort_mode,
        })?;
        self.handle_db_response(response);
        let count_resp = self.db.send(DbCommand::CountAllEntries)?;
        if let DbResponse::Count(Ok(count)) = count_resp {
            self.state.reduce(Action::UpdateTotalEntryCount(count));
        }
        Ok(())
    }

    fn load_entries_for_group(&mut self, group_id: Option<i64>, unread_only: bool, saved_only: bool, since: Option<i64>) -> Result<(), DbWorkerError> {
        self.state.viewing_group = true;
        self.state.selected_feed = None;
        self.state.selected_feed_index = None;
        let response = self.db.send(DbCommand::EntriesForGroupFiltered {
            group_id, unread_only, saved_only, since, sort_mode: self.state.sort_mode,
        })?;
        self.handle_db_response(response);
        let count_resp = self.db.send(DbCommand::CountEntriesForGroup(group_id))?;
        if let DbResponse::Count(Ok(count)) = count_resp {
            self.state.reduce(Action::UpdateTotalEntryCount(count));
        }
        Ok(())
    }

    fn set_search_query(&mut self, query: String) -> Result<(), DbWorkerError> {
        self.state.reduce(Action::SetSearchQuery(query.clone()));
        let since = self.since_cutoff();
        if let Some(feed_id) = self.state.selected_feed {
            if self.state.search_query.is_some() {
                self.send_and_handle(DbCommand::SearchEntries {
                    feed_id: Some(feed_id), query, unread_only: self.state.unread_only,
                    saved_only: self.state.saved_only, since,
                })?;
            } else {
                self.send_and_handle(DbCommand::EntriesForFeedFiltered {
                    feed_id, unread_only: self.state.unread_only,
                    saved_only: self.state.saved_only, since, sort_mode: self.state.sort_mode,
                })?;
            }
        }
        Ok(())
    }

    fn handle_db_response(&mut self, response: DbResponse) {
        match response {
            DbResponse::Feeds(Ok(feeds)) => self.state.reduce(Action::FeedsLoaded(feeds)),
            DbResponse::Entries(Ok(entries)) => self.state.reduce(Action::EntriesLoaded(entries)),
            DbResponse::Counts(Ok(counts)) => self.state.reduce(Action::UpdateUnreadCounts(counts)),
            DbResponse::Count(Ok(total)) => self.state.reduce(Action::UpdateTotalUnread(total)),
            DbResponse::Feed(Ok(feed)) => {
                self.state.reduce(Action::SetStatus(self.lang.feed_saved(&feed.url)));
            }
            DbResponse::Updated(Ok(count)) => {
                self.state.reduce(Action::SetStatus(format!("{}: {}", self.lang.updated_entries, count)));
            }
            DbResponse::Groups(Ok(groups)) => self.state.reduce(Action::GroupsLoaded(groups)),
            DbResponse::Ok(Ok(())) | DbResponse::Group(Ok(_)) => {}

            DbResponse::Feeds(Err(e))
            | DbResponse::Entries(Err(e))
            | DbResponse::Counts(Err(e))
            | DbResponse::Count(Err(e))
            | DbResponse::Feed(Err(e))
            | DbResponse::Updated(Err(e))
            | DbResponse::Ok(Err(e))
            | DbResponse::Group(Err(e))
            | DbResponse::Groups(Err(e)) => {
                warn!("DB error: {e}");
                self.state.reduce(Action::DbError(e));
            }
        }
    }

    fn refresh_entry_count(&mut self, feed_id: i64) -> Result<(), DbWorkerError> {
        let response = self.db.send(DbCommand::CountEntriesForFeed(feed_id))?;
        if let DbResponse::Count(Ok(count)) = response {
            self.state.reduce(Action::UpdateTotalEntryCount(count));
        }
        Ok(())
    }

    fn reload_groups_and_feeds(&mut self) -> Result<(), DbWorkerError> {
        let response = self.db.send(DbCommand::ListGroups)?;
        self.handle_db_response(response);
        let response = self.db.send(DbCommand::ListFeeds)?;
        self.handle_db_response(response);
        Ok(())
    }

    fn refresh_feeds(&mut self) -> Result<(), DbWorkerError> {
        if self.refresh_rx.is_some() {
            self.state
                .reduce(Action::SetStatus(self.lang.already_refreshing.to_string()));
            return Ok(());
        }

        let response = self.db.send(DbCommand::ListFeeds)?;
        self.handle_db_response(response);
        if self.state.feeds.is_empty() {
            self.state
                .reduce(Action::SetStatus(self.lang.no_feeds_to_refresh.to_string()));
            return Ok(());
        }

        self.state.refreshing = true;
        info!("Refreshing {} feeds", self.state.feeds.len());
        self.state
            .reduce(Action::SetStatus(self.lang.refreshing.to_string()));

        let feeds: Arc<Vec<Feed>> = Arc::new(self.state.feeds.clone());

        let jobs: Vec<FetchJob> = feeds
            .iter()
            .map(|feed| FetchJob {
                feed_id: feed.id,
                url: feed.url.clone(),
                cache: Some(crate::fetch::client::CacheState {
                    etag: feed.etag.clone(),
                    last_modified: feed.last_modified.clone(),
                }),
            })
            .collect();

        let client = self.client.clone();
        let db = self.db.clone();
        let max_concurrency = self.max_concurrency;
        let (tx, rx) = std::sync::mpsc::channel();
        self.refresh_rx = Some(rx);

        let runtime = Arc::clone(&self.runtime);

        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let scheduler = Scheduler::new(client, max_concurrency);
                let results = runtime.block_on(scheduler.run(jobs));
                let mut refreshed = 0;
                let mut updated_entries: i64 = 0;
                let mut errors = 0;
                let mut last_error: Option<String> = None;

                for (job, result) in results {
                    match result {
                        Ok(fetch) => {
                            refreshed += 1;
                            let now = now_timestamp();
                            let body = fetch.body;
                            let _ = db.send(DbCommand::UpdateFeedFetchState {
                                feed_id: job.feed_id,
                                etag: fetch.etag,
                                last_modified: fetch.last_modified,
                                last_checked_at: Some(now),
                            });

                            if let Some(body) = body {
                                match parse_feed(&body) {
                                    Ok(parsed) => {
                                        if let Some(feed) = feeds.iter().find(|f| f.id == job.feed_id)
                                            && let Some(title) = parsed
                                                .title
                                                .as_ref()
                                                .map(|t| t.content.trim().to_string())
                                                .filter(|value| !value.is_empty())
                                                && feed.title.as_deref()
                                                    != Some(title.as_str())
                                        {
                                            let mut updated_feed = feed.clone();
                                            updated_feed.title = Some(title);
                                            let _ = db.send(DbCommand::UpdateFeed(updated_feed));
                                        }

                                        let entries = map_entries(job.feed_id, &parsed, now);
                                        if let Ok(DbResponse::Updated(Ok(count))) =
                                            db.send(DbCommand::UpsertEntries(entries))
                                        {
                                            updated_entries += count as i64;
                                        }
                                    }
                                    Err(error) => {
                                        errors += 1;
                                        last_error =
                                            Some(format!("Parse error for {}: {:?}", job.url, error));
                                    }
                                }
                            }
                        }
                        Err(error) => {
                            errors += 1;
                            last_error = Some(format!("Fetch error for {}: {:?}", job.url, error));
                        }
                    }
                }

                RefreshComplete {
                    refreshed,
                    updated_entries,
                    errors,
                    last_error,
                }
            }));

            match result {
                Ok(complete) => {
                    let _ = tx.send(complete);
                }
                Err(panic_info) => {
                    let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                        format!("Refresh thread panicked: {s}")
                    } else if let Some(s) = panic_info.downcast_ref::<String>() {
                        format!("Refresh thread panicked: {s}")
                    } else {
                        "Refresh thread panicked with unknown error".to_string()
                    };
                    error!("{msg}");
                    let _ = tx.send(RefreshComplete {
                        refreshed: 0,
                        updated_entries: 0,
                        errors: 1,
                        last_error: Some(msg),
                    });
                }
            }
        });

        Ok(())
    }

    pub fn poll_refresh(&mut self) {
        if self.refreshing() {
            self.state.tick = self.state.tick.wrapping_add(1);
        }
        let result = match &self.refresh_rx {
            Some(rx) => match rx.try_recv() {
                Ok(result) => Some(Ok(result)),
                Err(std::sync::mpsc::TryRecvError::Disconnected) => Some(Err(())),
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
            },
            None => None,
        };
        match result {
            Some(Ok(result)) => {
                self.refresh_rx = None;
                self.state.refreshing = false;

                let summary = self.lang.refreshed_summary(
                    result.refreshed,
                    result.updated_entries,
                    result.errors,
                );

                if let Ok(response) = self.db.send(DbCommand::ListFeeds) {
                    self.handle_db_response(response);
                }

                if result.errors > 0 {
                    let message = if let Some(error) = &result.last_error {
                        format!("{summary}. Last error: {error}")
                    } else {
                        summary.clone()
                    };
                    warn!("Refresh completed with errors: {message}");
                    self.state.reduce(Action::DbError(message));
                } else {
                    info!("Refresh completed: {summary}");
                    self.state.reduce(Action::SetStatus(summary));
                }

                let _ = self.refresh_current_entries();
                let _ = self.dispatch(Action::RefreshUnreadCounts);
            }
            Some(Err(())) => {
                error!("Refresh thread crashed unexpectedly");
                self.refresh_rx = None;
                self.state.refreshing = false;
                self.state.reduce(Action::DbError(
                    self.lang.refresh_thread_crashed.to_string(),
                ));
            }
            None => {}
        }
    }

    pub fn refreshing(&self) -> bool {
        self.refresh_rx.is_some()
    }

    fn refresh_current_entries(&mut self) -> Result<(), DbWorkerError> {
        let since = self.since_cutoff();
        let unread_only = self.state.unread_only;
        let saved_only = self.state.saved_only;
        let sort_mode = self.state.sort_mode;

        if let Some(feed_id) = self.state.selected_feed {
            if let Some(query) = self.state.search_query.clone() {
                let response = self.db.send(DbCommand::SearchEntries {
                    feed_id: Some(feed_id),
                    query,
                    unread_only,
                    saved_only,
                    since,
                })?;
                self.handle_db_response(response);
            } else {
                let response = self.db.send(DbCommand::EntriesForFeedFiltered {
                    feed_id,
                    unread_only,
                    saved_only,
                    since,
                    sort_mode,
                })?;
                self.handle_db_response(response);
            }
        } else if self.state.viewing_group {
            // Reload group/all entries based on current feed row selection
            if let Some(row_idx) = self.state.selected_feed_row_index {
                match self.state.feed_rows.get(row_idx).cloned() {
                    Some(state::FeedRow::GroupHeader { group_id, .. }) => {
                        let response = self.db.send(DbCommand::EntriesForGroupFiltered {
                            group_id: Some(group_id),
                            unread_only,
                            saved_only,
                            since,
                            sort_mode,
                        })?;
                        self.handle_db_response(response);
                    }
                    Some(state::FeedRow::UngroupedHeader { .. }) => {
                        let response = self.db.send(DbCommand::EntriesForGroupFiltered {
                            group_id: None,
                            unread_only,
                            saved_only,
                            since,
                            sort_mode,
                        })?;
                        self.handle_db_response(response);
                    }
                    _ => {
                        let response = self.db.send(DbCommand::AllEntriesFiltered {
                            unread_only,
                            saved_only,
                            since,
                            sort_mode,
                        })?;
                        self.handle_db_response(response);
                    }
                }
            } else {
                let response = self.db.send(DbCommand::AllEntriesFiltered {
                    unread_only,
                    saved_only,
                    since,
                    sort_mode,
                })?;
                self.handle_db_response(response);
            }
        }

        Ok(())
    }
}

fn is_valid_feed_url(url: &str) -> bool {
    match url::Url::parse(url.trim()) {
        Ok(parsed) => matches!(parsed.scheme(), "http" | "https") && parsed.host().is_some(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::models::NewFeed;

    pub(crate) fn test_app() -> App {
        let db = DbWorker::start_in_memory().expect("in-memory db");
        App::new(db, Lang::from_code("en"), 30).expect("app")
    }

    #[test]
    fn poll_refresh_noop_when_not_refreshing() {
        let mut app = test_app();
        assert!(!app.refreshing());
        app.poll_refresh();
        assert!(app.state.status.is_none());
    }

    #[test]
    fn poll_refresh_receives_result() {
        let mut app = test_app();
        let (tx, rx) = std::sync::mpsc::channel();
        app.refresh_rx = Some(rx);
        assert!(app.refreshing());

        tx.send(RefreshComplete {
            refreshed: 3,
            updated_entries: 10,
            errors: 0,
            last_error: None,
        })
        .unwrap();

        app.poll_refresh();

        assert!(!app.refreshing());
        let status = app.state.status.as_ref().expect("status should be set");
        assert_eq!(status.kind, state::StatusKind::Info);
        assert!(status.message.contains("3 feeds"));
        assert!(status.message.contains("10 entries"));
    }

    #[test]
    fn poll_refresh_shows_error_status() {
        let mut app = test_app();
        let (tx, rx) = std::sync::mpsc::channel();
        app.refresh_rx = Some(rx);

        tx.send(RefreshComplete {
            refreshed: 2,
            updated_entries: 5,
            errors: 1,
            last_error: Some("timeout".to_string()),
        })
        .unwrap();

        app.poll_refresh();

        assert!(!app.refreshing());
        let status = app.state.status.as_ref().expect("status should be set");
        assert_eq!(status.kind, state::StatusKind::Error);
        assert!(status.message.contains("1 errors"));
        assert!(status.message.contains("timeout"));
    }

    #[test]
    fn refresh_feeds_rejects_concurrent() {
        let mut app = test_app();
        let (_tx, rx) = std::sync::mpsc::channel();
        app.refresh_rx = Some(rx);

        let result = app.dispatch(Action::RefreshFeeds);
        assert!(result.is_ok());

        let status = app.state.status.as_ref().expect("status should be set");
        assert!(status.message.contains("Already refreshing"));
        assert!(app.refreshing());
    }

    #[test]
    fn refresh_feeds_no_feeds() {
        let mut app = test_app();
        assert!(app.state.feeds.is_empty());

        let result = app.dispatch(Action::RefreshFeeds);
        assert!(result.is_ok());

        let status = app.state.status.as_ref().expect("status should be set");
        assert!(status.message.contains("No feeds to refresh"));
        assert!(!app.refreshing());
    }

    #[test]
    fn refresh_feeds_spawns_background_thread() {
        let mut app = test_app();

        // Add a feed to the DB so refresh has something to work with
        let feed = NewFeed {
            title: Some("Test".to_string()),
            url: "http://invalid.test/feed.xml".to_string(),
            created_at: 0,
        };
        let _ = app.db.send(DbCommand::CreateFeed(feed));
        let _ = app.dispatch(Action::LoadFeeds);
        assert!(!app.state.feeds.is_empty());

        let result = app.dispatch(Action::RefreshFeeds);
        assert!(result.is_ok());
        assert!(app.refreshing());

        // Wait for the background thread to finish (will fail fetching the invalid URL)
        let rx = app.refresh_rx.as_ref().unwrap();
        let complete = rx.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
        app.refresh_rx = None;

        // The invalid URL should produce errors
        assert_eq!(complete.errors, 1);
        assert!(complete.last_error.is_some());
    }

    #[test]
    fn add_feed_rejects_invalid_url() {
        let mut app = test_app();
        let result = app.dispatch(Action::AddFeed {
            title: Some("Bad".to_string()),
            url: "not-a-url".to_string(),
            group_id: None,
        });
        assert!(result.is_ok());
        let status = app.state.status.as_ref().expect("status");
        assert_eq!(status.kind, state::StatusKind::Error);
        assert!(status.message.contains("Invalid URL"));
    }

    #[test]
    fn add_feed_accepts_valid_url() {
        let mut app = test_app();
        let result = app.dispatch(Action::AddFeed {
            title: Some("Good".to_string()),
            url: "https://example.com/feed.xml".to_string(),
            group_id: None,
        });
        assert!(result.is_ok());
        // Should not be an error status
        let status = app.state.status.as_ref().expect("status");
        assert_eq!(status.kind, state::StatusKind::Info);
    }

    #[test]
    fn url_validation() {
        assert!(is_valid_feed_url("https://example.com/feed"));
        assert!(is_valid_feed_url("http://example.com/rss.xml"));
        assert!(!is_valid_feed_url("ftp://example.com"));
        assert!(!is_valid_feed_url("not a url"));
        assert!(!is_valid_feed_url("https://"));
        assert!(!is_valid_feed_url(""));
    }
}
