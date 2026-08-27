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

/// The three list columns every Scope tab shows (spec §7, §12; v0.2.0 §2.1).
///
/// `#` is a sign rather than a word, and both shipped languages write it the
/// same. It is a msgid all the same: the shell looks up everything it shows, so
/// a language that spells the sign differently — «№» — needs its own catalogue
/// and no code change.
pub const COLUMN_INDEX: &str = "#";
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

/// The Search field's label (v0.2.0 spec §3, §14): constant, no mnemonic, and
/// never carrying the count — a changing label is a `NAMECHANGE`, measured
/// dead in v0.1.0.
pub const SEARCH_LABEL: &str = "Search:";

/// Announcement 8 (v0.2.0 spec §13 item 8): Expansion Mode was toggled — which
/// rendering both Scope lists now show. Two whole sentences and no placeholder:
/// the message is the mode, and the mode is the whole message.
///
/// The Ukrainian for the raw mode is «Показано збережені значення» — what the
/// registry holds — because "raw" names a text's provenance, and that is the
/// word Ukrainian has for it.
pub const SHOWING_EXPANDED_VALUES: &str = "Showing expanded values";
pub const SHOWING_RAW_VALUES: &str = "Showing raw values";

/// Announcement 9 (v0.2.0 spec §13 item 9): the filtered count on a criteria
/// change — a typing pause, ESC into a still-filtered view, an Expansion
/// toggle, Filter → All with query text. `{n}` is the visible count, `{m}` the
/// Scope's total; **the plural form is selected by `{m}`**, the total —
/// written down or lost, because the i18n gate checks plural presence, not
/// which number chose them. The zero case is worded, its own msgid: Ukrainian's
/// three plural forms have no zero form, and "No matching entries" is better
/// speech than "0".
pub const FILTERED_COUNT: &str = "{n} of {m} entry";
pub const FILTERED_COUNT_PLURAL: &str = "{n} of {m} entries";
pub const FILTERED_COUNT_NONE: &str = "No matching entries";

/// Announcement 10 (v0.2.0 spec §13 item 10): the Scope-named filtered count
/// on tab activation and Refresh while that Scope has a Filtered View. Whole
/// strings per Scope, never one frame — «PATH користувача: …» agrees with its
/// subject the way [`APPLIED_USER`]'s does — and the same composition is
/// StatusBar field 0's per-Scope fragment while that Scope is narrowed
/// (v0.2.0 spec §16). Plural by `{m}`; the zero cases are their own msgids,
/// one per Scope.
pub const FILTERED_USER: &str = "User PATH: {n} of {m} entry";
pub const FILTERED_USER_PLURAL: &str = "User PATH: {n} of {m} entries";
pub const FILTERED_USER_NONE: &str = "User PATH: no matching entries";
pub const FILTERED_SYSTEM: &str = "System PATH: {n} of {m} entry";
pub const FILTERED_SYSTEM_PLURAL: &str = "System PATH: {n} of {m} entries";
pub const FILTERED_SYSTEM_NONE: &str = "System PATH: no matching entries";

/// The two Filter states that are not an Issue type (v0.2.0 spec §4, §12).
///
/// The other five reuse the Status column's own words — [`ISSUE_MISSING`] and
/// its four siblings — so the Filter adds a name for exactly what the
/// Catalogue did not already hold. **Neither carries a mnemonic**: these two
/// strings are read in the Filter submenu, in Announcement 11 and in StatusBar
/// field 0, and the two surfaces that are not a menu would print the `&`. The
/// submenu is walked with the arrow keys, which is how a radio group is
/// walked; Ctrl+I is the keystroke, and it rides its own item.
pub const FILTER_ALL: &str = "All";
pub const FILTER_WITH_ISSUES: &str = "With issues";

/// Announcement 11 (v0.2.0 spec §13 item 11): the composed Search∧Filter count
/// a change to a non-All Filter speaks — **one announcement, never two**, so
/// the Filter's name and the count it produced are one sentence.
///
/// `{filter}` is the state's own name, translated before it is filled in, the
/// way [`APPLY_FAILED`]'s cause is. Plural by `{m}` like every other count
/// here; the zero case is worded and still names the state, because "which
/// filter found nothing" is the whole of what the user needs back.
pub const FILTER_COUNT: &str = "{filter}: {n} of {m} entry";
pub const FILTER_COUNT_PLURAL: &str = "{filter}: {n} of {m} entries";
pub const FILTER_COUNT_NONE: &str = "{filter}: no matching entries";

/// StatusBar field 0's per-Scope fragment while that Scope's Filter is not
/// `All` (v0.2.0 spec §16): the state named, then the count it produced.
///
/// Whole strings per Scope, like [`FILTERED_USER`] and for its reason — and
/// separate from it rather than composed onto it, because the name lands
/// *inside* the sentence and no suffix can put it there. The issue
/// parenthetical is appended as ever and never changes meaning: it counts the
/// Scope's Issues, not the view's.
pub const FILTERED_USER_NAMED: &str = "User PATH: {filter} — {n} of {m} entry";
pub const FILTERED_USER_NAMED_PLURAL: &str = "User PATH: {filter} — {n} of {m} entries";
pub const FILTERED_USER_NAMED_NONE: &str = "User PATH: {filter} — no matching entries";
pub const FILTERED_SYSTEM_NAMED: &str = "System PATH: {filter} — {n} of {m} entry";
pub const FILTERED_SYSTEM_NAMED_PLURAL: &str = "System PATH: {filter} — {n} of {m} entries";
pub const FILTERED_SYSTEM_NAMED_NONE: &str = "System PATH: {filter} — no matching entries";

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
/// startup. `{reason}` is itself Catalogue text — one of the four reasons
/// below — translated before it is filled in. The same assembled string is
/// StatusBar field 0 in that state (spec §12): the mode and its reason, where
/// the entry counts would otherwise stand.
///
/// The fourth reason is the `--data-dir` one (v0.2.0 §10). It names **the
/// switch** rather than the directory, because that is the whole of what
/// distinguishes it: the third reason would be equally true of it, and a Run
/// pointed somewhere it cannot write needs to hear which of the two locations
/// failed. One reason covers all its ways of failing — a target that could not
/// be created, one that could not be written, and a switch that carried
/// nothing to resolve — because there is one thing to say about all of them
/// and one thing to do about it.
pub const READONLY: &str = "Read-only: {reason}";
pub const READONLY_REASON_OWN_LOCATION_UNKNOWN: &str = "the application's own location is unknown";
pub const READONLY_REASON_CANNOT_CREATE: &str = "the data directory cannot be created";
pub const READONLY_REASON_NOT_WRITABLE: &str = "the data directory is not writable";
pub const READONLY_REASON_OVERRIDE_UNUSABLE: &str = "the --data-dir location cannot be used";

/// The command line's three strings (v0.2.0 §10, §14).
///
/// The application has no console to print to, so a command-line answer is a
/// dialog — Firefox's GUI-build help is literally a message box, and it is the
/// convention here too. These two are the one shape whose **body is not a
/// repetition of its title**: the title is the message, as everywhere else,
/// and the body carries [`USAGE`] — help arriving exactly when someone who
/// types arguments needs it.
///
/// [`USAGE`] is one string shared by both, so the two dialogs can never
/// describe two different command lines. The switch spellings inside it are
/// not translated, because they are what the user types.
pub const DIALOG_UNKNOWN_ARGUMENT: &str = "Unknown argument {arg} was ignored";
pub const DIALOG_COMMAND_LINE: &str = "PathMaster command line";
pub const USAGE: &str =
    "Usage: PathMaster.exe [--tab user|system|backups] [--data-dir <path>] [--help]";

/// The startup dialog an unreadable `settings.json` earns (spec §13). All of
/// it is the title: NVDA never speaks a `MessageDialog`'s body, so a dialog's
/// critical information lives in its title and buttons (spec §10, D6). One
/// dialog, one string — the [OK] button is the stock one, which carries no
/// meaning of its own to lose.
pub const DIALOG_SETTINGS_UNREADABLE: &str = "Settings could not be read — defaults are in use";

/// The dialog a failed write of `settings.json` earns (spec §13) — the mirror
/// of the one above, and shaped like it: the whole message is the title, the
/// [OK] button is the stock one, and it says what became of what the user
/// asked for rather than why the disk refused.
///
/// It exists because **nothing was recorded**. The window adopts an amended
/// document only once the file has taken it, so a write that failed leaves the
/// run exactly as it was — and a user who pressed OK and saw the dialog close
/// has no other way to learn that. The shutdown path's geometry write earns no
/// dialog and keeps its `WARN` line alone: nobody asked for that one, and the
/// window is already going.
pub const DIALOG_SETTINGS_UNWRITABLE: &str = "Settings could not be written — nothing was changed";

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
/// reaches a user, which is why they are the `MENU_GROUP_` constants absent
/// from [`REGISTRY`].
pub const MENU_GROUP_BAR: &str = "menu bar";
pub const MENU_GROUP_EDIT: &str = "Edit";
pub const MENU_GROUP_FILE: &str = "File";
pub const MENU_GROUP_HELP: &str = "Help";
pub const MENU_GROUP_TOOLS: &str = "Tools";
pub const MENU_GROUP_VIEW: &str = "View";

/// The menu bar's titles (spec §15; v0.2.0 §12): the mnemonics are F, E, V,
/// T, H — unique across the bar, and gated. View returns with v0.2.0, between
/// Edit and Tools: commands that change *what the list shows* live there.
pub const MENU_TITLE_EDIT: &str = "&Edit";
pub const MENU_TITLE_FILE: &str = "&File";
pub const MENU_TITLE_HELP: &str = "&Help";
pub const MENU_TITLE_TOOLS: &str = "&Tools";
pub const MENU_TITLE_VIEW: &str = "&View";

/// The View menu's first item (v0.2.0 §12): Ctrl+F's menu home, which focuses
/// the Search field and selects its contents. The accelerator is appended by
/// the code, never typed here (ADR-0004). Its mnemonic is S, unique in the
/// menu it opens.
pub const MENU_SEARCH: &str = "&Search";

/// The View menu's rendering item (v0.2.0 §5, §12): Ctrl+E's menu home, and the
/// state carrier for Expansion Mode.
///
/// The label is **constant** in both directions — a `wxITEM_CHECK` item's mark
/// is what NVDA reads to say which way it went, and a label that changed with
/// the mode would be a `NAMECHANGE`, measured dead in v0.1.0. The accelerator
/// is appended by the code (ADR-0004). Its mnemonic is E, unique in the menu it
/// opens.
pub const MENU_EXPANDED_VALUES: &str = "&Expanded Values";

/// The View menu's Filter submenu (v0.2.0 §4, §12): the title the seven
/// `wxITEM_RADIO` states hang under, and the plain command item Ctrl+I rides.
///
/// **The toggle is its own item, not a mark on this one.** A radio item
/// carrying Ctrl+I would fire that radio's selection rather than the toggle,
/// and a check item would carry a mark that lies whenever a per-type state is
/// active — so the coarse gesture gets a plain item with a constant label, and
/// the submenu's radio marks are the state. The seven state names carry no
/// mnemonic and are not menu entries here: they are shared with the Status
/// column, and the submenu is walked with the arrow keys.
///
/// The mnemonics are F and I, unique in the View menu they open beside
/// [`MENU_SEARCH`]'s S and [`MENU_EXPANDED_VALUES`]'s E.
pub const MENU_FILTER: &str = "&Filter";
pub const MENU_TOGGLE_ISSUES_FILTER: &str = "Toggle &Issues Filter";

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

/// The Tools menu's items (spec §15). Open Backups Folder hands the
/// Snapshots' own directory to the shell, which is why it opens rather than
/// asks — a folder picker would be a question, and nothing here is asking
/// one. Restart as Administrator is the **one entry point into elevation**
/// (spec §9, ADR-0005): no `…` because what follows is the UAC prompt and a
/// relaunch, not a dialog of ours. The mnemonic letters are S, O and R,
/// unique within the menu, and gated.
pub const MENU_SETTINGS: &str = "&Settings…";
pub const MENU_OPEN_BACKUPS_FOLDER: &str = "&Open Backups Folder";
pub const MENU_RESTART_AS_ADMIN: &str = "&Restart as Administrator";

/// The Help menu's one item, and the dialog it opens (spec §15, §16).
///
/// No `…` on the item: §15 spells it "About", and the `…` in this application
/// marks the two items that open a dialog *asking* something — About states
/// and is dismissed. Its mnemonic is A, and being alone in its menu it cannot
/// collide.
///
/// The dialog is one sentence because it is one title, which is all NVDA
/// speaks of a dialog (§10, D6). It carries the three things §16 makes it
/// carry — name, version, licence — and only `{version}` is filled in: the
/// product name and the licence identifier are proper nouns no translation may
/// vary, and the same two the exe's `VERSIONINFO` carries, which is the only
/// other place an unsigned binary says who it is.
pub const MENU_ABOUT: &str = "&About";
pub const DIALOG_ABOUT: &str = "PathMaster {version} — MIT License";

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
pub const SETTINGS_SNAPSHOTS_TO_KEEP: &str = "Snapshots to keep per PATH";

/// The dialog's three Filtered View controls (v0.2.0 §15), which are how the
/// narrowing behaviour is tuned without hand-editing `settings.json`.
///
/// Two of them are checkboxes, and a checkbox's own label **is** its
/// accessible name on the free native path — there is no `StaticText` before
/// one, and adding a second name would be the label read twice. The delay is a
/// typed number and is labelled the way the budget is.
///
/// Each label says what the setting *does when it is on*, never what it is
/// called in the file: "Speak filtered entry counts" is a sentence a user can
/// answer yes or no to, where "speakFilteredCount" is an identifier. The
/// delay's carries its unit, because a number with no unit is one the user has
/// to guess at, and carries it as "(ms)" so the label stays the width of a
/// label.
pub const SETTINGS_SPEAK_FILTERED_COUNT: &str = "Speak filtered entry counts";
pub const SETTINGS_COUNT_DELAY: &str = "Delay before speaking the count (ms)";
pub const SETTINGS_SEARCH_ESCAPE_RETURNS_FOCUS: &str = "Escape returns focus to the list";

/// Validation's error dialog, whose **title is the message** — NVDA never
/// speaks a `MessageDialog`'s body (spec §6, §10). Its single OK is the one
/// stock button left in the application.
///
/// The two typed numbers' rejections are worded like the first two: what was
/// wanted, never what was typed. The field keeps the text, so repeating it
/// back would say nothing the user cannot already read there. Each opens with
/// its own control's words, because the dialog now has two fields a number can
/// be wrong in and the message is the only thing said about which.
pub const REJECTED_FORBIDDEN_CHARACTER: &str =
    "The entry contains a forbidden character: {character}";
pub const REJECTED_EMPTY: &str = "The entry cannot be empty";
pub const REJECTED_SNAPSHOTS_TO_KEEP: &str = "Snapshots to keep must be a whole number, 1 or more";
pub const REJECTED_COUNT_DELAY: &str =
    "Delay before speaking the count must be a whole number, 0 to 5000";

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

/// The elevation texts (spec §9, FR-uac-elevation; ADR-0005).
///
/// The first pair is the close-confirm flow's **dedicated dialog** for the one
/// command that discards and relaunches: the title names User changes and no
/// other, because only the User Session can be dirty in an instance that
/// offers the command — System is non-writable unelevated, and elevated the
/// command is disabled. There is deliberately no [Save] here: the standard
/// close-confirm offers it, this dialog is the relaunch's own, and its two
/// buttons are the two outcomes. [`BUTTON_DIALOG_CANCEL`] is the second.
///
/// The cancelled dialog answers a declined UAC prompt — `ShellExecuteEx`
/// reports `ERROR_CANCELLED`, and silence after a security prompt is treated
/// as a defect (ADR-0005). A dialog and not an Announcement: it answers an
/// explicit user action, and the Announcement catalogue is closed at seven.
///
/// The window title is the cmd.exe convention, and it is Catalogue text
/// because Alt+Tab speaks the title first — the cheapest always-available
/// answer to "which instance am I in" (ticket 12 D11).
pub const DIALOG_DISCARD_AND_RESTART: &str =
    "Discard unsaved User changes and restart as administrator?";
pub const BUTTON_DISCARD_AND_RESTART: &str = "Discard and Restart";
pub const DIALOG_ELEVATION_CANCELLED: &str =
    "Elevation was cancelled — still running without administrator rights";
pub const WINDOW_TITLE_ELEVATED: &str = "Administrator: PathMaster";

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
    CatalogueEntry::text(COLUMN_INDEX),
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
    CatalogueEntry::text(SEARCH_LABEL),
    CatalogueEntry::text(SHOWING_EXPANDED_VALUES),
    CatalogueEntry::text(SHOWING_RAW_VALUES),
    CatalogueEntry::plural(FILTERED_COUNT, FILTERED_COUNT_PLURAL),
    CatalogueEntry::text(FILTERED_COUNT_NONE),
    CatalogueEntry::plural(FILTERED_USER, FILTERED_USER_PLURAL),
    CatalogueEntry::text(FILTERED_USER_NONE),
    CatalogueEntry::plural(FILTERED_SYSTEM, FILTERED_SYSTEM_PLURAL),
    CatalogueEntry::text(FILTERED_SYSTEM_NONE),
    CatalogueEntry::text(FILTER_ALL),
    CatalogueEntry::text(FILTER_WITH_ISSUES),
    CatalogueEntry::plural(FILTER_COUNT, FILTER_COUNT_PLURAL),
    CatalogueEntry::text(FILTER_COUNT_NONE),
    CatalogueEntry::plural(FILTERED_USER_NAMED, FILTERED_USER_NAMED_PLURAL),
    CatalogueEntry::text(FILTERED_USER_NAMED_NONE),
    CatalogueEntry::plural(FILTERED_SYSTEM_NAMED, FILTERED_SYSTEM_NAMED_PLURAL),
    CatalogueEntry::text(FILTERED_SYSTEM_NAMED_NONE),
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
    CatalogueEntry::text(READONLY_REASON_OVERRIDE_UNUSABLE),
    CatalogueEntry::text(DIALOG_UNKNOWN_ARGUMENT),
    CatalogueEntry::text(DIALOG_COMMAND_LINE),
    CatalogueEntry::text(USAGE),
    CatalogueEntry::text(DIALOG_SETTINGS_UNREADABLE),
    CatalogueEntry::text(DIALOG_SETTINGS_UNWRITABLE),
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
    CatalogueEntry::menu_item(MENU_TITLE_HELP, MENU_GROUP_BAR),
    CatalogueEntry::menu_item(MENU_TITLE_TOOLS, MENU_GROUP_BAR),
    CatalogueEntry::menu_item(MENU_TITLE_VIEW, MENU_GROUP_BAR),
    CatalogueEntry::menu_item(MENU_SEARCH, MENU_GROUP_VIEW),
    CatalogueEntry::menu_item(MENU_FILTER, MENU_GROUP_VIEW),
    CatalogueEntry::menu_item(MENU_TOGGLE_ISSUES_FILTER, MENU_GROUP_VIEW),
    CatalogueEntry::menu_item(MENU_EXPANDED_VALUES, MENU_GROUP_VIEW),
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
    CatalogueEntry::menu_item(MENU_RESTART_AS_ADMIN, MENU_GROUP_TOOLS),
    CatalogueEntry::menu_item(MENU_ABOUT, MENU_GROUP_HELP),
    CatalogueEntry::text(DIALOG_ABOUT),
    CatalogueEntry::text(DIALOG_DISCARD_AND_RESTART),
    CatalogueEntry::text(BUTTON_DISCARD_AND_RESTART),
    CatalogueEntry::text(DIALOG_ELEVATION_CANCELLED),
    CatalogueEntry::text(WINDOW_TITLE_ELEVATED),
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
    CatalogueEntry::text(SETTINGS_SNAPSHOTS_TO_KEEP),
    CatalogueEntry::text(SETTINGS_SPEAK_FILTERED_COUNT),
    CatalogueEntry::text(SETTINGS_COUNT_DELAY),
    CatalogueEntry::text(SETTINGS_SEARCH_ESCAPE_RETURNS_FOCUS),
    CatalogueEntry::text(REJECTED_FORBIDDEN_CHARACTER),
    CatalogueEntry::text(REJECTED_EMPTY),
    CatalogueEntry::text(REJECTED_SNAPSHOTS_TO_KEEP),
    CatalogueEntry::text(REJECTED_COUNT_DELAY),
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
