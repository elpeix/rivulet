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
    PanelSearch,
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
    FeedInfo,
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
    pub hide_read_feeds: bool,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub selected_entries: HashSet<i64>,
    pub show_help: bool,
    pub help_scroll: u16,
    pub modal_selection: usize,
    feed_rows_dirty: bool,
    entry_id_to_index: HashMap<i64, usize>,
    pub panel_search_focus: Option<Focus>,
    pub feed_filter_query: Option<String>,
    pub preview_search_query: Option<String>,
    pub preview_match_lines: Vec<usize>,
    pub preview_match_current: Option<usize>,
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
            hide_read_feeds: false,
            input_mode: InputMode::None,
            input_buffer: String::new(),
            selected_entries: HashSet::new(),
            show_help: false,
            help_scroll: 0,
            modal_selection: 0,
            feed_rows_dirty: false,
            entry_id_to_index: HashMap::new(),
            panel_search_focus: None,
            feed_filter_query: None,
            preview_search_query: None,
            preview_match_lines: Vec::new(),
            preview_match_current: None,
        }
    }
}

impl AppState {
    pub fn entry_position(&self, id: i64) -> Option<usize> {
        self.entry_id_to_index.get(&id).copied()
    }

    /// Returns the appropriate refresh action based on the current view context.
    /// - Entries panel with a single feed selected → RefreshFeed
    /// - Entries panel viewing a group/ungrouped → RefreshFeedsByGroup
    /// - Otherwise → RefreshFeeds (all)
    pub fn contextual_refresh_action(&self) -> Action {
        if self.focus == Focus::Entries {
            if let Some(feed_id) = self.selected_feed {
                return Action::RefreshFeed(feed_id);
            }
            if self.viewing_group {
                if let Some(row_idx) = self.selected_feed_row_index {
                    match self.feed_rows.get(row_idx) {
                        Some(FeedRow::GroupHeader { group_id, .. }) => {
                            return Action::RefreshFeedsByGroup(Some(*group_id));
                        }
                        Some(FeedRow::UngroupedHeader { .. }) => {
                            return Action::RefreshFeedsByGroup(None);
                        }
                        _ => {}
                    }
                }
            }
        }
        Action::RefreshFeeds
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
                self.selected_entries.clear();
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
                let old_counts =
                    std::mem::replace(&mut self.unread_counts, counts.into_iter().collect());
                if self.hide_read_feeds
                    && self.feeds.iter().any(|f| {
                        let was_zero = old_counts.get(&f.id).copied().unwrap_or(0) == 0;
                        let is_zero = self.unread_counts.get(&f.id).copied().unwrap_or(0) == 0;
                        was_zero != is_zero
                    })
                {
                    self.feed_rows_dirty = true;
                } else {
                    self.update_feed_row_counts();
                }
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
            | Action::RefreshFeed(_)
            | Action::RefreshFeedsByGroup(_)
            | Action::RefreshUnreadCounts
            | Action::AddFeed { .. }
            | Action::DeleteFeed(_)
            | Action::RenameFeed { .. }
            | Action::MarkRead(_)
            | Action::MarkUnread(_)
            | Action::MarkAllRead(_)
            | Action::MarkAllUnread(_)
            | Action::MarkFeedRead(_)
            | Action::MarkSaved(_)
            | Action::MarkUnsaved(_)
            | Action::MarkAllSaved(_)
            | Action::MarkAllUnsaved(_)
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
            self.preview_scroll = 0;
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

    /// Update preview search match state from computed match line indices.
    /// Called during rendering because matches depend on the rendered body lines,
    /// which are only available after layout and rich-text conversion.
    pub fn update_preview_matches(&mut self, match_lines: Vec<usize>) {
        self.preview_match_lines = match_lines;
        if self.preview_match_lines.is_empty() {
            self.preview_match_current = None;
        } else if let Some(cur) = self.preview_match_current {
            if cur >= self.preview_match_lines.len() {
                self.preview_match_current = Some(0);
                self.preview_scroll = self.preview_match_lines[0] as u16;
            }
        } else {
            // First search: auto-select and scroll to first match
            self.preview_match_current = Some(0);
            self.preview_scroll = self.preview_match_lines[0] as u16;
        }
    }

    /// Adjust the local unread count for a feed by `delta` (+1 or -1).
    /// Updates both per-feed and total counters, then refreshes feed row display.
    pub fn adjust_unread_count(&mut self, feed_id: i64, delta: i64) {
        let count = self.unread_counts.entry(feed_id).or_insert(0);
        *count = (*count + delta).max(0);
        self.total_unread = (self.total_unread + delta).max(0);
        self.update_feed_row_counts();
    }

    /// Update unread counters in existing feed rows without changing structure.
    /// Feeds that became empty stay visible until an explicit rebuild.
    fn update_feed_row_counts(&mut self) {
        let mut feeds_by_group: HashMap<Option<i64>, Vec<usize>> = HashMap::new();
        for (i, feed) in self.feeds.iter().enumerate() {
            feeds_by_group.entry(feed.group_id).or_default().push(i);
        }
        for row in &mut self.feed_rows {
            match row {
                FeedRow::GroupHeader {
                    group_id, unread, ..
                } => {
                    *unread = feeds_by_group.get(&Some(*group_id)).map_or(0, |indices| {
                        indices
                            .iter()
                            .map(|&i| {
                                self.feeds
                                    .get(i)
                                    .and_then(|f| self.unread_counts.get(&f.id).copied())
                                    .unwrap_or(0)
                            })
                            .sum()
                    });
                }
                FeedRow::UngroupedHeader { unread } => {
                    *unread = feeds_by_group.get(&None).map_or(0, |indices| {
                        indices
                            .iter()
                            .map(|&i| {
                                self.feeds
                                    .get(i)
                                    .and_then(|f| self.unread_counts.get(&f.id).copied())
                                    .unwrap_or(0)
                            })
                            .sum()
                    });
                }
                FeedRow::AllFeeds | FeedRow::FeedItem { .. } => {}
            }
        }
    }

    /// Rebuild feed rows if dirty. Returns `true` when the selected feed changed
    /// (e.g. because the previous one was hidden), so the caller can reload entries.
    pub fn flush_feed_rows(&mut self) -> bool {
        if !self.feed_rows_dirty {
            return false;
        }
        self.feed_rows_dirty = false;
        let prev_feed = self.selected_feed;
        self.rebuild_feed_rows();
        if self.selected_feed_row_index.is_none() && !self.feed_rows.is_empty() {
            self.selected_feed_row_index = Some(0);
        }
        // Sync selected_feed to whatever feed_row is now selected
        self.sync_selected_feed_from_row();
        self.selected_feed != prev_feed
    }

    /// Update selected_feed / selected_feed_index to match current feed row.
    fn sync_selected_feed_from_row(&mut self) {
        if let Some(row_idx) = self.selected_feed_row_index {
            if let Some(FeedRow::FeedItem { feed_index }) = self.feed_rows.get(row_idx) {
                if let Some(feed) = self.feeds.get(*feed_index) {
                    self.selected_feed = Some(feed.id);
                    self.selected_feed_index = Some(*feed_index);
                    return;
                }
            }
        }
        self.selected_feed = None;
        self.selected_feed_index = None;
    }

    pub fn rebuild_feed_rows(&mut self) {
        let prev_row_index = self.selected_feed_row_index;
        self.feed_rows.clear();
        // "All" row always first
        self.feed_rows.push(FeedRow::AllFeeds);

        // Helper: should this feed be hidden?
        let hiding = self.hide_read_feeds;
        let feed_query = self.feed_filter_query.as_deref().map(|q| q.to_lowercase());
        let feed_visible =
            |feed_index: usize, feeds: &[Feed], counts: &HashMap<i64, i64>| -> bool {
                if hiding {
                    let unread = feeds
                        .get(feed_index)
                        .and_then(|f| counts.get(&f.id).copied())
                        .unwrap_or(0);
                    if unread == 0 {
                        return false;
                    }
                }
                if let Some(ref q) = feed_query {
                    let title = feeds
                        .get(feed_index)
                        .and_then(|f| f.display_title())
                        .unwrap_or("");
                    if !title.to_lowercase().contains(q) {
                        return false;
                    }
                }
                true
            };

        if self.groups.is_empty() {
            // No groups: flat list
            for (i, _feed) in self.feeds.iter().enumerate() {
                if feed_visible(i, &self.feeds, &self.unread_counts) {
                    self.feed_rows.push(FeedRow::FeedItem { feed_index: i });
                }
            }
            // Sync selected_feed_row_index (+1 offset for AllFeeds)
            if let Some(fi) = self.selected_feed_index {
                let found = self.feed_rows.iter().position(
                    |row| matches!(row, FeedRow::FeedItem { feed_index } if *feed_index == fi),
                );
                self.selected_feed_row_index = found
                    .or(prev_row_index)
                    .map(|i| i.min(self.feed_rows.len().saturating_sub(1)));
            }
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
            // Hide empty groups when hiding empty feeds
            if hiding && unread == 0 {
                continue;
            }
            self.feed_rows.push(FeedRow::GroupHeader {
                group_id: group.id,
                name: group.name.clone(),
                unread,
            });
            if !self.collapsed_groups.contains(&group.id) {
                if let Some(indices) = group_feeds {
                    for &fi in indices {
                        if feed_visible(fi, &self.feeds, &self.unread_counts) {
                            self.feed_rows.push(FeedRow::FeedItem { feed_index: fi });
                        }
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
            if !hiding || unread > 0 {
                self.feed_rows.push(FeedRow::UngroupedHeader { unread });
                for &fi in ungrouped {
                    if feed_visible(fi, &self.feeds, &self.unread_counts) {
                        self.feed_rows.push(FeedRow::FeedItem { feed_index: fi });
                    }
                }
            }
        }

        // Update selected_feed_row_index to match selected_feed
        if let Some(feed_id) = self.selected_feed {
            let found = self.feed_rows.iter().position(|row| {
                if let FeedRow::FeedItem { feed_index } = row {
                    self.feeds.get(*feed_index).map(|f| f.id) == Some(feed_id)
                } else {
                    false
                }
            });
            self.selected_feed_row_index = found
                .or(prev_row_index)
                .map(|i| i.min(self.feed_rows.len().saturating_sub(1)));
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
    fn hide_read_feeds_filters_rows() {
        let mut state = AppState::default();
        let feeds = vec![sample_feed(1, "Alpha", None), sample_feed(2, "Beta", None)];
        state.reduce(Action::FeedsLoaded(feeds));
        // Alpha has 3 unread, Beta has 0
        state.reduce(Action::UpdateUnreadCounts(vec![(1, 3)]));
        state.flush_feed_rows();
        // All + Alpha + Beta = 3 rows
        assert_eq!(state.feed_rows.len(), 3);

        // Enable hide_read_feeds
        state.hide_read_feeds = true;
        state.rebuild_feed_rows();
        // All + Alpha only = 2 rows (Beta hidden)
        assert_eq!(state.feed_rows.len(), 2);
        assert!(matches!(state.feed_rows[0], FeedRow::AllFeeds));
        assert!(matches!(
            state.feed_rows[1],
            FeedRow::FeedItem { feed_index: 0 }
        ));

        // Disable again
        state.hide_read_feeds = false;
        state.rebuild_feed_rows();
        assert_eq!(state.feed_rows.len(), 3);
    }

    #[test]
    fn hide_read_feeds_hides_empty_groups() {
        let mut state = AppState::default();
        let feeds = vec![
            sample_feed(1, "A", Some(100)),
            sample_feed(2, "B", Some(200)),
        ];
        let groups = vec![
            Group {
                id: 100,
                name: "Tech".to_string(),
                position: 0,
            },
            Group {
                id: 200,
                name: "News".to_string(),
                position: 1,
            },
        ];
        state.reduce(Action::FeedsLoaded(feeds));
        state.reduce(Action::GroupsLoaded(groups));
        // Only feed 1 has unread
        state.reduce(Action::UpdateUnreadCounts(vec![(1, 5)]));
        state.flush_feed_rows();
        // All + Tech header + feed A + News header + feed B = 5
        assert_eq!(state.feed_rows.len(), 5);

        state.hide_read_feeds = true;
        state.rebuild_feed_rows();
        // All + Tech header + feed A = 3 (News group hidden entirely)
        assert_eq!(state.feed_rows.len(), 3);
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
    fn entry_change_resets_preview_scroll() {
        let mut state = AppState::default();
        state.entries = vec![
            Entry {
                id: 1,
                feed_id: 1,
                title: Some("A".into()),
                url: None,
                author: None,
                content: None,
                summary: None,
                published_at: None,
                fetched_at: 0,
                read_at: None,
                saved_at: None,
            },
            Entry {
                id: 2,
                feed_id: 1,
                title: Some("B".into()),
                url: None,
                author: None,
                content: None,
                summary: None,
                published_at: None,
                fetched_at: 0,
                read_at: None,
                saved_at: None,
            },
        ];
        state.focus = Focus::Entries;
        state.selected_entry = Some(1);
        state.selected_entry_index = Some(0);
        state.preview_scroll = 15;
        state.reduce(Action::MoveDown);
        assert_eq!(state.selected_entry, Some(2));
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

    #[test]
    fn feed_filter_query_filters_rows() {
        let mut state = AppState::default();
        let feeds = vec![
            sample_feed(1, "Rust Blog", None),
            sample_feed(2, "Go Weekly", None),
            sample_feed(3, "Rust News", None),
        ];
        state.reduce(Action::FeedsLoaded(feeds));
        state.flush_feed_rows();
        assert_eq!(state.feed_rows.len(), 4); // All + 3 feeds

        state.feed_filter_query = Some("rust".to_string());
        state.rebuild_feed_rows();
        // All + Rust Blog + Rust News = 3
        assert_eq!(state.feed_rows.len(), 3);
        assert!(matches!(state.feed_rows[0], FeedRow::AllFeeds));
        assert!(matches!(
            state.feed_rows[1],
            FeedRow::FeedItem { feed_index: 0 }
        ));
        assert!(matches!(
            state.feed_rows[2],
            FeedRow::FeedItem { feed_index: 2 }
        ));

        // Clear filter restores all
        state.feed_filter_query = None;
        state.rebuild_feed_rows();
        assert_eq!(state.feed_rows.len(), 4);
    }

    #[test]
    fn feed_filter_query_case_insensitive() {
        let mut state = AppState::default();
        state.reduce(Action::FeedsLoaded(vec![sample_feed(1, "RUST Blog", None)]));
        state.flush_feed_rows();

        state.feed_filter_query = Some("rust".to_string());
        state.rebuild_feed_rows();
        assert_eq!(state.feed_rows.len(), 2); // All + matching feed
    }

    #[test]
    fn adjust_unread_count_increments() {
        let mut state = AppState::default();
        state.total_unread = 5;
        state.unread_counts.insert(1, 3);

        state.adjust_unread_count(1, -1);
        assert_eq!(state.unread_counts[&1], 2);
        assert_eq!(state.total_unread, 4);

        state.adjust_unread_count(1, 1);
        assert_eq!(state.unread_counts[&1], 3);
        assert_eq!(state.total_unread, 5);
    }

    #[test]
    fn adjust_unread_count_floors_at_zero() {
        let mut state = AppState::default();
        state.total_unread = 0;
        state.unread_counts.insert(1, 0);

        state.adjust_unread_count(1, -1);
        assert_eq!(state.unread_counts[&1], 0);
        assert_eq!(state.total_unread, 0);
    }

    #[test]
    fn adjust_unread_count_creates_entry() {
        let mut state = AppState::default();
        state.total_unread = 0;
        assert!(!state.unread_counts.contains_key(&42));

        state.adjust_unread_count(42, 1);
        assert_eq!(state.unread_counts[&42], 1);
        assert_eq!(state.total_unread, 1);
    }

    #[test]
    fn update_preview_matches_auto_scrolls_to_first() {
        let mut state = AppState::default();
        state.preview_scroll = 50;
        state.preview_match_current = None;

        state.update_preview_matches(vec![10, 30, 60]);
        assert_eq!(state.preview_match_current, Some(0));
        assert_eq!(state.preview_scroll, 10);
    }

    #[test]
    fn update_preview_matches_clamps_out_of_bounds() {
        let mut state = AppState::default();
        state.preview_match_current = Some(5);

        state.update_preview_matches(vec![3, 7]);
        assert_eq!(state.preview_match_current, Some(0));
        assert_eq!(state.preview_scroll, 3);
    }

    #[test]
    fn update_preview_matches_empty_clears() {
        let mut state = AppState::default();
        state.preview_match_current = Some(2);

        state.update_preview_matches(Vec::new());
        assert_eq!(state.preview_match_current, None);
    }

    #[test]
    fn contextual_refresh_entries_with_feed_returns_refresh_feed() {
        let mut state = AppState::default();
        state.focus = Focus::Entries;
        state.selected_feed = Some(42);
        assert!(matches!(
            state.contextual_refresh_action(),
            Action::RefreshFeed(42)
        ));
    }

    #[test]
    fn contextual_refresh_entries_group_header_returns_refresh_by_group() {
        let mut state = AppState::default();
        state.focus = Focus::Entries;
        state.selected_feed = None;
        state.viewing_group = true;
        state.feed_rows = vec![
            FeedRow::AllFeeds,
            FeedRow::GroupHeader {
                group_id: 10,
                name: "Tech".to_string(),
                unread: 0,
            },
        ];
        state.selected_feed_row_index = Some(1);
        assert!(matches!(
            state.contextual_refresh_action(),
            Action::RefreshFeedsByGroup(Some(10))
        ));
    }

    #[test]
    fn contextual_refresh_entries_ungrouped_header_returns_refresh_none_group() {
        let mut state = AppState::default();
        state.focus = Focus::Entries;
        state.selected_feed = None;
        state.viewing_group = true;
        state.feed_rows = vec![FeedRow::AllFeeds, FeedRow::UngroupedHeader { unread: 0 }];
        state.selected_feed_row_index = Some(1);
        assert!(matches!(
            state.contextual_refresh_action(),
            Action::RefreshFeedsByGroup(None)
        ));
    }

    #[test]
    fn contextual_refresh_feeds_panel_returns_refresh_all() {
        let mut state = AppState::default();
        state.focus = Focus::Feeds;
        state.selected_feed = Some(42);
        assert!(matches!(
            state.contextual_refresh_action(),
            Action::RefreshFeeds
        ));
    }

    #[test]
    fn contextual_refresh_entries_no_feed_no_group_returns_refresh_all() {
        let mut state = AppState::default();
        state.focus = Focus::Entries;
        state.selected_feed = None;
        state.viewing_group = false;
        assert!(matches!(
            state.contextual_refresh_action(),
            Action::RefreshFeeds
        ));
    }
}
