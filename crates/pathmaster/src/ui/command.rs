//! The twenty-nine commands the window carries, and the menus they live in
//! (spec §15, §5, §6; v0.2.0 §9, §12).
//!
//! One enum is the whole map: it names the menu items, the menu each belongs
//! to, the ids the menu events arrive under, the accelerators, the per-Scope
//! buttons, and — in one `match` — when each is available. That matters more
//! than tidiness here, because `wxAcceleratorTable` is absent from wxdragon at
//! every level: a keyboard shortcut can only exist as a menu item's label, so
//! **every shortcut in PathMaster must have a menu home**, and the menu's
//! enabled state is therefore also the shortcut's. Ctrl+S is why Apply is in
//! this enum at all rather than being a button the window binds on its own.
//!
//! Accelerators are appended by this code and never typed into the Catalogue:
//! a translated `"\tCtrl+Я"` would not misread, it would delete the shortcut
//! (ADR-0004).

use pathmaster_core::expansion::Mode;
use pathmaster_core::filtered::Filter;
use pathmaster_core::msgids;
use pathmaster_core::session::Session;
use wxdragon::prelude::*;

use crate::catalog::translate;

/// One user-visible command.
///
/// Twenty-three of them are about a Scope; the last six are not, and are here
/// for the reason the enum exists at all: they are menu items, and a menu item's
/// id, label and enabled state are answered in one place or in three.
///
/// **Restore is deliberately not one of them.** §15 gives it no menu item and
/// no accelerator — the Backups tab covers it — and a button on that tab is not
/// a Scope button, which is what [`button_label`](Command::button_label) means
/// here. It has exactly one route, so the rule this enum enforces has nothing
/// to enforce for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Add,
    Edit,
    Delete,
    /// The one per-Entry command that reads rather than changes: it puts the
    /// focused visible Entry's **currently displayed** rendering on the
    /// clipboard (v0.2.0 §8).
    Copy,
    MoveUp,
    MoveDown,
    /// Edit → "Fix Issues…": the modal, per-Scope repair surface over the
    /// active Scope (v0.2.0 §7). The one Edit command that is about the whole
    /// Working Copy rather than one Entry — which is why it sits after the Move
    /// pair and before the history block it feeds a single Checkpoint into.
    FixIssues,
    Undo,
    Redo,
    Apply,
    Cancel,
    Refresh,
    Search,
    /// One of the seven exclusive Filter states, each a `wxITEM_RADIO` item in
    /// the View → Filter submenu (v0.2.0 §4). The state it carries is both
    /// what the item's mark reads and what choosing it sets, so a state can
    /// neither be shown without being reachable nor set without being shown.
    Filter(Filter),
    /// Ctrl+I's own item: the coarse axis, `All` ⇄ `With issues`.
    ///
    /// A **plain** item with a constant label, deliberately (v0.2.0 §12): a
    /// radio item carrying the accelerator would fire that radio's selection
    /// rather than the toggle, and a check item would carry a mark that lies
    /// whenever one of the five per-type states is active. The submenu's radio
    /// marks are the state; this is only a gesture.
    ToggleIssuesFilter,
    ExpandedValues,
    /// Ctrl+T: opens the Tree View over the active Scope (v0.2.0 §6). The one
    /// View command that opens a dialog rather than changing the list — which
    /// is why its label carries the `…` and why it still lives here: what it
    /// shows is a reading of the list, snapshotted.
    PathTree,
    Settings,
    OpenBackupsFolder,
    RestartAsAdministrator,
    Exit,
    /// F1: the User Guide, written into the Data Directory and handed to the
    /// browser (v0.2.0 §9). Enabled in every state this application can be in
    /// — how to use it is true on the Backups tab and in Read-only Data alike.
    UserGuide,
    About,
}

/// What a command's availability is decided from.
///
/// The active Scope's Editing Session answers for the ten that edit one, and
/// `None` — the Backups tab, which is not a Scope — closes every one of them.
/// The Run's own facts answer for Open Backups Folder and for Restart as
/// Administrator, and nothing at all answers for Exit, for Settings, for About
/// or for the User Guide: a way out of the application is available on every
/// tab and in every state, dirty Sessions included — that is what the
/// close-confirm is for — the settings belong to the Run rather than to
/// whichever tab happens to be showing, and how to use the application is true
/// in every state it can be in (v0.2.0 §9).
pub struct Availability<'a> {
    pub session: Option<&'a Session>,
    /// Whether this Scope's Filtered View is active — a non-empty Search text
    /// **or** a narrowing Filter, which is [`Criteria::narrowing`]'s one
    /// answer. A reorder's effect concerns positions the user cannot see, and
    /// Add appends below them, so a narrowed view closes Move Up, Move Down
    /// and Add (v0.2.0 §2).
    ///
    /// [`Criteria::narrowing`]: pathmaster_core::filtered::Criteria::narrowing
    pub narrowed: bool,
    /// How many Entries the view now shows — the Scope's whole count when
    /// nothing narrows it. Edit, Delete and Copy act on the focused **visible**
    /// Entry, so zero visible rows closes all three even over a non-empty
    /// Scope.
    pub visible_rows: usize,
    /// How many of this Scope's Entries the Fix Issues dialog could act on —
    /// [`fix::fixable`] over the last completed pass (v0.2.0 §7).
    ///
    /// Not merely "this Scope has Issues": an all-Relative or Over-length-only
    /// Scope would open an empty dialog, and menu enablement is the only
    /// indicator this command has. It is counted over the whole Working Copy
    /// and never over the Filtered View — the surface is per Scope, so a
    /// narrowing must not be able to close it.
    ///
    /// [`fix::fixable`]: pathmaster_core::fix::fixable
    pub fixable: usize,
    /// Whether this Run has a Data Directory at all. The one Run that has none
    /// does not know where it is (`ReadOnlyReason::OwnLocationUnknown`), and so
    /// has no folder of Snapshots to show.
    pub data_dir: bool,
    /// Whether this Run is elevated — decided once at startup, a property of
    /// the process (spec §9). It answers for exactly one command: Restart as
    /// Administrator is disabled where it could only restart into what the
    /// user already has.
    pub elevated: bool,
    /// How the application is rendering Entries right now — app-wide, both
    /// Scopes alike (v0.2.0 §5).
    ///
    /// It decides no command's *availability*: it is the state the one
    /// `wxITEM_CHECK` item carries, and it rides here because an item's
    /// enabled state and its check mark are written in one pass and must not
    /// be answered in two places.
    pub expansion: Mode,
    /// The active Scope's Filter — the state the submenu's radio mark reads
    /// (v0.2.0 §4). Like [`expansion`](Self::expansion) it decides no
    /// command's availability, and unlike it, it is per Scope.
    ///
    /// `None` is the Backups tab, which is not a Scope and has no Filter: the
    /// marks are then **left exactly as they are**, still showing the last
    /// Scope's state under items that read as disabled. Writing `All` there
    /// would be a mark claiming a narrowed Scope is not.
    pub filter: Option<Filter>,
}

/// What kind of menu item a command appends — which decides what its mark can
/// say, and so is fixed at build time and never afterwards. Appending the
/// wrong kind is an item whose mark can never be set at all.
///
/// Spelled out rather than reusing wx's own `ItemKind`, whose fourth variant is
/// a separator — a thing no command is, and an arm every `match` here would
/// have to answer with a lie.
enum MenuItemKind {
    /// A plain command item: it does something and carries no state.
    Plain,
    /// `wxITEM_CHECK`: a mark NVDA reads in both directions.
    Check,
    /// `wxITEM_RADIO`: one of a contiguous group of which exactly one is
    /// selected, and whose selected state NVDA reads (ticket 16, probes 3).
    Radio,
}

impl Command {
    /// Every command, in **button** order — §15's Add, Edit, Delete, Move Up,
    /// Move Down, Apply, Cancel — which is also the order each menu takes its
    /// own items in, since [`menu`](Self::menu) preserves it.
    pub const ALL: [Command; 29] = [
        Command::Add,
        Command::Edit,
        Command::Delete,
        // Copy closes the per-Entry group rather than opening it: it is the
        // group's one read-only member (v0.2.0 §12).
        Command::Copy,
        Command::MoveUp,
        Command::MoveDown,
        Command::FixIssues,
        Command::Undo,
        Command::Redo,
        Command::Apply,
        Command::Cancel,
        Command::Refresh,
        Command::Search,
        // The seven states, **read positionally off `Filter::ALL`** rather
        // than named again here: the submenu's order is v0.2.0 §4's order, and
        // indexing is what makes that a compile-time fact instead of two lists
        // a reader has to compare. The binary is bin-only and the Release
        // Checklist is its coverage (ADR-0007), so a rule worth keeping here
        // has to be structural or it is not kept at all.
        Command::Filter(Filter::ALL[0]), // All
        Command::Filter(Filter::ALL[1]), // With issues
        Command::Filter(Filter::ALL[2]), // Missing
        Command::Filter(Filter::ALL[3]), // Relative
        Command::Filter(Filter::ALL[4]), // Quoted
        Command::Filter(Filter::ALL[5]), // Duplicate
        Command::Filter(Filter::ALL[6]), // Empty
        Command::ToggleIssuesFilter,
        Command::ExpandedValues,
        // The dialog closes the View menu: §12's order is the two narrowing
        // criteria, then the rendering, then this.
        Command::PathTree,
        Command::Settings,
        Command::OpenBackupsFolder,
        Command::RestartAsAdministrator,
        Command::Exit,
        // Help, in §12's order: the guide first, About last.
        Command::UserGuide,
        Command::About,
    ];

    /// The menus, in menu-bar order — the whole of spec §15's table, with
    /// v0.2.0 §12's View between Edit and Tools: commands that change *what
    /// the list shows* live there.
    const MENUS: [(&'static str, &'static str); 5] = [
        (msgids::MENU_TITLE_FILE, msgids::MENU_GROUP_FILE),
        (msgids::MENU_TITLE_EDIT, msgids::MENU_GROUP_EDIT),
        (msgids::MENU_TITLE_VIEW, msgids::MENU_GROUP_VIEW),
        (msgids::MENU_TITLE_TOOLS, msgids::MENU_GROUP_TOOLS),
        (msgids::MENU_TITLE_HELP, msgids::MENU_GROUP_HELP),
    ];

    /// The id this command's menu item is appended under, and the id its
    /// events arrive with. Fixed offsets from `ID_HIGHEST` so no two commands
    /// can drift onto one number.
    pub fn id(self) -> Id {
        ID_HIGHEST
            + 1
            + Self::ALL
                .iter()
                .position(|command| *command == self)
                .expect("every command is in ALL") as Id
    }

    /// The command a menu event belongs to, or `None` for an id from
    /// somewhere else entirely.
    pub fn from_id(id: Id) -> Option<Command> {
        Self::ALL.into_iter().find(|command| command.id() == id)
    }

    /// The menu this command's item lives under. §15 puts Apply in File and
    /// everything else in Edit — and the mnemonic gate reads the same grouping
    /// through [`msgids::REGISTRY`], so an item cannot sit in one menu and be
    /// gated against another's siblings.
    pub fn menu(self) -> &'static str {
        // Exhaustive, like every other `match` over this enum: a catch-all
        // would land the next command someone adds in whichever menu happened
        // to be the default, silently.
        match self {
            Command::Apply | Command::Exit => msgids::MENU_GROUP_FILE,
            Command::Add
            | Command::Edit
            | Command::Delete
            | Command::Copy
            | Command::MoveUp
            | Command::MoveDown
            | Command::FixIssues
            | Command::Undo
            | Command::Redo
            | Command::Cancel
            | Command::Refresh => msgids::MENU_GROUP_EDIT,
            Command::Search
            | Command::Filter(_)
            | Command::ToggleIssuesFilter
            | Command::ExpandedValues
            | Command::PathTree => msgids::MENU_GROUP_VIEW,
            Command::Settings | Command::OpenBackupsFolder | Command::RestartAsAdministrator => {
                msgids::MENU_GROUP_TOOLS
            }
            Command::UserGuide | Command::About => msgids::MENU_GROUP_HELP,
        }
    }

    /// The submenu this command's item sits in, named by the title that
    /// submenu is appended under — `None` for an item that sits directly in
    /// the menu [`menu`](Self::menu) names.
    ///
    /// The seven Filter states are the only ones, and their submenu takes the
    /// place of the **first** of them in [`ALL`](Self::ALL) — so §12's View
    /// order is read off one list rather than stated twice.
    fn submenu(self) -> Option<&'static str> {
        // Exhaustive, like every other `match` over this enum: an item that
        // silently defaulted to the menu bar would be one the user finds in a
        // different place from the one this file describes.
        match self {
            Command::Filter(_) => Some(msgids::MENU_FILTER),
            Command::Add
            | Command::Edit
            | Command::Delete
            | Command::Copy
            | Command::MoveUp
            | Command::MoveDown
            | Command::FixIssues
            | Command::Undo
            | Command::Redo
            | Command::Apply
            | Command::Cancel
            | Command::Refresh
            | Command::Search
            | Command::ToggleIssuesFilter
            | Command::ExpandedValues
            | Command::PathTree
            | Command::Settings
            | Command::OpenBackupsFolder
            | Command::RestartAsAdministrator
            | Command::Exit
            | Command::UserGuide
            | Command::About => None,
        }
    }

    /// The menu item's label: Catalogue text with its accelerator appended.
    pub fn menu_label(self) -> String {
        let label = translate(match self {
            Command::Apply => msgids::MENU_APPLY,
            Command::Exit => msgids::MENU_EXIT,
            Command::Add => msgids::MENU_ADD_ENTRY,
            Command::Edit => msgids::MENU_EDIT_ENTRY,
            Command::Delete => msgids::MENU_DELETE_ENTRY,
            Command::Copy => msgids::MENU_COPY,
            Command::MoveUp => msgids::MENU_MOVE_UP,
            Command::MoveDown => msgids::MENU_MOVE_DOWN,
            Command::FixIssues => msgids::MENU_FIX_ISSUES,
            Command::Undo => msgids::MENU_UNDO,
            Command::Redo => msgids::MENU_REDO,
            Command::Cancel => msgids::MENU_CANCEL,
            Command::Refresh => msgids::MENU_REFRESH,
            Command::Search => msgids::MENU_SEARCH,
            // The state's own name — the Status column's word, for the five
            // type states — so no Issue is called two things (v0.2.0 §4).
            Command::Filter(filter) => filter.catalogue_msgid(),
            Command::ToggleIssuesFilter => msgids::MENU_TOGGLE_ISSUES_FILTER,
            Command::ExpandedValues => msgids::MENU_EXPANDED_VALUES,
            Command::PathTree => msgids::MENU_PATH_TREE,
            Command::Settings => msgids::MENU_SETTINGS,
            Command::OpenBackupsFolder => msgids::MENU_OPEN_BACKUPS_FOLDER,
            Command::RestartAsAdministrator => msgids::MENU_RESTART_AS_ADMIN,
            Command::UserGuide => msgids::MENU_USER_GUIDE,
            Command::About => msgids::MENU_ABOUT,
        });
        match self.accelerator() {
            Some(accelerator) => format!("{label}\t{accelerator}"),
            None => label,
        }
    }

    /// The label of this command's button under a Scope's list, or `None`
    /// where §15 gives it none — Undo, Redo and Refresh are menu and keyboard
    /// commands, and the Tools items are not about a Scope at all. Which
    /// commands have a button is therefore answered here and nowhere else; the
    /// Tab order is `ALL`'s order, filtered by this.
    ///
    /// The English differs from both the menu item and the operation name
    /// Announcement 4 speaks, because the three need different Ukrainian
    /// (ADR-0004).
    pub fn button_label(self) -> Option<String> {
        let msgid = match self {
            Command::Add => msgids::BUTTON_ADD,
            Command::Edit => msgids::BUTTON_EDIT,
            Command::Delete => msgids::BUTTON_DELETE,
            Command::MoveUp => msgids::BUTTON_MOVE_UP,
            Command::MoveDown => msgids::BUTTON_MOVE_DOWN,
            Command::Apply => msgids::BUTTON_APPLY,
            Command::Cancel => msgids::BUTTON_CANCEL,
            // Copy is menu-and-keyboard like Undo beside it: §15's seven
            // buttons are the whole row, and v0.2.0 adds none — a Ctrl+C
            // with a button under the list it copies from would be an
            // eighth control in the Tab cycle for a gesture that already
            // has a home.
            //
            // Every View command is menu-only too: the Filter has **no
            // on-window control** at all (v0.2.0 §4), and neither does its
            // toggle.
            Command::Copy
            | Command::FixIssues
            | Command::Undo
            | Command::Redo
            | Command::Refresh
            | Command::Search
            | Command::Filter(_)
            | Command::ToggleIssuesFilter
            | Command::ExpandedValues
            | Command::PathTree
            | Command::Settings
            | Command::OpenBackupsFolder
            | Command::RestartAsAdministrator
            | Command::Exit
            | Command::UserGuide
            | Command::About => return None,
        };
        Some(translate(msgid))
    }

    /// The keystroke wx parses out of the label. F2 and Del are the list's own
    /// gestures given a menu home; Enter and double-click reach the same
    /// dialog through the list's activation event instead (spec §6, §15).
    ///
    /// Alt+F4 is Windows' own gesture given one too. Naming it here does not
    /// create it — the system closes a window on Alt+F4 whatever any menu
    /// says — it makes the item **read** as the shortcut it already is, which
    /// is the only way a screen-reader user learns of one (ADR-0004).
    ///
    /// **Which commands carry one at all** is two rules, and between them they
    /// leave nothing over:
    ///
    /// - A command about a **Scope** carries a key. Cancel is the one
    ///   exception, and deliberately: it discards every unapplied edit in the
    ///   Session, so its reach is the menu item a user has to open and choose,
    ///   and never a keystroke a slip can land on. The Filter states are not
    ///   an exception but an impossibility — a radio item's key fires its own
    ///   selection rather than the gesture it was meant to be, which is why
    ///   Ctrl+I sits on a plain item beside them.
    /// - A command about the **Run** carries a key only where the platform
    ///   already owns one and the item does no more than name it: Alt+F4 and
    ///   F1. Settings, Open Backups Folder, Restart as Administrator and About
    ///   would each need one invented for them, and carry none.
    ///
    /// What this does not relax is the module's own rule: every shortcut needs
    /// a menu home, because there is nowhere else to put one. Not every item
    /// needs a shortcut.
    fn accelerator(self) -> Option<&'static str> {
        match self {
            // Ctrl+N, which Microsoft's shortcut guidance both designates for
            // "new" and reserves against being given to anything else.
            //
            // **Insert is what a list editor would otherwise reach for, and it
            // is unavailable here**: NVDA takes both Insert keys as its own
            // modifier by default, so the keystroke never arrives at all. The
            // convention is sound and the reason it loses is particular to the
            // user this application is built for.
            Command::Add => Some("Ctrl+N"),
            Command::Edit => Some("F2"),
            Command::Delete => Some("Del"),
            // **Scoped by the platform, not by us** (v0.2.0 §8): wxMSW text
            // entries claim Ctrl+C/X/V/A *before* accelerator translation
            // (`wxMSWTextEntryShouldPreProcessMessage`, pinned 3.3.3), so this
            // label can never steal the Search field's or a dialog field's own
            // copy. No focus-checking handler, no dynamic table — and
            // everywhere else it fires frame-wide like every Entry command.
            Command::Copy => Some("Ctrl+C"),
            Command::MoveUp => Some("Alt+Up"),
            Command::MoveDown => Some("Alt+Down"),
            // Ctrl+Shift+I: the Issues axis Ctrl+I already names, made to act
            // on them rather than to narrow to them (v0.2.0 §7, §12). Ctrl
            // because the dialog is about the whole Working Copy and not about
            // the focused row — the object-scoped commands are the F2 class —
            // and Shift because that is the modifier a key complementing
            // another one takes. Outside a named application NVDA binds no
            // Ctrl+Shift+letter of its own, so nothing of the screen reader's
            // is shadowed.
            Command::FixIssues => Some("Ctrl+Shift+I"),
            Command::Undo => Some("Ctrl+Z"),
            Command::Redo => Some("Ctrl+Y"),
            Command::Apply => Some("Ctrl+S"),
            Command::Exit => Some("Alt+F4"),
            Command::Refresh => Some("F5"),
            // Fires frame-wide even with focus already in the Search field —
            // wxMSW's text-entry preprocessing claims only Ctrl+C/X/V/A
            // (v0.2.0 §12), which is intended: one keystroke, one meaning.
            Command::Search => Some("Ctrl+F"),
            // Ctrl+E, the key the ticket-16 prototype carried through the
            // user's own NVDA verification (v0.2.0 §12).
            Command::ExpandedValues => Some("Ctrl+E"),
            // The coarse Filter axis (v0.2.0 §4, §12). Ctrl+I's "italic"
            // convention belongs to rich-text editors, which this is not.
            Command::ToggleIssuesFilter => Some("Ctrl+I"),
            // The Tree View (v0.2.0 §6, §12) — the PRD's Alt+T is a recorded
            // deviation: an Alt+letter shortcut shadows a mnemonic, and this
            // one would shadow the very menu it lives in.
            Command::PathTree => Some("Ctrl+T"),
            // The User Guide (v0.2.0 §9, §12). **In a dialog F1 does nothing**,
            // as a decision: a frame's accelerators do not reach a modal's own
            // event loop, and the alternative is an `EVT_CHAR_HOOK` in every
            // dialog as a standing obligation no gate would catch a future one
            // breaking. The against-silence rule governs commands that
            // *failed*; F1 in a dialog is an unbound key.
            Command::UserGuide => Some("F1"),
            // The five per-type states are menu-only, and so are the two
            // coarse ones: a radio item carrying a key would fire its own
            // selection rather than the toggle it was meant to be.
            //
            // Cancel is the one Scope command that could carry a key and does
            // not, and that is what it is for: it throws the Session's whole
            // unapplied history away, and the menu item is the only reach a
            // command like that should have.
            //
            // The last four are about the Run and not about a Scope, and none
            // of them has a key the platform already owns — there is no
            // Alt+F4 for "show me the settings".
            Command::Filter(_)
            | Command::Cancel
            | Command::Settings
            | Command::OpenBackupsFolder
            | Command::RestartAsAdministrator
            | Command::About => None,
        }
    }

    /// Whether this command is available — over the active Scope's Session,
    /// `None` being the Backups tab, which is not a Scope and offers no
    /// editing at all.
    ///
    /// A non-writable Session closes every command that edits, and a disabled
    /// item is how a screen reader is told so (spec §5, §15). **Refresh is not
    /// one of them**: it re-reads, and a Scope the user cannot edit is still
    /// one they can look at — Read-only Data "still reads, diagnoses and
    /// lists", and an unelevated System tab would otherwise never see an
    /// external change without a restart.
    pub fn enabled(self, available: &Availability) -> bool {
        // Exhaustive, like every `match` over this enum: the split is between
        // the commands that answer to the Run, the ones that answer to nothing
        // at all, and the ones that answer to a Scope — and the next command
        // someone adds has to say which it is.
        match self {
            // Not about a Scope: it shows this Run's own directory, so it is
            // available on the Backups tab, where there is no Session at all.
            Command::OpenBackupsFolder => available.data_dir,
            // About the Run alone: the one entry point into elevation
            // (spec §9, ADR-0005), available on every tab and in every state
            // that is not already elevated — Read-only Data included, where
            // this command is the standing remedy and the reason the state
            // never grows a second elevation offer. A dirty Session does not
            // close it either; the command runs through the close-confirm
            // flow, never around it.
            Command::RestartAsAdministrator => !available.elevated,
            // Not about anything. An application a dirty Session could disable
            // the way out of is one the user has to kill, and the whole of the
            // close-confirm is that they do not have to.
            //
            // Settings is the other: it is about the Run rather than a Scope,
            // and a Read-only Data run reaches it too. **That state is
            // answered inside the dialog**, which disables its own controls,
            // because an item reading as unavailable would say the settings
            // cannot be looked at — and in that run looking at them is exactly
            // what is still possible.
            //
            // About is the third, and the least conditional of them: it names
            // the build, which is true in every state this application can be
            // in — and a user checking what they are running is likeliest to
            // do it when something else has gone wrong.
            //
            // The User Guide is the fourth and the least conditional of all:
            // how to use the application is true in every state it can be in,
            // Backups tab and Read-only Data included (v0.2.0 §9) — and a run
            // that cannot write its own guide still opens the online copy.
            Command::Exit | Command::Settings | Command::About | Command::UserGuide => true,
            Command::Add
            | Command::Edit
            | Command::Delete
            | Command::Copy
            | Command::MoveUp
            | Command::MoveDown
            | Command::FixIssues
            | Command::Undo
            | Command::Redo
            | Command::Apply
            | Command::Cancel
            | Command::Refresh
            | Command::Search
            | Command::Filter(_)
            | Command::ToggleIssuesFilter
            | Command::ExpandedValues
            | Command::PathTree => available
                .session
                .is_some_and(|session| self.over(session, available)),
        }
    }

    /// What a Scope command asks of the Session it would act on — and, for the
    /// commands a Filtered View narrows, of the view (v0.2.0 §2).
    fn over(self, session: &Session, available: &Availability) -> bool {
        match self {
            Command::Refresh => true,
            // Read-only Data searches normally: a Run that cannot edit still
            // reads, diagnoses and lists (v0.2.0 §3).
            //
            // Expanded Values is the same kind of command and the same answer:
            // it changes how paths are read, not what they are, so a Session
            // nobody may edit is one whose rendering may still be changed.
            // What closes it is the Backups tab, where there is no Session at
            // all — like every other View item (v0.2.0 §5, §12).
            //
            // The Tree View is one of them and reads the most: it shows the
            // Filtered View as it stands, which a run that may edit nothing
            // still has.
            Command::Search
            | Command::Filter(_)
            | Command::ToggleIssuesFilter
            | Command::ExpandedValues
            | Command::PathTree => true,
            // Copy answers **above** the writability line for the same reason,
            // and it is the only Entry command that does: it reads the Working
            // Copy and never changes it (v0.2.0 §8, §12). An unelevated System
            // tab and a Read-only Data run are Scopes the user may look at, and
            // a row that can be read aloud is one that can be copied. What it
            // still asks is the same question Edit and Delete ask — a focused
            // **visible** Entry to act on, which an empty result set has none
            // of (v0.2.0 §2, §3).
            Command::Copy => available.visible_rows > 0,
            _ if !session.writable() => false,
            // Add appends at the end — a position a narrowed view may be
            // hiding, so it closes with Move Up and Move Down (v0.2.0 §2).
            Command::Add => !available.narrowed,
            // The focused **visible** Entry is what these act on: an empty
            // result set shows zero rows and closes both (v0.2.0 §2, §3).
            Command::Edit | Command::Delete => available.visible_rows > 0,
            Command::MoveUp | Command::MoveDown => {
                !available.narrowed && available.visible_rows > 0
            }
            // A repair surface with nothing to repair is a dialog that opens
            // empty, and this item's enabled state is the only indicator the
            // command has — the Status column and the StatusBar already say
            // there is work, so nothing here may say there is none (v0.2.0
            // §7). The count is the Scope's and not the view's: the narrowing
            // that closes Move Up does not close this.
            Command::FixIssues => available.fixable > 0,
            Command::Undo => session.can_undo(),
            Command::Redo => session.can_redo(),
            // Both are disabled while clean, and read as disabled (spec §5).
            // Apply asks nothing about whether the Data Directory can be
            // written: startup predicts, Apply verifies at write time
            // (ADR-0002) — and a Read-only Data run has already reached this
            // `match` through the non-writable Session above.
            Command::Apply | Command::Cancel => session.is_dirty(),
            // Not Scope commands, and never routed here — but answered rather
            // than caught, so that adding one cannot inherit a default.
            Command::Settings
            | Command::OpenBackupsFolder
            | Command::RestartAsAdministrator
            | Command::Exit
            | Command::UserGuide
            | Command::About => false,
        }
    }

    /// What kind of menu item this command appends (v0.2.0 §4, §5; ticket 16
    /// probes 1 and 3).
    ///
    /// Asked once, at build time — the kind is what an item's mark *can* say,
    /// and [`state`](Self::state) is what it says now.
    fn item(self) -> MenuItemKind {
        // Exhaustive, like every other `match` over this enum: the next
        // command someone adds has to say what kind of item it is, because an
        // item that silently carried no mark would read as a command in a menu
        // where its state is the whole point.
        match self {
            Command::ExpandedValues => MenuItemKind::Check,
            Command::Filter(_) => MenuItemKind::Radio,
            Command::Add
            | Command::Edit
            | Command::Delete
            | Command::Copy
            | Command::MoveUp
            | Command::MoveDown
            | Command::FixIssues
            | Command::Undo
            | Command::Redo
            | Command::Apply
            | Command::Cancel
            | Command::Refresh
            | Command::Search
            | Command::ToggleIssuesFilter
            | Command::PathTree
            | Command::Settings
            | Command::OpenBackupsFolder
            | Command::RestartAsAdministrator
            | Command::Exit
            | Command::UserGuide
            | Command::About => MenuItemKind::Plain,
        }
    }

    /// The state this item's mark now shows, or `None` for an item that has no
    /// mark — **and for one whose state nothing can answer**, whose mark is
    /// then left exactly as it stands.
    ///
    /// A mark is written whether or not its item is enabled: on the Backups
    /// tab every View item is disabled and Expanded Values' check mark stays
    /// readable there, because the mode is app-wide and a tab the command
    /// cannot be given from is not a tab it is off in (v0.2.0 §5). The Filter
    /// is per Scope rather than app-wide, so that tab answers `None` for it
    /// instead — the marks keep showing the Scope the user came from.
    fn state(self, available: &Availability) -> Option<bool> {
        match self {
            Command::ExpandedValues => Some(available.expansion.expanded()),
            Command::Filter(filter) => Some(available.filter? == filter),
            // No mark to write — answered rather than caught, so the next
            // state-carrying command has to say what its mark reads.
            Command::Add
            | Command::Edit
            | Command::Delete
            | Command::Copy
            | Command::MoveUp
            | Command::MoveDown
            | Command::FixIssues
            | Command::Undo
            | Command::Redo
            | Command::Apply
            | Command::Cancel
            | Command::Refresh
            | Command::Search
            | Command::ToggleIssuesFilter
            | Command::PathTree
            | Command::Settings
            | Command::OpenBackupsFolder
            | Command::RestartAsAdministrator
            | Command::Exit
            | Command::UserGuide
            | Command::About => None,
        }
    }
}

/// The menu bar, and the items in it that carry no command id of their own.
///
/// One submenu exists — View → Filter (v0.2.0 §4) — and `AppendSubMenu` gives
/// its title item no id, so [`MenuBar::enable_item`] cannot reach it. Keeping
/// the item is what lets the submenu itself read as disabled on the Backups
/// tab, rather than opening onto seven greyed states.
pub struct Menus {
    bar: MenuBar,
    /// The one submenu the bar holds, and the title it was appended under.
    /// `Option` and not a collection: v0.2.0 §12's bar has exactly one, and
    /// room for a second would be a promise [`submenu_slot`] does not make.
    submenu: Option<(&'static str, MenuItem)>,
}

impl Menus {
    /// Builds the bar from [`Command::MENUS`], [`Command::menu`] and
    /// [`Command::submenu`], gives it to `frame`, and keeps what syncing it
    /// needs. A command with no menu is a command with no shortcut — which
    /// for Ctrl+S would be a command with no way to reach it at all.
    ///
    /// Every item's help string is deliberately empty: wx writes it to the
    /// status bar as the user moves through the menu, and the status bar is
    /// command-only — nothing must-hear goes there (spec §10, §12).
    pub fn build(frame: &Frame) -> Menus {
        let mut bar = MenuBar::builder();
        let mut submenu = None;
        for (title, group) in Command::MENUS {
            let menu = build_menu(|command| command.menu() == group && command.submenu().is_none());
            if let Some((position, held)) = submenu_slot(group) {
                let items = build_menu(|command| command.submenu() == Some(held));
                submenu = menu
                    .insert_submenu(position, items, &translate(held), "")
                    .map(|item| (held, item));
            }
            bar = bar.append(menu, &translate(title));
        }
        frame.set_menu_bar(bar.build());
        Menus {
            bar: frame.get_menu_bar().expect("the menu bar was just set"),
            submenu,
        }
    }

    /// Points every menu item at the state that now holds. Called after every
    /// operation and every tab change, so what the menu reads is never stale.
    ///
    /// The marks are written in the same pass and from the same
    /// [`Availability`]: wx toggles a check item's mark — and moves a radio
    /// group's — itself before the command reaches the window, so a mark is
    /// only ever as true as the state it is written back from (v0.2.0 §4, §5).
    ///
    /// A submenu reads as enabled when the commands it holds do; it has no
    /// enablement rule of its own, so there is nothing here to keep in step
    /// with [`Command::enabled`].
    pub fn sync(&self, available: &Availability) {
        for command in Command::ALL {
            self.bar
                .enable_item(command.id(), command.enabled(available));
            if let Some(state) = command.state(available) {
                self.bar.check_item(command.id(), state);
            }
        }
        if let Some((title, item)) = &self.submenu {
            item.enable(
                Command::ALL
                    .into_iter()
                    .any(|command| command.submenu() == Some(*title) && command.enabled(available)),
            );
        }
    }
}

/// One menu built from the commands `belongs` selects, in [`Command::ALL`]'s
/// order — which is the order §15 and v0.2.0 §12 put them in.
fn build_menu(belongs: impl Fn(Command) -> bool) -> Menu {
    let mut menu = Menu::builder();
    for command in Command::ALL.into_iter().filter(|command| belongs(*command)) {
        let (id, label) = (command.id(), command.menu_label());
        menu = match command.item() {
            MenuItemKind::Plain => menu.append_item(id, &label, ""),
            MenuItemKind::Check => menu.append_check_item(id, &label, ""),
            MenuItemKind::Radio => menu.append_radio_item(id, &label, ""),
        };
    }
    menu.build()
}

/// Where `group`'s submenu stands in it, if it has one: how many of the menu's
/// own items come before it, and the title it is appended under.
///
/// A submenu takes the place of its **first** item in [`Command::ALL`], so the
/// menu's order is read off that one list — a second table saying where the
/// submenu goes could only promise an order `ALL` does not keep. The first is
/// also the last: v0.2.0 §12 gives the bar one submenu, and this answers for
/// it.
fn submenu_slot(group: &'static str) -> Option<(usize, &'static str)> {
    let mut position = 0;
    for command in Command::ALL.into_iter().filter(|c| c.menu() == group) {
        match command.submenu() {
            Some(title) => return Some((position, title)),
            None => position += 1,
        }
    }
    None
}
