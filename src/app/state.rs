use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::app::actions::Action;
use crate::store::models::{Entry, Feed, Group};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Feeds,
    Entries,
    Preview,
}

#[derive(Debug, Clone)]
pub enum FeedRow {
    GroupHeader { group_id: i64, name: String, unread: i64 },
    FeedItem { feed_index: usize },
    UngroupedHeader { unread: i64 },
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub feeds: Vec<Feed>,
    pub entries: Vec<Entry>,
    pub selected_feed: Option<i64>,
    pub selected_entry: Option<i64>,
    pub selected_feed_index: Option<usize>,
    pub selected_entry_index: Option<usize>,
    pub focus: Focus,
    pub unread_only: bool,
    pub saved_only: bool,
    pub search_query: Option<String>,
    pub preview_scroll: u16,
    pub preview_content_len: usize,
    pub unread_counts: HashMap<i64, i64>,
    pub total_unread: i64,
    pub status: Option<StatusMessage>,
    pub refreshing: bool,
    pub tick: usize,
    pub groups: Vec<Group>,
    pub collapsed_groups: HashSet<i64>,
    pub feed_rows: Vec<FeedRow>,
    pub selected_feed_row_index: Option<usize>,
    pub panel_ratios: [u16; 3],
    pub status_set_at: Option<Instant>,
    pub total_entry_count: i64,
}

#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub message: String,
    pub kind: StatusKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Info,
    Error,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            feeds: Vec::new(),
            entries: Vec::new(),
            selected_feed: None,
            selected_entry: None,
            selected_feed_index: None,
            selected_entry_index: None,
            focus: Focus::Feeds,
            unread_only: true,
            saved_only: false,
            search_query: None,
            preview_scroll: 0,
            preview_content_len: 0,
            unread_counts: HashMap::new(),
            total_unread: 0,
            status: None,
            refreshing: false,
            tick: 0,
            groups: Vec::new(),
            collapsed_groups: HashSet::new(),
            feed_rows: Vec::new(),
            selected_feed_row_index: None,
            panel_ratios: [20, 30, 50],
            status_set_at: None,
            total_entry_count: 0,
        }
    }
}

impl AppState {
    pub fn reduce(&mut self, action: Action) {
        match action {
            Action::FeedsLoaded(feeds) => {
                self.feeds = feeds;
                self.rebuild_feed_rows();
                // Position cursor on first row if nothing selected yet, but don't load entries
                if self.selected_feed_row_index.is_none() && !self.feed_rows.is_empty() {
                    self.selected_feed_row_index = Some(0);
                }
            }
            Action::EntriesLoaded(entries) => {
                self.entries = entries;
                if self.entries.is_empty() {
                    self.selected_entry = None;
                    self.selected_entry_index = None;
                    self.preview_scroll = 0;
                } else if let Some(selected_id) = self.selected_entry {
                    self.selected_entry_index = self
                        .entries
                        .iter()
                        .position(|entry| entry.id == selected_id);
                } else if self.selected_entry_index.is_none() {
                    self.select_entry_index(0);
                }
            }
            Action::SelectFeed(feed_id) => {
                self.selected_feed = feed_id;
                self.selected_feed_index =
                    feed_id.and_then(|id| self.feeds.iter().position(|feed| feed.id == id));
                self.entries.clear();
                self.selected_entry = None;
                self.selected_entry_index = None;
                self.preview_scroll = 0;
            }
            Action::SelectEntry(entry_id) => {
                self.selected_entry = entry_id;
                self.selected_entry_index =
                    entry_id.and_then(|id| self.entries.iter().position(|entry| entry.id == id));
                self.preview_scroll = 0;
            }
            Action::FocusEntries => {
                self.focus = Focus::Entries;
            }
            Action::FocusPreview => {
                self.focus = Focus::Preview;
            }
            Action::FocusFeeds => {
                self.focus = Focus::Feeds;
            }
            Action::ToggleUnreadFilter => {
                self.unread_only = !self.unread_only;
            }
            Action::ToggleSavedFilter => {
                self.saved_only = !self.saved_only;
            }
            Action::SetSearchQuery(query) => {
                let trimmed = query.trim();
                if trimmed.is_empty() {
                    self.search_query = None;
                } else {
                    self.search_query = Some(trimmed.to_string());
                }
            }
            Action::FocusNext => {
                self.focus = match self.focus {
                    Focus::Feeds => Focus::Entries,
                    Focus::Entries => Focus::Preview,
                    Focus::Preview => Focus::Feeds,
                };
            }
            Action::FocusPrev => {
                self.focus = match self.focus {
                    Focus::Feeds => Focus::Preview,
                    Focus::Entries => Focus::Feeds,
                    Focus::Preview => Focus::Entries,
                };
            }
            Action::MoveUp => match self.focus {
                Focus::Feeds => self.move_feed_selection(-1),
                Focus::Entries => self.move_entry_selection(-1),
                Focus::Preview => self.scroll_preview(-1),
            },
            Action::MoveDown => match self.focus {
                Focus::Feeds => self.move_feed_selection(1),
                Focus::Entries => self.move_entry_selection(1),
                Focus::Preview => self.scroll_preview(1),
            },
            Action::PageUp => if self.focus == Focus::Preview { self.scroll_preview(-10) },
            Action::PageDown => if self.focus == Focus::Preview { self.scroll_preview(10) },
            Action::ScrollTop => if self.focus == Focus::Preview { self.preview_scroll = 0 },
            Action::ScrollBottom => if self.focus == Focus::Preview { self.preview_scroll = self.preview_content_len.saturating_sub(1) as u16 },
            Action::UpdateUnreadCounts(counts) => {
                self.unread_counts = counts.into_iter().collect();
                self.rebuild_feed_rows();
            }
            Action::UpdateTotalUnread(total) => {
                self.total_unread = total;
            }
            Action::SetStatus(message) => {
                self.status = Some(StatusMessage {
                    message,
                    kind: StatusKind::Info,
                });
                self.status_set_at = Some(Instant::now());
            }
            Action::ClearStatus => {
                self.status = None;
                self.status_set_at = None;
            }
            Action::DbError(error) => {
                self.status = Some(StatusMessage {
                    message: error,
                    kind: StatusKind::Error,
                });
                self.status_set_at = Some(Instant::now());
            }
            Action::GroupsLoaded(groups) => {
                self.groups = groups;
                self.rebuild_feed_rows();
            }
            Action::ToggleGroupCollapse(group_id) => {
                if !self.collapsed_groups.remove(&group_id) {
                    self.collapsed_groups.insert(group_id);
                }
                self.rebuild_feed_rows();
                // Stay on the group header after toggle
                self.selected_feed_row_index = self.feed_rows.iter().position(|r| {
                    matches!(r, FeedRow::GroupHeader { group_id: gid, .. } if *gid == group_id)
                });
            }
            Action::ResizePanel(delta) => {
                self.resize_panel(delta);
            }
            Action::UpdateTotalEntryCount(count) => {
                self.total_entry_count = count;
            }
            Action::LoadFeeds
            | Action::LoadEntriesFiltered { .. }
            | Action::SearchEntries { .. }
            | Action::RefreshFeeds
            | Action::RefreshUnreadCounts
            | Action::AddFeed { .. }
            | Action::DeleteFeed(_)
            | Action::MarkRead(_)
            | Action::MarkUnread(_)
            | Action::MarkSaved(_)
            | Action::MarkUnsaved(_)
            | Action::LoadGroups
            | Action::AddGroup { .. }
            | Action::DeleteGroup(_)
            | Action::RenameGroup { .. }
            | Action::AssignFeedToGroup { .. } => {}
        }
    }

    fn select_feed_index(&mut self, index: usize) {
        if let Some(feed) = self.feeds.get(index) {
            self.selected_feed = Some(feed.id);
            self.selected_feed_index = Some(index);
            self.entries.clear();
            self.selected_entry = None;
            self.selected_entry_index = None;
        }
    }

    fn select_entry_index(&mut self, index: usize) {
        if let Some(entry) = self.entries.get(index) {
            self.selected_entry = Some(entry.id);
            self.selected_entry_index = Some(index);
        }
    }

    fn move_feed_selection(&mut self, delta: isize) {
        if self.feed_rows.is_empty() {
            return;
        }
        let current = self.selected_feed_row_index.unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, (self.feed_rows.len() - 1) as isize) as usize;
        self.selected_feed_row_index = Some(next);
        match &self.feed_rows[next] {
            FeedRow::FeedItem { feed_index } => {
                let fi = *feed_index;
                self.select_feed_index(fi);
            }
            FeedRow::GroupHeader { .. } | FeedRow::UngroupedHeader { .. } => {
                // Don't change selected_feed when on a header
            }
        }
    }

    pub fn rebuild_feed_rows(&mut self) {
        self.feed_rows.clear();
        if self.groups.is_empty() {
            // No groups: flat list (current behavior)
            for (i, _feed) in self.feeds.iter().enumerate() {
                self.feed_rows.push(FeedRow::FeedItem { feed_index: i });
            }
            // Sync selected_feed_row_index with selected_feed_index
            self.selected_feed_row_index = self.selected_feed_index;
            return;
        }

        // Grouped mode
        for group in &self.groups {
            let group_feeds: Vec<usize> = self
                .feeds
                .iter()
                .enumerate()
                .filter(|(_, f)| f.group_id == Some(group.id))
                .map(|(i, _)| i)
                .collect();
            let unread: i64 = group_feeds
                .iter()
                .map(|&i| self.unread_counts.get(&self.feeds[i].id).copied().unwrap_or(0))
                .sum();
            self.feed_rows.push(FeedRow::GroupHeader {
                group_id: group.id,
                name: group.name.clone(),
                unread,
            });
            if !self.collapsed_groups.contains(&group.id) {
                for fi in group_feeds {
                    self.feed_rows.push(FeedRow::FeedItem { feed_index: fi });
                }
            }
        }

        // Ungrouped feeds
        let ungrouped: Vec<usize> = self
            .feeds
            .iter()
            .enumerate()
            .filter(|(_, f)| f.group_id.is_none())
            .map(|(i, _)| i)
            .collect();
        if !ungrouped.is_empty() {
            let unread: i64 = ungrouped
                .iter()
                .map(|&i| self.unread_counts.get(&self.feeds[i].id).copied().unwrap_or(0))
                .sum();
            self.feed_rows.push(FeedRow::UngroupedHeader { unread });
            for fi in ungrouped {
                self.feed_rows.push(FeedRow::FeedItem { feed_index: fi });
            }
        }

        // Update selected_feed_row_index to match selected_feed
        if let Some(feed_id) = self.selected_feed {
            self.selected_feed_row_index = self.feed_rows.iter().position(|row| {
                if let FeedRow::FeedItem { feed_index } = row {
                    self.feeds.get(*feed_index).map(|f| f.id) == Some(feed_id)
                } else {
                    false
                }
            });
        }
    }

    fn move_entry_selection(&mut self, delta: isize) {
        if self.entries.is_empty() {
            return;
        }
        let current = self.selected_entry_index.unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, (self.entries.len() - 1) as isize) as usize;
        self.select_entry_index(next);
    }

    fn scroll_preview(&mut self, delta: isize) {
        let max = self.preview_content_len.saturating_sub(1) as u16;
        if delta < 0 {
            let shift = delta.unsigned_abs() as u16;
            self.preview_scroll = self.preview_scroll.saturating_sub(shift);
        } else if delta > 0 {
            self.preview_scroll = self.preview_scroll.saturating_add(delta as u16).min(max);
        }
    }

    fn resize_panel(&mut self, delta: i8) {
        // L (delta>0) = grow focused panel, H (delta<0) = shrink focused panel
        // Neighbour is always the adjacent panel to the right, except for Preview which uses Entries.
        let step = 5u16;
        let idx = match self.focus {
            Focus::Feeds => 0,
            Focus::Entries => 1,
            Focus::Preview => 2,
        };
        let neighbour = if idx < 2 { idx + 1 } else { 1 };
        // For Preview (rightmost), H grows and L shrinks (directions are mirrored)
        let growing = if idx == 2 { delta < 0 } else { delta > 0 };
        let (grow, shrink) = if growing { (idx, neighbour) } else { (neighbour, idx) };
        if self.panel_ratios[shrink] > step + 10 {
            self.panel_ratios[grow] += step;
            self.panel_ratios[shrink] -= step;
        }
    }
}
