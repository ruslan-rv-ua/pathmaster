//! The seventeen commands the window carries, and the menus they live in
//! (spec §15, §5, §6; v0.2.0 §12).
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
use pathmaster_core::msgids;
use pathmaster_core::session::Session;
use wxdragon::prelude::*;

use crate::catalog::translate;

/// One user-visible command.
///
/// Twelve of them are about a Scope; the last five are not, and are here for
/// the reason the enum exists at all: they are menu items, and a menu item's
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
    MoveUp,
    MoveDown,
    Undo,
    Redo,
    Apply,
    Cancel,
    Refresh,
    Search,
    ExpandedValues,
    Settings,
    OpenBackupsFolder,
    RestartAsAdministrator,
    Exit,
    About,
}

/// What a command's availability is decided from.
///
/// The active Scope's Editing Session answers for the ten that edit one, and
/// `None` — the Backups tab, which is not a Scope — closes every one of them.
/// The Run's own facts answer for Open Backups Folder and for Restart as
/// Administrator, and nothing at all answers for Exit or for Settings: a way
/// out of the application is available on every tab and in every state, dirty
/// Sessions included — that is what the close-confirm is for — and the
/// settings belong to the Run, not to whichever tab happens to be showing.
pub struct Availability<'a> {
    pub session: Option<&'a Session>,
    /// Whether this Scope's Filtered View is active — a non-empty Search text
    /// (the Filter axis composes in with ticket 05). A reorder's effect
    /// concerns positions the user cannot see, and Add appends below them, so
    /// a narrowed view closes Move Up, Move Down and Add (v0.2.0 §2).
    pub narrowed: bool,
    /// How many Entries the view now shows — the Scope's whole count when
    /// nothing narrows it. Edit and Delete act on the focused **visible**
    /// Entry, so zero visible rows closes them even over a non-empty Scope.
    pub visible_rows: usize,
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
}

impl Command {
    /// Every command, in **button** order — §15's Add, Edit, Delete, Move Up,
    /// Move Down, Apply, Cancel — which is also the order each menu takes its
    /// own items in, since [`menu`](Self::menu) preserves it.
    pub const ALL: [Command; 17] = [
        Command::Add,
        Command::Edit,
        Command::Delete,
        Command::MoveUp,
        Command::MoveDown,
        Command::Undo,
        Command::Redo,
        Command::Apply,
        Command::Cancel,
        Command::Refresh,
        Command::Search,
        Command::ExpandedValues,
        Command::Settings,
        Command::OpenBackupsFolder,
        Command::RestartAsAdministrator,
        Command::Exit,
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
            | Command::MoveUp
            | Command::MoveDown
            | Command::Undo
            | Command::Redo
            | Command::Cancel
            | Command::Refresh => msgids::MENU_GROUP_EDIT,
            Command::Search | Command::ExpandedValues => msgids::MENU_GROUP_VIEW,
            Command::Settings | Command::OpenBackupsFolder | Command::RestartAsAdministrator => {
                msgids::MENU_GROUP_TOOLS
            }
            Command::About => msgids::MENU_GROUP_HELP,
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
            Command::MoveUp => msgids::MENU_MOVE_UP,
            Command::MoveDown => msgids::MENU_MOVE_DOWN,
            Command::Undo => msgids::MENU_UNDO,
            Command::Redo => msgids::MENU_REDO,
            Command::Cancel => msgids::MENU_CANCEL,
            Command::Refresh => msgids::MENU_REFRESH,
            Command::Search => msgids::MENU_SEARCH,
            Command::ExpandedValues => msgids::MENU_EXPANDED_VALUES,
            Command::Settings => msgids::MENU_SETTINGS,
            Command::OpenBackupsFolder => msgids::MENU_OPEN_BACKUPS_FOLDER,
            Command::RestartAsAdministrator => msgids::MENU_RESTART_AS_ADMIN,
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
            Command::Undo
            | Command::Redo
            | Command::Refresh
            | Command::Search
            | Command::ExpandedValues
            | Command::Settings
            | Command::OpenBackupsFolder
            | Command::RestartAsAdministrator
            | Command::Exit
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
    fn accelerator(self) -> Option<&'static str> {
        match self {
            Command::Edit => Some("F2"),
            Command::Delete => Some("Del"),
            Command::MoveUp => Some("Alt+Up"),
            Command::MoveDown => Some("Alt+Down"),
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
            Command::Add
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
        // at all, and the ten that answer to a Scope — and the next command
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
            Command::Exit | Command::Settings | Command::About => true,
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
            | Command::ExpandedValues => available
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
            Command::Search | Command::ExpandedValues => true,
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
            | Command::About => false,
        }
    }

    /// Whether this command's menu item **carries state**: a `wxITEM_CHECK`
    /// item, whose mark NVDA reads in both directions, rather than a plain
    /// item that does something and says nothing (v0.2.0 §5, ticket 16
    /// probe 1).
    ///
    /// Asked twice and answered here once — at build time, to append the right
    /// kind of item, and at every sync, to write [`state`](Self::state) into
    /// it. A plain item and a check item are not interchangeable: appending
    /// the wrong kind is an item whose mark can never be set at all.
    fn carries_state(self) -> bool {
        // Exhaustive, like every other `match` over this enum: the next
        // command someone adds has to say whether it carries a mark, because
        // an item that silently carried none would read as a command in a menu
        // where its state is the whole point.
        match self {
            Command::ExpandedValues => true,
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
            | Command::Settings
            | Command::OpenBackupsFolder
            | Command::RestartAsAdministrator
            | Command::Exit
            | Command::About => false,
        }
    }

    /// The state that item's mark now shows.
    ///
    /// **It is written whether or not the item is enabled**: on the Backups
    /// tab Expanded Values is disabled like every other View item, and its
    /// check mark stays readable there — the mode is app-wide and a tab the
    /// command cannot be given from is not a tab it is off in (v0.2.0 §5).
    fn state(self, available: &Availability) -> bool {
        match self {
            Command::ExpandedValues => available.expansion.expanded(),
            // Never asked — `carries_state` closes them — and answered rather
            // than caught all the same, so the next state-carrying command has
            // to say what its mark reads.
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
            | Command::Settings
            | Command::OpenBackupsFolder
            | Command::RestartAsAdministrator
            | Command::Exit
            | Command::About => false,
        }
    }
}

/// Builds the menu bar from [`Command::MENUS`] and [`Command::menu`], so a
/// command with no menu is a command with no shortcut — which for Ctrl+S would
/// be a command with no way to reach it at all.
///
/// Every item's help string is deliberately empty: wx writes it to the status
/// bar as the user moves through the menu, and the status bar is command-only
/// — nothing must-hear goes there (spec §10, §12).
pub fn build_menu_bar() -> MenuBar {
    let mut bar = MenuBar::builder();
    for (title, group) in Command::MENUS {
        let mut menu = Menu::builder();
        for command in Command::ALL.into_iter().filter(|c| c.menu() == group) {
            let (id, label) = (command.id(), command.menu_label());
            menu = if command.carries_state() {
                menu.append_check_item(id, &label, "")
            } else {
                menu.append_item(id, &label, "")
            };
        }
        bar = bar.append(menu.build(), &translate(title));
    }
    bar.build()
}

/// Points every menu item at the state that now holds. Called after every
/// operation and every tab change, so what the menu reads is never stale.
///
/// The check items' marks are written in the same pass and from the same
/// [`Availability`]: wx toggles a check item's mark itself before the command
/// reaches the window, so the mark is only ever as true as the flag it is
/// written back from (v0.2.0 §5).
pub fn sync_menu_bar(bar: &MenuBar, available: &Availability) {
    for command in Command::ALL {
        bar.enable_item(command.id(), command.enabled(available));
        if command.carries_state() {
            bar.check_item(command.id(), command.state(available));
        }
    }
}
