//! One Scope tab: the Search field, the list of Entries it narrows, and the
//! buttons that edit them (spec §7, §12, §15; v0.2.0 §3).
//!
//! Everything about focus lives here, because focus is how this application
//! speaks. Delete has no Announcement and no confirmation; the commit of an
//! Add has none either — what the user hears is NVDA reading the row focus
//! landed on. So every operation that changes the list ends by putting a row
//! into the focused state **and giving the list the keyboard focus**: a row
//! that is focused in a control that is not is silent, which for this
//! application is the same as not having happened.

use pathmaster_core::catalogue::Catalogue;
use pathmaster_core::diagnostics::Findings;
use pathmaster_core::msgids;
use pathmaster_core::session::Session;
use wxdragon::prelude::*;
use wxdragon::timer::Timer;

use crate::catalog::translate;
use crate::ui::command::{Availability, Command};
use crate::ui::list;
use crate::ui::rendering::Rendering;

/// The `#` and Status column widths in DIP — this tab's two deliberate pixel
/// constants, scaled through [`list::from_dip`] like every other column width
/// in the application (spec §12 D2, D4; v0.2.0 §2.1). Both columns hold text of
/// a predictable length — a position, and comma-joined one-word Issue types —
/// while paths are unbounded, so both are fixed and Path takes all remaining
/// width.
///
/// `#` is sized for the four digits no real `PATH` reaches: the whole value is
/// capped at 32 767 UTF-16 units ([`thresholds::HARD_CAP`]), and one at that
/// length made of one-character Entries is not a machine anyone has. Past four
/// the cell would clip on screen and stay whole in speech, since a screen
/// reader reads the item's text and not its pixels.
///
/// [`thresholds::HARD_CAP`]: pathmaster_core::thresholds::HARD_CAP
const INDEX_COLUMN_DIP: i32 = 48;
const STATUS_COLUMN_DIP: i32 = 220;

/// One row as the list shows it: the `#` cell, the Path cell and the Status
/// cell, owned.
///
/// Owned is the point (ADR-0011): rows are composed under the Session's and
/// the findings' scoped access and rendered after both closures have died, so
/// a rebuild — which runs the list's own events — is never inside one.
pub struct Row {
    /// The Entry's 1-based position in the Working Copy (v0.2.0 §2.1). A
    /// number rather than the digits the cell shows: nothing about it is
    /// language, and the Fix Issues dialog carries the same value in its own
    /// `#` column (`fix::Row::position`).
    pub position: usize,
    /// The Entry's **displayed rendering** — its raw text, or the expanded
    /// reading of it under Expansion Mode (v0.2.0 §5). It is the same text
    /// Search matches, because both come through the one [`Rendering`].
    pub path: String,
    pub status: String,
}

impl Row {
    /// Composes every row of a Scope: each Entry's position, its rendering
    /// under the mode now in force, and the Status column the last completed
    /// pass gives it — nothing, until one has run (spec §7, FR-diag-async).
    ///
    /// The position is where the Entry stands **now**, counted off the Working
    /// Copy this call reads. Every operation that reorders or removes Entries
    /// ends in a rebuild through this function, so the column renumbers with
    /// the data and can never describe a list that has moved on.
    pub fn compose(
        session: &Session,
        findings: Option<&Findings>,
        catalogue: &Catalogue,
        rendering: &Rendering,
    ) -> Vec<Row> {
        let all: Vec<usize> = (0..session.entries().len()).collect();
        Self::compose_visible(session, findings, catalogue, rendering, &all)
    }

    /// [`compose`](Self::compose) narrowed to a Filtered View's visible set —
    /// `visible` is the Working-Copy indices the view shows, in order (v0.2.0
    /// §2). The `#` cell carries each Entry's **original** position, which is
    /// exactly what makes it worth carrying under any narrowing (§2.1).
    pub fn compose_visible(
        session: &Session,
        findings: Option<&Findings>,
        catalogue: &Catalogue,
        rendering: &Rendering,
        visible: &[usize],
    ) -> Vec<Row> {
        visible
            .iter()
            .map(|&index| {
                let entry = &session.entries()[index];
                Row {
                    position: index + 1,
                    path: rendering.render(entry.raw()).into_owned(),
                    status: catalogue
                        .status_column(findings.map_or(&[][..], |findings| findings.issues(entry))),
                }
            })
            .collect()
    }
}

/// One Scope's tab.
pub struct ScopePage {
    pub panel: Panel,
    /// The permanent Search field above the list (v0.2.0 §3): a native
    /// `TextCtrl`, never `SearchCtrl` — the generic composite on MSW is
    /// unmeasured with NVDA, while `TextCtrl` is the exact control ticket 04
    /// proved. Its label is constant and never carries the count.
    pub search: TextCtrl,
    /// The Search debounce: one-shot, restarted on every keystroke, owned by
    /// the field and not the Frame — wxdragon binds `on_tick` on the owner
    /// with no id filter, so a second Timer on the [`Pump`]'s Frame would fire
    /// the diagnostic drain's handler too (and vice versa).
    ///
    /// [`Pump`]: crate::pump::Pump
    pub debounce: Timer<TextCtrl>,
    pub list: ListCtrl,
    /// The commands with a button, in Tab order — `Command::ALL` filtered by
    /// [`Command::button_label`], so the two can never disagree.
    buttons: Vec<(Command, Button)>,
}

impl ScopePage {
    /// Builds the tab over a Scope's rows as they stand at startup.
    ///
    /// The list is report mode with exactly three columns, `#`, Path and
    /// Status, and no icons (spec §7, §10; v0.2.0 §2.1) — and `SingleSel`,
    /// which is the app's real shape: Delete, Move Up and Move Down act on one
    /// Entry. `ListCtrlStyle::EditLabels` is deliberately absent: editing is
    /// the modal dialog and nothing else (spec §6).
    ///
    /// The three columns are built once, here, and are never added or removed
    /// afterwards. §12's layout rule is that the window does not reflow under
    /// the user, so `#` is present before there is any narrowing to need it.
    pub fn build(notebook: &Notebook, rows: &[Row]) -> ScopePage {
        let panel = Panel::builder(notebook).build();
        // The Search field is created before the list because creation order
        // is the Tab order, and v0.2.0 §3 fixes it: tabs → search field →
        // list → buttons. The label is a `StaticText` — not a Tab stop — and
        // never carries a mnemonic or the count.
        let search_label = StaticText::builder(&panel)
            .with_label(&translate(msgids::SEARCH_LABEL))
            .build();
        let search = TextCtrl::builder(&panel).build();
        let list = ListCtrl::builder(&panel)
            .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel)
            .build();
        let index_width = list::from_dip(&list, INDEX_COLUMN_DIP);
        let status_width = list::from_dip(&list, STATUS_COLUMN_DIP);
        // `#` would read better right-aligned, and cannot be: comctl32 forces
        // LVCFMT_LEFT on the leftmost report column and silently ignores any
        // other format there. Left is what it will be either way, said out loud.
        list.insert_column(
            0,
            &translate(msgids::COLUMN_INDEX),
            ListColumnFormat::Left,
            index_width,
        );
        // Path's width is never a constant: the fit below sets it on the initial
        // layout and on every resize, so it is inserted at zero.
        list.insert_column(
            1,
            &translate(msgids::COLUMN_PATH),
            ListColumnFormat::Left,
            0,
        );
        list.insert_column(
            2,
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

        let search_row = BoxSizer::builder(Orientation::Horizontal).build();
        search_row.add(
            &search_label,
            0,
            SizerFlag::AlignCenterVertical | SizerFlag::All,
            4,
        );
        search_row.add(&search, 1, SizerFlag::Expand | SizerFlag::All, 4);

        let sizer = BoxSizer::builder(Orientation::Vertical).build();
        sizer.add_sizer(&search_row, 0, SizerFlag::Expand | SizerFlag::All, 0);
        sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 4);
        sizer.add_sizer(&button_row, 0, SizerFlag::Expand | SizerFlag::All, 4);
        panel.set_sizer(sizer, true);

        // Path takes all remaining width (spec §12 D2). Lay the page out first so
        // the list's client size is current, then hand the event on. The zero floor
        // is unreachable at the 800×600 window minimum; it only guards degenerate
        // sizes during construction.
        panel.on_size(move |event| {
            panel.layout();
            let path_width = list.get_client_size().width - index_width - status_width;
            list.set_column_width(1, path_width.max(0));
            event.skip(true);
        });

        let debounce = Timer::new(&search);
        let page = ScopePage {
            panel,
            search,
            debounce,
            list,
            buttons,
        };
        // No pass has run yet, so every Status column starts empty — which is
        // also what a healthy Scope looks like, and stays so for one Timer
        // tick (spec §7, FR-diag-async).
        page.render(rows, None);
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
    ///
    /// It takes composed [`Row`]s rather than a Session on purpose: a rebuild
    /// runs the list's own events, so it may not happen inside a scoped-access
    /// closure — the caller copies the rows out first (ADR-0011).
    pub fn render(&self, rows: &[Row], row: Option<usize>) {
        self.rebuild(rows);
        if let Some(row) = row {
            self.focus_row(row);
        }
    }

    /// [`render`](Self::render) without the keyboard focus: the row state is
    /// set but the list is not focused — the rebuild a Search keystroke asks
    /// for, where focus stays in the field and moving it would be the one
    /// uninvited jump v0.2.0 §2 forbids. Measured silent under NVDA exactly
    /// like this: rows rebuilt under the unfocused list, no chatter (ticket
    /// 04's verdict).
    pub fn render_quiet(&self, rows: &[Row], row: Option<usize>) {
        self.rebuild(rows);
        if let Some(row) = row {
            self.mark_row(row);
        }
    }

    /// The one rebuild: plain `DeleteAllItems` + reinsert, no Freeze/Thaw —
    /// it earned nothing under NVDA and is dropped (v0.2.0 §3).
    fn rebuild(&self, rows: &[Row]) {
        self.list.delete_all_items();
        for (index, data) in rows.iter().enumerate() {
            self.list
                .insert_item(index as i64, &data.position.to_string(), None);
            self.list
                .set_item_text_by_column(index as i64, 1, &data.path);
            self.list
                .set_item_text_by_column(index as i64, 2, &data.status);
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
    /// It writes the same [`Row`]s a rebuild takes and reads only their Status
    /// third — one composition path for the column, whichever way it reaches
    /// the screen. The `#` and Path cells it leaves alone are by construction
    /// already right: only a Working-Copy change can move an Entry or renumber
    /// one, and every such change goes through [`render`](Self::render).
    pub fn render_status(&self, rows: &[Row]) {
        for (index, data) in rows.iter().enumerate() {
            self.list
                .set_item_text_by_column(index as i64, 2, &data.status);
        }
    }

    /// Writes the Path column from the rendering now in force and touches
    /// nothing else — the Expansion toggle's redraw when the mode changed how
    /// the Entries read but not which of them the view shows (v0.2.0 §5).
    ///
    /// Separate from [`render`](Self::render) for
    /// [`render_status`](Self::render_status)'s reason, and **measured** for
    /// it: a rebuild has to re-mark the landing row, and NVDA re-reads a row
    /// marked in a list that holds the keyboard focus — so a rebuilt list
    /// would speak the row before the toggle's own message, where §5 says the
    /// toggle speaks its message and an arrow key re-reads the row. Writing
    /// the cells leaves every item state alone, which is silent.
    ///
    /// The `#` and Status cells it leaves alone are by construction already
    /// right: the mode is not an edit, so no Entry moved, was renumbered or
    /// was re-diagnosed.
    pub fn render_paths(&self, rows: &[Row]) {
        for (index, data) in rows.iter().enumerate() {
            self.list
                .set_item_text_by_column(index as i64, 1, &data.path);
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
        self.mark_row(row);
    }

    /// The row-state half of [`focus_row`](Self::focus_row): selects and
    /// focuses `row` (clamped) inside the list without giving the list the
    /// keyboard focus — which is what a rebuild under a focused Search field
    /// needs, so the landing row is ready to be read the moment the user
    /// arrows or Tabs in.
    fn mark_row(&self, row: usize) {
        let Some(last) = self.last_row() else { return };
        let row = row.min(last) as i64;
        self.list.set_item_state(
            row,
            ListItemState::Selected | ListItemState::Focused,
            ListItemState::Selected | ListItemState::Focused,
        );
        self.list.ensure_visible(row);
    }

    /// Whether the keyboard focus is stranded — held by a button the operation
    /// just turned off. `true` asks the caller for a [`focus_list`] rescue;
    /// `false` means focus stays exactly where it is.
    ///
    /// Both halves are spec §10. "After Apply — stays on the current Entry"
    /// means a Ctrl+S pressed from a list row leaves the user on that row —
    /// and "focus never jumps without a reason" means a Ctrl+S pressed from
    /// the Move Up button leaves them on Move Up. The one control that cannot
    /// keep the focus is the one the operation just turned off: Apply and
    /// Cancel Changes both disable themselves the moment the Session goes
    /// clean, and focus left on a disabled button is focus nowhere, which for
    /// this application is the same as silence.
    ///
    /// It is asked before the buttons are re-synced, so the answer comes from
    /// two live facts: which button wx says has the focus, and what the
    /// Session now says that button's command is worth. The question and the
    /// rescue are deliberately two calls: [`Availability`] lives inside the
    /// scoped access, while moving focus runs the toolkit's own events — a
    /// dispatch, which no closure body may make (ADR-0011).
    ///
    /// [`focus_list`]: Self::focus_list
    pub fn focus_stranded(&self, available: &Availability) -> bool {
        self.buttons
            .iter()
            .any(|(command, button)| button.has_focus() && !command.enabled(available))
    }

    /// Puts the keyboard focus into this tab's list, on the row the user was
    /// last on — or on the list itself, which is where an empty list, or one
    /// nothing has reached yet, leaves it.
    ///
    /// Focus without a row is still focus somewhere; a row without focus is
    /// silence, which is why the two answers are one function and not a choice
    /// at each call site.
    pub fn focus_list(&self) {
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

    /// The row the user is on, if any (see [`list::focused_row`]). Under a
    /// Filtered View this is a **view** row; mapping it onto an Entry is the
    /// tab's business, because only the tab holds the view's criteria.
    pub fn focused_row(&self) -> Option<usize> {
        list::focused_row(&self.list)
    }

    /// Points every button at the state that now holds — the same `match` the
    /// menu items answer to, so a command cannot be dead in one place and live
    /// in the other.
    ///
    /// `available` carries **this tab's** Session and not the active one: a
    /// Scope's buttons answer to the Scope they sit under, whichever tab the
    /// user is looking at.
    pub fn sync_buttons(&self, available: &Availability) {
        for (command, button) in &self.buttons {
            button.enable(command.enabled(available));
        }
    }
}
