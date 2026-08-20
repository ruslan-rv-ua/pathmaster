//! The main window shell (spec §12): one vertical sizer — Banner above the notebook —
//! with the native status bar attached to the frame outside the sizer, and the menu
//! bar on the frame itself.
//!
//! The tab order is the whole map: tabs → list → buttons, full traversal, no traps.
//! `announce()` speaks without touching focus; the status bar is command-only
//! (`NVDA+End`), absent from the Tab order.
//!
//! Every editing command in the application arrives here, from a menu item, its
//! accelerator, a button, or the list's own activation gesture, and leaves through
//! one `run` — so "what is available" and "what happens" are each answered once.
//!
//! Diagnostics ride along behind that one door: every command that changes a
//! Working Copy ends in `after_edit`, which asks the [`Pump`] for a pass over
//! both Scopes (spec §7, FR-diag-async).

mod command;
mod entry_dialog;
mod question;
mod scope_page;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use pathmaster_core::catalogue::{Announcement, Catalogue, ScopeCounts, UndoDirection};
use pathmaster_core::diagnostics::{Diagnosis, Findings};
use pathmaster_core::msgids;
use pathmaster_core::normalize::has_variable_reference;
use pathmaster_core::session::{EntryId, Operation, Scope, Session, ValueType};
use pathmaster_platform::datadir::ReadOnlyReason;
use pathmaster_platform::registry::ScopeKey;
use wxdragon::prelude::*;

use crate::announce::Announcer;
use crate::catalog::{self, translate};
use crate::pump::Pump;
use crate::ui::command::Command;
use crate::ui::scope_page::ScopePage;

/// The notebook's page order (spec §12): the two Scopes, then Backups —
/// which is not a Scope, so activating it announces nothing and offers no
/// editing at all.
const TAB_INDEX_USER: i32 = 0;
const TAB_INDEX_SYSTEM: i32 = 1;

/// One Scope, everything the application holds of it: which Scope it is, the
/// Session being edited, and the tab showing it. They travel together through
/// every command, so they are one thing rather than three arrays indexed alike.
struct ScopeTab {
    scope: Scope,
    session: Rc<RefCell<Session>>,
    page: ScopePage,
    /// What the last completed pass found here — the Status column's whole
    /// content, and the issue count StatusBar field 0 reports. A derived view,
    /// held beside the Session and never inside it (ADR-0001).
    ///
    /// `None` until the first pass lands, and that is not the same as a pass
    /// that found nothing: the column reads both as empty, but the StatusBar
    /// must not claim "0 issues" about a Scope no pass has yet looked at.
    findings: RefCell<Option<Findings>>,
}

impl ScopeTab {
    /// This Scope's Working Copy as the worker takes it: the Entries' raw text,
    /// in list order. Cloned because the pass runs on another thread and a
    /// Session is `Rc<RefCell<…>>` — the pass diagnoses the state it was handed,
    /// not whatever the user has reached by the time it finishes.
    fn raw_entries(&self) -> Vec<String> {
        self.session
            .borrow()
            .entries()
            .iter()
            .map(|entry| entry.raw().to_string())
            .collect()
    }

    /// The two numbers StatusBar field 0 reports about this Scope: how many
    /// Entries it holds now, and how many findings the last pass made here —
    /// `None` until one has run (see [`ScopeCounts`]).
    fn counts(&self) -> ScopeCounts {
        ScopeCounts {
            scope: self.scope,
            entries: self.session.borrow().entries().len(),
            issues: self.findings.borrow().as_ref().map(Findings::issue_count),
        }
    }

    /// The registry value behind this tab. Built where it is used rather than
    /// held: a `ScopeKey` is a key path and a value name, not a handle.
    fn key(&self) -> ScopeKey {
        match self.scope {
            Scope::User => ScopeKey::user(),
            Scope::System => ScopeKey::system(),
        }
    }

    /// The Entry the user is on: its row and its id.
    ///
    /// The borrow is taken and dropped inside this one function on purpose.
    /// Every command asks this question immediately before opening a modal
    /// dialog, and a dialog runs its own event loop — a handler firing inside
    /// it would find the Session still borrowed and panic. Having one place to
    /// ask is what keeps that rule from being four places to remember.
    fn focused_entry(&self) -> Option<(usize, EntryId)> {
        self.page.focused_entry(&self.session.borrow())
    }

    /// The row `id` now stands at.
    fn row_of(&self, id: EntryId) -> Option<usize> {
        ScopePage::row_of(&self.session.borrow(), id)
    }
}

/// The window and everything an editing command needs to reach: the two Scope
/// tabs, the menu whose enabled states follow the active one, and the single
/// voice.
///
/// It rides an `Rc` into every event handler, which is why every command takes
/// `&self`: the Sessions' interior mutability is the `RefCell`'s, and **no
/// borrow is ever held across a call that can run someone else's code**.
///
/// Two kinds of call can. A modal dialog runs its own event loop, and
/// `ScopePage::render` fires the list's own events synchronously — of which
/// only `on_item_activated` is bound, so `render` cannot re-enter. The second
/// arrived with diagnostics and is the sharper one: **the Timer ticks inside a
/// modal dialog's loop too**, so `collect_pass` can run — taking the Pump's
/// borrow, the Sessions' and the findings' — while `ask_for_entry` or
/// `question::ask` is open. Every dialog call site was checked against that
/// and holds no borrow across it: `focused_entry` scopes its own, `edit` reads
/// the raw text through a temporary that dies with its statement, and
/// `convert_or_keep` drops the Session before it asks. What a pass landing
/// under an open dialog does is write the Status column and re-sync controls
/// the dialog has disabled anyway — invisible, and correct once it closes.
struct App {
    frame: Frame,
    notebook: Notebook,
    menu: MenuBar,
    /// The one Catalogue (ADR-0009): every string this window composes is
    /// composed here, and the Announcer holds the same one.
    catalogue: Rc<Catalogue>,
    announcer: Announcer,
    status: StatusBar,
    tabs: [ScopeTab; 2],
    readonly: Option<ReadOnlyReason>,
    /// The diagnostic pass: the worker thread and the Timer that drains it
    /// (spec §17, `pump`). Never called into off the UI thread, because
    /// widgets may only be touched from it.
    pump: Pump,
    /// The merged length the last pass measured — StatusBar field 1, which
    /// keeps showing it until the next pass replaces it. `None` only before
    /// the first pass has landed.
    merged_length: Cell<Option<usize>>,
}

/// Builds and shows the main window over the two loaded Sessions, and hands it
/// back so a startup dialog has a parent to sit on and a window to hand focus
/// back to. A Read-only Data run passes its reason; announcing it is the last
/// step of startup (spec §11: … → UI → writability → announce).
pub fn build_main_window(
    user: Rc<RefCell<Session>>,
    system: Rc<RefCell<Session>>,
    readonly: Option<ReadOnlyReason>,
) -> Frame {
    let frame = Frame::builder()
        .with_title("PathMaster")
        // Crosses the FFI boundary through the implicit FromDIP → 900×650 DIP (spec §12 D2).
        .with_size(Size::new(900, 650))
        .build();
    frame.set_min_size(Size::new(800, 600));
    frame.set_menu_bar(command::build_menu_bar());

    // The one Catalogue, built here and shared by everything that composes a
    // string out of it: this window, the Announcer, and each Scope tab's
    // Status column (ADR-0009). `install` has already given wx its own, which
    // is what `catalog::Installed` asks.
    let catalogue = Rc::new(Catalogue::new(catalog::Installed));

    let root = Panel::builder(&frame).build();

    // The Banner: always visible, fixed height, its StaticText empty at rest — the layout
    // never reflows under the user when announce() sets a message (spec §12 D1, §10).
    // get_char_height() and set_min_size are both physical pixels: SetMinSize is one of the
    // FFI calls wxdragon does NOT route through its implicit FromDIP, so no double scaling.
    let banner = StaticText::builder(&root).with_label("").build();
    banner.set_min_size(Size::new(-1, banner.get_char_height()));

    let notebook = Notebook::builder(&root).build();
    let user_page = ScopePage::build(&notebook, &catalogue, &user.borrow());
    let system_page = ScopePage::build(&notebook, &catalogue, &system.borrow());
    let tabs = [
        ScopeTab {
            scope: Scope::User,
            session: user,
            page: user_page,
            findings: RefCell::new(None),
        },
        ScopeTab {
            scope: Scope::System,
            session: system,
            page: system_page,
            findings: RefCell::new(None),
        },
    ];
    // The Backups tab is not a Scope; its Snapshot list arrives with the backups ticket.
    let backups_page = Panel::builder(&notebook).build();
    notebook.add_page(
        &tabs[0].page.panel,
        &translate(msgids::TAB_USER),
        true,
        None,
    );
    notebook.add_page(
        &tabs[1].page.panel,
        &translate(msgids::TAB_SYSTEM),
        false,
        None,
    );
    notebook.add_page(&backups_page, &translate(msgids::TAB_BACKUPS), false, None);

    let root_sizer = BoxSizer::builder(Orientation::Vertical).build();
    root_sizer.add(&banner, 0, SizerFlag::Expand | SizerFlag::All, 4);
    root_sizer.add(&notebook, 1, SizerFlag::Expand | SizerFlag::All, 4);
    root.set_sizer(root_sizer, true);

    // Command-only (NVDA+End), absent from the Tab order: field 0 general status,
    // field 1 the passive merged-length field (spec §12 D10). No field is ever
    // styled: text carries everything.
    let status = frame.create_status_bar(2, 0, ID_ANY as Id, "");
    status.set_status_widths(&[-3, -2]);

    let app = Rc::new(App {
        frame,
        notebook,
        menu: frame.get_menu_bar().expect("the menu bar was just set"),
        announcer: Announcer::new(banner, Rc::clone(&catalogue)),
        catalogue,
        status,
        tabs,
        readonly,
        pump: Pump::new(&frame),
        merged_length: Cell::new(None),
    });
    app.bind();
    app.sync();

    frame.centre();
    frame.show(true);

    // The pass at load (spec §7, FR-diag-async). Asked for after show, so the
    // first results land in a window that exists to receive them; until they
    // do, every Status column is empty and StatusBar field 1 is blank — one
    // Timer tick, in a window no keystroke has reached yet.
    app.request_pass();

    // Announcement 7 (spec §10.1), once at startup: a Read-only Data run names
    // its reason. Fired after show so the Banner's window exists to speak from.
    if let Some(reason) = &app.readonly {
        app.announcer.announce(Announcement::ReadOnly {
            reason: reason.catalogue_msgid(),
        });
    }

    frame
}

impl App {
    /// Every route a command can arrive by. Each handler owns its own `Rc`,
    /// which is what keeps the App alive for as long as the widgets are.
    fn bind(self: &Rc<Self>) {
        // Menu items and their accelerators, which in wxdragon are the same
        // thing: `wxAcceleratorTable` is not bound at any level, so a menu
        // item's label is the only place a shortcut can live.
        let app = Rc::clone(self);
        self.frame.on_menu(move |event| {
            if let Some(command) = Command::from_id(event.get_id()) {
                app.run(command);
            }
        });

        for tab in &self.tabs {
            // Enter and double-click on a row: the list's own gesture for
            // "open this", which is the Edit dialog (spec §6, FR-edit-f2).
            let app = Rc::clone(self);
            tab.page
                .list
                .on_item_activated(move |_| app.run(Command::Edit));
            for (command, button) in tab.page.buttons() {
                let app = Rc::clone(self);
                let command = *command;
                button.on_click(move |_| app.run(command));
            }
        }

        // Announcement 1 (spec §10.1): activating a Scope tab speaks its entry
        // count. The count is read at activation time, not captured — Refresh
        // and editing change it under the same handler.
        let app = Rc::clone(self);
        self.notebook.on_page_changed(move |event| {
            // The selection the event carries, not the notebook's: on Windows
            // the widget has not caught up when this fires.
            let active = app.tab_at(event.get_selection());
            if let Some(tab) = active {
                let count = tab.session.borrow().entries().len();
                app.announcer.announce(Announcement::EntryCount {
                    scope: tab.scope,
                    count,
                });
            }
            app.sync_for(active);
            event.base.skip(true);
        });

        // The one place a finished pass crosses onto the UI thread (spec §7,
        // FR-diag-async).
        let app = Rc::clone(self);
        self.pump.on_tick(move |_| app.collect_pass());
    }

    /// Asks for a pass over both Working Copies.
    ///
    /// One pass covers both Scopes because they are diagnosed together: a
    /// System edit changes what a User Entry is a duplicate of, so there is no
    /// such thing as re-diagnosing one Scope alone (spec §7, FR-diag-duplicate).
    fn request_pass(&self) {
        // Named rather than indexed: System goes first because that is the
        // order Windows merges the two Scopes in, and reversing the pair would
        // silently move every cross-scope duplicate flag onto the other Scope.
        let system = self.tab_of(Scope::System).raw_entries();
        let user = self.tab_of(Scope::User).raw_entries();
        self.pump.request(system, user);
    }

    /// The Timer's tick: put a finished pass on screen if one has landed.
    fn collect_pass(&self) {
        if let Some(diagnosis) = self.pump.take() {
            self.apply_pass(&diagnosis);
        }
    }

    /// Puts a completed pass on screen: both Status columns, then both
    /// StatusBar fields.
    ///
    /// The lists are **not** rebuilt — only their Status column is written —
    /// because a pass lands on its own schedule, and rebuilding would clear the
    /// focused row out from under whoever is arrowing through it. Both tabs are
    /// written, not just the active one: the tab the user is not looking at was
    /// diagnosed by the same pass.
    fn apply_pass(&self, diagnosis: &Diagnosis) {
        for tab in &self.tabs {
            let session = tab.session.borrow();
            let findings = Findings::of(session.entries(), diagnosis.scope(tab.scope));
            tab.page.render_status(&session, Some(&findings));
            *tab.findings.borrow_mut() = Some(findings);
        }
        self.merged_length.set(Some(diagnosis.merged_length()));
        self.sync();
    }

    /// The one door every command comes through.
    ///
    /// The availability check is not belt-and-braces: a menu accelerator can
    /// fire against an enabled state set before the last operation, and the
    /// answer must be the same one the menu is showing.
    fn run(&self, command: Command) {
        let Some(tab) = self.active_tab() else { return };
        let available = command.enabled(Some(&tab.session.borrow()));
        if !available {
            return;
        }
        match command {
            Command::Add => self.add(tab),
            Command::Edit => self.edit(tab),
            Command::Delete => self.delete(tab),
            // The direction is the command, so it travels as the command: a
            // bare `true` at this call site would say nothing.
            Command::MoveUp | Command::MoveDown => self.move_entry(tab, command),
            Command::Undo | Command::Redo => self.undo_redo(tab, command),
            Command::Cancel => self.cancel(tab),
            Command::Refresh => self.refresh(tab),
        }
    }

    /// Add is dialog-first: the dialog opens empty, and OK appends at the end
    /// — the lowest search precedence, which is the safe place for a path the
    /// user has not ranked (spec §6, FR-add-delete). Abandoning it leaves no
    /// Entry, no Checkpoint and no Issue behind.
    fn add(&self, tab: &ScopeTab) {
        let title = translate(msgids::DIALOG_ADD_ENTRY);
        let Some(text) = entry_dialog::ask_for_entry(&self.frame, &self.catalogue, &title, "")
        else {
            return;
        };
        let convert = self.convert_or_keep(tab, &text);
        let mut added = None;
        {
            let mut session = tab.session.borrow_mut();
            if convert {
                session.batch(Operation::ChangeValueType, |working| {
                    working.set_value_type(ValueType::RegExpandSz);
                    added = working.add(&text);
                    added
                });
            } else {
                added = session.add(&text);
            }
        }
        self.after_edit(tab, added.and_then(|id| tab.row_of(id)));
    }

    /// Edit opens the same dialog over the focused Entry's raw text. Focus
    /// lands back on the edited row whatever the outcome (spec §6 D7).
    fn edit(&self, tab: &ScopeTab) {
        let focused = tab.focused_entry();
        let Some((row, id)) = focused else { return };
        let raw = tab.session.borrow().entries()[row].raw().to_string();
        let title = translate(msgids::DIALOG_EDIT_ENTRY);
        let Some(text) = entry_dialog::ask_for_entry(&self.frame, &self.catalogue, &title, &raw)
        else {
            return;
        };
        let convert = self.convert_or_keep(tab, &text);
        {
            let mut session = tab.session.borrow_mut();
            if convert {
                session.batch(Operation::ChangeValueType, |working| {
                    working.set_value_type(ValueType::RegExpandSz);
                    working.edit(id, &text);
                    Some(id)
                });
            } else {
                session.edit(id, &text);
            }
        }
        self.after_edit(tab, tab.row_of(id));
    }

    /// Delete has no confirmation — undo is the safety net (spec §6 D4).
    /// Focus stays at the same index, clamped to the new last row, and the row
    /// NVDA reads there is the whole of the feedback.
    fn delete(&self, tab: &ScopeTab) {
        let Some((row, id)) = tab.focused_entry() else {
            return;
        };
        if !tab.session.borrow_mut().delete(id) {
            return;
        }
        self.after_edit(tab, Some(row));
    }

    /// One Move Up or Move Down, one Checkpoint. Moving the first Entry up is
    /// not an operation and changes nothing, including focus.
    fn move_entry(&self, tab: &ScopeTab, command: Command) {
        let Some((_, id)) = tab.focused_entry() else {
            return;
        };
        let moved = {
            let mut session = tab.session.borrow_mut();
            match command {
                Command::MoveUp => session.move_up(id),
                _ => session.move_down(id),
            }
        };
        if !moved {
            return;
        }
        self.after_edit(tab, tab.row_of(id));
    }

    /// Undo and Redo restore a Checkpoint, move focus to the Entry it hints,
    /// and speak Announcement 4 — or 5, when the step took the Working Copy
    /// back across an Apply (spec §10.1). The operation name is the one thing
    /// focus cannot say.
    fn undo_redo(&self, tab: &ScopeTab, command: Command) {
        let previous_row = tab.page.focused_row();
        // One match, because the command decides two things that must not be
        // allowed to disagree: which way the history is walked, and which of
        // Announcement 4's two sentences says so.
        let (direction, outcome) = {
            let mut session = tab.session.borrow_mut();
            match command {
                Command::Redo => (UndoDirection::Redo, session.redo()),
                _ => (UndoDirection::Undo, session.undo()),
            }
        };
        let Some(outcome) = outcome else { return };
        let row = outcome.focus.and_then(|id| tab.row_of(id)).or(previous_row);
        self.after_edit(tab, row);
        self.announcer
            .announce(Announcement::UndoRedo { direction, outcome });
    }

    /// Cancel discards the Working Copy back to the Baseline. It is itself a
    /// Checkpoint, so Ctrl+Z restores the discarded work — which is why its
    /// confirmation says no more than "Discard changes?" (spec §5, FR-cancel).
    fn cancel(&self, tab: &ScopeTab) {
        if !self.confirm(msgids::DIALOG_DISCARD_CHANGES) {
            return;
        }
        let previous_row = tab.page.focused_row();
        if !tab.session.borrow_mut().cancel() {
            return;
        }
        self.after_edit(tab, previous_row);
        self.announcer.announce(Announcement::ChangesDiscarded);
    }

    /// Refresh re-reads the active Scope alone and clears its Undo/Redo stacks
    /// — so unlike Cancel it cannot be taken back, which its confirmation says
    /// (spec §5, FR-refresh). Focus keeps the Entry with the same id, else its
    /// nearest neighbour, else the list.
    ///
    /// A re-read that fails leaves the Session exactly as it was: an
    /// unreadable value is not an Absent one, and blanking a Scope over a
    /// transient failure would be the one unrecoverable thing this screen can
    /// do. The Announcement catalogue is closed at seven, so nothing is
    /// spoken; the §9 taxonomy that will name it arrives with Apply.
    fn refresh(&self, tab: &ScopeTab) {
        let dirty = tab.session.borrow().is_dirty();
        if dirty && !self.confirm(msgids::DIALOG_REFRESH_DISCARDS) {
            return;
        }
        let Ok(raw) = tab.key().read() else { return };
        let previous_row = tab.page.focused_row();
        let focused = tab.focused_entry().map(|(_, id)| id);
        let landing = tab.session.borrow_mut().refresh(raw.decode(), focused);
        let row = landing.and_then(|id| tab.row_of(id)).or(previous_row);
        self.after_edit(tab, row);
        let count = tab.session.borrow().entries().len();
        self.announcer.announce(Announcement::EntryCount {
            scope: tab.scope,
            count,
        });
    }

    /// The `%VAR%`-into-`REG_SZ` question, asked between validation and the
    /// commit and only by a text that raises it (spec §6). `true` means the
    /// user chose to convert the Scope, which then commits with the edit as
    /// one Checkpoint. Both answers are legal and both are undoable — the
    /// negative button is the one that leaves the Value Type alone, which is
    /// the only half of the outcome it can spare.
    fn convert_or_keep(&self, tab: &ScopeTab, text: &str) -> bool {
        let asks = {
            let session = tab.session.borrow();
            session.value_type() == ValueType::RegSz && has_variable_reference(text)
        };
        asks && question::ask(
            &self.frame,
            &translate(msgids::DIALOG_VAR_IN_REG_SZ),
            &translate(msgids::BUTTON_CHANGE_VALUE_TYPE),
            &translate(msgids::BUTTON_KEEP_LITERAL),
        )
    }

    /// A [Yes] [No] confirmation whose whole meaning is its title.
    fn confirm(&self, title_msgid: &str) -> bool {
        question::ask(
            &self.frame,
            &translate(title_msgid),
            &translate(msgids::BUTTON_YES),
            &translate(msgids::BUTTON_NO),
        )
    }

    /// Redraws the Scope, lands focus, points every control at the state that
    /// now holds, and asks for a fresh pass — the tail of every operation, so
    /// no screen can show one Working Copy while a menu reads another.
    ///
    /// The redraw carries the *last* pass's findings, read by Entry id: a row
    /// that only moved keeps its Status words, and the one whose text just
    /// changed shows none until the new pass lands (spec §7, FR-diag-async).
    fn after_edit(&self, tab: &ScopeTab, row: Option<usize>) {
        {
            let session = tab.session.borrow();
            tab.page
                .render(&session, tab.findings.borrow().as_ref(), row);
        }
        self.sync();
        self.request_pass();
    }

    /// The Scope tab the notebook is showing; the Backups tab is not one.
    fn active_tab(&self) -> Option<&ScopeTab> {
        self.tab_at(Some(self.notebook.selection()))
    }

    /// The tab showing `scope`. Both Scopes have exactly one tab each, which
    /// is what makes this total — and what lets the callers that care about
    /// *which* Scope they mean say so by name rather than by array position.
    fn tab_of(&self, scope: Scope) -> &ScopeTab {
        self.tabs
            .iter()
            .find(|tab| tab.scope == scope)
            .expect("every Scope has a tab")
    }

    /// The Scope tab at a notebook page index, if that page is a Scope at all.
    fn tab_at(&self, selection: Option<i32>) -> Option<&ScopeTab> {
        match selection {
            Some(TAB_INDEX_USER) => self.tabs.first(),
            Some(TAB_INDEX_SYSTEM) => self.tabs.get(1),
            _ => None,
        }
    }

    fn sync(&self) {
        self.sync_for(self.active_tab());
    }

    /// Points the menu, the buttons and the status bar at `active`'s Session.
    /// Taken as an argument rather than read back, because the notebook's own
    /// selection lags the page-changed event that carries it.
    fn sync_for(&self, active: Option<&ScopeTab>) {
        let session = active.map(|tab| tab.session.borrow());
        command::sync_menu_bar(&self.menu, session.as_deref());
        drop(session);
        for tab in &self.tabs {
            tab.page.sync_buttons(&tab.session.borrow());
        }
        // User first, then System: the order the tabs are in, not the runtime
        // order a pass evaluates them in (spec §12).
        self.status.set_status_text(
            &self.catalogue.general_status(
                [
                    self.tab_of(Scope::User).counts(),
                    self.tab_of(Scope::System).counts(),
                ],
                self.readonly.as_ref().map(ReadOnlyReason::catalogue_msgid),
            ),
            0,
        );
        self.status
            .set_status_text(&self.catalogue.merged_length(self.merged_length.get()), 1);
    }
}

/// The one startup dialog `settings.json` can earn: it could not be read, so
/// this run is on defaults (spec §13).
///
/// Shown after the main window rather than before it, so that dismissing it
/// leaves focus in the window the user came for.
pub fn show_settings_unreadable(parent: &Frame) {
    question::tell(parent, &translate(msgids::DIALOG_SETTINGS_UNREADABLE));
}
