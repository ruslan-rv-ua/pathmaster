//! The Backups tab: every Snapshot on disk, and the one button that brings one
//! back (spec §8, FR-backup-ui; §12, §15).
//!
//! It is not a Scope, and almost every difference from `scope_page` follows
//! from that. Nothing here is edited, so nothing here is announced: activating
//! the tab says nothing, and a Corrupted file says `[Corrupted]` in its own row
//! rather than through the Banner — passive list text, read for free when the
//! row takes focus, exactly as the Status column is (`CONTEXT.md`,
//! **Corrupted**).
//!
//! Restore is the tab's whole action, and its availability is a property of the
//! **focused row**: a Corrupted Snapshot has nothing to load, and a Snapshot
//! whose Scope cannot be written has nowhere to load it. Both read as a
//! disabled button, which is how a screen reader is told (spec §5, §15).

use std::cell::RefCell;
use std::rc::Rc;

use pathmaster_core::backups::Row;
use pathmaster_core::catalogue::Catalogue;
use pathmaster_core::msgids;
use pathmaster_core::session::{Scope, ValueType};
use wxdragon::prelude::*;

use crate::catalog::translate;
use crate::ui::list;

/// `wxLIST_AUTOSIZE_USEHEADER`: fit the column to the wider of its content and
/// its header, and — on the last column — to whatever width is left.
///
/// Not a pixel constant, which is the point. §12 D2 gives the application
/// exactly one of those and one explicit `FromDIP` call, both spent on the
/// Scope list's Status column; three more here would be three more numbers to
/// keep true across every DPI. What these columns hold is short and known —
/// a fixed-width stamp, a Scope's name, a count — so wx can measure them.
const AUTOSIZE_USEHEADER: i32 = -2;

/// The Backups tab.
pub struct BackupsPage {
    pub panel: Panel,
    pub list: ListCtrl,
    pub restore: Button,
    /// What the rows on screen stand for, in the same order.
    ///
    /// Held rather than re-read at Restore time: the two-layer validation has
    /// already opened every file to build this list, so restoring from what
    /// was read is one fewer read that can fail — and it cannot disagree with
    /// the row the user is looking at.
    rows: RefCell<Vec<Row>>,
    /// The one Catalogue, for the three columns this tab composes (ADR-0009).
    catalogue: Rc<Catalogue>,
}

impl BackupsPage {
    /// Builds the tab, empty. Its content arrives with the first
    /// [`show`](Self::show) — the directory is read when the tab is activated,
    /// not at startup, because a Snapshot may have been written by the other
    /// instance since.
    pub fn build(notebook: &Notebook, catalogue: &Rc<Catalogue>) -> BackupsPage {
        let panel = Panel::builder(notebook).build();
        let list = ListCtrl::builder(&panel)
            .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel)
            .build();
        for (column, heading) in [
            msgids::COLUMN_DATE_AND_TIME,
            msgids::COLUMN_SCOPE,
            msgids::COLUMN_ENTRIES,
        ]
        .into_iter()
        .enumerate()
        {
            list.insert_column(
                column as i64,
                &translate(heading),
                ListColumnFormat::Left,
                0,
            );
        }
        // Inserted at zero and measured here: a column's width is never a
        // constant in this application.
        fit_columns(&list);

        let restore = Button::builder(&panel)
            .with_label(&translate(msgids::BUTTON_RESTORE))
            .build();
        let button_row = BoxSizer::builder(Orientation::Horizontal).build();
        button_row.add(&restore, 0, SizerFlag::All, 4);

        let sizer = BoxSizer::builder(Orientation::Vertical).build();
        sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 4);
        sizer.add_sizer(&button_row, 0, SizerFlag::Expand | SizerFlag::All, 4);
        panel.set_sizer(sizer, true);

        // The last column takes the width the first two leave, which is what
        // `AUTOSIZE_USEHEADER` means on the last one; re-asked on every resize
        // for the same reason the Scope list re-fits its Path column.
        panel.on_size(move |event| {
            panel.layout();
            fit_columns(&list);
            event.skip(true);
        });

        BackupsPage {
            panel,
            list,
            restore,
            rows: RefCell::new(Vec::new()),
            catalogue: Rc::clone(catalogue),
        }
    }

    /// Redraws the list over `rows`, newest first as they arrive.
    ///
    /// Focus is left exactly where it is. Nothing here is an operation, so
    /// there is no row an operation points at — the user Tabs into the list and
    /// arrows through it, and a rebuild that moved them would be a rebuild they
    /// did not ask for.
    ///
    /// The widget is filled first and [`rows`](Self::rows) replaced last, and
    /// **no borrow of it is held across either**: a list fires its own events
    /// synchronously, `on_item_focused` among them, and that handler reads this
    /// cell. The window re-syncs the button immediately afterwards, so a row
    /// focused mid-rebuild is a stale answer for the length of one call and not
    /// a panic.
    pub fn show(&self, rows: Vec<Row>) {
        self.list.delete_all_items();
        for (index, row) in rows.iter().enumerate() {
            let [taken, scope, entries] = self.catalogue.backup_row(row);
            self.list.insert_item(index as i64, &taken, None);
            self.list.set_item_text_by_column(index as i64, 1, &scope);
            self.list.set_item_text_by_column(index as i64, 2, &entries);
        }
        fit_columns(&self.list);
        *self.rows.borrow_mut() = rows;
    }

    /// The Scope the focused row would be restored into, or `None` when there
    /// is nothing to restore: an empty list, a row nothing has reached yet, or
    /// a Corrupted Snapshot, which has nothing to load.
    ///
    /// It answers the Scope rather than a bare `true` because the caller's
    /// other half of the question is about that Scope: a Snapshot goes back
    /// into the Scope it was taken from, and a Session that cannot be written
    /// is the second reason Restore reads as disabled.
    pub fn restore_target(&self) -> Option<Scope> {
        let rows = self.rows.borrow();
        let row = rows.get(list::focused_row(&self.list)?)?;
        row.restores().is_some().then_some(row.scope())
    }

    /// What restoring the focused row loads: the Scope it goes into, its
    /// Entries and the Value Type they were stored under. Owned, because the
    /// caller is about to hand it to a Session that this page's borrow must
    /// not still be open across.
    pub fn restore_payload(&self) -> Option<(Scope, Vec<String>, ValueType)> {
        let rows = self.rows.borrow();
        let row = rows.get(list::focused_row(&self.list)?)?;
        let (entries, value_type) = row.restores()?;
        Some((row.scope(), entries.to_vec(), value_type))
    }

    /// Points the Restore button at what the focused row is worth — the same
    /// shape as a Scope tab's `sync_buttons`, over the one button this tab has.
    pub fn sync_button(&self, restorable: bool) {
        self.restore.enable(restorable);
    }
}

/// Re-fits all three columns to what they now hold.
fn fit_columns(list: &ListCtrl) {
    for column in 0..list.get_column_count() {
        list.set_column_width(column as i64, AUTOSIZE_USEHEADER);
    }
}
