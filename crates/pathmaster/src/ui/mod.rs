//! The main window shell (spec §12): one vertical sizer — Banner above the notebook —
//! with the native status bar attached to the frame outside the sizer, and the menu
//! bar on the frame itself.
//!
//! The tab order is the whole map: tabs → search field → list → buttons, full
//! traversal, no traps (v0.2.0 §3).
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
mod door;
mod entry_dialog;
mod list;
mod question;
mod rendering;
mod scope_page;
mod settings_dialog;

use std::cell::{Cell, RefCell};
use std::path::Path;
use std::rc::Rc;

use pathmaster_core::catalogue::{Announcement, Catalogue, ScopeCounts, UndoDirection};
use pathmaster_core::diagnostics::{Diagnosis, Findings};
use pathmaster_core::filtered::{self, Criteria, Filter};
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
use crate::scoped::Scoped;
use crate::ui::backups_page::BackupsPage;
use crate::ui::command::{Availability, Command, Menus};
use crate::ui::rendering::Rendering;
use crate::ui::scope_page::{Row, ScopePage};
use crate::SharedScope;

/// The notebook's page order (spec §12): the two Scopes, then Backups —
/// which is not a Scope, so activating it announces nothing and offers no
/// editing at all.
const TAB_INDEX_USER: i32 = 0;
const TAB_INDEX_SYSTEM: i32 = 1;
const TAB_INDEX_BACKUPS: i32 = 2;

/// The key codes the Search field answers to (v0.2.0 §3). wxdragon names no
/// `WXK_` constants at any level, so the wxWidgets values are spelled here —
/// the same ones the ticket-04 prototype keyed on. Both Enters are consumed:
/// one gesture, one meaning, whichever side of the keyboard it came from.
const WXK_RETURN: i32 = 13;
const WXK_ESCAPE: i32 = 27;
const WXK_DOWN: i32 = 317;
const WXK_NUMPAD_ENTER: i32 = 372;

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
    /// [`Scoped`], not a bare `RefCell`: a Session is reached by commands, by
    /// the Timer's tick and by synchronous toolkit callbacks (ADR-0011).
    session: Rc<Scoped<Session>>,
    page: ScopePage,
    /// The **applied** criteria — the Search text and Filter the list on
    /// screen was last rebuilt under, which is not always what the field
    /// holds: typing sits in the field until the debounce tick applies it.
    /// Per-Editing-Session derived view state (v0.2.0 §2): both die with the
    /// Run, no Checkpoint captures them, and no command changes them — only
    /// the user's own narrowing actions do, typing in the field and choosing
    /// a state in the submenu. [`Scoped`] because commands and the debounce's
    /// tick both reach it.
    criteria: Scoped<Criteria>,
    /// Whether this Scope owes a **spoken count** at the next debounce tick.
    ///
    /// The Expansion toggle is what arms it: with a Filtered View active the
    /// toggle changes membership, so it speaks twice — its own mode message,
    /// then the count through this same debounced path one
    /// `filteredCountDelayMs` later, never combined into one msgid (v0.2.0
    /// §13 item 8). A tick answers one count however many reasons it has, so
    /// a toggle landing inside a typing window is spoken by the tick the
    /// typing already asked for.
    count_due: Cell<bool>,
    /// What the last completed pass found here — the Status column's whole
    /// content, and the issue count StatusBar field 0 reports. A derived view,
    /// held beside the Session and never inside it (ADR-0001), and [`Scoped`]
    /// for the Session's own reason.
    ///
    /// `None` until the first pass lands, and that is not the same as a pass
    /// that found nothing: the column reads both as empty, but the StatusBar
    /// must not claim "0 issues" about a Scope no pass has yet looked at.
    findings: Scoped<Option<Findings>>,
    /// What this Scope's registry value was the last time it was read.
    ///
    /// The comparison subject external-change detection needs (spec §4): it is
    /// `(vtype, bytes)` because decoding stops at the first NUL, so a decoded
    /// copy would miss a real change. It cannot live in the `Session` —
    /// `RawValue` is a `pathmaster-platform` type and core may not reach it —
    /// so the window holds it, and three paths keep it current: startup,
    /// Refresh, and whatever an Apply Run hands back (ADR-0008).
    last_read: RefCell<RawValue>,
    /// The one app-wide Expansion Mode, shared with the window and the other
    /// Scope (v0.2.0 §5). Held here rather than passed in because every
    /// question this tab answers about its view — the visible set, the rows,
    /// the counts — is asked under the mode now in force, and a mode handed in
    /// per call is one two callers can disagree about.
    rendering: Rc<Rendering>,
}

impl ScopeTab {
    /// This Scope's Working Copy as the worker takes it: the Entries' raw text,
    /// in list order. Cloned because the pass runs on another thread and the
    /// Session stays behind its scoped access — the pass diagnoses the state
    /// it was handed, not whatever the user has reached by the time it
    /// finishes.
    fn raw_entries(&self) -> Vec<String> {
        self.session.with(|session| {
            session
                .entries()
                .iter()
                .map(|entry| entry.raw().to_string())
                .collect()
        })
    }

    /// The numbers StatusBar field 0 reports about this Scope: how many
    /// Entries it holds now, how many its Filtered View shows (`None` while
    /// nothing narrows it), and how many findings the last pass made here —
    /// `None` until one has run (see [`ScopeCounts`]).
    fn counts(&self) -> ScopeCounts {
        ScopeCounts {
            scope: self.scope,
            entries: self.session.with(|session| session.entries().len()),
            visible: self.narrowed().then(|| self.visible().len()),
            filter: self.filter(),
            issues: self
                .findings
                .with(|findings| findings.as_ref().map(Findings::issue_count)),
        }
    }

    /// Whether this Scope has a Filtered View — a non-empty applied Search
    /// text **or** a narrowing Filter, composed by `Criteria` (v0.2.0 §2).
    fn narrowed(&self) -> bool {
        self.criteria.with(Criteria::narrowing)
    }

    /// Whether the Search half alone is narrowing — what ESC answers to, since
    /// it clears the text and leaves any Filter standing (v0.2.0 §3).
    fn searching(&self) -> bool {
        self.criteria.with(Criteria::searching)
    }

    /// This Scope's applied Filter: the state its submenu's radio mark reads,
    /// and the one StatusBar field 0 names while it narrows (v0.2.0 §4, §16).
    fn filter(&self) -> Filter {
        self.criteria.with(|criteria| criteria.filter)
    }

    /// The visible set: the Working-Copy indices of the Entries this Scope's
    /// Filtered View shows, in order — every index, while nothing narrows.
    /// Recomputed from the live Working Copy on each ask, so it can never
    /// describe a list that has moved on.
    ///
    /// **Matching reads the currently displayed rendering** (v0.2.0 §3): the
    /// same text the rows carry, so what the spoken count counts is exactly
    /// what the arrow keys will read. Its consequence is paid deliberately —
    /// toggling Expansion Mode changes membership, because a raw
    /// `%JAVA_HOME%` Entry and its expanded reading are different haystacks
    /// (v0.2.0 §5).
    ///
    /// **The Filter reads the last completed pass** (v0.2.0 §4), which is the
    /// same clock the Status column runs on: an Entry no pass has looked at
    /// carries no Issues, so a narrowing state shows nothing until one lands
    /// and then shows what it found (spec §7, FR-diag-async).
    fn visible(&self) -> Vec<usize> {
        self.session.with(|session| {
            self.findings.with(|findings| {
                self.criteria.with(|criteria| {
                    filtered::visible_indices(
                        session.entries().iter().map(|entry| {
                            (
                                self.rendering.render(entry.raw()),
                                findings
                                    .as_ref()
                                    .map_or(&[][..], |findings| findings.issues(entry)),
                            )
                        }),
                        criteria,
                    )
                })
            })
        })
    }

    /// The registry value behind this tab. Built where it is used rather than
    /// held: a `ScopeKey` is a key path and a value name, not a handle.
    fn key(&self) -> ScopeKey {
        match self.scope {
            Scope::User => ScopeKey::user(),
            Scope::System => ScopeKey::system(),
        }
    }

    /// The Entry the user is on: its **view** row and its id — owned out of
    /// the scoped access, because every command asks this question immediately
    /// before opening a modal dialog. The view row is mapped through the
    /// visible set: under a Filtered View the list's row 0 can be the Working
    /// Copy's Entry 7, and every allowed command touches exactly the visible
    /// Entry the user is on, never a hidden one (v0.2.0 §2).
    fn focused_entry(&self) -> Option<(usize, EntryId)> {
        let row = self.page.focused_row()?;
        let index = *self.visible().get(row)?;
        self.session
            .with(|session| session.entries().get(index).map(|entry| (row, entry.id())))
    }

    /// The view row `id` now stands at — `None` when the view does not show it,
    /// which under a Filtered View is not the same as the Entry being gone.
    fn view_row_of(&self, id: EntryId) -> Option<usize> {
        self.session.with(|session| {
            let entries = session.entries();
            self.visible()
                .into_iter()
                .position(|index| entries[index].id() == id)
        })
    }

    /// This Scope's rows as the list would show them now: the Filtered View's
    /// visible Entries under the last completed pass's findings, composed
    /// inside the scoped access and handed out owned — which is what lets the
    /// caller rebuild the list with no closure open (ADR-0011).
    fn rows(&self, catalogue: &Catalogue) -> Vec<Row> {
        let visible = self.visible();
        self.session.with(|session| {
            self.findings.with(|findings| {
                Row::compose_visible(
                    session,
                    findings.as_ref(),
                    catalogue,
                    &self.rendering,
                    &visible,
                )
            })
        })
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
    /// Answers the **Entry** focus should land on: the one with the same id if
    /// it survived the re-read, else its nearest neighbour by index (spec §5,
    /// FR-refresh). Turning that into a row is the caller's focus rule —
    /// `None` falls back to the visual position the user was already at.
    fn adopt(&self, raw: RawValue) -> Option<EntryId> {
        let focus = self.focused_entry().map(|(_, id)| id);
        let landing = self
            .session
            .with_mut(|session| session.refresh(raw.decode(), focus));
        *self.last_read.borrow_mut() = raw;
        landing
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
        self.session.with_mut(Session::mark_applied);
    }
}

/// The window and everything an editing command needs to reach: the two Scope
/// tabs, the menu whose enabled states follow the active one, and the single
/// voice.
///
/// It rides an `Rc` into every event handler, which is why every command takes
/// `&self`. The borrow discipline is structural, not a list kept here
/// (ADR-0011): state more than one kind of call reaches lives behind
/// [`Scoped`], whose closures nothing can escape and whose one rule is that a
/// closure body must not dispatch; every dialog opens through [`door`], whose
/// modal depth is what keeps the diagnostic Timer's tick inert while one is
/// up.
struct App {
    frame: Frame,
    notebook: Notebook,
    /// The Backups tab: every Snapshot on disk, and the Restore that brings one
    /// back into a Working Copy (spec §8). Not a Scope, so it is not one of
    /// [`tabs`](App::tabs) and no editing command reaches it.
    backups: BackupsPage,
    menus: Menus,
    /// The one Catalogue (ADR-0009): every string this window composes is
    /// composed here, and the Announcer holds the same one.
    catalogue: Rc<Catalogue>,
    announcer: Announcer,
    status: StatusBar,
    tabs: [ScopeTab; 2],
    /// Expansion Mode: the one app-wide flag, shared with both Scope tabs so
    /// they render alike (v0.2.0 §5). The window holds it because the window
    /// is what flips it; the tabs hold it because they are what reads it.
    rendering: Rc<Rendering>,
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
    ///
    /// [`Scoped`] since v0.2.0's Search landed: the debounce Timer's tick and
    /// the field's synchronous handlers read it, and the Settings command
    /// writes it — more than one kind of call, which is ADR-0011's whole
    /// classification rule.
    settings: Scoped<SettingsFile>,
    /// Whether the elevated instance has been spawned and this one is exiting
    /// (spec §9, ADR-0005). Read in exactly one place: the close path, whose
    /// standard close-confirm must not re-ask what the restart command's
    /// dedicated dialog has already answered.
    relaunched: Cell<bool>,
}

/// Gives the window its icon — which the exe's own icon resource does **not**
/// do (spec §12 D7).
///
/// wxMSW never adopts the executable's `RT_GROUP_ICON` for a frame: with the
/// resource correctly embedded, `WM_GETICON` and the class icon both still
/// answer 0 (research/04 §4.2 measured exactly that). So a build with a
/// perfect Explorer icon shows the generic Windows one in the taskbar, the
/// title bar and Alt+Tab — the half of the job that is easy to miss precisely
/// because the other half looks right.
///
/// One embedded SVG covers every DPI from a single asset, and it is the same
/// source design `resources/app.ico` is rasterised from, so the two assets
/// cannot become two designs. `get_bitmap_for` renders it at this window's
/// scale, which matters because wx takes **one** icon rather than a bundle.
///
/// A failure is silent by design: an icon that would not render is a window
/// with the generic icon, which is exactly what this run would have had
/// anyway, and nothing a user could act on.
fn set_frame_icon(frame: &Frame) {
    const ICON: &[u8] = include_bytes!("../../resources/icon.svg");

    if let Some(bitmap) = BitmapBundle::from_svg_data(ICON, Size::new(32, 32))
        .and_then(|bundle| bundle.get_bitmap_for(frame))
    {
        frame.set_icon(&bitmap);
    }
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
    // The one Catalogue, built before the window it titles and shared by
    // everything that composes a string out of it: this window, the Announcer,
    // and each Scope tab's Status column (ADR-0009). `install` has already
    // given wx its own, which is what `catalog::Installed` asks.
    let catalogue = Rc::new(Catalogue::new(catalog::Installed));

    let frame = Frame::builder()
        // Which instance this window is, said where Alt+Tab reads first
        // (spec §9, ticket 12 D11). Composed rather than chosen here: which of
        // the two titles an elevated process earns is a rule, and rules live
        // beside the msgids they fill.
        .with_title(&catalogue.window_title(run.elevated()))
        // Crosses the FFI boundary through the implicit FromDIP → 900×650 DIP (spec §12 D2).
        .with_size(Size::new(900, 650))
        .build();
    frame.set_min_size(Size::new(800, 600));
    set_frame_icon(&frame);
    // The bar is given to the frame here, before anything is laid out under
    // it, and kept: the Filter submenu's own item carries no command id, so
    // what disables it on the Backups tab is the item itself (v0.2.0 §4).
    let menus = Menus::build(&frame);

    let root = Panel::builder(&frame).build();

    // The Banner: always visible, fixed height, its StaticText empty at rest — the layout
    // never reflows under the user when announce() sets a message (spec §12 D1, §10).
    // get_char_height() and set_min_size are both physical pixels: SetMinSize is one of the
    // FFI calls wxdragon does NOT route through its implicit FromDIP, so no double scaling.
    let banner = StaticText::builder(&root).with_label("").build();
    banner.set_min_size(Size::new(-1, banner.get_char_height()));

    let notebook = Notebook::builder(&root).build();
    // Raw, like every Run: the mode is per-Run derived view state with no
    // `settings.json` field to open expanded from (v0.2.0 §5).
    let rendering = Rc::new(Rendering::new());
    // No pass has run yet, so the rows are composed under no findings — and
    // outside the Sessions' scoped access, because building a page renders it.
    let rows_at_start = |scope: &SharedScope| {
        scope
            .session
            .with(|session| Row::compose(session, None, &catalogue, &rendering))
    };
    let user_page = ScopePage::build(&notebook, &rows_at_start(&user));
    let system_page = ScopePage::build(&notebook, &rows_at_start(&system));
    let tabs = [
        ScopeTab {
            scope: Scope::User,
            session: user.session,
            page: user_page,
            criteria: Scoped::new(Criteria::default()),
            count_due: Cell::new(false),
            findings: Scoped::new(None),
            last_read: RefCell::new(user.last_read),
            rendering: Rc::clone(&rendering),
        },
        ScopeTab {
            scope: Scope::System,
            session: system.session,
            page: system_page,
            criteria: Scoped::new(Criteria::default()),
            count_due: Cell::new(false),
            findings: Scoped::new(None),
            last_read: RefCell::new(system.last_read),
            rendering: Rc::clone(&rendering),
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
        menus,
        announcer: Announcer::new(banner, Rc::clone(&catalogue)),
        catalogue,
        status,
        tabs,
        rendering,
        readonly,
        pump: Pump::new(&frame),
        merged_length: Cell::new(None),
        run,
        settings: Scoped::new(settings),
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

            // The Search field (v0.2.0 §3). Typing restarts the one-shot
            // debounce; the criteria apply — rebuild, sync, count — at the
            // tick. The delay is read per keystroke, so a settings change is
            // in force with no restart.
            let scope = tab.scope;
            let app = Rc::clone(self);
            tab.page.search.on_text_changed(move |_| {
                let delay = app.settings.with(SettingsFile::filtered_count_delay_ms) as i32;
                app.tab_of(scope).page.debounce.start(delay, true);
            });
            // Inert while a dialog is up, like the Pump's tick (ADR-0011): an
            // accelerator can open a dialog inside the debounce window, and a
            // rebuild plus an Announcement must not land under its modal loop.
            // Restarting rather than dropping keeps the promise the keystroke
            // made: the count speaks, one delay after the dialog closes.
            let app = Rc::clone(self);
            tab.page.debounce.on_tick(move |_| {
                let tab = app.tab_of(scope);
                if door::modal_open() {
                    let delay = app.settings.with(SettingsFile::filtered_count_delay_ms) as i32;
                    tab.page.debounce.start(delay, true);
                    return;
                }
                app.apply_criteria(tab);
            });
            // The field's keyboard contract (v0.2.0 §3): Enter is consumed
            // and does nothing — unhandled it would reach the default button —
            // Down-arrow enters the list (Tab does so on its own: the list is
            // next in the Tab order), and ESC clears and returns focus.
            let app = Rc::clone(self);
            tab.page.search.on_key_down(move |event| {
                let key = match &event {
                    WindowEventData::Keyboard(keyboard) => keyboard.get_key_code(),
                    _ => None,
                };
                match key {
                    Some(WXK_RETURN) | Some(WXK_NUMPAD_ENTER) => {}
                    Some(WXK_DOWN) => app.tab_of(scope).page.focus_list(),
                    Some(WXK_ESCAPE) => app.clear_search(app.tab_of(scope)),
                    _ => event.skip(true),
                }
            });
        }

        // The Backups tab's own two events. Restore has no menu item and no
        // accelerator (spec §15), so the button is its whole route — and the
        // list's focus decides what that button is worth, because a row is
        // what a Restore is of.
        let app = Rc::clone(self);
        self.backups.restore.on_click(move |_| app.restore());
        let app = Rc::clone(self);
        self.backups.list.on_item_focused(move |_| app.sync());

        // Announcement 1 (spec §10.1) — or, while the Scope has a Filtered
        // View, the Scope-named filtered count (v0.2.0 §13 item 10):
        // activating a Scope tab speaks what its list holds. The counts are
        // read at activation time, not captured — Refresh and editing change
        // them under the same handler.
        let app = Rc::clone(self);
        self.notebook.on_page_changed(move |event| {
            // The selection the event carries, not the notebook's: on Windows
            // the widget has not caught up when this fires.
            let selection = event.get_selection();
            let active = app.tab_at(selection);
            if let Some(tab) = active {
                app.speak_scope_status(tab);
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
        // FR-diag-async). Inert while a dialog is up (ADR-0011): the Timer
        // keeps firing inside the modal loop — which preserves `Pump`'s
        // self-healing restart — and a pass landing mid-dialog is collected by
        // the first tick after it closes, ≤ 100 ms later.
        let app = Rc::clone(self);
        self.pump.on_tick(move |_| {
            if !door::modal_open() {
                app.collect_pass();
            }
        });
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
    /// The lists are **not** rebuilt where their membership held — only the
    /// Status column is written — because a pass lands on its own schedule and
    /// rebuilding would clear the focused row out from under whoever is
    /// arrowing through it. Both tabs are written, not just the active one:
    /// the tab the user is not looking at was diagnosed by the same pass.
    ///
    /// **A pass can move membership**, though, and only under a Filter: the
    /// state selects on the Issue set, and the Issue set is exactly what a
    /// pass replaces (v0.2.0 §4). Where it moved, the rows on screen are the
    /// wrong rows and the Status cells would land on the wrong Entries, so the
    /// list is rebuilt and lands on §2's row like any other membership change
    /// — silently, which is what §2 says recomputation is.
    fn apply_pass(&self, diagnosis: &Diagnosis) {
        for tab in &self.tabs {
            // Read before the findings land, for the reason the Expansion
            // toggle reads its own: a list row is a position in the visible
            // set, and a pass under a Filter can change which Entries that set
            // holds (v0.2.0 §4).
            let (showing, concerned) = (tab.visible(), tab.focused_entry().map(|(_, id)| id));
            let findings = tab
                .session
                .with(|session| Findings::of(session.entries(), diagnosis.scope(tab.scope)));
            tab.findings.with_mut(|held| *held = Some(findings));
            let rows = tab.rows(&self.catalogue);
            if tab.visible() == showing {
                tab.page.render_status(&rows);
            } else {
                // Membership moved, so the cells no longer line up with the
                // rows on screen and only a rebuild can. Silent and quiet:
                // §2's recomputation says nothing, and a pass lands on its own
                // schedule — taking the keyboard focus for it would be the
                // uninvited jump §2 forbids. The first row stands in where the
                // rule answers nothing, as it does on the typing path: a Run
                // that chose a Filter before its first pass landed was looking
                // at an empty list, so there is no row to keep and §2's "if no
                // rows remain" does not apply to the rows that just arrived.
                tab.page
                    .render_quiet(&rows, self.landing_row(tab, concerned).or(Some(0)));
            }
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
        // Answered before anything below can open a dialog, and owned out of
        // the scoped access.
        let available =
            self.with_availability(active, |availability| command.enabled(availability));
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
            Command::About => return self.about(),
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
            | Command::Refresh
            | Command::Search
            | Command::Filter(_)
            | Command::ToggleIssuesFilter
            | Command::ExpandedValues => {}
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
            Command::Search => self.focus_search(tab),
            // The state is the command, so it travels as the command — and the
            // toggle reads the state now in force rather than carrying one,
            // which is what keeps Ctrl+I's two rules in one place (v0.2.0 §4).
            Command::Filter(filter) => self.set_filter(tab, filter),
            Command::ToggleIssuesFilter => self.set_filter(tab, tab.filter().toggled()),
            // The mode is app-wide, so the active tab is not passed on — but
            // the command is a Scope tab's all the same: reaching here is what
            // "disabled on Backups, like every other View item" means.
            Command::ExpandedValues => self.toggle_expansion(),
            // Answered above, before there was a Scope to answer them over.
            Command::Settings
            | Command::OpenBackupsFolder
            | Command::RestartAsAdministrator
            | Command::Exit
            | Command::About => {}
        }
    }

    /// Ctrl+F, View → Search: focuses the active Scope's Search field and
    /// selects its whole contents, so typing replaces the old query rather
    /// than appending to it (v0.2.0 §3).
    fn focus_search(&self, tab: &ScopeTab) {
        tab.page.search.set_focus();
        tab.page.search.select_all();
    }

    /// The debounce tick: applies what the Search field now holds as this
    /// Scope's criteria — rebuild, enablement, StatusBar, and the spoken
    /// count, in that order so what is heard describes what is on screen.
    ///
    /// A tick whose text equals the applied criteria is not a change — typing
    /// `a` then Backspace inside one debounce window lands here with nothing
    /// to do, and §2's "speaks only when the criteria change" says to do
    /// nothing, loudly included. It still speaks when the tick was armed by
    /// the Expansion toggle, which changed the view without touching the
    /// field: that count is what the toggle owes, and it arrives here so that
    /// **one tick speaks one count** however many reasons it had (v0.2.0 §13
    /// item 8). Only the speaking is debounced — the toggle re-rendered its
    /// rows the moment it was given.
    fn apply_criteria(&self, tab: &ScopeTab) {
        let typed = tab.page.search.get_value();
        let retyped = tab.criteria.with(|criteria| criteria.query != typed);
        let owed = tab.count_due.replace(false);
        if !retyped && !owed {
            return;
        }
        if retyped {
            let concerned = tab.focused_entry().map(|(_, id)| id);
            tab.criteria.with_mut(|criteria| criteria.query = typed);
            // Quiet: focus stays in the field the user is typing in — the
            // rebuild marks the landing row without taking the keyboard focus,
            // which is the mechanism ticket 04 measured silent under NVDA. The
            // first row stands in when the rule answers nothing (a Run whose
            // first gesture is typing has no row to keep), so Down and Tab
            // always land on a row NVDA reads.
            let row = self.landing_row(tab, concerned).or(Some(0));
            tab.page.render_quiet(&tab.rows(&self.catalogue), row);
            self.sync();
        }
        // Spoken only about the list the user is on: the debounce survives a
        // tab switch, and a count describing a hidden list would be noise —
        // the criteria still applied, and arrival speech covers the return.
        if self
            .active_tab()
            .is_some_and(|active| active.scope == tab.scope)
        {
            self.speak_view(tab, |shown, total| Announcement::FilteredCount {
                shown,
                total,
            });
        }
    }

    /// View → Filter, and Ctrl+I: applies a Scope's chosen Filter state
    /// (v0.2.0 §4).
    ///
    /// A **discrete** gesture, unlike typing — so it applies and speaks at
    /// once, and takes the pending Search text with it. The field is what the
    /// query is: a keystroke inside the debounce window has not reached the
    /// criteria yet, and narrowing by the Filter alone while the field holds
    /// text would show a list neither axis describes. Flushing it here is also
    /// what makes **one announcement, never two**: the debounce is stopped and
    /// whatever count it owed is dropped, because this speaks the composed one.
    ///
    /// Re-choosing the state already in force changes no criteria and does
    /// nothing, loudly included — the same rule a debounce tick whose text has
    /// not moved answers to (§2: what is spoken is a *criteria* change).
    ///
    /// Focus is never taken: the command arrives from a menu, wx hands focus
    /// back where it was, and a list that gains rows must not also gain the
    /// keyboard (§2's uninvited-jump rule). The rebuild still marks §2's
    /// landing row, so Down and Tab land on a row NVDA reads.
    fn set_filter(&self, tab: &ScopeTab, filter: Filter) {
        let applied = Criteria::new(tab.page.search.get_value(), filter);
        if tab.criteria.with(|criteria| *criteria == applied) {
            return;
        }
        // The composed Search∧Filter count, in the sentence the new state
        // earns: item 11 while the Filter narrows, item 9 when only the query
        // is left doing it, and Announcement 1 when neither is — the two-part
        // condition, completed (v0.2.0 §13 items 1, 9 and 11).
        self.narrow(tab, applied, |shown, total| match filter {
            Filter::All => Announcement::FilteredCount { shown, total },
            filter => Announcement::FilterCount {
                filter,
                shown,
                total,
            },
        });
    }

    /// The one path a **discrete** narrowing gesture takes: adopt `criteria`
    /// as what this Scope's list is showing, redraw under them, point every
    /// control at the result, and say what changed (v0.2.0 §2, §13).
    ///
    /// Discrete, as against the debounced typing path: the gesture is complete
    /// when it arrives, so the pending debounce dies with the criteria it was
    /// about and **so does any count it owed** — this speaks one of its own,
    /// and an owed count spoken beside it would be the second announcement §4
    /// rules out. A gesture that changes nothing must therefore not come here
    /// at all: its caller answers that first.
    ///
    /// Quiet, and never focus-taking: the rebuild marks §2's landing row
    /// without giving the list the keyboard, which is the uninvited jump §2
    /// forbids and the mechanism ticket 04 measured silent. The first row
    /// stands in where the rule answers nothing — a Run whose first gesture is
    /// this one has no row to keep — so Down and Tab always land on a row NVDA
    /// reads.
    fn narrow(
        &self,
        tab: &ScopeTab,
        criteria: Criteria,
        speaks: impl FnOnce(usize, usize) -> Announcement,
    ) {
        tab.page.debounce.stop();
        tab.count_due.set(false);
        let concerned = tab.focused_entry().map(|(_, id)| id);
        tab.criteria.with_mut(|held| *held = criteria);
        let row = self.landing_row(tab, concerned).or(Some(0));
        tab.page.render_quiet(&tab.rows(&self.catalogue), row);
        self.sync();
        self.speak_view(tab, speaks);
    }

    /// Ctrl+E, View → Expanded Values: flips the one app-wide rendering flag
    /// (v0.2.0 §5).
    ///
    /// **Both Scope tabs re-render**, because the mode is not per Scope, and
    /// **nothing about a Working Copy is touched**: no Checkpoint, invisible
    /// to Undo and Redo both ways, so a Ctrl+Z under expanded mode shows the
    /// rolled-back Working Copy, still expanded.
    ///
    /// The order is the whole of it. What each tab was showing — its visible
    /// set, and the Entry it is on — is read **before** the flip: a list row
    /// is a position in the visible set, and after the flip that set can be a
    /// different one, so the old row read against the new membership would
    /// name another Entry.
    ///
    /// **How each list is redrawn is decided by whether its membership moved**
    /// (v0.2.0 §5). Where it did not, only the Path cells differ, and writing
    /// them touches no item state — which is what keeps the toggle silent
    /// under a list holding the keyboard focus, so its own message is what is
    /// heard and an arrow key re-reads the row. Where it did — the Filtered
    /// View case, where the two renderings are different haystacks — the list
    /// is rebuilt and lands on §2's row, exactly as any other membership
    /// change does. Focus is never *taken* either way.
    fn toggle_expansion(&self) {
        let showing = self
            .tabs
            .each_ref()
            .map(|tab| (tab.visible(), tab.focused_entry().map(|(_, id)| id)));
        let mode = self.rendering.toggle();
        for (tab, (visible, concerned)) in self.tabs.iter().zip(showing) {
            let rows = tab.rows(&self.catalogue);
            if tab.visible() == visible {
                tab.page.render_paths(&rows);
            } else {
                tab.page
                    .render_quiet(&rows, self.landing_row(tab, concerned));
            }
        }
        self.sync();
        self.announcer
            .announce(Announcement::ExpansionMode { mode });
        // Under a Filtered View the toggle changed membership, so the count
        // follows — through the same debounced path a typing pause uses, one
        // `filteredCountDelayMs` later and never combined into one msgid
        // (v0.2.0 §13 item 8). Only the visible Scope's: a count about a list
        // the user is not on would be noise, and arrival speech covers the
        // return.
        let Some(tab) = self.active_tab().filter(|tab| tab.narrowed()) else {
            return;
        };
        tab.count_due.set(true);
        let delay = self.settings.with(SettingsFile::filtered_count_delay_ms) as i32;
        tab.page.debounce.start(delay, true);
    }

    /// ESC in the Search field: clears the text and returns focus to the list
    /// (v0.2.0 §3) — the second half honouring `searchEscapeReturnsFocus`,
    /// the first happening either way. One gesture, one meaning: on an
    /// already-idle field it still returns focus and says nothing.
    fn clear_search(&self, tab: &ScopeTab) {
        // `change_value` — never `set_value` — because the programmatic clear
        // must not fire the typing path on top of this one.
        tab.page.search.change_value("");
        // The Search half alone, never `narrowed`: ESC clears the text, so a
        // Scope narrowed only by its Filter has nothing here to change — and
        // "ESC on an already-empty field says nothing" is one rule, whether or
        // not a Filter is standing (v0.2.0 §3). A gesture that says nothing
        // also cancels nothing: an Expansion toggle's owed count is still owed,
        // and `narrow` is what would have taken it (v0.2.0 §13 item 8).
        if tab.searching() {
            let cleared = tab
                .criteria
                .with(|criteria| Criteria::new("", criteria.filter));
            // Announcement 1 where the clear left no Filtered View at all, and
            // item 9's count where a Filter is still narrowing — "ESC into a
            // still-filtered view" is one of that item's own occasions
            // (v0.2.0 §13 items 1 and 9).
            self.narrow(tab, cleared, |shown, total| Announcement::FilteredCount {
                shown,
                total,
            });
        }
        if self
            .settings
            .with(SettingsFile::search_escape_returns_focus)
        {
            tab.page.focus_list();
        }
    }

    /// The view's voice, one shape for both count Announcements (v0.2.0 §13):
    /// with no Filtered View, Announcement 1; narrowed, the count `narrowed`
    /// builds — item 9 for a criteria change, item 10 for an arrival — gated
    /// by `speakFilteredCount`, which items 9/10/11 answer to and
    /// Announcement 1 does not. The zero case is the msgid's own business.
    fn speak_view(&self, tab: &ScopeTab, narrowed: impl FnOnce(usize, usize) -> Announcement) {
        let total = tab.session.with(|session| session.entries().len());
        if !tab.narrowed() {
            self.announcer.announce(Announcement::EntryCount {
                scope: tab.scope,
                count: total,
            });
        } else if self.settings.with(SettingsFile::speak_filtered_count) {
            self.announcer
                .announce(narrowed(tab.visible().len(), total));
        }
    }

    /// The arrival voice — tab activation and Refresh: Announcement 1, or the
    /// Scope-named filtered count (item 10) while the Scope has a Filtered
    /// View (v0.2.0 §13 items 1 and 10).
    fn speak_scope_status(&self, tab: &ScopeTab) {
        let scope = tab.scope;
        self.speak_view(tab, move |shown, total| Announcement::ScopeFilteredCount {
            scope,
            shown,
            total,
        });
    }

    /// Help → About: what this build is (spec §15, §16).
    ///
    /// Name, version and licence, in the title, because the title is what NVDA
    /// speaks of a dialog. For an **unsigned** binary this is not decoration:
    /// it and the exe's `VERSIONINFO` are the only two places the application
    /// says who it is, and the one of them a screen-reader user can reach
    /// without leaving the keyboard.
    ///
    /// The version comes from Cargo rather than from a constant of ours, so
    /// there is one version in the build and no second place to forget. That
    /// it agrees with the `VERSIONINFO` compiled into the exe is gated by
    /// `pathmaster-core`'s `versioninfo.rs`, which reads the resource script
    /// without linking wxWidgets to do it.
    fn about(&self) {
        let version = env!("CARGO_PKG_VERSION");
        question::inform(&self.frame, &self.catalogue.about_dialog(version));
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
                question::warn(&self.frame, &translate(msgids::DIALOG_ELEVATION_CANCELLED));
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
    /// **before** it is shown, and what it answers is recorded and written
    /// afterwards — `settings` lives behind [`Scoped`] (the Search debounce's
    /// tick reads it too), whose closures nothing can escape: everything the
    /// dialog needs is owned before its modal loop starts (ADR-0011).
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
        let opening = self.settings.with(SettingsFile::choices);
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
        let mut amended = self.settings.with(SettingsFile::clone);
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
            question::warn(&self.frame, &translate(msgids::DIALOG_SETTINGS_UNWRITABLE));
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
        self.settings.with_mut(|settings| *settings = amended);
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
        // Owned out of the page's cell before the Session is touched; the
        // `activate` below — a dispatch — runs with every closure closed.
        let Some((scope, entries, value_type)) = self.backups.restore_payload() else {
            return;
        };
        let tab = self.tab_of(scope);
        if !tab
            .session
            .with_mut(|session| session.restore(entries, value_type))
        {
            return;
        }
        self.activate(scope);
        // The first row, or — over a Snapshot that restored nothing — the list
        // itself, which is where `focus_row` lands when there is no row. Past
        // the focus rule on purpose: a Restore concerns no single Entry, and
        // the top of the restored list is v0.1.0's contract.
        self.after_edit_at(tab, Some(0));
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
        let added = tab.session.with_mut(|session| {
            if convert {
                let mut added = None;
                session.batch(Operation::ChangeValueType, |working| {
                    working.set_value_type(ValueType::RegExpandSz);
                    added = working.add(&text);
                    added
                });
                added
            } else {
                session.add(&text)
            }
        });
        self.after_edit(tab, added);
    }

    /// Edit opens the same dialog over the focused visible Entry's raw text.
    /// Focus lands back on the edited row whatever the outcome (spec §6 D7) —
    /// unless the edit took the Entry out of the match set, in which case it
    /// vanishes at OK and §2's focus rule lands on the same visual position.
    fn edit(&self, tab: &ScopeTab) {
        let Some((_, id)) = tab.focused_entry() else {
            return;
        };
        let Some(raw) = tab.session.with(|session| {
            session
                .entries()
                .iter()
                .find(|entry| entry.id() == id)
                .map(|entry| entry.raw().to_string())
        }) else {
            return;
        };
        let title = translate(msgids::DIALOG_EDIT_ENTRY);
        let Some(text) = entry_dialog::ask_for_entry(&self.frame, &self.catalogue, &title, &raw)
        else {
            return;
        };
        let convert = self.convert_or_keep(tab, &text);
        tab.session.with_mut(|session| {
            if convert {
                session.batch(Operation::ChangeValueType, |working| {
                    working.set_value_type(ValueType::RegExpandSz);
                    working.edit(id, &text);
                    Some(id)
                });
            } else {
                session.edit(id, &text);
            }
        });
        self.after_edit(tab, Some(id));
    }

    /// Delete has no confirmation — undo is the safety net (spec §6 D4).
    /// Focus stays at the same visual position, clamped to the new last row —
    /// §2's rule with the concerned Entry gone — and the row NVDA reads there
    /// is the whole of the feedback.
    fn delete(&self, tab: &ScopeTab) {
        let Some((_, id)) = tab.focused_entry() else {
            return;
        };
        if !tab.session.with_mut(|session| session.delete(id)) {
            return;
        }
        self.after_edit(tab, None);
    }

    /// One Move Up or Move Down, one Checkpoint. Moving the first Entry up is
    /// not an operation and changes nothing, including focus.
    fn move_entry(&self, tab: &ScopeTab, command: Command) {
        let Some((_, id)) = tab.focused_entry() else {
            return;
        };
        let moved = tab.session.with_mut(|session| match command {
            Command::MoveUp => session.move_up(id),
            _ => session.move_down(id),
        });
        if !moved {
            return;
        }
        self.after_edit(tab, Some(id));
    }

    /// Undo and Redo restore a Checkpoint, move focus to the Entry it hints,
    /// and speak Announcement 4 — or 5, when the step took the Working Copy
    /// back across an Apply (spec §10.1). The operation name is the one thing
    /// focus cannot say.
    fn undo_redo(&self, tab: &ScopeTab, command: Command) {
        // One match, because the command decides two things that must not be
        // allowed to disagree: which way the history is walked, and which of
        // Announcement 4's two sentences says so.
        let (direction, outcome) = tab.session.with_mut(|session| match command {
            Command::Redo => (UndoDirection::Redo, session.redo()),
            _ => (UndoDirection::Undo, session.undo()),
        });
        let Some(outcome) = outcome else { return };
        self.after_edit(tab, outcome.focus);
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
        if !tab.session.with_mut(Session::cancel) {
            return;
        }
        self.after_edit(tab, None);
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
        let dirty = tab.session.with(Session::is_dirty);
        if dirty && !self.confirm(msgids::DIALOG_REFRESH_DISCARDS) {
            return;
        }
        let Ok(raw) = tab.key().read() else { return };
        let landing = tab.adopt(raw);
        self.after_edit(tab, landing);
        // Announcement 1 — or, while this Scope has a Filtered View, item 10:
        // the criteria survive a Refresh untouched (v0.2.0 §2), so what is
        // spoken is what the narrowed list now shows.
        self.speak_scope_status(tab);
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
    /// Everything the run needs is copied out **before** it is called — owned
    /// values, because the run opens modal dialogs and nothing may still be
    /// inside a scoped-access closure when one's nested event loop starts
    /// (ADR-0008, ADR-0011).
    ///
    /// A run with no Data Directory has nowhere to put the backup that must
    /// precede any write, so there is no run to make. It is unreachable in
    /// practice — such a run is Read-only Data, whose Sessions are all
    /// non-writable, so nothing is dirty and neither Ctrl+S nor the
    /// close-confirm can reach here — but the Data Directory is an `Option`
    /// and this is what its `None` means.
    fn apply_scopes(&self, order: &[Scope]) -> Option<apply::Outcome> {
        let data_dir = self.run.data_dir()?;
        // Read out here rather than in the struct below: the scoped access
        // must be closed before the run opens its dialogs, not live inside
        // the call expression across every one of them.
        let max_backups = self.settings.with(SettingsFile::max_backups);
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
    /// Everything in it is owned, which is what lets the caller hand the
    /// result to a sequence that opens dialogs.
    fn scope_input(&self, scope: Scope) -> ScopeInput {
        let tab = self.tab_of(scope);
        ScopeInput {
            scope,
            key: tab.key(),
            entries: tab.raw_entries(),
            value_type: tab.session.with(Session::value_type),
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
                    // disabled out from under the user: asked under the scoped
                    // access, moved after it has closed.
                    let stranded = self.with_availability(Some(tab), |availability| {
                        tab.page.focus_stranded(availability)
                    });
                    if stranded {
                        tab.page.focus_list();
                    }
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
                    let landing = tab.adopt(found);
                    let row = self.landing_row(tab, landing);
                    tab.page.render(&tab.rows(&self.catalogue), row);
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
    /// the sequence does not keep.
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
            .filter(|tab| tab.session.with(Session::is_dirty))
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
        let mut amended = self.settings.with(SettingsFile::clone);
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
        let asks = tab.session.with(|session| {
            session.value_type() == ValueType::RegSz && has_variable_reference(text)
        });
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

    /// §2's focus rule, answered as the view row to land on: (1) the Entry the
    /// operation concerned, if the view shows it; (2) else the visual position
    /// the user was already at — `render`'s clamp makes "else the last visible
    /// row" fall out of the same number; (3) `None` over an empty list, where
    /// focus stays on the list itself and never jumps to the Search field
    /// uninvited.
    ///
    /// Asked **before** the rebuild: `focused_row` still reads the list as the
    /// user left it, which is what "same visual position" means.
    fn landing_row(&self, tab: &ScopeTab, concerned: Option<EntryId>) -> Option<usize> {
        concerned
            .and_then(|id| tab.view_row_of(id))
            .or_else(|| tab.page.focused_row())
    }

    /// Redraws the Scope, lands focus by §2's rule on the Entry the operation
    /// concerned (`None` for an operation that concerned no surviving Entry),
    /// points every control at the state that now holds, and asks for a fresh
    /// pass — the tail of every operation, so no screen can show one Working
    /// Copy while a menu reads another.
    fn after_edit(&self, tab: &ScopeTab, concerned: Option<EntryId>) {
        let row = self.landing_row(tab, concerned);
        self.after_edit_at(tab, row);
    }

    /// [`after_edit`](Self::after_edit) with the landing row already decided —
    /// Restore's first-row landing comes here directly, past the focus rule.
    ///
    /// The redraw carries the *last* pass's findings, read by Entry id: a row
    /// that only moved keeps its Status words, and the one whose text just
    /// changed shows none until the new pass lands (spec §7, FR-diag-async).
    fn after_edit_at(&self, tab: &ScopeTab, row: Option<usize>) {
        tab.page.render(&tab.rows(&self.catalogue), row);
        self.sync();
        self.request_pass();
    }

    /// Activates a Scope's tab.
    ///
    /// One line, with a cast and a hazard in it, and both belong in one place:
    /// `set_selection` runs the page-changed handler **synchronously**, and
    /// that handler reads every Session in the window — a dispatch, which is
    /// why no scoped-access closure may call this (ADR-0011).
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

    /// Reads `tab`'s availability through a closure — [`Availability`] borrows
    /// the Session, so it lives inside the scoped access and the answer comes
    /// out owned. `None` is a tab that is not a Scope, whose availability is
    /// the Run's facts alone.
    ///
    /// The view facts ride along (v0.2.0 §2): each tab answers under **its
    /// own** Filtered View, so a Scope narrowed in the background keeps its
    /// buttons honest however long the user looks elsewhere.
    fn with_availability<R>(
        &self,
        tab: Option<&ScopeTab>,
        read: impl FnOnce(&Availability) -> R,
    ) -> R {
        let data_dir = self.run.data_dir().is_some();
        let elevated = self.run.elevated();
        let expansion = self.rendering.mode();
        match tab {
            Some(tab) => {
                let narrowed = tab.narrowed();
                let visible_rows = tab.visible().len();
                let filter = Some(tab.filter());
                tab.session.with(|session| {
                    read(&Availability {
                        session: Some(session),
                        narrowed,
                        visible_rows,
                        data_dir,
                        elevated,
                        expansion,
                        filter,
                    })
                })
            }
            // The Backups tab, where every View item is disabled — and where
            // the mode rides along all the same, because a disabled Expanded
            // Values keeps a readable check mark (v0.2.0 §5). The Filter does
            // not: it is per Scope, and this tab is not one, so the radio
            // marks are left showing the Scope the user came from.
            None => read(&Availability {
                session: None,
                narrowed: false,
                visible_rows: 0,
                data_dir,
                elevated,
                expansion,
                filter: None,
            }),
        }
    }

    fn sync(&self) {
        self.sync_for(self.active_tab());
    }

    /// Points the menu, the buttons and the status bar at `active`'s Session.
    /// Taken as an argument rather than read back, because the notebook's own
    /// selection lags the page-changed event that carries it.
    fn sync_for(&self, active: Option<&ScopeTab>) {
        self.with_availability(active, |availability| self.menus.sync(availability));
        for tab in &self.tabs {
            self.with_availability(Some(tab), |availability| {
                tab.page.sync_buttons(availability)
            });
        }
        // Restore is worth something only over a row that can be loaded into a
        // Session that can be written: a Corrupted Snapshot has nothing to
        // load, and System unelevated or a Read-only Data run has nowhere to
        // load it (spec §8). Both read as a disabled button.
        let restorable = self
            .backups
            .restore_target()
            .is_some_and(|scope| self.tab_of(scope).session.with(Session::writable));
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
    question::warn(parent, &translate(msgids::DIALOG_SETTINGS_UNREADABLE));
}
