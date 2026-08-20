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

mod command;
mod entry_dialog;
mod question;
mod scope_page;

use std::cell::RefCell;
use std::rc::Rc;

use pathmaster_core::msgids::{self, fill};
use pathmaster_core::normalize::has_variable_reference;
use pathmaster_core::session::{EntryId, Operation, Scope, Session, UndoOutcome, ValueType};
use pathmaster_platform::datadir::ReadOnlyReason;
use pathmaster_platform::registry::ScopeKey;
use wxdragon::prelude::*;

use crate::announce::Announcer;
use crate::catalog::{translate, translate_plural};
use crate::ui::command::Command;
use crate::ui::scope_page::ScopePage;

/// The notebook's page order (spec §12): the two Scopes, then Backups —
/// which is not a Scope, so activating it announces nothing and offers no
/// editing at all.
const TAB_INDEX_USER: i32 = 0;
const TAB_INDEX_SYSTEM: i32 = 1;

/// The window and everything an editing command needs to reach: the two
/// Sessions, the two tabs that show them, the menu whose enabled states
/// follow the active one, and the single voice.
///
/// It rides an `Rc` into every event handler, which is why every command
/// takes `&self`: the Sessions' interior mutability is the `RefCell`'s, and
/// no borrow is ever held across a modal dialog — a dialog runs its own event
/// loop, and a handler firing inside it would find the Session already
/// borrowed.
struct App {
    frame: Frame,
    notebook: Notebook,
    menu: MenuBar,
    announcer: Announcer,
    status: StatusBar,
    pages: [ScopePage; 2],
    sessions: [Rc<RefCell<Session>>; 2],
    readonly: Option<ReadOnlyReason>,
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

    let root = Panel::builder(&frame).build();

    // The Banner: always visible, fixed height, its StaticText empty at rest — the layout
    // never reflows under the user when announce() sets a message (spec §12 D1, §10).
    // get_char_height() and set_min_size are both physical pixels: SetMinSize is one of the
    // FFI calls wxdragon does NOT route through its implicit FromDIP, so no double scaling.
    let banner = StaticText::builder(&root).with_label("").build();
    banner.set_min_size(Size::new(-1, banner.get_char_height()));

    let notebook = Notebook::builder(&root).build();
    let pages = [
        ScopePage::build(&notebook, &user.borrow()),
        ScopePage::build(&notebook, &system.borrow()),
    ];
    // The Backups tab is not a Scope; its Snapshot list arrives with the backups ticket.
    let backups_page = Panel::builder(&notebook).build();
    notebook.add_page(&pages[0].panel, &translate(msgids::TAB_USER), true, None);
    notebook.add_page(&pages[1].panel, &translate(msgids::TAB_SYSTEM), false, None);
    notebook.add_page(&backups_page, &translate(msgids::TAB_BACKUPS), false, None);

    let root_sizer = BoxSizer::builder(Orientation::Vertical).build();
    root_sizer.add(&banner, 0, SizerFlag::Expand | SizerFlag::All, 4);
    root_sizer.add(&notebook, 1, SizerFlag::Expand | SizerFlag::All, 4);
    root.set_sizer(root_sizer, true);

    // Command-only (NVDA+End), absent from the Tab order: field 0 general status,
    // field 1 the passive merged-length field (spec §12 D10) — text arrives with
    // diagnostics. No field is ever styled: text carries everything.
    let status = frame.create_status_bar(2, 0, ID_ANY as Id, "");
    status.set_status_widths(&[-3, -2]);

    let app = Rc::new(App {
        frame,
        notebook,
        menu: frame.get_menu_bar().expect("the menu bar was just set"),
        announcer: Announcer::new(banner),
        status,
        pages,
        sessions: [user, system],
        readonly,
    });
    app.bind();
    app.sync();

    frame.centre();
    frame.show(true);

    // Announcement 7 (spec §10.1), once at startup: a Read-only Data run names
    // its reason. Fired after show so the Banner's window exists to speak from.
    if let Some(reason) = &app.readonly {
        app.announcer.announce(&readonly_text(reason));
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

        for page in &self.pages {
            // Enter and double-click on a row: the list's own gesture for
            // "open this", which is the Edit dialog (spec §6, FR-edit-f2).
            let app = Rc::clone(self);
            page.list.on_item_activated(move |_| app.run(Command::Edit));
            for (command, button) in page.buttons() {
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
            if let Some(scope) = scope_of(event.get_selection()) {
                let session = app.sessions[scope].borrow();
                app.announcer
                    .announce(&entry_count_text(session.scope(), session.entries().len()));
            }
            app.sync_for(scope_of(event.get_selection()));
            event.base.skip(true);
        });
    }

    /// The one door every command comes through.
    ///
    /// The availability check is not belt-and-braces: a menu accelerator can
    /// fire against an enabled state set before the last operation, and the
    /// answer must be the same one the menu is showing.
    fn run(&self, command: Command) {
        let Some(scope) = self.active_scope() else {
            return;
        };
        let available = command.enabled(Some(&self.sessions[scope].borrow()));
        if !available {
            return;
        }
        match command {
            Command::Add => self.add(scope),
            Command::Edit => self.edit(scope),
            Command::Delete => self.delete(scope),
            Command::MoveUp => self.move_entry(scope, true),
            Command::MoveDown => self.move_entry(scope, false),
            Command::Undo => self.undo_redo(scope, false),
            Command::Redo => self.undo_redo(scope, true),
            Command::Cancel => self.cancel(scope),
            Command::Refresh => self.refresh(scope),
        }
    }

    /// Add is dialog-first: the dialog opens empty, and OK appends at the end
    /// — the lowest search precedence, which is the safe place for a path the
    /// user has not ranked (spec §6, FR-add-delete). Abandoning it leaves no
    /// Entry, no Checkpoint and no Issue behind.
    fn add(&self, scope: usize) {
        let title = translate(msgids::DIALOG_ADD_ENTRY);
        let Some(text) = entry_dialog::ask_for_entry(&self.frame, &title, "") else {
            return;
        };
        let convert = self.convert_or_keep(scope, &text);
        let mut added = None;
        {
            let mut session = self.sessions[scope].borrow_mut();
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
        self.after_edit(scope, added.and_then(|id| self.row_of(scope, id)));
    }

    /// Edit opens the same dialog over the focused Entry's raw text. Focus
    /// lands back on the edited row whatever the outcome (spec §6 D7).
    fn edit(&self, scope: usize) {
        let focused = {
            let session = self.sessions[scope].borrow();
            self.pages[scope]
                .focused_entry(&session)
                .map(|(row, id)| (id, session.entries()[row].raw().to_string()))
        };
        let Some((id, raw)) = focused else { return };
        let title = translate(msgids::DIALOG_EDIT_ENTRY);
        let Some(text) = entry_dialog::ask_for_entry(&self.frame, &title, &raw) else {
            return;
        };
        let convert = self.convert_or_keep(scope, &text);
        {
            let mut session = self.sessions[scope].borrow_mut();
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
        self.after_edit(scope, self.row_of(scope, id));
    }

    /// Delete has no confirmation — undo is the safety net (spec §6 D4).
    /// Focus stays at the same index, clamped to the new last row, and the row
    /// NVDA reads there is the whole of the feedback.
    fn delete(&self, scope: usize) {
        let focused = {
            let session = self.sessions[scope].borrow();
            self.pages[scope].focused_entry(&session)
        };
        let Some((row, id)) = focused else { return };
        if !self.sessions[scope].borrow_mut().delete(id) {
            return;
        }
        self.after_edit(scope, Some(row));
    }

    /// One Move Up or Move Down, one Checkpoint. Moving the first Entry up is
    /// not an operation and changes nothing, including focus.
    fn move_entry(&self, scope: usize, up: bool) {
        let focused = {
            let session = self.sessions[scope].borrow();
            self.pages[scope].focused_entry(&session)
        };
        let Some((_, id)) = focused else { return };
        let moved = {
            let mut session = self.sessions[scope].borrow_mut();
            if up {
                session.move_up(id)
            } else {
                session.move_down(id)
            }
        };
        if !moved {
            return;
        }
        self.after_edit(scope, self.row_of(scope, id));
    }

    /// Undo and Redo restore a Checkpoint, move focus to the Entry it hints,
    /// and speak Announcement 4 — or 5, when the step re-dirtied a Session
    /// that had just been applied (spec §10.1). The operation name is the one
    /// thing focus cannot say.
    fn undo_redo(&self, scope: usize, redo: bool) {
        let previous_row = self.pages[scope].focused_row();
        let outcome = {
            let mut session = self.sessions[scope].borrow_mut();
            if redo {
                session.redo()
            } else {
                session.undo()
            }
        };
        let Some(outcome) = outcome else { return };
        let row = outcome
            .focus
            .and_then(|id| self.row_of(scope, id))
            .or(previous_row);
        self.after_edit(scope, row);
        self.announcer.announce(&undo_text(redo, outcome));
    }

    /// Cancel discards the Working Copy back to the Baseline. It is itself a
    /// Checkpoint, so Ctrl+Z restores the discarded work — which is why its
    /// confirmation says no more than "Discard changes?" (spec §5, FR-cancel).
    fn cancel(&self, scope: usize) {
        if !self.confirm(msgids::DIALOG_DISCARD_CHANGES) {
            return;
        }
        let previous_row = self.pages[scope].focused_row();
        if !self.sessions[scope].borrow_mut().cancel() {
            return;
        }
        self.after_edit(scope, previous_row);
        self.announcer
            .announce(&translate(msgids::CHANGES_DISCARDED));
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
    fn refresh(&self, scope: usize) {
        let dirty = self.sessions[scope].borrow().is_dirty();
        if dirty && !self.confirm(msgids::DIALOG_REFRESH_DISCARDS) {
            return;
        }
        let Ok(raw) = scope_key(scope).read() else {
            return;
        };
        let previous_row = self.pages[scope].focused_row();
        let focused = {
            let session = self.sessions[scope].borrow();
            self.pages[scope].focused_entry(&session).map(|(_, id)| id)
        };
        let landing = self.sessions[scope]
            .borrow_mut()
            .refresh(raw.decode(), focused);
        let row = landing
            .and_then(|id| self.row_of(scope, id))
            .or(previous_row);
        self.after_edit(scope, row);
        let session = self.sessions[scope].borrow();
        self.announcer
            .announce(&entry_count_text(session.scope(), session.entries().len()));
    }

    /// The `%VAR%`-into-`REG_SZ` question, asked between validation and the
    /// commit and only by a text that raises it (spec §6). `true` means the
    /// user chose to convert the Scope, which then commits with the edit as
    /// one Checkpoint. Both answers are legal and both are undoable.
    fn convert_or_keep(&self, scope: usize, text: &str) -> bool {
        let asks = {
            let session = self.sessions[scope].borrow();
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

    /// Redraws the Scope, lands focus, and points every control at the state
    /// that now holds — the tail of every operation, so no screen can show one
    /// Working Copy while a menu reads another.
    fn after_edit(&self, scope: usize, row: Option<usize>) {
        {
            let session = self.sessions[scope].borrow();
            self.pages[scope].render(&session, row);
        }
        self.sync();
    }

    /// The two Scope pages of the notebook; the Backups tab is not one.
    fn active_scope(&self) -> Option<usize> {
        scope_of(Some(self.notebook.selection()))
    }

    fn row_of(&self, scope: usize, id: EntryId) -> Option<usize> {
        ScopePage::row_of(&self.sessions[scope].borrow(), id)
    }

    fn sync(&self) {
        self.sync_for(self.active_scope());
    }

    /// Points the menu, the buttons and the status bar at `scope`'s Session.
    /// Taken as an argument rather than read back, because the notebook's own
    /// selection lags the page-changed event that carries it.
    fn sync_for(&self, scope: Option<usize>) {
        let active = scope.map(|index| self.sessions[index].borrow());
        command::sync_menu_bar(&self.menu, active.as_deref());
        drop(active);
        for index in [0, 1] {
            let session = self.sessions[index].borrow();
            self.pages[index].sync_buttons(&session);
        }
        let user = self.sessions[0].borrow();
        let system = self.sessions[1].borrow();
        self.status
            .set_status_text(&general_status(&user, &system, self.readonly.as_ref()), 0);
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

/// Which Session a notebook page belongs to, if it is a Scope at all.
fn scope_of(selection: Option<i32>) -> Option<usize> {
    match selection {
        Some(TAB_INDEX_USER) => Some(0),
        Some(TAB_INDEX_SYSTEM) => Some(1),
        _ => None,
    }
}

/// The registry value behind a Scope page. Built where it is used rather than
/// held: a `ScopeKey` is a key path and a value name, not a handle.
fn scope_key(scope: usize) -> ScopeKey {
    if scope == 0 {
        ScopeKey::user()
    } else {
        ScopeKey::system()
    }
}

/// Announcements 4 and 5 (spec §10.1): what was undone or redone, and — when
/// the step re-dirtied a Session that had just been applied — that there are
/// unsaved changes again. No path text: focus lands on the row and NVDA reads
/// it for free.
fn undo_text(redo: bool, outcome: UndoOutcome) -> String {
    let template = translate(if redo { msgids::REDONE } else { msgids::UNDONE });
    let operation = translate(outcome.operation.catalogue_msgid());
    let mut text = fill(&template, &[("operation", &operation)]);
    if outcome.crossed_apply {
        text.push_str(&translate(msgids::UNSAVED_CHANGES_SUFFIX));
    }
    text
}

/// Announcement 1's text: the Scope's entry count, with the zero case as its
/// own msgid — "no entries" is better speech than "0", and Ukrainian's three
/// plural forms have no zero form to give it (spec §10.1 item 1).
fn entry_count_text(scope: Scope, count: usize) -> String {
    let (none, singular, plural) = match scope {
        Scope::User => (
            msgids::ENTRIES_USER_NONE,
            msgids::ENTRIES_USER,
            msgids::ENTRIES_USER_PLURAL,
        ),
        Scope::System => (
            msgids::ENTRIES_SYSTEM_NONE,
            msgids::ENTRIES_SYSTEM,
            msgids::ENTRIES_SYSTEM_PLURAL,
        ),
    };
    if count == 0 {
        translate(none)
    } else {
        fill(
            &translate_plural(singular, plural, count as u32),
            &[("n", &count.to_string())],
        )
    }
}

/// Announcement 7's text, which is also StatusBar field 0 in Read-only Data:
/// the mode and its reason, both halves Catalogue text (spec §10.1 item 7).
fn readonly_text(reason: &ReadOnlyReason) -> String {
    fill(
        &translate(msgids::READONLY),
        &[("reason", &translate(reason.catalogue_msgid()))],
    )
}

/// StatusBar field 0, the general status (spec §12): the two entry counts —
/// issue counts join with the diagnostics ticket — or, in Read-only Data, the
/// mode and its reason in their place.
fn general_status(user: &Session, system: &Session, readonly: Option<&ReadOnlyReason>) -> String {
    match readonly {
        Some(reason) => readonly_text(reason),
        None => format!(
            "{} | {}",
            entry_count_text(user.scope(), user.entries().len()),
            entry_count_text(system.scope(), system.entries().len()),
        ),
    }
}
