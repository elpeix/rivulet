mod keyboard;
mod modals;
mod mouse;

pub use keyboard::handle_key;
pub use modals::{current_modal, handle_help_key, handle_input_mode};
pub use mouse::handle_mouse;

use crate::app::actions::Action;
use crate::app::state::FeedRow;
use crate::app::App;

/// Reload entries based on the currently selected feed row.
/// Shared helper that eliminates duplication between reload and navigation.
pub(crate) fn dispatch_load_entries(app: &mut App) {
    if let Some(row_idx) = app.state.selected_feed_row_index {
        if let Some(row) = app.state.feed_rows.get(row_idx) {
            match row {
                FeedRow::AllFeeds => {
                    let _ = app.dispatch(Action::LoadAllEntries {
                        unread_only: app.state.unread_only,
                        saved_only: app.state.saved_only,
                        since: app.since_cutoff(),
                    });
                    return;
                }
                FeedRow::GroupHeader { group_id, .. } => {
                    let gid = *group_id;
                    let _ = app.dispatch(Action::LoadEntriesForGroup {
                        group_id: Some(gid),
                        unread_only: app.state.unread_only,
                        saved_only: app.state.saved_only,
                        since: app.since_cutoff(),
                    });
                    return;
                }
                FeedRow::UngroupedHeader { .. } => {
                    let _ = app.dispatch(Action::LoadEntriesForGroup {
                        group_id: None,
                        unread_only: app.state.unread_only,
                        saved_only: app.state.saved_only,
                        since: app.since_cutoff(),
                    });
                    return;
                }
                FeedRow::FeedItem { .. } => {}
            }
        }
    }
    if let Some(feed_id) = app.state.selected_feed {
        let _ = app.dispatch(Action::LoadEntriesFiltered {
            feed_id,
            unread_only: app.state.unread_only,
            saved_only: app.state.saved_only,
            since: app.since_cutoff(),
        });
    }
}
