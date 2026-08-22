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

mod backups_page;
mod command;
mod entry_dialog;
mod list;
mod question;
mod scope_page;
mod settings_dialog;

use std::cell::{Cell, RefCell};
use std::path::Path;
use std::rc::Rc;

use pathmaster_core::catalogue::{Announcement, Catalogue, ScopeCounts, UndoDirection};
use pathmaster_core::diagnostics::{Diagnosis, Findings};
use pathmaster_core::logfmt::{FailureCause, Record};
use pathmaster_core::msgids;
use pathmaster_core::normalize::has_variable_reference;
use pathmaster_core::session::{EntryId, Operation, Scope, Session, ValueType};
use pathmaster_core::settings::SettingsFile;
use pathmaster_platform::apply::{self, ApplyRun, Ask, ExternalChange, ScopeInput, ScopeOutcome};
use pathmaster_platform::datadir::ReadOnlyReason;
use pathmaster_platform::diagnostics::ProcessEnvironment;
use pathmaster_platform::elevation::{self, RelaunchFailure, StartTab};
use pathmaster_platform::geometry::{self, Placement};
use pathmaster_platform::logwriter;
use pathmaster_platform::registry::{RawValue, ScopeKey};
// The file in the Data Directory. `pathmaster_core::settings` above contributes
// the type it holds; this contributes the atomic replace that rewrites it.
use pathmaster_platform::settings;
use pathmaster_platform::snapshots;
use pathmaster_platform::startup::Run;
use wxdragon::prelude::*;

use crate::announce::Announcer;
use crate::catalog::{self, translate};
use crate::pump::Pump;
use crate::ui::backups_page::BackupsPage;
use crate::ui::command::{Availability, Command};
use crate::ui::scope_page::ScopePage;
use crate::SharedScope;

/// The notebook's page order (spec §12): the two Scopes, then Backups —
/// which is not a Scope, so activating it announces nothing and offers no
/// editing at all.
const TAB_INDEX_USER: i32 = 0;
const TAB_INDEX_SYSTEM: i32 = 1;
const TAB_INDEX_BACKUPS: i32 = 2;

/// The page a Scope's tab sits at. The one place the order above is read as a
/// mapping, so activating a Scope's tab and finding the tab a page belongs to
/// cannot disagree.
fn tab_index(scope: Scope) -> i32 {
    match scope {
        Scope::User => TAB_INDEX_USER,
        Scope::System => TAB_INDEX_SYSTEM,
    }
}

/// The page a [`StartTab`] asks for — the boundary-crossing cousin of
/// [`tab_index`], kept beside it so the two readings of the page order cannot
/// drift apart (spec §9, ticket 12 D5).
fn start_tab_index(tab: StartTab) -> i32 {
    match tab {
        StartTab::User => TAB_INDEX_USER,
        StartTab::System => TAB_INDEX_SYSTEM,
        StartTab::Backups => TAB_INDEX_BACKUPS,
    }
}

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
    /// What this Scope's registry value was the last time it was read.
    ///
    /// The comparison subject external-change detection needs (spec §4): it is
    /// `(vtype, bytes)` because decoding stops at the first NUL, so a decoded
    /// copy would miss a real change. It cannot live in the `Session` —
    /// `RawValue` is a `pathmaster-platform` type and core may not reach it —
    /// so the window holds it, and three paths keep it current: startup,
    /// Refresh, and whatever an Apply Run hands back (ADR-0008).
    last_read: RefCell<RawValue>,
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

    /// Takes a freshly read value as the truth: Working Copy and Baseline both
    /// become it, the Undo/Redo stacks clear, and it becomes what the next
    /// external-change comparison is made against.
    ///
    /// Both readers of a fresh value do exactly this — F5, and the
    /// external-change dialog's middle answer — and none of it may come apart:
    /// a Session refreshed from a value the tab did not keep would report an
    /// external change at the very next Apply, and a focus rule applied at one
    /// of the two call sites and not the other would be a rule only half kept.
    ///
    /// Answers the **row** focus lands on: the Entry with the same id if it
    /// survived the re-read, else its nearest neighbour by index, else wherever
    /// the user already was (spec §5, FR-refresh).
    fn adopt(&self, raw: RawValue) -> Option<usize> {
        let previous_row = self.page.focused_row();
        let focus = self.focused_entry().map(|(_, id)| id);
        let landing = self.session.borrow_mut().refresh(raw.decode(), focus);
        *self.last_read.borrow_mut() = raw;
        landing.and_then(|id| self.row_of(id)).or(previous_row)
    }

    /// Records a successful Apply: the value now in the registry becomes what
    /// the next external-change comparison is made against, and the Baseline
    /// moves onto the Working Copy.
    ///
    /// Apply is a barrier, not a flush — the Undo/Redo stacks are untouched, so
    /// Ctrl+Z afterwards moves the Working Copy back and simply re-dirties the
    /// Session, saying so as it goes (spec §5, §10.1 item 5).
    fn applied(&self, stored: RawValue) {
        *self.last_read.borrow_mut() = stored;
        self.session.borrow_mut().mark_applied();
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
/// Four kinds of call can. A modal dialog runs its own event loop, and
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
///
/// The Backups tab added the other two, and unlike `render`'s these **are**
/// bound. `BackupsPage::show` rebuilds a list under a live `on_item_focused`,
/// whose handler is `sync` — so it reads every Session *and* the page's own
/// cell of Snapshot files; `show` therefore fills the widget first and replaces
/// that cell last, holding no borrow of it across either. And
/// `Notebook::set_selection`, which `restore` calls to activate the target
/// Scope's tab, runs the page-changed handler synchronously, which borrows
/// every Session — so `restore` copies what it needs out of the page before it
/// touches one, and the `borrow_mut` that performs the Restore is a temporary
/// that dies with the `if` that tests it.
struct App {
    frame: Frame,
    notebook: Notebook,
    /// The Backups tab: every Snapshot on disk, and the Restore that brings one
    /// back into a Working Copy (spec §8). Not a Scope, so it is not one of
    /// [`tabs`](App::tabs) and no editing command reaches it.
    backups: BackupsPage,
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
    /// This Run's facts: the log and the Data Directory, decided once by
    /// `startup::decide` and handed to every Apply Run (ADR-0008, ADR-0010).
    run: Run,
    /// The settings as they now stand. Held rather than read per Apply because
    /// `maxBackups` is a setting the user changes while the application runs,
    /// which is exactly why it is not one of the Run's facts (ADR-0010); the
    /// Settings dialog replaces what is here.
    settings: RefCell<SettingsFile>,
    /// Whether the elevated instance has been spawned and this one is exiting
    /// (spec §9, ADR-0005). Read in exactly one place: the close path, whose
    /// standard close-confirm must not re-ask what the restart command's
    /// dedicated dialog has already answered.
    relaunched: Cell<bool>,
}

/// Builds and shows the main window over the two loaded Sessions, and hands it
/// back so a startup dialog has a parent to sit on and a window to hand focus
/// back to. A Read-only Data run passes its reason; announcing it is the last
/// step of startup (spec §11: … → UI → writability → announce).
///
/// `start_tab` is the one thing an elevation relaunch carries across the
/// process boundary — the tab the user left (spec §9, ticket 12 D5). `None`
/// is a plain launch and opens on the User tab.
pub fn build_main_window(
    user: SharedScope,
    system: SharedScope,
    readonly: Option<ReadOnlyReason>,
    run: Run,
    settings: SettingsFile,
    start_tab: Option<StartTab>,
) -> Frame {
    // Which instance this window is, said where Alt+Tab reads first: the
    // elevated title is the cmd.exe convention and Catalogue text (ticket 12
    // D11); the unelevated one is the product name, which no translation may
    // vary.
    let title = match run.elevated() {
        true => translate(msgids::WINDOW_TITLE_ELEVATED),
        false => "PathMaster".to_string(),
    };
    let frame = Frame::builder()
        .with_title(&title)
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
    let user_page = ScopePage::build(&notebook, &catalogue, &user.session.borrow());
    let system_page = ScopePage::build(&notebook, &catalogue, &system.session.borrow());
    let tabs = [
        ScopeTab {
            scope: Scope::User,
            session: user.session,
            page: user_page,
            findings: RefCell::new(None),
            last_read: RefCell::new(user.last_read),
        },
        ScopeTab {
            scope: Scope::System,
            session: system.session,
            page: system_page,
            findings: RefCell::new(None),
            last_read: RefCell::new(system.last_read),
        },
    ];
    // The Backups tab is not a Scope: it lists files rather than Entries, and
    // its content is read from the directory when it is activated.
    let backups = BackupsPage::build(&notebook, &catalogue);
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
    notebook.add_page(&backups.panel, &translate(msgids::TAB_BACKUPS), false, None);

    let root_sizer = BoxSizer::builder(Orientation::Vertical).build();
    root_sizer.add(&banner, 0, SizerFlag::Expand | SizerFlag::All, 4);
    root_sizer.add(&notebook, 1, SizerFlag::Expand | SizerFlag::All, 4);
    root.set_sizer(root_sizer, true);

    // Command-only (NVDA+End), absent from the Tab order: field 0 general status,
    // field 1 the passive merged-length field (spec §12 D10). No field is ever
    // styled: text carries everything.
    let status = frame.create_status_bar(2, 0, ID_ANY as Id, "");
    status.set_status_widths(&[-3, -2]);

    // Read before the settings move into the window, and used below: where
    // this window opens is decided from the file, once, before it is shown.
    let remembered = settings.window();

    let app = Rc::new(App {
        frame,
        notebook,
        backups,
        menu: frame.get_menu_bar().expect("the menu bar was just set"),
        announcer: Announcer::new(banner, Rc::clone(&catalogue)),
        catalogue,
        status,
        tabs,
        readonly,
        pump: Pump::new(&frame),
        merged_length: Cell::new(None),
        run,
        settings: RefCell::new(settings),
        relaunched: Cell::new(false),
    });
    // The tab the user left, honoured before the handlers exist: a plain
    // launch announces nothing about the tab it opens on, and an elevated
    // relaunch is the same start on a different tab — so `set_selection` runs
    // here, where there is no page-changed handler to speak. The Backups tab
    // reads its directory on activation, and with no handler yet that read is
    // made by hand.
    if let Some(tab) = start_tab {
        app.notebook.set_selection(start_tab_index(tab) as usize);
        if tab == StartTab::Backups {
            app.reload_backups();
        }
    }
    app.bind();
    app.sync();

    // Where the window opens (spec §12): what the file remembered, clamped
    // onto the monitors plugged in now — decided before `show`, so it opens
    // where it belongs rather than jumping once the user can see it.
    match geometry::place(remembered, &geometry::work_areas()) {
        // The builder's 900×650 stands, and wx centres it. Both already
        // happened, which is why this arm carries no numbers.
        Placement::Centred => frame.centre(),
        Placement::Remembered(window) => {
            // Physical pixels, set on the built frame: wxdragon routes a
            // *builder's* size through an implicit `FromDIP` and this call
            // through nothing, so what was measured on the way out is what is
            // set on the way back in (spec §12).
            frame.set_size_with_pos(window.x, window.y, window.width, window.height);
            // After the geometry, never instead of it: what is set above is
            // where the window goes the moment the user restores it.
            if window.maximised {
                frame.maximize(true);
            }
        }
    }
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

        // The Backups tab's own two events. Restore has no menu item and no
        // accelerator (spec §15), so the button is its whole route — and the
        // list's focus decides what that button is worth, because a row is
        // what a Restore is of.
        let app = Rc::clone(self);
        self.backups.restore.on_click(move |_| app.restore());
        let app = Rc::clone(self);
        self.backups.list.on_item_focused(move |_| app.sync());

        // Announcement 1 (spec §10.1): activating a Scope tab speaks its entry
        // count. The count is read at activation time, not captured — Refresh
        // and editing change it under the same handler.
        let app = Rc::clone(self);
        self.notebook.on_page_changed(move |event| {
            // The selection the event carries, not the notebook's: on Windows
            // the widget has not caught up when this fires.
            let selection = event.get_selection();
            let active = app.tab_at(selection);
            if let Some(tab) = active {
                let count = tab.session.borrow().entries().len();
                app.announcer.announce(Announcement::EntryCount {
                    scope: tab.scope,
                    count,
                });
            }
            // The directory is re-read here rather than held: the other
            // instance writes Snapshots into it too, and this tab is the only
            // place that shows them. Activating it announces nothing — it is
            // not a Scope, and there is no seventh Announcement for a list.
            if selection == Some(TAB_INDEX_BACKUPS) {
                app.reload_backups();
            }
            app.sync_for(active);
            event.base.skip(true);
        });

        // Closing the application, however it was asked for: the title bar's
        // [X], Alt+F4, File → Exit and the taskbar's own Close all arrive here
        // as one event, so the close-confirm is asked once, in one place
        // (spec §5, FR-close-confirm).
        let app = Rc::clone(self);
        self.frame.on_close(move |event| {
            if app.closing() {
                // Nothing here destroys the window. Skipping hands the event
                // on to wx's own top-level handler, which is what does — a
                // handler that neither skips nor vetoes leaves the window open
                // and says nothing about why.
                event.skip(true);
                return;
            }
            // A close event is never one of the typed variants, so this
            // always matches. Vetoing says "do not close" outright, rather
            // than leaving it to be inferred from an event nobody skipped.
            if let WindowEventData::General(close) = &event {
                close.veto();
            }
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
        let active = self.active_tab();
        // The borrow dies here, before anything below can open a dialog.
        let available = {
            let session = active.map(|tab| tab.session.borrow());
            command.enabled(&self.availability(session.as_deref()))
        };
        if !available {
            return;
        }
        // The commands that are not about a Scope — and so the ones reachable
        // from the Backups tab, where there is no Scope tab to hand one.
        //
        // Exhaustive, like every other `match` over this enum: a catch-all
        // would route the next command someone adds to a Scope it may not be
        // about, silently.
        match command {
            Command::Settings => return self.open_settings(),
            Command::OpenBackupsFolder => return self.open_backups_folder(),
            Command::RestartAsAdministrator => return self.restart_as_administrator(),
            // `false`, never `true`: a forced close would be a way past the
            // very dialog the close-confirm exists to ask (spec §5).
            Command::Exit => return self.frame.close(false),
            Command::Add
            | Command::Edit
            | Command::Delete
            | Command::MoveUp
            | Command::MoveDown
            | Command::Undo
            | Command::Redo
            | Command::Apply
            | Command::Cancel
            | Command::Refresh => {}
        }
        let Some(tab) = active else { return };
        match command {
            Command::Add => self.add(tab),
            Command::Edit => self.edit(tab),
            Command::Delete => self.delete(tab),
            // The direction is the command, so it travels as the command: a
            // bare `true` at this call site would say nothing.
            Command::MoveUp | Command::MoveDown => self.move_entry(tab, command),
            Command::Undo | Command::Redo => self.undo_redo(tab, command),
            Command::Apply => self.apply(tab),
            Command::Cancel => self.cancel(tab),
            Command::Refresh => self.refresh(tab),
            // Answered above, before there was a Scope to answer them over.
            Command::Settings
            | Command::OpenBackupsFolder
            | Command::RestartAsAdministrator
            | Command::Exit => {}
        }
    }

    /// Tools → Restart as Administrator: the **one entry point into
    /// elevation** (spec §9, ADR-0005). The command does what it says — the
    /// dedicated close-confirm when anything is dirty, the UAC prompt, and on
    /// a successful spawn this instance exits, through the same close path as
    /// every other exit with the question already answered.
    ///
    /// A declined prompt is never silent: `ERROR_CANCELLED` earns a dialog
    /// and the application carries on, fully functional. Focus returns to
    /// where it was because a modal dialog hands it back to the control that
    /// opened it, the same rule every dialog here rides. Any other spawn
    /// failure keeps the application running and earns one `ERROR` log line
    /// with the raw code; the on-screen reporting stays with `ShellExecuteEx`'s
    /// own error UI, which `relaunch_elevated` deliberately leaves on — and on
    /// the one path that fails before that call can show anything, a process
    /// that cannot name its own executable, the log line is the only witness.
    fn restart_as_administrator(&self) {
        // The close-confirm flow, run through rather than around (ADR-0005):
        // the dedicated title names what is lost. It can only be User changes
        // — System is non-writable unelevated, and elevated this command is
        // disabled — and there is deliberately no [Save]: this dialog is the
        // relaunch's own, and its two buttons are its two outcomes. The safe
        // answer holds the default, the focus and Escape (`question::choose`).
        if !self.dirty_scopes().is_empty()
            && question::choose(
                &self.frame,
                &translate(msgids::DIALOG_DISCARD_AND_RESTART),
                &[
                    &translate(msgids::BUTTON_DISCARD_AND_RESTART),
                    &translate(msgids::BUTTON_DIALOG_CANCEL),
                ],
            ) != 0
        {
            return;
        }
        // Only the active tab crosses the boundary (ticket 12 D5): Sessions
        // are dead at a process boundary and stay dead.
        match elevation::relaunch_elevated(self.active_start_tab()) {
            Ok(()) => {
                // The elevated instance is up; on success the original
                // instance exits (spec §9). Through `close`, not around it,
                // so the geometry write and the shutdown record still happen
                // — the flag is what keeps the standard close-confirm from
                // asking again what the dedicated dialog just answered.
                self.relaunched.set(true);
                self.frame.close(false);
            }
            Err(RelaunchFailure::Declined) => {
                question::tell(&self.frame, &translate(msgids::DIALOG_ELEVATION_CANCELLED));
            }
            // Nothing was spawned; the application keeps running (spec §9
            // names only the declined prompt) and the failure lands its one
            // log record with the raw code — the shell's own error UI covers
            // the screen where the call itself ran, and the log covers the
            // path that failed before it.
            Err(RelaunchFailure::Failed { os_error }) => {
                self.run
                    .log(&Record::relaunch_failed(FailureCause::Io { os_error }));
            }
        }
    }

    /// The tab the user is on, as the relaunch's one argument names it — the
    /// inverse of [`start_tab_index`], and computed as that inverse rather
    /// than written out again, so a reordered notebook cannot leave the two
    /// disagreeing. A selection no `StartTab` names cannot occur (every page
    /// is one), but the search's degraded answer is the User tab, the same
    /// one a plain launch opens on.
    fn active_start_tab(&self) -> StartTab {
        let selection = self.notebook.selection();
        StartTab::ALL
            .into_iter()
            .find(|tab| start_tab_index(*tab) == selection)
            .unwrap_or(StartTab::User)
    }

    /// Tools → Settings…: the two settings the user may change while the
    /// application runs (spec §13, §11).
    ///
    /// The order is the whole of it. What the dialog opens on is read out
    /// **before** it is shown — a `Ref` taken inside the call would live
    /// across a modal loop, and the diagnostic Timer ticks inside one — and
    /// what it answers is recorded and written afterwards, both under a
    /// borrow that dies here.
    ///
    /// **`maxBackups` is in force the moment this returns**: the settings the
    /// window holds are what the next Apply Run reads its rotation budget from
    /// (ADR-0010), so there is nothing further to apply. The language is not,
    /// and says so on its own label — nothing is re-translated and nothing is
    /// announced.
    ///
    /// Escape and [Cancel] leave the file untouched, and so does an OK over
    /// controls the user only looked at: `record_choices` compares, and a
    /// comparison that finds nothing has nothing to write.
    fn open_settings(&self) {
        let opening = self.settings.borrow().choices();
        // The one run that may write is the one whose OK is worth pressing.
        // Asked once, and used twice: the dialog disables its controls on this
        // answer, and the write below needs the very directory it names.
        let data_dir = self.writable_data_dir();
        let Some(chosen) = settings_dialog::ask_for_settings(
            &self.frame,
            &self.catalogue,
            opening,
            data_dir.is_some(),
        ) else {
            return;
        };
        // Unreachable: a Read-only Data run has no enabled OK to answer with.
        // Answered anyway, and answered by changing nothing — a run that may
        // not write its directory may not quietly change what it is doing
        // either, because the two would then disagree at the next start.
        let Some(data_dir) = data_dir else { return };
        // Amended on a copy, because what the file takes is what this run
        // adopts (`record_settings`). The borrow dies with this statement.
        let mut amended = self.settings.borrow().clone();
        if !amended.record_choices(chosen) {
            return;
        }
        // **A failed write is told, because nothing happened.** Nobody asked
        // for the geometry write, and on that path the window is already
        // going, so its `WARN` line stands alone (spec §13); this one the user
        // did ask for, and since the run adopts nothing the file did not take,
        // the whole of what they get for pressing OK is a dialog closing. The
        // one stock [OK] and the whole message in the title, like every other
        // dialog here — and not an Announcement: that catalogue is closed at
        // seven, and this belongs beside the startup dialog its unreadable
        // twin already earns.
        if !self.record_settings(data_dir, amended) {
            question::tell(&self.frame, &translate(msgids::DIALOG_SETTINGS_UNWRITABLE));
        }
    }

    /// Writes an amended `settings.json` and adopts it into this run **only if
    /// the file took it** — answering whether it did (spec §13).
    ///
    /// The order is the whole of it. The document is amended on a copy and
    /// becomes what this window holds once the write has succeeded, so the file
    /// and the run can never disagree about what the settings are — and a write
    /// that failed leaves the very difference that makes the next attempt a
    /// change again. Recording in memory first would make a second OK compare
    /// equal and write nothing, which is the one state a user whose setting did
    /// not persist must not be left in; and the condition that fails this write
    /// is one §3 calls a designed state, the other instance holding the file.
    ///
    /// The write is the atomic replace every write to this file goes through:
    /// the other instance never reads a half-written file, and a failure leaves
    /// the previous one intact. It earns one `WARN settings:` line here,
    /// whoever asked for it; what else a caller owes the user is the caller's,
    /// because only the caller knows whether anyone asked.
    fn record_settings(&self, data_dir: &Path, amended: SettingsFile) -> bool {
        if let Err(error) = settings::write(data_dir, &amended) {
            self.run
                .log(&Record::settings_write_failed(FailureCause::Io {
                    os_error: error.raw_os_error(),
                }));
            return false;
        }
        *self.settings.borrow_mut() = amended;
        true
    }

    /// Tools → Open Backups Folder: the Snapshots' own directory, handed to
    /// the shell (spec §15). Not a file dialog — nothing here is asking the
    /// user for a folder.
    fn open_backups_folder(&self) {
        // Unreachable with `None` — `Command::enabled` has already turned the
        // item off for the one Run that does not know where its data lives —
        // and this is what that `None` means.
        if let Some(data_dir) = self.run.data_dir() {
            snapshots::open_folder(data_dir);
        }
    }

    /// Restore loads the chosen Snapshot's Entries and Value Type into its own
    /// Scope's Working Copy, as one ordinary Checkpoint (spec §8, ADR-0006).
    /// **Nothing reaches the registry**: what a Restore has done is make the
    /// Session dirty, and Apply is what writes it — which is also why an
    /// accidental one is Ctrl+Z, and why there is no confirmation to sit
    /// through, exactly as Delete has none.
    ///
    /// The target Scope's tab is then activated with focus on the restored
    /// list, so what happened is heard through focus rather than through an
    /// eighth Announcement. Activating a Scope's tab speaks its entry count
    /// like any other activation — this is one, and hiding it from the handler
    /// that answers them would be a second kind of tab switch.
    fn restore(&self) {
        // Read out before the Session is touched: the page's own borrow must
        // not still be open when the notebook fires its page-changed handler,
        // which reads every Session in the window.
        let Some((scope, entries, value_type)) = self.backups.restore_payload() else {
            return;
        };
        let tab = self.tab_of(scope);
        if !tab.session.borrow_mut().restore(entries, value_type) {
            return;
        }
        self.activate(scope);
        // The first row, or — over a Snapshot that restored nothing — the list
        // itself, which is where `focus_row` lands when there is no row.
        self.after_edit(tab, Some(0));
    }

    /// Re-reads `data\backups\` and puts it on screen.
    ///
    /// A directory that cannot be read reads as no Snapshots, and says nothing
    /// about it: the Announcement catalogue is closed at seven and none of
    /// them is about a list, and the only run that can reach it is one whose
    /// Data Directory the user has taken away underneath it.
    fn reload_backups(&self) {
        let files = self
            .run
            .data_dir()
            .and_then(|data_dir| snapshots::load(&snapshots::dir(data_dir)).ok())
            .unwrap_or_default();
        self.backups.show(files);
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
        let row = tab.adopt(raw);
        self.after_edit(tab, row);
        let count = tab.session.borrow().entries().len();
        self.announcer.announce(Announcement::EntryCount {
            scope: tab.scope,
            count,
        });
    }

    /// Ctrl+S: the Apply Run over the active Scope alone (spec §5, FR-apply).
    fn apply(&self, tab: &ScopeTab) {
        if let Some(outcome) = self.apply_scopes(&[tab.scope]) {
            self.after_apply(outcome);
        }
    }

    /// One Apply Run over `order`, in that order, stopping at the first Scope
    /// that does not complete (spec §5, FR-apply; ADR-0008). Ctrl+S is a run
    /// of one; the close-confirm's Save is a run over every dirty Scope, User
    /// first.
    ///
    /// Everything the run needs is copied out **before** it is called, and
    /// nothing is borrowed across it: the run opens modal dialogs, a modal
    /// dialog runs its own event loop, and the diagnostic Timer ticks inside
    /// that loop — a Session borrow held across it would meet the pass's own
    /// and panic (ADR-0008).
    ///
    /// A run with no Data Directory has nowhere to put the backup that must
    /// precede any write, so there is no run to make. It is unreachable in
    /// practice — such a run is Read-only Data, whose Sessions are all
    /// non-writable, so nothing is dirty and neither Ctrl+S nor the
    /// close-confirm can reach here — but the Data Directory is an `Option`
    /// and this is what its `None` means.
    fn apply_scopes(&self, order: &[Scope]) -> Option<apply::Outcome> {
        let data_dir = self.run.data_dir()?;
        // Read out here rather than in the struct below: a `Ref` taken inside
        // the call expression would live until the call returned, which is to
        // say across every dialog the run opens.
        let max_backups = self.settings.borrow().max_backups();
        Some(apply::apply(
            ApplyRun {
                scopes: [
                    self.scope_input(Scope::User),
                    self.scope_input(Scope::System),
                ],
                order,
                data_dir,
                log_path: self.run.log_path(),
                // Read here rather than inside the run, because a Snapshot's
                // name and its collision suffix must both come from one
                // reading of the clock (ADR-0008).
                at: logwriter::now(),
                // From the settings as they now stand, not from a copy taken
                // at startup (ADR-0010).
                max_backups,
            },
            &ProcessEnvironment,
            &Dialogs {
                frame: &self.frame,
                catalogue: &self.catalogue,
            },
        ))
    }

    /// One Scope as the run takes it. Both are handed over however few are
    /// being applied: the merged length the over-length gate reads is a fact
    /// about the pair (spec §7).
    ///
    /// Every borrow it takes dies inside it, which is what lets the caller
    /// hand the result to a sequence that opens dialogs.
    fn scope_input(&self, scope: Scope) -> ScopeInput {
        let tab = self.tab_of(scope);
        ScopeInput {
            scope,
            key: tab.key(),
            entries: tab.raw_entries(),
            value_type: tab.session.borrow().value_type(),
            last_read: tab.last_read.borrow().clone(),
        }
    }

    /// The rest of FR-apply's fixed order, which is the window's half: move the
    /// Baseline, re-run diagnostics, announce.
    ///
    /// A failure reaches none of it — the Working Copy is untouched and the
    /// Baseline stays where it was, which is the taxonomy's first invariant
    /// (spec §9) and is why the run is handed no Baseline at all.
    fn after_apply(&self, outcome: apply::Outcome) {
        for record in &outcome.records {
            self.run.log(record);
        }
        for (scope, done) in outcome.scopes {
            let tab = self.tab_of(scope);
            match done {
                ScopeOutcome::Applied { stored } => {
                    tab.applied(stored);
                    // Focus stays on the current Entry (spec §10), so the list
                    // is not redrawn — nothing in it changed — and focus is
                    // moved only if the control holding it has just been
                    // disabled out from under the user.
                    tab.page
                        .rescue_focus(&self.availability(Some(&tab.session.borrow())));
                    self.announcer.announce(Announcement::Applied { scope });
                }
                ScopeOutcome::Refreshed { found } => {
                    // The external-change dialog's middle answer: Working Copy
                    // and Baseline both become what was just read, and the
                    // stacks clear. Nothing was written and nothing is
                    // announced — Apply did not happen.
                    //
                    // The list is redrawn here rather than through `after_edit`
                    // because the sync and the diagnostic pass belong to the
                    // run as a whole, and happen once below however many Scopes
                    // it reached.
                    let row = tab.adopt(found);
                    let session = tab.session.borrow();
                    tab.page
                        .render(&session, tab.findings.borrow().as_ref(), row);
                }
                // Nothing happened, and the user is the one who decided so.
                ScopeOutcome::Cancelled => {}
                ScopeOutcome::Failed(failure) => {
                    self.announcer.announce(Announcement::ApplyFailed {
                        cause: failure.catalogue_msgid(),
                    });
                }
            }
        }
        self.sync();
        // One pass covers both Scopes, so it is asked for once however many
        // the run reached (spec §7, FR-diag-duplicate).
        self.request_pass();
    }

    /// Everything closing the application has to get right (spec §5
    /// FR-close-confirm, §12, §14). `true` lets the close proceed.
    ///
    /// The order is the whole of it. The user's answer comes first, because a
    /// [Cancel] means none of the rest happens; the geometry is written only
    /// once the close is certain, which is what "on clean shutdown only"
    /// means; and the shutdown line is last, so its presence in the log says
    /// every step above it ran. A killed process shows as that line's absence.
    fn closing(&self) -> bool {
        // An elevation relaunch arrives here with its question already asked
        // — the dedicated dialog, run through the close-confirm flow rather
        // than around it (spec §9, ADR-0005) — so the standard one would be
        // the same question twice, this time offering the [Save] the
        // dedicated dialog deliberately does not.
        if !self.relaunched.get() && !self.save_or_discard() {
            return false;
        }
        self.remember_geometry();
        self.run.log(&Record::shutdown_clean());
        true
    }

    /// The close-confirm: one dialog for the application, raised only when a
    /// Session is dirty (spec §5, FR-close-confirm). `true` lets the close
    /// proceed.
    ///
    /// **Clean Sessions close with no dialog** — dirty is a comparison, so an
    /// edit and its exact reversal are not something to ask about.
    ///
    /// One list of dirty Scopes, read once: it is what the title names and
    /// what the Apply Run is handed, so the sentence cannot promise an order
    /// the sequence does not keep. The borrows it takes die inside it, before
    /// the dialog's own event loop starts.
    fn save_or_discard(&self) -> bool {
        let dirty = self.dirty_scopes();
        if dirty.is_empty() {
            return true;
        }
        match question::choose(
            &self.frame,
            &self.catalogue.close_confirm_dialog(&dirty),
            &[
                &translate(msgids::BUTTON_SAVE),
                &translate(msgids::BUTTON_DISCARD),
                &translate(msgids::BUTTON_DIALOG_CANCEL),
            ],
        ) {
            0 => self.save_then_close(&dirty),
            // Close without writing. The Sessions die with the process, which
            // is what makes this the cheap answer to give: nothing on the
            // machine changes.
            1 => true,
            // [Cancel], which is also Escape, the default and the close box:
            // the outcome that changes least (`question::choose`).
            _ => false,
        }
    }

    /// [Save]: one Apply Run over every dirty Scope, User first, each through
    /// the full Apply path — external-change detection, backup, taxonomy and
    /// all (spec §5, FR-close-confirm). `true` lets the close proceed.
    ///
    /// **Partial failure aborts the close.** A run that did not complete
    /// leaves the window open, and a run that *failed* also sends focus to the
    /// tab it failed on — activated **before** the outcome is applied, because
    /// activating a tab speaks its entry count while the failure's reason is
    /// spoken by `after_apply`: last spoken is what the user hears, and the
    /// reason is what they need (spec §10.1 items 1 and 3). A [Cancel] inside
    /// the run stops the close just as a failure does and moves nothing: the
    /// user chose it, and there is nothing to announce.
    fn save_then_close(&self, dirty: &[Scope]) -> bool {
        // Unreachable: a dirty Session is a writable one, and the only Run
        // without a Data Directory is Read-only Data, whose Sessions are both
        // non-writable. Answered anyway, and answered by staying open —
        // nothing was saved, so nothing may be thrown away.
        let Some(outcome) = self.apply_scopes(dirty) else {
            return false;
        };
        let completed = outcome.completed();
        if let Some(scope) = outcome.failed_scope() {
            // Both halves, because activating a tab is not landing focus in
            // it: a row focused in a control that is not focused is silent,
            // which for this application is the same as not having happened
            // (`scope_page`). The Working Copy is untouched by a failure, so
            // the row the user was already on is the row they return to.
            self.activate(scope);
            self.tab_of(scope).page.focus_list();
        }
        self.after_apply(outcome);
        completed
    }

    /// The Scopes whose Sessions are dirty, **in tab order** — which is User
    /// first, and so is the order FR-close-confirm's Save applies them in
    /// (spec §5, §12).
    ///
    /// One constant answers both: `TAB_INDEX_USER` is 0, so the tab the user
    /// reads first and the Scope the run writes first cannot come apart.
    fn dirty_scopes(&self) -> Vec<Scope> {
        let mut dirty: Vec<Scope> = self
            .tabs
            .iter()
            .filter(|tab| tab.session.borrow().is_dirty())
            .map(|tab| tab.scope)
            .collect();
        dirty.sort_by_key(|scope| tab_index(*scope));
        dirty
    }

    /// Writes where the window is into `settings.json` — on a clean shutdown
    /// only, and in Writable Data only (spec §12, §13).
    ///
    /// It amends the document rather than serialising the settings, so a hand
    /// edit's unknown fields and its key order both survive; the write itself
    /// is [`record_settings`](Self::record_settings)'s.
    ///
    /// **A failed write is not the user's problem to hear about.** They asked
    /// to close, and that is happening; the window is already going, so a
    /// dialog would outlive what it is about, and the Announcement catalogue
    /// is closed at seven. The log is the only witness there is room for, and
    /// it is the one a developer asked "why does it not remember where I put
    /// it?" would reach for — which is why this is the caller that drops the
    /// answer the Settings dialog acts on.
    fn remember_geometry(&self) {
        let Some(data_dir) = self.writable_data_dir() else {
            return;
        };
        // **A minimised window is not a place.** Windows parks one far off
        // every monitor and reports that as its position, and `is_maximized`
        // answers `false` for it whatever it was before — so recording it
        // would overwrite a perfectly good remembered geometry with one the
        // next start can only read as off-screen, and the window would come
        // back centred at the default size. Writing nothing leaves the file
        // saying what it already said, which is the last place the user could
        // actually see it.
        if self.frame.is_iconized() {
            return;
        }
        let position = self.frame.get_position();
        let size = self.frame.get_size();
        let mut amended = self.settings.borrow().clone();
        // `pathmaster_core::settings::Window` spelled out: `Window` in a wx
        // module is wx's own, and this is the record in the file.
        amended.set_window(pathmaster_core::settings::Window {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
            // Recorded beside the geometry wx reports **while** maximised,
            // never instead of it: restoring sets that geometry and maximises
            // over it, so an un-maximise afterwards lands on a window the size
            // of the screen rather than on nothing at all.
            maximised: self.frame.is_maximized(),
        });
        // The answer is deliberately dropped: this is the one writer nobody
        // asked for, and its `WARN` line — which `record_settings` has already
        // written — is the only witness there is room for on a path where the
        // window is already going.
        self.record_settings(data_dir, amended);
    }

    /// The Data Directory this Run may **write**, which is not the same as the
    /// one it can name: `Run::data_dir` is `DataDirState::dir()`, and a
    /// Read-only Data run has one of those too. `readonly` is the UI's own
    /// record of the state that decided both, and it is `None` in exactly one
    /// case — `DataDirState::Writable`.
    ///
    /// Apply deliberately does not ask this question: startup predicts, Apply
    /// verifies at write time (ADR-0002). Geometry is the other side of that
    /// rule — nobody asked for it, so a run that could only find out by
    /// failing does not try, and "not written in Read-only Data" is visible
    /// here rather than left to a `write` that would have refused anyway.
    fn writable_data_dir(&self) -> Option<&Path> {
        match self.readonly {
            Some(_) => None,
            None => self.run.data_dir(),
        }
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

    /// Activates a Scope's tab.
    ///
    /// One line, with a cast and a hazard in it, and both belong in one place:
    /// `set_selection` runs the page-changed handler **synchronously**, and
    /// that handler reads every Session in the window — so no caller may hold
    /// a borrow across this.
    fn activate(&self, scope: Scope) {
        self.notebook.set_selection(tab_index(scope) as usize);
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
        let selection = selection?;
        self.tabs
            .iter()
            .find(|tab| tab_index(tab.scope) == selection)
    }

    /// What a command's availability is decided from: a Session, and the facts
    /// of this Run (see [`Availability`]).
    fn availability<'a>(&self, session: Option<&'a Session>) -> Availability<'a> {
        Availability {
            session,
            data_dir: self.run.data_dir().is_some(),
            elevated: self.run.elevated(),
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
        command::sync_menu_bar(&self.menu, &self.availability(session.as_deref()));
        drop(session);
        for tab in &self.tabs {
            tab.page
                .sync_buttons(&self.availability(Some(&tab.session.borrow())));
        }
        // Restore is worth something only over a row that can be loaded into a
        // Session that can be written: a Corrupted Snapshot has nothing to
        // load, and System unelevated or a Read-only Data run has nowhere to
        // load it (spec §8). Both read as a disabled button.
        let restorable = self
            .backups
            .restore_target()
            .is_some_and(|scope| self.tab_of(scope).session.borrow().writable());
        self.backups.sync_button(restorable);
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

/// The window's half of the Apply Run's question port: three dialogs, and
/// nothing else (ADR-0008).
///
/// It is built for the length of one run and holds only what a dialog needs —
/// a parent to sit on and the Catalogue that composes the two titles carrying
/// a number. No Session and no `Rc<App>`: a dialog runs its own event loop,
/// and everything reachable from here must already be free of borrows.
struct Dialogs<'a> {
    frame: &'a Frame,
    catalogue: &'a Catalogue,
}

impl Ask for Dialogs<'_> {
    /// The value moved under the Session since it was last read (spec §5,
    /// FR-apply). All three answers are legal, so all three are buttons; the
    /// last is Cancel, which is where Escape and the default land.
    fn external_change(&self, _scope: Scope) -> ExternalChange {
        match question::choose(
            self.frame,
            &translate(msgids::DIALOG_EXTERNAL_CHANGE),
            &[
                &translate(msgids::BUTTON_OVERWRITE),
                &translate(msgids::BUTTON_REFRESH_AND_DISCARD),
                &translate(msgids::BUTTON_DIALOG_CANCEL),
            ],
        ) {
            0 => ExternalChange::Overwrite,
            1 => ExternalChange::RefreshAndDiscard,
            _ => ExternalChange::Cancel,
        }
    }

    /// Past 8,191 (spec §7). Proceeding is legal and the button says so; the
    /// title carries the number, because a `MessageDialog`'s body is never
    /// spoken.
    fn cmd_limit(&self, length: usize) -> bool {
        question::ask(
            self.frame,
            &self.catalogue.cmd_limit_dialog(length),
            &translate(msgids::BUTTON_APPLY_ANYWAY),
            &translate(msgids::BUTTON_DIALOG_CANCEL),
        )
    }

    /// At 32,767 there is nothing to offer but Cancel, so this dialog has one
    /// button and this function has no answer to give (spec §7).
    fn hard_cap(&self, length: usize) {
        question::choose(
            self.frame,
            &self.catalogue.hard_cap_dialog(length),
            &[&translate(msgids::BUTTON_DIALOG_CANCEL)],
        );
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
