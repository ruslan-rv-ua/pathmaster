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

/// The three the Backups tab shows (spec §8, FR-backup-ui). The Scope column's
/// *values* are [`TAB_USER`] and [`TAB_SYSTEM`] — a Scope has one name, and the
/// tab that shows it is where it is already written.
pub const COLUMN_DATE_AND_TIME: &str = "Date and time";
pub const COLUMN_SCOPE: &str = "Scope";
pub const COLUMN_ENTRIES: &str = "Entries";

/// What the Entries column says of a Snapshot that failed validation (spec §8):
/// passive list text, read for free when the row takes focus, and never an
/// Announcement (`CONTEXT.md`, **Corrupted**). It stands where a count would
/// because it answers the same question — how many Entries restoring this
/// file would load — for a file that cannot be read at all. The brackets are
/// the spec's own and mark it as a state rather than a number.
pub const SNAPSHOT_CORRUPTED: &str = "[Corrupted]";

/// Announcement 1 (spec §10.1): the entry count on tab activation and Refresh.
/// The zero case is its own msgid rather than a plural form — Ukrainian's
/// `nplurals=3` has no zero form, and "no entries" is better speech than "0".
pub const ENTRIES_USER: &str = "User PATH: {n} entry";
pub const ENTRIES_USER_PLURAL: &str = "User PATH: {n} entries";
pub const ENTRIES_USER_NONE: &str = "User PATH: no entries";
pub const ENTRIES_SYSTEM: &str = "System PATH: {n} entry";
pub const ENTRIES_SYSTEM_PLURAL: &str = "System PATH: {n} entries";
pub const ENTRIES_SYSTEM_NONE: &str = "System PATH: no entries";

/// Announcement 2 (spec §10.1 item 2): a Scope's Working Copy reached the
/// registry. Two whole strings rather than one frame with the Scope filled in:
/// «PATH користувача застосовано» agrees with its subject, and a Scope name
/// dropped into a shared frame would not. Which of the two is spoken is the
/// Catalogue's own rule, decided from the [`Scope`](crate::session::Scope) the
/// Announcement carries.
pub const APPLIED_USER: &str = "User PATH applied";
pub const APPLIED_SYSTEM: &str = "System PATH applied";

/// Announcement 3 (spec §10.1 item 3): the §9 taxonomy's texts — one frame and
/// the three causes it is filled with.
///
/// Every row of the taxonomy says "Apply failed" and then names its own reason,
/// so the sentence is written once. The frame carries the final stop, which
/// leaves each cause a phrase a translator may reorder; `{cause}` is itself
/// Catalogue text, translated before it is filled in, exactly as
/// [`READONLY`]'s `{reason}` is.
///
/// A **failed re-read takes the registry row's cause** and not one of its own
/// (spec §9's fifth row): nothing was written either way, which is the whole of
/// what the user needs to know.
pub const APPLY_FAILED: &str = "Apply failed — {cause}.";
pub const APPLY_FAILED_BACKUP: &str = "could not write a backup, no changes were made";
pub const APPLY_FAILED_ACCESS_DENIED: &str = "access denied";
pub const APPLY_FAILED_REGISTRY: &str = "the registry could not be written";

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

/// The five Issue types' words, and the whole of what the Status column shows
/// (spec §7, FR-diag-status). One word each, comma-joined most-severe-first —
/// no severity prefix, no icons, and no word for a healthy Entry: an empty
/// column is the only healthy state, so "OK" is a string this Catalogue must
/// never be able to say.
pub const ISSUE_MISSING: &str = "Missing";
pub const ISSUE_RELATIVE: &str = "Relative";
pub const ISSUE_QUOTED: &str = "Quoted";
pub const ISSUE_DUPLICATE: &str = "Duplicate";
pub const ISSUE_EMPTY: &str = "Empty";

/// The issue half of StatusBar field 0 (spec §12). A **suffix** rather than one
/// string carrying both numbers, because a gettext lookup selects its plural
/// form on one number and this line has two — the shape Announcement 5's
/// [`UNSAVED_CHANGES_SUFFIX`] already takes. Keep the leading space.
pub const ISSUES_SUFFIX: &str = " ({m} issue)";
pub const ISSUES_SUFFIX_PLURAL: &str = " ({m} issues)";

/// StatusBar field 1, the passive merged-length field (spec §12,
/// FR-diag-overlength): the length always, with the `cmd.exe` warning appended
/// past 8,191. That threshold is literal text and not a placeholder — it is a
/// measured constant of the OS, not a number this sentence is filled with, and
/// the one fact the warning exists to carry must not be droppable by a
/// translation. Over-length is never in the Status column and never spoken.
pub const MERGED_LENGTH: &str = "Merged PATH: {n} char";
pub const MERGED_LENGTH_PLURAL: &str = "Merged PATH: {n} chars";
pub const MERGED_LENGTH_EXCEEDS: &str = " — exceeds 8,191 (cmd.exe limit)";

/// The mnemonic groups the menus so far form: the menu bar's own titles, the
/// Edit menu's items, and the File menu's. A group is a set of siblings whose
/// `&` letters must not repeat; the gate walks each one (spec §15).
///
/// These are **not** Catalogue strings and are never translated — they are
/// group keys, and the only thing that reads them is the gate. Nothing here
/// reaches a user, which is why they are the three `MENU_GROUP_` constants
/// absent from [`REGISTRY`].
pub const MENU_GROUP_BAR: &str = "menu bar";
pub const MENU_GROUP_EDIT: &str = "Edit";
pub const MENU_GROUP_FILE: &str = "File";
pub const MENU_GROUP_TOOLS: &str = "Tools";

/// The menu bar's titles. Help arrives with the ticket that fills it
/// (spec §15). The mnemonics are F, E, T — unique across the bar, and gated.
pub const MENU_TITLE_EDIT: &str = "&Edit";
pub const MENU_TITLE_FILE: &str = "&File";
pub const MENU_TITLE_TOOLS: &str = "&Tools";

/// The File menu's items (spec §15). Apply is here because Ctrl+S can only
/// live on a menu item's label — wxdragon binds no accelerator table at any
/// level, so **every shortcut has a menu home** (ADR-0004) — and Exit is here
/// for the same reason and one more: Alt+F4 already closes the window, so
/// without this item the close-confirm would be a dialog with no menu route to
/// it at all. The mnemonic letters are A and x, unique within the menu, and
/// gated.
pub const MENU_APPLY: &str = "&Apply";
pub const MENU_EXIT: &str = "E&xit";

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

/// The Tools menu's items (spec §15). Restart as Administrator arrives with
/// the ticket that owns it. Open Backups Folder hands the Snapshots' own
/// directory to the shell, which is why it opens rather than asks — a folder
/// picker would be a question, and nothing here is asking one. The mnemonic
/// letters are S and O, unique within the menu, and gated.
pub const MENU_SETTINGS: &str = "&Settings…";
pub const MENU_OPEN_BACKUPS_FOLDER: &str = "&Open Backups Folder";

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
pub const BUTTON_APPLY: &str = "Apply";
pub const BUTTON_CANCEL: &str = "Cancel Changes";

/// The Backups tab's one button (spec §15): it loads the chosen Snapshot into
/// its Scope's Working Copy, which is why it says no more than what it does —
/// nothing is written until that Scope is applied.
pub const BUTTON_RESTORE: &str = "Restore";

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

/// The Settings dialog (Tools → Settings…, spec §13, §11).
///
/// **The restart notice rides the language selector's own label**, which is
/// why that label is a sentence: FR-i18n-runtime says the language applies
/// after a restart, and the Announcement catalogue is closed at seven, so the
/// one place left to say so is the label of the control that changes it.
///
/// The auto choice is Catalogue text because it names a rule rather than a
/// language; the two languages beside it in the selector are their own
/// endonyms and deliberately outside the Catalogue, so a user who cannot read
/// the current Interface Language can still find theirs (spec §11).
///
/// The budget's label says **Snapshots** and not "backups": `CONTEXT.md`
/// reserves the latter for the act of taking one and for the directory they
/// live in, which is what the tab and the Tools item above are named for.
pub const DIALOG_SETTINGS: &str = "Settings";
pub const SETTINGS_LANGUAGE: &str = "Language (takes effect after restart)";
pub const SETTINGS_LANGUAGE_FOLLOWS_SYSTEM: &str = "Follow the system language";
pub const SETTINGS_MAX_BACKUPS: &str = "Snapshots to keep per PATH";

/// Validation's error dialog, whose **title is the message** — NVDA never
/// speaks a `MessageDialog`'s body (spec §6, §10). Its single OK is the one
/// stock button left in the application.
///
/// The budget's rejection is the second of them and is worded like the first
/// two: what was wanted, never what was typed. The field keeps the text, so
/// repeating it back would say nothing the user cannot already read there.
pub const REJECTED_FORBIDDEN_CHARACTER: &str =
    "The entry contains a forbidden character: {character}";
pub const REJECTED_EMPTY: &str = "The entry cannot be empty";
pub const REJECTED_MAX_BACKUPS: &str = "Snapshots to keep must be a whole number, 1 or more";

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

/// The close-confirm (spec §5, FR-close-confirm): one dialog for the whole
/// application, raised only when a Session is dirty. `{scopes}` is filled with
/// the dirty Scopes' own tab labels, so a Scope has one name here too
/// (ADR-0004); the title carries the question because NVDA never speaks a
/// `MessageDialog` body.
///
/// **`Save` is the one place that word is ours to say.** `CONTEXT.md` keeps it
/// off **Apply** and off **Apply Run**, because naming the operation "save"
/// hides that a write to the registry is what it is — but this is a button on
/// a close-confirm, and FR-close-confirm fixes its English. What it performs is
/// still an Apply Run, and the log and every Announcement still say so.
///
/// `Discard` is not [`BUTTON_CANCEL`] and not [`OPERATION_CANCEL`]: three
/// meanings, three English strings (ADR-0004). This one closes the application
/// without writing; that one discards a Working Copy back to its Baseline and
/// leaves the application open. [`BUTTON_DIALOG_CANCEL`] is the third button,
/// meaning here exactly what it means everywhere — do not commit.
pub const DIALOG_CLOSE_CONFIRM: &str = "Unsaved changes in: {scopes} — save before closing?";
pub const BUTTON_SAVE: &str = "Save";
pub const BUTTON_DISCARD: &str = "Discard";

/// The external-change dialog (spec §5, FR-apply): the value moved under the
/// Session between the last read and this Apply. All three answers are legal,
/// so all three are named — the middle one adopts what was just read and
/// writes nothing, which is why its label says both halves of what it does.
///
/// [`BUTTON_DIALOG_CANCEL`] is the third button: here too it means "do not
/// commit", which is exactly what it says everywhere else.
pub const DIALOG_EXTERNAL_CHANGE: &str = "PATH was modified externally since last refresh";
pub const BUTTON_OVERWRITE: &str = "Overwrite";
pub const BUTTON_REFRESH_AND_DISCARD: &str = "Refresh and discard my changes";

/// The two over-length gates at Apply (spec §7, FR-diag-overlength). Each is a
/// title and nothing else — NVDA never speaks a `MessageDialog`'s body — and
/// each names both numbers: the threshold, which is a measured constant of the
/// OS and so literal text, and `{n}`, the length this Apply would leave behind.
///
/// The first is a warning with a way past it; the second has a single
/// [`BUTTON_DIALOG_CANCEL`] and no way past at all, which is why it offers no
/// affirmative button to name.
pub const DIALOG_OVER_CMD_LIMIT: &str =
    "cmd.exe will ignore a PATH longer than 8,191 characters ({n} after this Apply)";
pub const BUTTON_APPLY_ANYWAY: &str = "Apply Anyway";
pub const DIALOG_OVER_HARD_CAP: &str =
    "PATH cannot exceed 32,767 characters ({n} after this Apply)";

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
    CatalogueEntry::text(COLUMN_DATE_AND_TIME),
    CatalogueEntry::text(COLUMN_SCOPE),
    CatalogueEntry::text(COLUMN_ENTRIES),
    CatalogueEntry::text(SNAPSHOT_CORRUPTED),
    CatalogueEntry::plural(ENTRIES_USER, ENTRIES_USER_PLURAL),
    CatalogueEntry::text(ENTRIES_USER_NONE),
    CatalogueEntry::plural(ENTRIES_SYSTEM, ENTRIES_SYSTEM_PLURAL),
    CatalogueEntry::text(ENTRIES_SYSTEM_NONE),
    CatalogueEntry::text(APPLIED_USER),
    CatalogueEntry::text(APPLIED_SYSTEM),
    CatalogueEntry::text(APPLY_FAILED),
    CatalogueEntry::text(APPLY_FAILED_BACKUP),
    CatalogueEntry::text(APPLY_FAILED_ACCESS_DENIED),
    CatalogueEntry::text(APPLY_FAILED_REGISTRY),
    CatalogueEntry::text(READONLY),
    CatalogueEntry::text(READONLY_REASON_OWN_LOCATION_UNKNOWN),
    CatalogueEntry::text(READONLY_REASON_CANNOT_CREATE),
    CatalogueEntry::text(READONLY_REASON_NOT_WRITABLE),
    CatalogueEntry::text(DIALOG_SETTINGS_UNREADABLE),
    CatalogueEntry::text(ISSUE_MISSING),
    CatalogueEntry::text(ISSUE_RELATIVE),
    CatalogueEntry::text(ISSUE_QUOTED),
    CatalogueEntry::text(ISSUE_DUPLICATE),
    CatalogueEntry::text(ISSUE_EMPTY),
    CatalogueEntry::plural(ISSUES_SUFFIX, ISSUES_SUFFIX_PLURAL),
    CatalogueEntry::plural(MERGED_LENGTH, MERGED_LENGTH_PLURAL),
    CatalogueEntry::text(MERGED_LENGTH_EXCEEDS),
    CatalogueEntry::menu_item(MENU_TITLE_EDIT, MENU_GROUP_BAR),
    CatalogueEntry::menu_item(MENU_TITLE_FILE, MENU_GROUP_BAR),
    CatalogueEntry::menu_item(MENU_TITLE_TOOLS, MENU_GROUP_BAR),
    CatalogueEntry::menu_item(MENU_APPLY, MENU_GROUP_FILE),
    CatalogueEntry::menu_item(MENU_EXIT, MENU_GROUP_FILE),
    CatalogueEntry::menu_item(MENU_ADD_ENTRY, MENU_GROUP_EDIT),
    CatalogueEntry::menu_item(MENU_EDIT_ENTRY, MENU_GROUP_EDIT),
    CatalogueEntry::menu_item(MENU_DELETE_ENTRY, MENU_GROUP_EDIT),
    CatalogueEntry::menu_item(MENU_MOVE_UP, MENU_GROUP_EDIT),
    CatalogueEntry::menu_item(MENU_MOVE_DOWN, MENU_GROUP_EDIT),
    CatalogueEntry::menu_item(MENU_UNDO, MENU_GROUP_EDIT),
    CatalogueEntry::menu_item(MENU_REDO, MENU_GROUP_EDIT),
    CatalogueEntry::menu_item(MENU_CANCEL, MENU_GROUP_EDIT),
    CatalogueEntry::menu_item(MENU_REFRESH, MENU_GROUP_EDIT),
    CatalogueEntry::menu_item(MENU_SETTINGS, MENU_GROUP_TOOLS),
    CatalogueEntry::menu_item(MENU_OPEN_BACKUPS_FOLDER, MENU_GROUP_TOOLS),
    CatalogueEntry::text(BUTTON_ADD),
    CatalogueEntry::text(BUTTON_EDIT),
    CatalogueEntry::text(BUTTON_DELETE),
    CatalogueEntry::text(BUTTON_MOVE_UP),
    CatalogueEntry::text(BUTTON_MOVE_DOWN),
    CatalogueEntry::text(BUTTON_APPLY),
    CatalogueEntry::text(BUTTON_CANCEL),
    CatalogueEntry::text(BUTTON_RESTORE),
    CatalogueEntry::text(DIALOG_EDIT_ENTRY),
    CatalogueEntry::text(DIALOG_ADD_ENTRY),
    CatalogueEntry::text(BUTTON_BROWSE),
    CatalogueEntry::text(DIALOG_CHOOSE_FOLDER),
    CatalogueEntry::text(BUTTON_OK),
    CatalogueEntry::text(BUTTON_DIALOG_CANCEL),
    CatalogueEntry::text(BUTTON_YES),
    CatalogueEntry::text(BUTTON_NO),
    CatalogueEntry::text(DIALOG_SETTINGS),
    CatalogueEntry::text(SETTINGS_LANGUAGE),
    CatalogueEntry::text(SETTINGS_LANGUAGE_FOLLOWS_SYSTEM),
    CatalogueEntry::text(SETTINGS_MAX_BACKUPS),
    CatalogueEntry::text(REJECTED_FORBIDDEN_CHARACTER),
    CatalogueEntry::text(REJECTED_EMPTY),
    CatalogueEntry::text(REJECTED_MAX_BACKUPS),
    CatalogueEntry::text(DIALOG_VAR_IN_REG_SZ),
    CatalogueEntry::text(BUTTON_CHANGE_VALUE_TYPE),
    CatalogueEntry::text(BUTTON_KEEP_LITERAL),
    CatalogueEntry::text(DIALOG_DISCARD_CHANGES),
    CatalogueEntry::text(DIALOG_REFRESH_DISCARDS),
    CatalogueEntry::text(DIALOG_CLOSE_CONFIRM),
    CatalogueEntry::text(BUTTON_SAVE),
    CatalogueEntry::text(BUTTON_DISCARD),
    CatalogueEntry::text(DIALOG_EXTERNAL_CHANGE),
    CatalogueEntry::text(BUTTON_OVERWRITE),
    CatalogueEntry::text(BUTTON_REFRESH_AND_DISCARD),
    CatalogueEntry::text(DIALOG_OVER_CMD_LIMIT),
    CatalogueEntry::text(BUTTON_APPLY_ANYWAY),
    CatalogueEntry::text(DIALOG_OVER_HARD_CAP),
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
