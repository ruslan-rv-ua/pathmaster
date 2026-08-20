//! The nine editing commands, and the Edit menu that carries them
//! (spec §15, §5, §6).
//!
//! One enum is the whole map: it names the menu items, the ids the menu events
//! arrive under, the accelerators, the per-Scope buttons, and — in one
//! `match` — when each is available. That matters more than tidiness here,
//! because `wxAcceleratorTable` is absent from wxdragon at every level: a
//! keyboard shortcut can only exist as a menu item's label, so **every
//! shortcut in PathMaster must have a menu home**, and the menu's enabled
//! state is therefore also the shortcut's.
//!
//! Accelerators are appended by this code and never typed into the Catalogue:
//! a translated `"\tCtrl+Я"` would not misread, it would delete the shortcut
//! (ADR-0004).

use pathmaster_core::msgids;
use pathmaster_core::session::Session;
use wxdragon::prelude::*;

use crate::catalog::translate;

/// One user-visible editing command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Add,
    Edit,
    Delete,
    MoveUp,
    MoveDown,
    Undo,
    Redo,
    Cancel,
    Refresh,
}

impl Command {
    /// The Edit menu, in the order §15 fixes.
    pub const ALL: [Command; 9] = [
        Command::Add,
        Command::Edit,
        Command::Delete,
        Command::MoveUp,
        Command::MoveDown,
        Command::Undo,
        Command::Redo,
        Command::Cancel,
        Command::Refresh,
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

    /// The menu item's label: Catalogue text with its accelerator appended.
    pub fn menu_label(self) -> String {
        let label = translate(match self {
            Command::Add => msgids::MENU_ADD_ENTRY,
            Command::Edit => msgids::MENU_EDIT_ENTRY,
            Command::Delete => msgids::MENU_DELETE_ENTRY,
            Command::MoveUp => msgids::MENU_MOVE_UP,
            Command::MoveDown => msgids::MENU_MOVE_DOWN,
            Command::Undo => msgids::MENU_UNDO,
            Command::Redo => msgids::MENU_REDO,
            Command::Cancel => msgids::MENU_CANCEL,
            Command::Refresh => msgids::MENU_REFRESH,
        });
        match self.accelerator() {
            Some(accelerator) => format!("{label}\t{accelerator}"),
            None => label,
        }
    }

    /// The label of this command's button under a Scope's list, or `None`
    /// where §15 gives it none — Undo, Redo and Refresh are menu and keyboard
    /// commands. Which commands have a button is therefore answered here and
    /// nowhere else; the Tab order is `ALL`'s order, filtered by this.
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
            Command::Cancel => msgids::BUTTON_CANCEL,
            Command::Undo | Command::Redo | Command::Refresh => return None,
        };
        Some(translate(msgid))
    }

    /// The keystroke wx parses out of the label. F2 and Del are the list's own
    /// gestures given a menu home; Enter and double-click reach the same
    /// dialog through the list's activation event instead (spec §6, §15).
    fn accelerator(self) -> Option<&'static str> {
        match self {
            Command::Edit => Some("F2"),
            Command::Delete => Some("Del"),
            Command::MoveUp => Some("Alt+Up"),
            Command::MoveDown => Some("Alt+Down"),
            Command::Undo => Some("Ctrl+Z"),
            Command::Redo => Some("Ctrl+Y"),
            Command::Refresh => Some("F5"),
            Command::Add | Command::Cancel => None,
        }
    }

    /// Whether this command is available over `session` — `None` being the
    /// Backups tab, which is not a Scope and offers no editing at all.
    ///
    /// A non-writable Session closes every command that edits, and a disabled
    /// item is how a screen reader is told so (spec §5, §15). **Refresh is not
    /// one of them**: it re-reads, and a Scope the user cannot edit is still
    /// one they can look at — Read-only Data "still reads, diagnoses and
    /// lists", and an unelevated System tab would otherwise never see an
    /// external change without a restart.
    pub fn enabled(self, session: Option<&Session>) -> bool {
        let Some(session) = session else { return false };
        match self {
            Command::Refresh => true,
            _ if !session.writable() => false,
            Command::Add => true,
            Command::Edit | Command::Delete | Command::MoveUp | Command::MoveDown => {
                !session.entries().is_empty()
            }
            Command::Undo => session.can_undo(),
            Command::Redo => session.can_redo(),
            Command::Cancel => session.is_dirty(),
        }
    }
}

/// Builds the menu bar. File, Tools and Help arrive with the tickets that fill
/// them; the Edit menu is complete here.
///
/// Every item's help string is deliberately empty: wx writes it to the status
/// bar as the user moves through the menu, and the status bar is command-only
/// — nothing must-hear goes there (spec §10, §12).
pub fn build_menu_bar() -> MenuBar {
    let mut edit = Menu::builder();
    for command in Command::ALL {
        edit = edit.append_item(command.id(), &command.menu_label(), "");
    }
    MenuBar::builder()
        .append(edit.build(), &translate(msgids::MENU_TITLE_EDIT))
        .build()
}

/// Points every menu item at the active Session's state. Called after every
/// operation and every tab change, so what the menu reads is never stale.
pub fn sync_menu_bar(bar: &MenuBar, session: Option<&Session>) {
    for command in Command::ALL {
        bar.enable_item(command.id(), command.enabled(session));
    }
}
