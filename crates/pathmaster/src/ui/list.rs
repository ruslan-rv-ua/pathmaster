//! What every `ListCtrl` in the application is asked, wherever it lives: which
//! row is the user on, and how wide is a column meant to be?
//!
//! Both answers are here rather than on any one surface because each has a rule
//! in it — two readings and a subtraction that must not happen for the first, an
//! FFI boundary wxdragon does not scale across for the second — and one rule
//! with two copies is one rule that can come apart. Everything else about a list
//! differs between its surfaces: the Scope list is rebuilt by operations and
//! lands focus deliberately, the Backups list is rebuilt by a directory and
//! never moves anyone, and the Fix Issues list is built once and never rebuilt
//! at all.

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

/// The application's explicit FromDIP conversion (spec §12 D4).
///
/// wxdragon applies `FromDIP` implicitly to the sizes that cross the FFI
/// boundary through a builder, but **`ListCtrl` column widths cross it raw** —
/// so every hardcoded column constant in the application is scaled here,
/// against the live DPI, and nowhere else.
pub fn from_dip(list: &ListCtrl, dip: i32) -> i32 {
    let dc = ClientDC::new(list);
    let (ppi_x, _) = dc.get_ppi();
    if ppi_x > 0 {
        dip * ppi_x / 96
    } else {
        dip
    }
}
