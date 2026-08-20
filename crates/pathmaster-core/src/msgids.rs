//! The Catalogue's registry: every msgid the application looks up, in one list
//! (spec §11, ADR-0004).
//!
//! `translate()` is a function, not a macro, so nothing extracts the set of
//! msgids from the code and nothing checks that one exists. Naming every msgid
//! here is what makes the set knowable — and turns "one Catalogue" from a rule
//! someone must remember into a list a test can walk. The completeness gate
//! walks it; the shipped `.po` files are measured against it.
//!
//! A msgid is English source text, and that English is an API surface: `msgctxt`
//! is unbound at every level, so where two strings mean different things their
//! English must differ (ADR-0004). Placeholders are named braces — `%d` would be
//! indistinguishable from the `%VAR%` this application exists to edit.

/// One Catalogue entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CatalogueEntry {
    /// The English source text: the lookup key, and what a miss returns.
    pub msgid: &'static str,
    /// The plural msgid, for the entries looked up through `translate_plural`.
    /// The singular in `msgid` is the key both forms are found by.
    pub plural: Option<&'static str>,
    /// The menu this label belongs to, when its mnemonic must stay unique among
    /// that menu's siblings. Menus land with the tickets that build them.
    pub menu: Option<&'static str>,
}

impl CatalogueEntry {
    /// A string with one form: a label, a title, an Announcement.
    pub const fn text(msgid: &'static str) -> Self {
        CatalogueEntry {
            msgid,
            plural: None,
            menu: None,
        }
    }

    /// A string whose wording depends on a count.
    pub const fn plural(singular: &'static str, plural: &'static str) -> Self {
        CatalogueEntry {
            msgid: singular,
            plural: Some(plural),
            menu: None,
        }
    }

    /// A menu label, whose `&` mnemonic is gated against its siblings'.
    pub const fn menu_item(msgid: &'static str, menu: &'static str) -> Self {
        CatalogueEntry {
            msgid,
            plural: None,
            menu: Some(menu),
        }
    }
}

/// The Scope tab labels (spec §12).
pub const TAB_USER: &str = "User PATH";
pub const TAB_SYSTEM: &str = "System PATH";
pub const TAB_BACKUPS: &str = "Backups";

/// The two list columns every Scope tab shows (spec §7, §12).
pub const COLUMN_PATH: &str = "Path";
pub const COLUMN_STATUS: &str = "Status";

/// Announcement 1 (spec §10.1): the entry count on tab activation and Refresh.
/// The zero case is its own msgid rather than a plural form — Ukrainian's
/// `nplurals=3` has no zero form, and "no entries" is better speech than "0".
pub const ENTRIES_USER: &str = "User PATH: {n} entry";
pub const ENTRIES_USER_PLURAL: &str = "User PATH: {n} entries";
pub const ENTRIES_USER_NONE: &str = "User PATH: no entries";
pub const ENTRIES_SYSTEM: &str = "System PATH: {n} entry";
pub const ENTRIES_SYSTEM_PLURAL: &str = "System PATH: {n} entries";
pub const ENTRIES_SYSTEM_NONE: &str = "System PATH: no entries";

/// Announcement 7 (spec §10.1): a Read-only Data run names its reason once at
/// startup. `{reason}` is itself Catalogue text — one of the three §3 reasons
/// below — translated before it is filled in. The same assembled string is
/// StatusBar field 0 in that state (spec §12): the mode and its reason, where
/// the entry counts would otherwise stand.
pub const READONLY: &str = "Read-only: {reason}";
pub const READONLY_REASON_OWN_LOCATION_UNKNOWN: &str = "the application's own location is unknown";
pub const READONLY_REASON_CANNOT_CREATE: &str = "the data directory cannot be created";
pub const READONLY_REASON_NOT_WRITABLE: &str = "the data directory is not writable";

/// The startup dialog an unreadable `settings.json` earns (spec §13). All of
/// it is the title: NVDA never speaks a `MessageDialog`'s body, so a dialog's
/// critical information lives in its title and buttons (spec §10, D6). One
/// dialog, one string — the [OK] button is the stock one, which carries no
/// meaning of its own to lose.
pub const DIALOG_SETTINGS_UNREADABLE: &str = "Settings could not be read — defaults are in use";

/// The two mnemonic groups the menus so far form: the menu bar's own titles,
/// and the Edit menu's items. A group is a set of siblings whose `&` letters
/// must not repeat; the gate walks each one (spec §15).
///
/// These are **not** Catalogue strings and are never translated — they are
/// group keys, and the only thing that reads them is the gate. Nothing here
/// reaches a user, which is why they are the two `MENU_` constants absent
/// from [`REGISTRY`].
pub const MENU_GROUP_BAR: &str = "menu bar";
pub const MENU_GROUP_EDIT: &str = "Edit";

/// The menu bar's titles. The Edit menu is the first to land; File, Tools and
/// Help arrive with the tickets that fill them (spec §15).
pub const MENU_TITLE_EDIT: &str = "&Edit";

/// The Edit menu's items (spec §15). Accelerators are **not** here: the code
/// appends `"\tCtrl+Z"` to the translated label, because a translated tab
/// would delete the shortcut rather than misread it (ADR-0004). The mnemonic
/// letters are A, E, D, M, V, U, R, C, F — unique within the menu, and gated.
pub const MENU_ADD_ENTRY: &str = "&Add Entry…";
pub const MENU_EDIT_ENTRY: &str = "&Edit Entry…";
pub const MENU_DELETE_ENTRY: &str = "&Delete Entry";
pub const MENU_MOVE_UP: &str = "&Move Up";
pub const MENU_MOVE_DOWN: &str = "Mo&ve Down";
pub const MENU_UNDO: &str = "&Undo";
pub const MENU_REDO: &str = "&Redo";
pub const MENU_CANCEL: &str = "&Cancel Changes";
pub const MENU_REFRESH: &str = "Re&fresh";

/// The per-Scope buttons (spec §15). Their English differs from both the menu
/// items that share their command and the operation names that announce it —
/// a `…` where a dialog follows, no mnemonic (the Tab order is the map, and a
/// button's `&` would race the menu bar's), and no "Entry" where the button
/// already sits under the list of them.
pub const BUTTON_ADD: &str = "Add…";
pub const BUTTON_EDIT: &str = "Edit…";
pub const BUTTON_DELETE: &str = "Delete";
pub const BUTTON_MOVE_UP: &str = "Move Up";
pub const BUTTON_MOVE_DOWN: &str = "Move Down";
pub const BUTTON_CANCEL: &str = "Cancel Changes";

/// The Add/Edit dialog (spec §6, FR-edit-f2). The titles double as the
/// operation names Announcement 4 speaks — the spec writes them identically
/// because they name the same operation, once at the top of the dialog that
/// performs it and once in the sentence that undoes it. The path field reuses
/// the `Path` column header: one label for one thing.
pub const DIALOG_EDIT_ENTRY: &str = "Edit entry";
pub const DIALOG_ADD_ENTRY: &str = "Add entry";
pub const BUTTON_BROWSE: &str = "Browse…";

/// The folder picker's own title. `wxDirDialog` is a stock Windows dialog, but
/// its title is ours to give — left unset it would speak wx's built-in English
/// in a Ukrainian run, and a dialog title is spoken (spec §6, §10).
pub const DIALOG_CHOOSE_FOLDER: &str = "Choose a folder";

/// The buttons our own dialogs carry, because `MessageDialog` cannot relabel
/// its own and `add_std_catalog()` is never called (spec §11).
///
/// `Cancel` here is Cancel-**the-dialog-button** — "do not commit", «Скасувати»
/// — and is deliberately not the Cancel *command*, which discards changes back
/// to the Baseline and is [`BUTTON_CANCEL`]/«Відхилити зміни» (ADR-0004).
pub const BUTTON_OK: &str = "OK";
pub const BUTTON_DIALOG_CANCEL: &str = "Cancel";
pub const BUTTON_YES: &str = "Yes";
pub const BUTTON_NO: &str = "No";

/// Validation's error dialog, whose **title is the message** — NVDA never
/// speaks a `MessageDialog`'s body (spec §6, §10). Its single OK is the one
/// stock button left in the application.
pub const REJECTED_FORBIDDEN_CHARACTER: &str =
    "The entry contains a forbidden character: {character}";
pub const REJECTED_EMPTY: &str = "The entry cannot be empty";

/// The convert-or-keep dialog: the single occasion a Value Type changes, and
/// only ever by asking (spec §5, §6). `%VAR%` and `REG_SZ` are data, not
/// placeholders — the braces are what this Catalogue substitutes.
pub const DIALOG_VAR_IN_REG_SZ: &str =
    "This entry uses %VAR%, but the value type (REG_SZ) does not expand variables";
pub const BUTTON_CHANGE_VALUE_TYPE: &str = "Change type to REG_EXPAND_SZ";
pub const BUTTON_KEEP_LITERAL: &str = "Keep as literal text";

/// The two confirmations that guard a discard (spec §5, FR-cancel and
/// FR-refresh). Cancel's is undoable and says only what it does; Refresh's is
/// not, and names the undo history it clears.
pub const DIALOG_DISCARD_CHANGES: &str = "Discard changes?";
pub const DIALOG_REFRESH_DISCARDS: &str =
    "Refresh discards your unsaved changes and the undo history — continue?";

/// Announcements 4, 5 and 6 (spec §10.1). `{operation}` is itself Catalogue
/// text — one of the [`Operation`](crate::session::Operation) names below,
/// translated before it is filled in, and translated as a verbal noun so the
/// Ukrainian composes ("Скасовано: додавання запису").
pub const UNDONE: &str = "Undone: {operation}";
pub const REDONE: &str = "Redone: {operation}";
pub const UNSAVED_CHANGES_SUFFIX: &str = ", unsaved changes";
pub const CHANGES_DISCARDED: &str = "Changes discarded";

/// The operation names Announcement 4 fills in (spec §10.1 item 4). Each is a
/// **different English string from the button that performs it** — the two
/// need different Ukrainian forms, so uniform English would collapse them into
/// one translation (ADR-0004, ticket 11 D14). Add and Edit share the dialog
/// titles above, which name the same operations.
pub const OPERATION_DELETE: &str = "Delete entry";
pub const OPERATION_MOVE: &str = "Move entry";
pub const OPERATION_CANCEL: &str = "Discard changes";
pub const OPERATION_CHANGE_VALUE_TYPE: &str = "Change value type";
pub const OPERATION_RESTORE: &str = "Restore snapshot";

/// Every msgid the application looks up. Later tickets append their strings;
/// nothing is looked up that is not named here.
pub const REGISTRY: &[CatalogueEntry] = &[
    CatalogueEntry::text(TAB_USER),
    CatalogueEntry::text(TAB_SYSTEM),
    CatalogueEntry::text(TAB_BACKUPS),
    CatalogueEntry::text(COLUMN_PATH),
    CatalogueEntry::text(COLUMN_STATUS),
    CatalogueEntry::plural(ENTRIES_USER, ENTRIES_USER_PLURAL),
    CatalogueEntry::text(ENTRIES_USER_NONE),
    CatalogueEntry::plural(ENTRIES_SYSTEM, ENTRIES_SYSTEM_PLURAL),
    CatalogueEntry::text(ENTRIES_SYSTEM_NONE),
    CatalogueEntry::text(READONLY),
    CatalogueEntry::text(READONLY_REASON_OWN_LOCATION_UNKNOWN),
    CatalogueEntry::text(READONLY_REASON_CANNOT_CREATE),
    CatalogueEntry::text(READONLY_REASON_NOT_WRITABLE),
    CatalogueEntry::text(DIALOG_SETTINGS_UNREADABLE),
    CatalogueEntry::menu_item(MENU_TITLE_EDIT, MENU_GROUP_BAR),
    CatalogueEntry::menu_item(MENU_ADD_ENTRY, MENU_GROUP_EDIT),
    CatalogueEntry::menu_item(MENU_EDIT_ENTRY, MENU_GROUP_EDIT),
    CatalogueEntry::menu_item(MENU_DELETE_ENTRY, MENU_GROUP_EDIT),
    CatalogueEntry::menu_item(MENU_MOVE_UP, MENU_GROUP_EDIT),
    CatalogueEntry::menu_item(MENU_MOVE_DOWN, MENU_GROUP_EDIT),
    CatalogueEntry::menu_item(MENU_UNDO, MENU_GROUP_EDIT),
    CatalogueEntry::menu_item(MENU_REDO, MENU_GROUP_EDIT),
    CatalogueEntry::menu_item(MENU_CANCEL, MENU_GROUP_EDIT),
    CatalogueEntry::menu_item(MENU_REFRESH, MENU_GROUP_EDIT),
    CatalogueEntry::text(BUTTON_ADD),
    CatalogueEntry::text(BUTTON_EDIT),
    CatalogueEntry::text(BUTTON_DELETE),
    CatalogueEntry::text(BUTTON_MOVE_UP),
    CatalogueEntry::text(BUTTON_MOVE_DOWN),
    CatalogueEntry::text(BUTTON_CANCEL),
    CatalogueEntry::text(DIALOG_EDIT_ENTRY),
    CatalogueEntry::text(DIALOG_ADD_ENTRY),
    CatalogueEntry::text(BUTTON_BROWSE),
    CatalogueEntry::text(DIALOG_CHOOSE_FOLDER),
    CatalogueEntry::text(BUTTON_OK),
    CatalogueEntry::text(BUTTON_DIALOG_CANCEL),
    CatalogueEntry::text(BUTTON_YES),
    CatalogueEntry::text(BUTTON_NO),
    CatalogueEntry::text(REJECTED_FORBIDDEN_CHARACTER),
    CatalogueEntry::text(REJECTED_EMPTY),
    CatalogueEntry::text(DIALOG_VAR_IN_REG_SZ),
    CatalogueEntry::text(BUTTON_CHANGE_VALUE_TYPE),
    CatalogueEntry::text(BUTTON_KEEP_LITERAL),
    CatalogueEntry::text(DIALOG_DISCARD_CHANGES),
    CatalogueEntry::text(DIALOG_REFRESH_DISCARDS),
    CatalogueEntry::text(UNDONE),
    CatalogueEntry::text(REDONE),
    CatalogueEntry::text(UNSAVED_CHANGES_SUFFIX),
    CatalogueEntry::text(CHANGES_DISCARDED),
    CatalogueEntry::text(OPERATION_DELETE),
    CatalogueEntry::text(OPERATION_MOVE),
    CatalogueEntry::text(OPERATION_CANCEL),
    CatalogueEntry::text(OPERATION_CHANGE_VALUE_TYPE),
    CatalogueEntry::text(OPERATION_RESTORE),
];

/// The placeholder names in `text`, in order of appearance.
///
/// A placeholder is `{name}` with `name` made of ASCII letters, digits and
/// underscores; anything else between braces is ordinary text, and `%VAR%` is
/// always data.
pub fn placeholders(text: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let mut rest = text;
    while let Some((_, close, name)) = next_braces(rest) {
        if is_placeholder_name(name) {
            names.push(name);
        }
        rest = &rest[close + 1..];
    }
    names
}

/// Substitutes `{name}` placeholders — the one substitution helper (spec §11).
///
/// Values are copied in verbatim and never rescanned, so Entry text carrying
/// braces or `%VAR%` cannot turn into a placeholder. A placeholder with no
/// value is left as it stands: the gate makes that unreachable in shipped text,
/// and a readable string beats a panic in an application NVDA is speaking.
pub fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some((open, close, name)) = next_braces(rest) {
        out.push_str(&rest[..open]);
        match values.iter().find(|(key, _)| *key == name) {
            Some((_, value)) if is_placeholder_name(name) => out.push_str(value),
            _ => out.push_str(&rest[open..=close]),
        }
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    out
}

/// The mnemonic letter of a label: the character after its first single `&`.
///
/// `&&` is an escaped ampersand, not a mnemonic. Ukrainian labels carry the
/// Latin letter in parentheses — `"Файл(&F)"` — so this answers `'F'` there too.
pub fn mnemonic(label: &str) -> Option<char> {
    let mut chars = label.chars();
    while let Some(c) = chars.next() {
        if c != '&' {
            continue;
        }
        match chars.next() {
            Some('&') => continue,
            other => return other,
        }
    }
    None
}

/// The first mnemonic letter two of `labels` share, compared case-insensitively
/// as Alt+F and Alt+f are the same keystroke. Labels without a mnemonic are
/// passed over — their absence is a separate defect.
pub fn duplicate_mnemonic<'a>(labels: impl IntoIterator<Item = &'a str>) -> Option<char> {
    let mut seen = std::collections::BTreeSet::new();
    for label in labels {
        if let Some(letter) = mnemonic(label) {
            let folded = letter.to_lowercase().next().unwrap_or(letter);
            if !seen.insert(folded) {
                return Some(folded);
            }
        }
    }
    None
}

/// The next `{...}` in `rest`: where its brace opens, where it closes, and what
/// stands between them — which is a placeholder name only if
/// [`is_placeholder_name`] says so. Both readers of the Catalogue's text scan it
/// through here, so "what counts as a placeholder" is answered in one place.
fn next_braces(rest: &str) -> Option<(usize, usize, &str)> {
    let open = rest.find('{')?;
    let close = open + 1 + rest[open + 1..].find('}')?;
    Some((open, close, &rest[open + 1..close]))
}

fn is_placeholder_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}
