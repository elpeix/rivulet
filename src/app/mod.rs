pub mod actions;
pub mod events;
pub mod input;
pub mod state;

use std::sync::mpsc::Receiver;

use crate::fetch::client::{FetchOptions, HttpClient};
use crate::fetch::parser::{map_entries, parse_feed};
use crate::fetch::scheduler::{FetchJob, Scheduler};
use crate::i18n::Lang;
use crate::store::models::{Feed, NewFeed, NewGroup};
use crate::util::time::now_timestamp;
use actions::Action;
use events::{DbCommand, DbResponse, DbWorker, DbWorkerError};
use state::AppState;

pub struct RefreshComplete {
    pub refreshed: usize,
    pub updated_entries: i64,
    pub errors: usize,
    pub last_error: Option<String>,
}

pub struct App {
    pub state: AppState,
    pub lang: Lang,
    db: DbWorker,
    client: HttpClient,
    max_concurrency: usize,
    refresh_rx: Option<Receiver<RefreshComplete>>,
}

impl App {
    pub fn new(db: DbWorker, lang: Lang) -> Result<Self, String> {
        let client =
            HttpClient::new(FetchOptions::default()).map_err(|error| format!("{:?}", error))?;

        Ok(Self {
            state: AppState::default(),
            lang,
            db,
            client,
            max_concurrency: 6,
            refresh_rx: None,
        })
    }

    pub fn dispatch(&mut self, action: Action) -> Result<(), DbWorkerError> {
        match action {
            Action::LoadFeeds => {
                let response = self.db.send(DbCommand::ListFeeds)?;
                self.handle_db_response(response);
            }
            Action::RefreshUnreadCounts => {
                let counts = self.db.send(DbCommand::UnreadCountsByFeed)?;
                self.handle_db_response(counts);
                let total = self.db.send(DbCommand::UnreadCountAll)?;
                self.handle_db_response(total);
            }
            Action::RefreshFeeds => {
                self.refresh_feeds()?;
            }
            Action::LoadEntriesFiltered {
                feed_id,
                unread_only,
                saved_only,
            } => {
                if self.state.selected_feed != Some(feed_id) {
                    self.state.reduce(Action::SelectFeed(Some(feed_id)));
                }
                let response = self.db.send(DbCommand::EntriesForFeedFiltered {
                    feed_id,
                    unread_only,
                    saved_only,
                })?;
                self.handle_db_response(response);
                self.refresh_entry_count(feed_id)?;
            }
            Action::SearchEntries { feed_id, query } => {
                let response = self.db.send(DbCommand::SearchEntries {
                    feed_id,
                    query,
                    unread_only: self.state.unread_only,
                    saved_only: self.state.saved_only,
                })?;
                self.handle_db_response(response);
                if let Some(fid) = feed_id {
                    self.refresh_entry_count(fid)?;
                }
            }
            Action::ToggleUnreadFilter => {
                self.state.reduce(Action::ToggleUnreadFilter);
                if let Some(feed_id) = self.state.selected_feed {
                    let response = self.db.send(DbCommand::EntriesForFeedFiltered {
                        feed_id,
                        unread_only: self.state.unread_only,
                        saved_only: self.state.saved_only,
                    })?;
                    self.handle_db_response(response);
                }
            }
            Action::ToggleSavedFilter => {
                self.state.reduce(Action::ToggleSavedFilter);
                if let Some(feed_id) = self.state.selected_feed {
                    let response = self.db.send(DbCommand::EntriesForFeedFiltered {
                        feed_id,
                        unread_only: self.state.unread_only,
                        saved_only: self.state.saved_only,
                    })?;
                    self.handle_db_response(response);
                }
            }
            Action::SetSearchQuery(query) => {
                self.state.reduce(Action::SetSearchQuery(query.clone()));
                if let Some(feed_id) = self.state.selected_feed {
                    if self.state.search_query.is_some() {
                        let response = self.db.send(DbCommand::SearchEntries {
                            feed_id: Some(feed_id),
                            query,
                            unread_only: self.state.unread_only,
                            saved_only: self.state.saved_only,
                        })?;
                        self.handle_db_response(response);
                    } else {
                        let response = self.db.send(DbCommand::EntriesForFeedFiltered {
                            feed_id,
                            unread_only: self.state.unread_only,
                            saved_only: self.state.saved_only,
                        })?;
                        self.handle_db_response(response);
                    }
                }
            }
            Action::AddFeed { title, url } => {
                if !is_valid_feed_url(&url) {
                    self.state.reduce(Action::DbError(self.lang.invalid_url(&url)));
                    return Ok(());
                }
                let feed = NewFeed {
                    title,
                    url: url.clone(),
                    created_at: now_timestamp(),
                };
                let response = self.db.send(DbCommand::CreateFeed(feed))?;
                self.handle_db_response(response);
                let response = self.db.send(DbCommand::ListFeeds)?;
                self.handle_db_response(response);
            }
            Action::DeleteFeed(feed_id) => {
                let response = self.db.send(DbCommand::DeleteFeed(feed_id))?;
                self.handle_db_response(response);
                let response = self.db.send(DbCommand::ListFeeds)?;
                self.handle_db_response(response);
            }
            Action::MarkRead(entry_id) => {
                let response = self.db.send(DbCommand::MarkRead {
                    entry_id,
                    read_at: now_timestamp(),
                })?;
                self.handle_db_response(response);
            }
            Action::MarkUnread(entry_id) => {
                let response = self.db.send(DbCommand::MarkUnread(entry_id))?;
                self.handle_db_response(response);
            }
            Action::MarkSaved(entry_id) => {
                let response = self.db.send(DbCommand::MarkSaved {
                    entry_id,
                    saved_at: now_timestamp(),
                })?;
                self.handle_db_response(response);
            }
            Action::MarkUnsaved(entry_id) => {
                let response = self.db.send(DbCommand::MarkUnsaved(entry_id))?;
                self.handle_db_response(response);
            }
            Action::LoadGroups => {
                let response = self.db.send(DbCommand::ListGroups)?;
                self.handle_db_response(response);
            }
            Action::AddGroup { name } => {
                let max_pos = self.state.groups.iter().map(|g| g.position).max().unwrap_or(-1);
                let new = NewGroup {
                    name,
                    position: max_pos + 1,
                    created_at: now_timestamp(),
                };
                let response = self.db.send(DbCommand::CreateGroup(new))?;
                self.handle_db_response(response);
                self.reload_groups_and_feeds()?;
            }
            Action::DeleteGroup(id) => {
                let response = self.db.send(DbCommand::DeleteGroup(id))?;
                self.handle_db_response(response);
                self.reload_groups_and_feeds()?;
            }
            Action::RenameGroup { id, name } => {
                let response = self.db.send(DbCommand::RenameGroup { id, name })?;
                self.handle_db_response(response);
                self.reload_groups_and_feeds()?;
            }
            Action::AssignFeedToGroup { feed_id, group_id } => {
                let response = self.db.send(DbCommand::SetFeedGroup { feed_id, group_id })?;
                self.handle_db_response(response);
                self.reload_groups_and_feeds()?;
            }
            other => {
                self.state.reduce(other);
            }
        }

        Ok(())
    }

    fn handle_db_response(&mut self, response: DbResponse) {
        match response {
            DbResponse::Feeds(result) => match result {
                Ok(feeds) => self.state.reduce(Action::FeedsLoaded(feeds)),
                Err(error) => self.state.reduce(Action::DbError(error)),
            },
            DbResponse::Entries(result) => match result {
                Ok(entries) => self.state.reduce(Action::EntriesLoaded(entries)),
                Err(error) => self.state.reduce(Action::DbError(error)),
            },
            DbResponse::Counts(result) => match result {
                Ok(counts) => self.state.reduce(Action::UpdateUnreadCounts(counts)),
                Err(error) => self.state.reduce(Action::DbError(error)),
            },
            DbResponse::Count(result) => match result {
                Ok(total) => self.state.reduce(Action::UpdateTotalUnread(total)),
                Err(error) => self.state.reduce(Action::DbError(error)),
            },
            DbResponse::Feed(result) => match result {
                Ok(feed) => self
                    .state
                    .reduce(Action::SetStatus(self.lang.feed_saved(&feed.url))),
                Err(error) => self.state.reduce(Action::DbError(error)),
            },
            DbResponse::Updated(result) => match result {
                Ok(count) => self
                    .state
                    .reduce(Action::SetStatus(format!("Updated entries: {}", count))),
                Err(error) => self.state.reduce(Action::DbError(error)),
            },
            DbResponse::Ok(result) => match result {
                Ok(()) => {}
                Err(error) => self.state.reduce(Action::DbError(error)),
            },
            DbResponse::Group(result) => match result {
                Ok(_group) => {}
                Err(error) => self.state.reduce(Action::DbError(error)),
            },
            DbResponse::Groups(result) => match result {
                Ok(groups) => self.state.reduce(Action::GroupsLoaded(groups)),
                Err(error) => self.state.reduce(Action::DbError(error)),
            },
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
        self.state
            .reduce(Action::SetStatus(self.lang.refreshing.to_string()));

        let feed_map: std::collections::HashMap<i64, Feed> = self
            .state
            .feeds
            .iter()
            .cloned()
            .map(|feed| (feed.id, feed))
            .collect();

        let jobs: Vec<FetchJob> = self
            .state
            .feeds
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

        std::thread::spawn(move || {
            let scheduler = Scheduler::new(client, max_concurrency);
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(_) => {
                    let _ = tx.send(RefreshComplete {
                        refreshed: 0,
                        updated_entries: 0,
                        errors: 1,
                        last_error: Some("Failed to create async runtime".to_string()),
                    });
                    return;
                }
            };

            let results = rt.block_on(scheduler.run(jobs));
            let mut refreshed = 0;
            let mut updated_entries: i64 = 0;
            let mut errors = 0;
            let mut last_error: Option<String> = None;

            for (job, result) in results {
                match result {
                    Ok(fetch) => {
                        refreshed += 1;
                        let now = now_timestamp();
                        let _ = db.send(DbCommand::UpdateFeedFetchState {
                            feed_id: job.feed_id,
                            etag: fetch.etag.clone(),
                            last_modified: fetch.last_modified.clone(),
                            last_checked_at: Some(now),
                        });

                        if let Some(body) = fetch.body {
                            match parse_feed(&body) {
                                Ok(parsed) => {
                                    if let Some(feed) = feed_map.get(&job.feed_id)
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

            let _ = tx.send(RefreshComplete {
                refreshed,
                updated_entries,
                errors,
                last_error,
            });
        });

        Ok(())
    }

    pub fn poll_refresh(&mut self) {
        if self.refreshing() {
            self.state.tick = self.state.tick.wrapping_add(1);
        }
        if let Some(rx) = &self.refresh_rx
            && let Ok(result) = rx.try_recv()
        {
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
                    let message = if let Some(error) = result.last_error {
                        format!("{}. Last error: {}", summary, error)
                    } else {
                        summary
                    };
                    self.state.reduce(Action::DbError(message));
                } else {
                    self.state.reduce(Action::SetStatus(summary));
                }

                let _ = self.refresh_current_entries();
                let _ = self.dispatch(Action::RefreshUnreadCounts);
        }
    }

    pub fn refreshing(&self) -> bool {
        self.refresh_rx.is_some()
    }

    #[cfg(test)]
    fn is_refreshing(&self) -> bool {
        self.refreshing()
    }

    fn refresh_current_entries(&mut self) -> Result<(), DbWorkerError> {
        if let Some(feed_id) = self.state.selected_feed {
            if let Some(query) = self.state.search_query.clone() {
                let response = self.db.send(DbCommand::SearchEntries {
                    feed_id: Some(feed_id),
                    query,
                    unread_only: self.state.unread_only,
                    saved_only: self.state.saved_only,
                })?;
                self.handle_db_response(response);
            } else {
                let response = self.db.send(DbCommand::EntriesForFeedFiltered {
                    feed_id,
                    unread_only: self.state.unread_only,
                    saved_only: self.state.saved_only,
                })?;
                self.handle_db_response(response);
            }
        }

        Ok(())
    }
}

fn is_valid_feed_url(url: &str) -> bool {
    let trimmed = url.trim();
    (trimmed.starts_with("http://") || trimmed.starts_with("https://"))
        && trimmed.len() > "https://x".len()
        && !trimmed.contains(' ')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::models::NewFeed;

    fn test_app() -> App {
        let db = DbWorker::start_in_memory().expect("in-memory db");
        App::new(db, Lang::from_code("en")).expect("app")
    }

    #[test]
    fn poll_refresh_noop_when_not_refreshing() {
        let mut app = test_app();
        assert!(!app.is_refreshing());
        app.poll_refresh();
        assert!(app.state.status.is_none());
    }

    #[test]
    fn poll_refresh_receives_result() {
        let mut app = test_app();
        let (tx, rx) = std::sync::mpsc::channel();
        app.refresh_rx = Some(rx);
        assert!(app.is_refreshing());

        tx.send(RefreshComplete {
            refreshed: 3,
            updated_entries: 10,
            errors: 0,
            last_error: None,
        })
        .unwrap();

        app.poll_refresh();

        assert!(!app.is_refreshing());
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

        assert!(!app.is_refreshing());
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
        assert!(app.is_refreshing());
    }

    #[test]
    fn refresh_feeds_no_feeds() {
        let mut app = test_app();
        assert!(app.state.feeds.is_empty());

        let result = app.dispatch(Action::RefreshFeeds);
        assert!(result.is_ok());

        let status = app.state.status.as_ref().expect("status should be set");
        assert!(status.message.contains("No feeds to refresh"));
        assert!(!app.is_refreshing());
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
        assert!(app.is_refreshing());

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
