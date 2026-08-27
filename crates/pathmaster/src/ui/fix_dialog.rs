//! Edit → "Fix Issues…": the modal, per-Scope repair surface (v0.2.0 §7,
//! `CONTEXT.md` **Fix Issues**).
//!
//! What the dialog shows is a [`Plan`] the caller built and handed over — the
//! active Scope's fixable Entries as the last completed pass found them.
//! Nothing here reads a Working Copy or a Session, and **nothing here is
//! live**: no diagnostics, no Timer of its own, which is what keeps a modal's
//! nested event loop free of the borrow hazard (ADR-0011). Modality is the
//! fence the staleness rule leans on: nothing can change underneath this
//! dialog, so the plan it was opened with is still true when it closes.
//!
//! **The checkboxes are the native ones, through the raw-`LVM_*` hatch.**
//! wxdragon exposes neither `LVS_EX_CHECKBOXES` nor the check events, so both
//! the style and the states go through `SendMessageW` on the list's own
//! `SysListView32` handle — in-process, which is what makes it safe (ticket
//! 01's research; the cross-process crash rule is about pointer-carrying
//! messages sent to another process). With the style set, comctl32 draws the
//! boxes, Space toggles the focused row, and NVDA reads "checked" / "not
//! checked" per row and announces a toggle in place — all of it native, none
//! of it ours (measured, ticket 16 probe 7).
//!
//! **The states are read once, at apply time**, by `LVM_GETITEMSTATE`. There
//! is no check event to listen for and none is wanted: what the user has
//! checked is a fact the control holds, and asking it once when the answer is
//! needed cannot drift from what is on screen.

use pathmaster_core::catalogue::Catalogue;
use pathmaster_core::fix::{Action, Plan};
use pathmaster_core::msgids;
use pathmaster_core::session::EntryId;
use wxdragon::prelude::*;

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::Controls::{
    LVIS_STATEIMAGEMASK, LVITEMW, LVM_GETITEMSTATE, LVM_SETEXTENDEDLISTVIEWSTYLE, LVM_SETITEMSTATE,
    LVS_EX_CHECKBOXES,
};
use windows_sys::Win32::UI::WindowsAndMessaging::SendMessageW;

use crate::catalog::translate;
use crate::ui::{door, list};

/// The two ways out. Local to this module, like every other dialog's.
const ID_COMMIT: Id = ID_HIGHEST + 141;
const ID_ABANDON: Id = ID_HIGHEST + 142;

/// The list's size in DIP, and the widths of the three columns that hold text
/// of a predictable length — a position, one or two comma-joined Issue words,
/// and one of two action names. Path takes what is left, as it does in the
/// main list, because a path is unbounded.
///
/// The size is given rather than fitted for the Tree View's reason: a fit
/// would size the dialog to whichever Entries happen to be broken today.
const LIST_WIDTH_DIP: i32 = 680;
const LIST_HEIGHT_DIP: i32 = 300;
const INDEX_COLUMN_DIP: i32 = 48;
const ISSUE_COLUMN_DIP: i32 = 160;
const ACTION_COLUMN_DIP: i32 = 130;

/// comctl32's state-image field is the top 12 bits of an item's state, and the
/// two values a checkbox uses are 1 (unchecked) and 2 (checked) shifted into
/// it. `LVIS_STATEIMAGEMASK` is the mask for exactly that field.
const STATE_IMAGE_SHIFT: u32 = 12;
const UNCHECKED_IMAGE: u32 = 1;
const CHECKED_IMAGE: u32 = 2;

/// Opens the Fix Issues dialog over `plan` and answers with the Entries the
/// user chose to repair, each with the repair its row proposed — in the plan's
/// own order, so applying them walks the Working Copy forwards.
///
/// `None` is "do nothing at all", and it covers both ways of meaning it:
/// Cancel or Escape, **and** [Fix selected] with nothing checked. §7 makes
/// them one outcome — no Checkpoint, no Announcement — which is also why the
/// button is never dynamically disabled: a button that vanished as the last
/// box was cleared would be a control moving under a screen reader for a
/// gesture that already means "nothing".
///
/// The answer carries [`EntryId`]s and never positions: the plan's rows are
/// resolved back to Entries by identity, which is the one thing that survives
/// a duplicate (v0.2.0 §7).
pub fn ask_which_to_fix(
    parent: &dyn WxWidget,
    catalogue: &Catalogue,
    title: &str,
    plan: &Plan,
) -> Option<Vec<(EntryId, Action)>> {
    let dialog = Dialog::builder(parent, title).build();
    let panel = Panel::builder(&dialog).build();

    // Created before the buttons because creation order is the Tab order:
    // list → Fix selected → Cancel. `SingleSel` is the app's shape in every
    // list — one row has the focus, and Space acts on that row — and the check
    // state is deliberately *not* the selection: a screen reader reads the two
    // apart, which is why the native checkbox is worth the hatch at all.
    let list = ListCtrl::builder(&panel)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel)
        .with_size(Size::new(LIST_WIDTH_DIP, LIST_HEIGHT_DIP))
        .build();
    let fixed_width = columns(&list);
    fill(&list, catalogue, plan);

    let commit = Button::builder(&panel)
        .with_id(ID_COMMIT)
        .with_label(&translate(msgids::BUTTON_FIX_SELECTED))
        .build();
    let abandon = Button::builder(&panel)
        .with_id(ID_ABANDON)
        .with_label(&translate(msgids::BUTTON_DIALOG_CANCEL))
        .build();
    // **Cancel is the default, and it is the whole point.** Every other dialog
    // in the application makes its commit button the default; §7 inverts it
    // here on purpose, because this is the one dialog whose commit deletes
    // Entries in bulk and whose whole gesture is Space on a row. With Enter
    // bound to Cancel, no keystroke a user is already making while reviewing
    // checkboxes can apply the change — leaving [Fix selected] reachable by
    // Tab and by mouse, which is what a destructive commit should cost.
    abandon.set_default();

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    buttons.add_stretch_spacer(1);
    buttons.add(&commit, 0, SizerFlag::All, 4);
    buttons.add(&abandon, 0, SizerFlag::All, 4);

    let inner = BoxSizer::builder(Orientation::Vertical).build();
    inner.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 8);
    inner.add_sizer(&buttons, 0, SizerFlag::Expand | SizerFlag::All, 8);
    panel.set_sizer(inner, true);

    let outer = BoxSizer::builder(Orientation::Vertical).build();
    outer.add(&panel, 1, SizerFlag::Expand, 0);
    dialog.set_sizer_and_fit(outer, true);
    dialog.centre();

    // Path takes all remaining width, on the first layout and on every resize
    // — the main list's own rule (spec §12 D2), bound on the panel because
    // that is where this application lays a list out.
    panel.on_size(move |event| {
        panel.layout();
        list.set_column_width(1, (list.get_client_size().width - fixed_width).max(0));
        event.skip(true);
    });

    commit.on_click(move |_| dialog.end_modal(ID_COMMIT));
    abandon.on_click(move |_| dialog.end_modal(ID_ABANDON));
    // "[Cancel] keeps default and Escape" (§7): both keys, one answer.
    dialog.set_escape_id(ID_ABANDON);

    // Initial focus on the first row (§7) — which every plan reaching here
    // has, since a Scope with none disables the menu item this arrived
    // through; the empty case is answered rather than assumed away. The list
    // takes the keyboard focus either way: a row focused in a control that is
    // not is silent, which for this application is the same as absent.
    if !plan.is_empty() {
        list.set_item_state(
            0,
            ListItemState::Selected | ListItemState::Focused,
            ListItemState::Selected | ListItemState::Focused,
        );
    }
    list.set_focus();

    // Read **before** the window goes: a destroyed control answers with
    // nothing, and this is the one moment the user's answer exists anywhere.
    let chosen = if door::show(&dialog) == ID_COMMIT {
        checked(&list, plan)
    } else {
        Vec::new()
    };
    dialog.destroy();
    // Nothing checked is a Cancel by another route (§7).
    (!chosen.is_empty()).then_some(chosen)
}

/// The four columns of §7 — `#`, Path, Issue, Action, the first two reusing the
/// main list's own headers because they carry the same two things — and the
/// width the three fixed ones take, which is what Path is sized against.
///
/// `#` would read better right-aligned and cannot be: comctl32 forces
/// `LVCFMT_LEFT` on the leftmost report column, exactly as it does in the main
/// list.
fn columns(list: &ListCtrl) -> i32 {
    // Path's is zero here: it is never a constant, and the panel's size
    // handler sets it on the first layout and on every resize.
    let widths = [
        (msgids::COLUMN_INDEX, INDEX_COLUMN_DIP),
        (msgids::COLUMN_PATH, 0),
        (msgids::COLUMN_ISSUE, ISSUE_COLUMN_DIP),
        (msgids::COLUMN_ACTION, ACTION_COLUMN_DIP),
    ];
    let mut fixed = 0;
    for (index, (msgid, dip)) in widths.into_iter().enumerate() {
        let width = list::from_dip(list, dip);
        fixed += width;
        list.insert_column(
            index as i64,
            &translate(msgid),
            ListColumnFormat::Left,
            width,
        );
    }
    fixed
}

/// Writes every row, then turns the checkboxes on and sets the defaults.
///
/// The style is set **after** the items exist rather than before: comctl32
/// gives an item its state image when the style arrives, so enabling it first
/// and inserting afterwards would leave rows whose box is drawn from a state
/// nobody wrote.
fn fill(list: &ListCtrl, catalogue: &Catalogue, plan: &Plan) {
    for (index, row) in plan.rows().iter().enumerate() {
        let index = index as i64;
        list.insert_item(index, &row.position.to_string(), None);
        list.set_item_text_by_column(index, 1, &row.raw);
        // The Issue column is the Status column's own join, in the Status
        // column's own words: one Issue has one name wherever it is read.
        list.set_item_text_by_column(index, 2, &catalogue.status_column(&row.issues));
        list.set_item_text_by_column(index, 3, &translate(row.action.catalogue_msgid()));
    }

    let Some(hwnd) = handle(list) else { return };
    // SAFETY: a value-carrying message to a live `SysListView32` in this
    // process — no pointer crosses, and the call is synchronous.
    unsafe {
        SendMessageW(
            hwnd,
            LVM_SETEXTENDEDLISTVIEWSTYLE,
            LVS_EX_CHECKBOXES as usize,
            LVS_EX_CHECKBOXES as isize,
        );
    }
    for (index, row) in plan.rows().iter().enumerate() {
        set_checked(hwnd, index, row.checked);
    }
}

/// The rows the user left checked, resolved to the Entries they repair.
///
/// Read straight off the control, once: there is no check event in wxdragon to
/// have listened for, and no copy of the state kept here that could disagree
/// with the boxes on screen.
///
/// A list whose handle has already gone answers "nothing checked", which the
/// caller reads as a Cancel — the safe answer for a window that is no longer
/// there to have been read.
fn checked(list: &ListCtrl, plan: &Plan) -> Vec<(EntryId, Action)> {
    let Some(hwnd) = handle(list) else {
        return Vec::new();
    };
    plan.rows()
        .iter()
        .enumerate()
        .filter(|(index, _)| is_checked(hwnd, *index))
        .map(|(_, row)| (row.id, row.action))
        .collect()
}

/// The list's own window handle, or `None` for a control wx has not made (or
/// has already unmade) — every raw message below goes through this.
fn handle(list: &ListCtrl) -> Option<HWND> {
    let hwnd: HWND = list.get_handle().cast();
    (!hwnd.is_null()).then_some(hwnd)
}

/// Writes one row's checkbox — the state-image half of its item state, and
/// nothing else: the mask is what keeps selection and focus untouched.
fn set_checked(hwnd: HWND, row: usize, checked: bool) {
    let image = if checked {
        CHECKED_IMAGE
    } else {
        UNCHECKED_IMAGE
    };
    // SAFETY: an in-process `SendMessage` on a live `SysListView32`. The
    // `LVITEMW` is a local that outlives the synchronous call, and only the
    // state-image bits the mask names are written.
    unsafe {
        let mut item: LVITEMW = std::mem::zeroed();
        item.stateMask = LVIS_STATEIMAGEMASK;
        item.state = image << STATE_IMAGE_SHIFT;
        SendMessageW(
            hwnd,
            LVM_SETITEMSTATE,
            row,
            std::ptr::from_ref(&item) as isize,
        );
    }
}

/// Whether one row's checkbox is checked, asked of the control itself.
fn is_checked(hwnd: HWND, row: usize) -> bool {
    // SAFETY: a value-carrying message, in-process; the mask is a scalar and
    // no pointer crosses.
    let state = unsafe { SendMessageW(hwnd, LVM_GETITEMSTATE, row, LVIS_STATEIMAGEMASK as isize) };
    ((state as u32 & LVIS_STATEIMAGEMASK) >> STATE_IMAGE_SHIFT) == CHECKED_IMAGE
}
