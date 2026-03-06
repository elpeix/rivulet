use std::collections::{HashMap, HashSet};
use std::time::Instant;

use ratatui::layout::Rect;

use crate::app::actions::Action;
use crate::fetch::discovery::DiscoveredFeed;
use crate::store::models::{Entry, Feed, Group};
use crate::ui::rich_text::LinkRegion;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    None,
    Search,
    AddFeed,
    AddFeedGroup {
        url: String,
    },
    RenameFeed,
    DeleteFeed,
    AssignGroup,
    ManageGroups,
    AddGroup,
    RenameGroup,
    DeleteGroup {
        group_id: i64,
    },
    Discovering,
    SelectDiscoveredFeed {
        feeds: Vec<DiscoveredFeed>,
        group_id: Option<i64>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    /// Three columns: Feeds | Entries | Preview
    Columns,
    /// Two columns: Feeds | (Entries / Preview stacked)
    Split,
}

impl LayoutMode {
    pub fn toggle(self) -> Self {
        match self {
            Self::Columns => Self::Split,
            Self::Split => Self::Columns,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    DateDesc,
    DateAsc,
    TitleAsc,
}

impl SortMode {
    pub fn next(self) -> Self {
        match self {
            Self::DateDesc => Self::DateAsc,
            Self::DateAsc => Self::TitleAsc,
            Self::TitleAsc => Self::DateDesc,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Feeds,
    Entries,
    Preview,
}

#[derive(Debug, Clone)]
pub enum FeedRow {
    AllFeeds,
    GroupHeader {
        group_id: i64,
        name: String,
        unread: i64,
    },
    FeedItem {
        feed_index: usize,
    },
    UngroupedHeader {
        unread: i64,
    },
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
    pub split_ratio: u16,
    pub layout_mode: LayoutMode,
    pub status_set_at: Option<Instant>,
    pub total_entry_count: i64,
    pub viewing_group: bool,
    pub recent_only: bool,
    pub preview_links: Vec<String>,
    pub selected_link_index: Option<usize>,
    pub preview_link_regions: Vec<LinkRegion>,
    pub preview_body_area: Rect,
    pub sort_mode: SortMode,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub show_help: bool,
    pub help_scroll: u16,
    pub modal_selection: usize,
    feed_rows_dirty: bool,
    entry_id_to_index: HashMap<i64, usize>,
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
            split_ratio: 50,
            layout_mode: LayoutMode::Columns,
            status_set_at: None,
            total_entry_count: 0,
            viewing_group: false,
            recent_only: true,
            preview_links: Vec::new(),
            selected_link_index: None,
            preview_link_regions: Vec::new(),
            preview_body_area: Rect::default(),
            sort_mode: SortMode::DateDesc,
            input_mode: InputMode::None,
            input_buffer: String::new(),
            show_help: false,
            help_scroll: 0,
            modal_selection: 0,
            feed_rows_dirty: false,
            entry_id_to_index: HashMap::new(),
        }
    }
}

impl AppState {
    pub fn entry_position(&self, id: i64) -> Option<usize> {
        self.entry_id_to_index.get(&id).copied()
    }

    fn rebuild_entry_index(&mut self) {
        self.entry_id_to_index.clear();
        for (i, entry) in self.entries.iter().enumerate() {
            self.entry_id_to_index.insert(entry.id, i);
        }
    }

    pub fn reduce(&mut self, action: Action) {
        match action {
            Action::FeedsLoaded(feeds) => {
                self.feeds = feeds;
                self.feed_rows_dirty = true;
            }
            Action::EntriesLoaded(entries) => {
                self.entries = entries;
                self.rebuild_entry_index();
                if self.entries.is_empty() {
                    self.selected_entry = None;
                    self.selected_entry_index = None;
                    self.preview_scroll = 0;
                } else if let Some(selected_id) = self.selected_entry {
                    self.selected_entry_index = self.entry_position(selected_id);
                    if self.selected_entry_index.is_none() {
                        self.select_entry_index(0);
                    }
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
                self.preview_links.clear();
                self.selected_link_index = None;
            }
            Action::SelectEntry(entry_id) => {
                self.selected_entry = entry_id;
                self.selected_entry_index = entry_id.and_then(|id| self.entry_position(id));
                self.preview_scroll = 0;
                self.selected_link_index = None;
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
            Action::PageUp => {
                self.scroll_preview(-10);
            }
            Action::PageDown => {
                self.scroll_preview(10);
            }
            Action::ScrollTop => {
                if self.focus == Focus::Preview {
                    self.preview_scroll = 0
                }
            }
            Action::ScrollBottom => {
                if self.focus == Focus::Preview {
                    self.preview_scroll = u16::try_from(self.preview_content_len.saturating_sub(1))
                        .unwrap_or(u16::MAX)
                }
            }
            Action::UpdateUnreadCounts(counts) => {
                self.unread_counts = counts.into_iter().collect();
                self.feed_rows_dirty = true;
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
                self.feed_rows_dirty = true;
            }
            Action::ToggleGroupCollapse(group_id) => {
                if !self.collapsed_groups.remove(&group_id) {
                    self.collapsed_groups.insert(group_id);
                }
                self.rebuild_feed_rows();
                // Stay on the group header after toggle
                self.selected_feed_row_index = self.feed_rows.iter().position(
                    |r| matches!(r, FeedRow::GroupHeader { group_id: gid, .. } if *gid == group_id),
                );
            }
            Action::ResizePanel(delta) => {
                self.resize_panel(delta);
            }
            Action::ToggleLayout => {
                self.layout_mode = self.layout_mode.toggle();
            }
            Action::UpdateTotalEntryCount(count) => {
                self.total_entry_count = count;
            }
            Action::LoadFeeds
            | Action::LoadEntriesFiltered { .. }
            | Action::LoadAllEntries { .. }
            | Action::LoadEntriesForGroup { .. }
            | Action::RefreshFeeds
            | Action::RefreshUnreadCounts
            | Action::AddFeed { .. }
            | Action::DeleteFeed(_)
            | Action::RenameFeed { .. }
            | Action::MarkRead(_)
            | Action::MarkUnread(_)
            | Action::MarkAllRead(_)
            | Action::MarkFeedRead(_)
            | Action::MarkSaved(_)
            | Action::MarkUnsaved(_)
            | Action::LoadGroups
            | Action::AddGroup { .. }
            | Action::DeleteGroup(_)
            | Action::RenameGroup { .. }
            | Action::AssignFeedToGroup { .. }
            | Action::SwapGroupOrder { .. }
            | Action::AddDiscoveredFeed { .. }
            | Action::DiscoveryResult { .. } => {}
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
            self.selected_link_index = None;
        }
    }

    fn move_feed_selection(&mut self, delta: isize) {
        if self.feed_rows.is_empty() {
            return;
        }
        let current = self.selected_feed_row_index.unwrap_or(0);
        let max = self.feed_rows.len() - 1;
        let next = if delta >= 0 {
            current.saturating_add(delta as usize).min(max)
        } else {
            current.saturating_sub(delta.unsigned_abs())
        };
        self.selected_feed_row_index = Some(next);
        if let Some(FeedRow::FeedItem { feed_index }) = self.feed_rows.get(next) {
            let fi = *feed_index;
            self.select_feed_index(fi);
        }
    }

    pub fn flush_feed_rows(&mut self) {
        if self.feed_rows_dirty {
            self.feed_rows_dirty = false;
            self.rebuild_feed_rows();
            if self.selected_feed_row_index.is_none() && !self.feed_rows.is_empty() {
                self.selected_feed_row_index = Some(0);
            }
        }
    }

    pub fn rebuild_feed_rows(&mut self) {
        self.feed_rows.clear();
        // "All" row always first
        self.feed_rows.push(FeedRow::AllFeeds);

        if self.groups.is_empty() {
            // No groups: flat list
            for (i, _feed) in self.feeds.iter().enumerate() {
                self.feed_rows.push(FeedRow::FeedItem { feed_index: i });
            }
            // Sync selected_feed_row_index (+1 offset for AllFeeds)
            self.selected_feed_row_index = self.selected_feed_index.map(|i| i + 1);
            return;
        }

        // Build group_id → feed indices map (O(feeds) instead of O(groups × feeds))
        let mut feeds_by_group: HashMap<Option<i64>, Vec<usize>> = HashMap::new();
        for (i, feed) in self.feeds.iter().enumerate() {
            feeds_by_group.entry(feed.group_id).or_default().push(i);
        }

        // Grouped mode
        for group in &self.groups {
            let group_feeds = feeds_by_group.get(&Some(group.id));
            let unread: i64 = group_feeds.map_or(0, |indices| {
                indices
                    .iter()
                    .map(|&i| {
                        self.unread_counts
                            .get(&self.feeds[i].id)
                            .copied()
                            .unwrap_or(0)
                    })
                    .sum()
            });
            self.feed_rows.push(FeedRow::GroupHeader {
                group_id: group.id,
                name: group.name.clone(),
                unread,
            });
            if !self.collapsed_groups.contains(&group.id) {
                if let Some(indices) = group_feeds {
                    for &fi in indices {
                        self.feed_rows.push(FeedRow::FeedItem { feed_index: fi });
                    }
                }
            }
        }

        // Ungrouped feeds
        if let Some(ungrouped) = feeds_by_group.get(&None) {
            let unread: i64 = ungrouped
                .iter()
                .map(|&i| {
                    self.unread_counts
                        .get(&self.feeds[i].id)
                        .copied()
                        .unwrap_or(0)
                })
                .sum();
            self.feed_rows.push(FeedRow::UngroupedHeader { unread });
            for &fi in ungrouped {
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
        let current = self.selected_entry_index.unwrap_or(0);
        let max = self.entries.len() - 1;
        let next = if delta >= 0 {
            current.saturating_add(delta as usize).min(max)
        } else {
            current.saturating_sub(delta.unsigned_abs())
        };
        self.select_entry_index(next);
    }

    fn scroll_preview(&mut self, delta: isize) {
        let max = u16::try_from(self.preview_content_len.saturating_sub(1)).unwrap_or(u16::MAX);
        if delta < 0 {
            let shift = delta.unsigned_abs() as u16;
            self.preview_scroll = self.preview_scroll.saturating_sub(shift);
        } else if delta > 0 {
            self.preview_scroll = self.preview_scroll.saturating_add(delta as u16).min(max);
        }
    }

    fn resize_panel(&mut self, delta: i8) {
        let step = 5u16;
        match self.layout_mode {
            LayoutMode::Columns => {
                // L (delta>0) = grow focused panel, H (delta<0) = shrink focused panel
                // Neighbour is always the adjacent panel to the right, except for Preview which uses Entries.
                let idx = match self.focus {
                    Focus::Feeds => 0,
                    Focus::Entries => 1,
                    Focus::Preview => 2,
                };
                let neighbour = if idx < 2 { idx + 1 } else { 1 };
                // For Preview (rightmost), H grows and L shrinks (directions are mirrored)
                let growing = if idx == 2 { delta < 0 } else { delta > 0 };
                let (grow, shrink) = if growing {
                    (idx, neighbour)
                } else {
                    (neighbour, idx)
                };
                if self.panel_ratios[shrink] > step + 10 {
                    self.panel_ratios[grow] += step;
                    self.panel_ratios[shrink] -= step;
                }
            }
            LayoutMode::Split => {
                match self.focus {
                    Focus::Feeds => {
                        // Resize horizontal split between feeds and right column
                        let growing = delta > 0;
                        if growing && self.panel_ratios[1] > step + 10 {
                            self.panel_ratios[0] += step;
                            self.panel_ratios[1] -= step;
                        } else if !growing && self.panel_ratios[0] > step + 10 {
                            self.panel_ratios[0] -= step;
                            self.panel_ratios[1] += step;
                        }
                    }
                    Focus::Entries | Focus::Preview => {
                        // Resize vertical split between entries and preview
                        let growing = if self.focus == Focus::Preview {
                            delta < 0
                        } else {
                            delta > 0
                        };
                        if growing && self.split_ratio < 90 {
                            self.split_ratio += step;
                        } else if !growing && self.split_ratio > 10 {
                            self.split_ratio -= step;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_feed(id: i64, title: &str, group_id: Option<i64>) -> Feed {
        Feed {
            id,
            title: Some(title.to_string()),
            custom_title: None,
            url: format!("https://{}.com/rss", title.to_lowercase()),
            etag: None,
            last_modified: None,
            last_checked_at: None,
            group_id,
        }
    }

    fn sample_entry(id: i64, feed_id: i64, title: &str) -> Entry {
        Entry {
            id,
            feed_id,
            title: Some(title.to_string()),
            url: None,
            author: None,
            published_at: None,
            fetched_at: 0,
            summary: None,
            content: None,
            read_at: None,
            saved_at: None,
        }
    }

    #[test]
    fn feeds_loaded_builds_rows() {
        let mut state = AppState::default();
        let feeds = vec![sample_feed(1, "Alpha", None), sample_feed(2, "Beta", None)];
        state.reduce(Action::FeedsLoaded(feeds));
        state.flush_feed_rows();
        // AllFeeds + 2 feed items
        assert_eq!(state.feed_rows.len(), 3);
        assert!(matches!(state.feed_rows[0], FeedRow::AllFeeds));
        assert_eq!(state.selected_feed_row_index, Some(0));
    }

    #[test]
    fn entries_loaded_selects_first() {
        let mut state = AppState::default();
        let entries = vec![sample_entry(10, 1, "Post A"), sample_entry(11, 1, "Post B")];
        state.reduce(Action::EntriesLoaded(entries));
        assert_eq!(state.selected_entry, Some(10));
        assert_eq!(state.selected_entry_index, Some(0));
    }

    #[test]
    fn entries_loaded_empty_clears_selection() {
        let mut state = AppState::default();
        state.selected_entry = Some(10);
        state.reduce(Action::EntriesLoaded(Vec::new()));
        assert!(state.selected_entry.is_none());
        assert!(state.selected_entry_index.is_none());
    }

    #[test]
    fn entries_loaded_preserves_selection() {
        let mut state = AppState::default();
        state.selected_entry = Some(11);
        let entries = vec![sample_entry(10, 1, "A"), sample_entry(11, 1, "B")];
        state.reduce(Action::EntriesLoaded(entries));
        assert_eq!(state.selected_entry, Some(11));
        assert_eq!(state.selected_entry_index, Some(1));
    }

    #[test]
    fn select_feed_clears_entries() {
        let mut state = AppState::default();
        state.entries = vec![sample_entry(10, 1, "X")];
        state.selected_entry = Some(10);
        state.reduce(Action::SelectFeed(Some(2)));
        assert_eq!(state.selected_feed, Some(2));
        assert!(state.entries.is_empty());
        assert!(state.selected_entry.is_none());
    }

    #[test]
    fn toggle_filters() {
        let mut state = AppState::default();
        assert!(state.unread_only);
        state.reduce(Action::ToggleUnreadFilter);
        assert!(!state.unread_only);
        state.reduce(Action::ToggleUnreadFilter);
        assert!(state.unread_only);

        assert!(!state.saved_only);
        state.reduce(Action::ToggleSavedFilter);
        assert!(state.saved_only);
    }

    #[test]
    fn toggle_group_collapse() {
        let mut state = AppState::default();
        let feeds = vec![sample_feed(1, "A", Some(100))];
        let groups = vec![Group {
            id: 100,
            name: "Tech".to_string(),
            position: 0,
        }];
        state.reduce(Action::FeedsLoaded(feeds));
        state.reduce(Action::GroupsLoaded(groups));
        state.flush_feed_rows();
        // Group header + 1 feed item + no ungrouped
        assert_eq!(state.feed_rows.len(), 3); // All + GroupHeader + FeedItem

        state.reduce(Action::ToggleGroupCollapse(100));
        assert!(state.collapsed_groups.contains(&100));
        assert_eq!(state.feed_rows.len(), 2); // All + GroupHeader (feed hidden)

        state.reduce(Action::ToggleGroupCollapse(100));
        assert!(!state.collapsed_groups.contains(&100));
        assert_eq!(state.feed_rows.len(), 3);
    }

    #[test]
    fn set_and_clear_status() {
        let mut state = AppState::default();
        state.reduce(Action::SetStatus("hello".to_string()));
        assert_eq!(state.status.as_ref().unwrap().message, "hello");
        assert_eq!(state.status.as_ref().unwrap().kind, StatusKind::Info);

        state.reduce(Action::ClearStatus);
        assert!(state.status.is_none());
    }

    #[test]
    fn db_error_sets_error_status() {
        let mut state = AppState::default();
        state.reduce(Action::DbError("oops".to_string()));
        assert_eq!(state.status.as_ref().unwrap().kind, StatusKind::Error);
    }

    #[test]
    fn resize_panel() {
        let mut state = AppState::default();
        let initial = state.panel_ratios;
        state.focus = Focus::Feeds;
        state.reduce(Action::ResizePanel(1)); // grow feeds
        assert!(state.panel_ratios[0] > initial[0]);
        assert!(state.panel_ratios[1] < initial[1]);
    }

    #[test]
    fn scroll_preview_clamped() {
        let mut state = AppState::default();
        state.focus = Focus::Preview;
        state.preview_content_len = 5;
        state.reduce(Action::MoveUp); // scroll -1 from 0
        assert_eq!(state.preview_scroll, 0);
        state.reduce(Action::MoveDown);
        assert_eq!(state.preview_scroll, 1);
        state.reduce(Action::ScrollBottom);
        assert_eq!(state.preview_scroll, 4);
        state.reduce(Action::ScrollTop);
        assert_eq!(state.preview_scroll, 0);
    }

    #[test]
    fn sort_mode_cycle() {
        let mode = SortMode::DateDesc;
        assert_eq!(mode.next(), SortMode::DateAsc);
        assert_eq!(mode.next().next(), SortMode::TitleAsc);
        assert_eq!(mode.next().next().next(), SortMode::DateDesc);
    }

    #[test]
    fn search_query_trims_whitespace() {
        let mut state = AppState::default();
        state.reduce(Action::SetSearchQuery("  hello  ".to_string()));
        assert_eq!(state.search_query.as_deref(), Some("hello"));
        state.reduce(Action::SetSearchQuery("   ".to_string()));
        assert!(state.search_query.is_none());
    }
}
