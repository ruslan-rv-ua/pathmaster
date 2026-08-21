//! The one question both tabs ask a `ListCtrl`: which row is the user on?
//!
//! It is here rather than on either page because the answer has a rule in it —
//! two readings, and a subtraction that must not happen — and one rule with two
//! copies is one rule that can come apart. Everything else about a list differs
//! between the two tabs: the Scope list is rebuilt by operations and lands
//! focus deliberately, the Backups list is rebuilt by a directory and never
//! moves anyone.

use wxdragon::prelude::*;

/// The row the user is on, if any: the **focused** one, or the selected one
/// when the list has been clicked but never arrowed through.
///
/// Both readings are needed. comctl32 tracks focus and selection separately,
/// and a fresh click sets selection first; asking only for focus would answer
/// `None` for a row the user is plainly looking at.
///
/// `-1` is comctl32's "no such item" and is what `try_from` rejects here — the
/// same value that, as an index, means *every* row.
pub fn focused_row(list: &ListCtrl) -> Option<usize> {
    let focused = list.get_next_item(-1, ListNextItemFlag::All, ListItemState::Focused);
    let row = if focused >= 0 {
        focused
    } else {
        list.get_first_selected_item()
    };
    usize::try_from(row).ok()
}
