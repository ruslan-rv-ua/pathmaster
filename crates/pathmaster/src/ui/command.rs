//! The thirteen commands the window carries, and the menus they live in
//! (spec §15, §5, §6).
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

use pathmaster_core::msgids;
use pathmaster_core::session::Session;
use wxdragon::prelude::*;

use crate::catalog::translate;

/// One user-visible command.
///
/// Ten of them are about a Scope; the last three are not, and are here for the
/// reason the enum exists at all: they are menu items, and a menu item's id,
/// label and enabled state are answered in one place or in three.
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
    Settings,
    OpenBackupsFolder,
    Exit,
}

/// What a command's availability is decided from.
///
/// The active Scope's Editing Session answers for the ten that edit one, and
/// `None` — the Backups tab, which is not a Scope — closes every one of them.
/// The Run's own facts answer for Open Backups Folder, and nothing at all
/// answers for Exit or for Settings: a way out of the application is available
/// on every tab and in every state, dirty Sessions included — that is what the
/// close-confirm is for — and the settings belong to the Run, not to whichever
/// tab happens to be showing. Ticket 17 adds its Tools item to the same shape.
pub struct Availability<'a> {
    pub session: Option<&'a Session>,
    /// Whether this Run has a Data Directory at all. The one Run that has none
    /// does not know where it is (`ReadOnlyReason::OwnLocationUnknown`), and so
    /// has no folder of Snapshots to show.
    pub data_dir: bool,
}

impl Command {
    /// Every command, in **button** order — §15's Add, Edit, Delete, Move Up,
    /// Move Down, Apply, Cancel — which is also the order each menu takes its
    /// own items in, since [`menu`](Self::menu) preserves it.
    pub const ALL: [Command; 13] = [
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
        Command::Settings,
        Command::OpenBackupsFolder,
        Command::Exit,
    ];

    /// The menus, in menu-bar order. Help arrives with the ticket that fills
    /// it; Tools gains Restart as Administrator with its own (spec §15).
    const MENUS: [(&'static str, &'static str); 3] = [
        (msgids::MENU_TITLE_FILE, msgids::MENU_GROUP_FILE),
        (msgids::MENU_TITLE_EDIT, msgids::MENU_GROUP_EDIT),
        (msgids::MENU_TITLE_TOOLS, msgids::MENU_GROUP_TOOLS),
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
            Command::Settings | Command::OpenBackupsFolder => msgids::MENU_GROUP_TOOLS,
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
            Command::Settings => msgids::MENU_SETTINGS,
            Command::OpenBackupsFolder => msgids::MENU_OPEN_BACKUPS_FOLDER,
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
            | Command::Settings
            | Command::OpenBackupsFolder
            | Command::Exit => return None,
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
            Command::Add | Command::Cancel | Command::Settings | Command::OpenBackupsFolder => None,
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
        // the two commands that answer to the Run and the ten that answer to a
        // Scope, and the next command someone adds has to say which it is.
        match self {
            // Not about a Scope: it shows this Run's own directory, so it is
            // available on the Backups tab, where there is no Session at all.
            Command::OpenBackupsFolder => available.data_dir,
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
            Command::Exit | Command::Settings => true,
            Command::Add
            | Command::Edit
            | Command::Delete
            | Command::MoveUp
            | Command::MoveDown
            | Command::Undo
            | Command::Redo
            | Command::Apply
            | Command::Cancel
            | Command::Refresh => available.session.is_some_and(|session| self.over(session)),
        }
    }

    /// What a Scope command asks of the Session it would act on.
    fn over(self, session: &Session) -> bool {
        match self {
            Command::Refresh => true,
            _ if !session.writable() => false,
            Command::Add => true,
            Command::Edit | Command::Delete | Command::MoveUp | Command::MoveDown => {
                !session.entries().is_empty()
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
            Command::Settings | Command::OpenBackupsFolder | Command::Exit => false,
        }
    }
}

/// Builds the menu bar from [`Command::MENUS`] and [`Command::menu`], so a
/// command with no menu is a command with no shortcut — which for Ctrl+S would
/// be a command with no way to reach it at all. Help arrives with the ticket
/// that fills it; Tools gains its third item with ticket 17.
///
/// Every item's help string is deliberately empty: wx writes it to the status
/// bar as the user moves through the menu, and the status bar is command-only
/// — nothing must-hear goes there (spec §10, §12).
pub fn build_menu_bar() -> MenuBar {
    let mut bar = MenuBar::builder();
    for (title, group) in Command::MENUS {
        let mut menu = Menu::builder();
        for command in Command::ALL.into_iter().filter(|c| c.menu() == group) {
            menu = menu.append_item(command.id(), &command.menu_label(), "");
        }
        bar = bar.append(menu.build(), &translate(title));
    }
    bar.build()
}

/// Points every menu item at the state that now holds. Called after every
/// operation and every tab change, so what the menu reads is never stale.
pub fn sync_menu_bar(bar: &MenuBar, available: &Availability) {
    for command in Command::ALL {
        bar.enable_item(command.id(), command.enabled(available));
    }
}
