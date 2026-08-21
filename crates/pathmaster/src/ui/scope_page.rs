//! One Scope tab: the list of Entries and the buttons that edit them
//! (spec §7, §12, §15).
//!
//! Everything about focus lives here, because focus is how this application
//! speaks. Delete has no Announcement and no confirmation; the commit of an
//! Add has none either — what the user hears is NVDA reading the row focus
//! landed on. So every operation that changes the list ends by putting a row
//! into the focused state **and giving the list the keyboard focus**: a row
//! that is focused in a control that is not is silent, which for this
//! application is the same as not having happened.

use std::rc::Rc;

use pathmaster_core::catalogue::Catalogue;
use pathmaster_core::diagnostics::Findings;
use pathmaster_core::msgids;
use pathmaster_core::session::{EntryId, Session};
use wxdragon::prelude::*;

use crate::catalog::translate;
use crate::ui::command::Command;

/// Status column width in DIP — the app's single deliberate pixel constant
/// (spec §12 D2). Status text is of predictable length (comma-joined one-word
/// Issue types) while paths are unbounded, so Status is fixed and Path takes
/// all remaining width.
const STATUS_COLUMN_DIP: i32 = 220;

/// One Scope's tab.
pub struct ScopePage {
    pub panel: Panel,
    pub list: ListCtrl,
    /// The commands with a button, in Tab order — `Command::ALL` filtered by
    /// [`Command::button_label`], so the two can never disagree.
    buttons: Vec<(Command, Button)>,
    /// The one Catalogue, for the one thing this tab composes: the Status
    /// column. Held rather than passed in, because the first write happens in
    /// [`build`](ScopePage::build), before there is anyone to pass it.
    catalogue: Rc<Catalogue>,
}

impl ScopePage {
    /// Builds the tab over a Session's current Entries.
    ///
    /// The list is report mode with exactly two columns, Path and Status — no
    /// index column, no icons (spec §7, §10) — and `SingleSel`, which is the
    /// app's real shape: Delete, Move Up and Move Down act on one Entry.
    /// `ListCtrlStyle::EditLabels` is deliberately absent: editing is the
    /// modal dialog and nothing else (spec §6).
    pub fn build(notebook: &Notebook, catalogue: &Rc<Catalogue>, session: &Session) -> ScopePage {
        let panel = Panel::builder(notebook).build();
        let list = ListCtrl::builder(&panel)
            .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel)
            .build();
        let status_width = from_dip(&list, STATUS_COLUMN_DIP);
        // Path's width is never a constant: the fit below sets it on the initial
        // layout and on every resize, so it is inserted at zero.
        list.insert_column(
            0,
            &translate(msgids::COLUMN_PATH),
            ListColumnFormat::Left,
            0,
        );
        list.insert_column(
            1,
            &translate(msgids::COLUMN_STATUS),
            ListColumnFormat::Left,
            status_width,
        );

        let button_row = BoxSizer::builder(Orientation::Horizontal).build();
        let buttons: Vec<(Command, Button)> = Command::ALL
            .into_iter()
            .filter_map(|command| Some((command, command.button_label()?)))
            .map(|(command, label)| {
                let button = Button::builder(&panel).with_label(&label).build();
                button_row.add(&button, 0, SizerFlag::All, 4);
                (command, button)
            })
            .collect();

        let sizer = BoxSizer::builder(Orientation::Vertical).build();
        sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 4);
        sizer.add_sizer(&button_row, 0, SizerFlag::Expand | SizerFlag::All, 4);
        panel.set_sizer(sizer, true);

        // Path takes all remaining width (spec §12 D2). Lay the page out first so
        // the list's client size is current, then hand the event on. The zero floor
        // is unreachable at the 800×600 window minimum; it only guards degenerate
        // sizes during construction.
        panel.on_size(move |event| {
            panel.layout();
            let path_width = list.get_client_size().width - status_width;
            list.set_column_width(0, path_width.max(0));
            event.skip(true);
        });

        let page = ScopePage {
            panel,
            list,
            buttons,
            catalogue: Rc::clone(catalogue),
        };
        // No pass has run yet, so every Status column starts empty — which is
        // also what a healthy Scope looks like, and stays so for one Timer
        // tick (spec §7, FR-diag-async).
        page.render(session, None, None);
        page
    }

    /// Every button, for the caller that binds them to their command.
    pub fn buttons(&self) -> &[(Command, Button)] {
        &self.buttons
    }

    /// Redraws the list from the Working Copy and puts focus on `row`, clamped
    /// to the new last row — or, over an emptied Scope, on the list itself.
    /// `None` leaves focus exactly where it was: it means the operation had no
    /// row to point at *and* the user was on none, which is not an occasion to
    /// move them.
    ///
    /// The whole list is rebuilt rather than patched: one code path for every
    /// operation means the focus rules below are the only thing that decides
    /// where the user lands.
    pub fn render(&self, session: &Session, findings: Option<&Findings>, row: Option<usize>) {
        self.list.delete_all_items();
        for (index, entry) in session.entries().iter().enumerate() {
            self.list.insert_item(index as i64, entry.raw(), None);
        }
        self.render_status(session, findings);
        if let Some(row) = row {
            self.focus_row(row);
        }
    }

    /// Writes the Status column from a completed pass and touches nothing else.
    ///
    /// Separate from [`render`](Self::render) because a pass landing must not
    /// move the user: rebuilding the list would clear the selected-and-focused
    /// row, and a pass lands whenever it finishes, including in the middle of
    /// someone arrowing through the list. It is also how a System edit reaches
    /// the User tab, whose rows did not change but whose duplicates did.
    ///
    /// `None` is "no pass has run yet", which this column shows exactly as it
    /// shows a healthy Scope — as nothing. The distinction matters only to the
    /// StatusBar, which must not report a count nothing has measured.
    pub fn render_status(&self, session: &Session, findings: Option<&Findings>) {
        for (index, entry) in session.entries().iter().enumerate() {
            let issues = findings.map_or(&[][..], |findings| findings.issues(entry));
            self.list.set_item_text_by_column(
                index as i64,
                1,
                &self.catalogue.status_column(issues),
            );
        }
    }

    /// Puts one row into the selected-and-focused state, clamps it to the last
    /// row, scrolls it into view, and gives the list the keyboard focus so
    /// NVDA reads it.
    ///
    /// **The list takes the keyboard focus whether or not a row survives to
    /// land on**: an emptied Scope has nothing to speak but the list itself,
    /// which is where FR-refresh's "else the list" ends, and leaving focus on
    /// the button that emptied it would say nothing at all.
    pub fn focus_row(&self, row: usize) {
        self.list.set_focus();
        let Some(last) = self.last_row() else { return };
        let row = row.min(last) as i64;
        self.list.set_item_state(
            row,
            ListItemState::Selected | ListItemState::Focused,
            ListItemState::Selected | ListItemState::Focused,
        );
        self.list.ensure_visible(row);
    }

    /// Gives the list the keyboard focus without moving the row it is on —
    /// what "focus stays on the current Entry" means after an Apply (spec §10).
    ///
    /// It has to be said rather than assumed, because the command may have
    /// arrived from the Apply button, and Apply disables itself the moment it
    /// succeeds: focus left on a disabled button is focus nowhere, which for
    /// this application is the same as silence. A list nobody has reached yet
    /// takes the focus without a row being chosen for them.
    pub fn keep_focus(&self) {
        match self.focused_row() {
            Some(row) => self.focus_row(row),
            None => self.list.set_focus(),
        }
    }

    /// The index of the last row, or `None` over an empty list.
    ///
    /// The count is asked for as an `i32`, so the empty case has to be caught
    /// before the subtraction rather than after: `0 - 1` is `-1`, and `-1` is
    /// comctl32's index for *every* row.
    fn last_row(&self) -> Option<usize> {
        usize::try_from(self.list.get_item_count())
            .ok()?
            .checked_sub(1)
    }

    /// The row the user is on, if any: the focused one, or the selected one
    /// when the list has been clicked but never arrowed through.
    pub fn focused_row(&self) -> Option<usize> {
        let focused = self
            .list
            .get_next_item(-1, ListNextItemFlag::All, ListItemState::Focused);
        let row = if focused >= 0 {
            focused
        } else {
            self.list.get_first_selected_item()
        };
        usize::try_from(row).ok()
    }

    /// The Entry the user is on: its row and its id. `None` when the list is
    /// empty or nothing in it has been reached yet.
    pub fn focused_entry(&self, session: &Session) -> Option<(usize, EntryId)> {
        let row = self.focused_row()?;
        Some((row, session.entries().get(row)?.id()))
    }

    /// The row `id` now stands at.
    pub fn row_of(session: &Session, id: EntryId) -> Option<usize> {
        session.entries().iter().position(|entry| entry.id() == id)
    }

    /// Points every button at the active Session's state — the same `match`
    /// the menu items answer to, so a command cannot be dead in one place and
    /// live in the other.
    pub fn sync_buttons(&self, session: &Session) {
        for (command, button) in &self.buttons {
            button.enable(command.enabled(Some(session)));
        }
    }
}

/// The app's single explicit FromDIP conversion (spec §12 D4). wxdragon applies
/// FromDIP implicitly to sizes crossing the FFI boundary, but ListCtrl column
/// widths cross it raw, so the one hardcoded pixel value is scaled here against
/// the live DPI.
fn from_dip(widget: &ListCtrl, dip: i32) -> i32 {
    let dc = ClientDC::new(widget);
    let (ppi_x, _) = dc.get_ppi();
    if ppi_x > 0 {
        dip * ppi_x / 96
    } else {
        dip
    }
}
