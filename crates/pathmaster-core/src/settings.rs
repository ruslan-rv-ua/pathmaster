//! `settings.json`: what the file may say, and what this run does when it says
//! something else (spec §13).
//!
//! The file is hand-editable, which is the whole reason this module is shaped
//! the way it is: every way a hand edit can be wrong has an answer that is not
//! "throw the file away". Two layers do that work.
//!
//! * The **parse layer** is all-or-nothing. Unparsable JSON, or a root that is
//!   not an object, means nothing in the file is used — the caller sets it
//!   aside and runs on [`SettingsFile::defaults`]. One mental model for both of
//!   the application's JSON file kinds: a Snapshot fails the same way.
//! * The **field layer** is per-field. An invalid value of a known field falls
//!   back to its own default *in memory* while the file keeps the raw value,
//!   and one [`Rejected`] witnesses it. No value is ever clamped: `-3 → 0`
//!   would mean "no backups", inventing a dangerous choice the user never made.
//!
//! What the file keeps is not a courtesy — it is the choice-not-outcome rule
//! made structural. A value this version cannot read (a v0.2 `language`) is not
//! a value this version may delete, so the raw text survives every rewrite
//! until the user changes *that* setting; unknown fields ride through the same
//! way. Rewriting is therefore not "serialise the settings" but "amend the
//! document", and [`SettingsFile`]'s setters are the only amendment there is.

use serde_json::{Map, Value};

use crate::language::LanguageChoice;
use crate::logfmt::Record;

/// The Interface Language when the file does not choose one (spec §13): follow
/// the system, which is what `auto` means.
pub const DEFAULT_LANGUAGE: LanguageChoice = LanguageChoice::Auto;

/// The backup budget when the file does not choose one (spec §13,
/// FR-backup-rotation). Valid stored values are ≥ 1: rotation at zero would
/// delete the pre-Apply safety net the product exists to provide.
pub const DEFAULT_MAX_BACKUPS: u32 = 50;

/// Whether the filtered count Announcements — items 9, 10 and 11 — speak at
/// all (v0.2.0 spec §15).
pub const DEFAULT_SPEAK_FILTERED_COUNT: bool = true;

/// The debounce before a filtered count speaks, in milliseconds (v0.2.0 spec
/// §15). The default is the primary user's own verdict from the ticket-04 NVDA
/// session — snappy 250 over the research's 1400; the setting exists precisely
/// so it can be slowed. `0` is a legal delay; the ceiling keeps a typo from
/// silently muting a feature the user believes is on.
pub const DEFAULT_FILTERED_COUNT_DELAY_MS: u32 = 250;
pub const MAX_FILTERED_COUNT_DELAY_MS: u32 = 5000;

/// Whether ESC in the Search field returns focus to the list (v0.2.0 spec §15)
/// — the PRD's shape by default, reversible toward the Windows/ARIA
/// stay-in-the-field convention.
pub const DEFAULT_SEARCH_ESCAPE_RETURNS_FOCUS: bool = true;

/// How the geometry default reads in the log — the only default with no value
/// to render, because having none *is* the default (spec §12: first run is
/// 900×650 centred, which the window decides, not the file).
const DEFAULT_WINDOW_SHOWN: &str = "none";

/// The field names, which are the file's API surface — never translated
/// (spec §11) and never spelled twice. Flat `camelCase`, deliberately not a
/// record (v0.2.0 spec §15): each of the three Filtered View fields has its
/// own default, and a record would let one typo silently reset all three.
const LANGUAGE: &str = "language";
const MAX_BACKUPS: &str = "maxBackups";
const WINDOW: &str = "window";
const SPEAK_FILTERED_COUNT: &str = "speakFilteredCount";
const FILTERED_COUNT_DELAY_MS: &str = "filteredCountDelayMs";
const SEARCH_ESCAPE_RETURNS_FOCUS: &str = "searchEscapeReturnsFocus";

/// The window's remembered geometry (spec §12): where it was, how big, and
/// whether it was maximised. Restoring it is clamped to the connected
/// monitors' work area — that is the restore's business, not the file's, so
/// nothing here is clamped or second-guessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    /// May be negative: a monitor left of or above the primary is a real place.
    pub x: i32,
    pub y: i32,
    /// Always positive — a window with no area is not a size.
    pub width: i32,
    pub height: i32,
    pub maximised: bool,
}

impl Window {
    const X: &'static str = "x";
    const Y: &'static str = "y";
    const WIDTH: &'static str = "width";
    const HEIGHT: &'static str = "height";
    const MAXIMISED: &'static str = "maximised";

    /// A window record is whole or it is nothing: geometry is one fact — where
    /// the window was — and half of it is not a place to put a window.
    fn read(value: &Value) -> Option<Window> {
        let object = value.as_object()?;
        let coordinate = |key| whole_number(object.get(key)?);
        let window = Window {
            x: coordinate(Self::X)?,
            y: coordinate(Self::Y)?,
            width: coordinate(Self::WIDTH)?,
            height: coordinate(Self::HEIGHT)?,
            maximised: object.get(Self::MAXIMISED)?.as_bool()?,
        };
        (window.width > 0 && window.height > 0).then_some(window)
    }

    /// Writes the five members into `object`, leaving anything else it holds
    /// alone — a nested field a later version adds is as unknown, and as
    /// preserved, as a top-level one.
    fn write(&self, object: &mut Map<String, Value>) {
        object.insert(Self::X.to_owned(), self.x.into());
        object.insert(Self::Y.to_owned(), self.y.into());
        object.insert(Self::WIDTH.to_owned(), self.width.into());
        object.insert(Self::HEIGHT.to_owned(), self.height.into());
        object.insert(Self::MAXIMISED.to_owned(), self.maximised.into());
    }
}

/// The two settings the Settings dialog offers (spec §13): what it opens on,
/// and what it answers with.
///
/// Geometry is deliberately not here. It is a setting in the file and nothing
/// the user sets — the window records where they left it — so a dialog that
/// carried it would be offering to type in a position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Choices {
    pub language: LanguageChoice,
    pub max_backups: u32,
}

/// One known field whose stored value was invalid, and the default that stands
/// in for it this run. The log is its only witness — no dialog, no
/// Announcement — so the wording lives here rather than at a call site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rejected {
    /// The field's name as the file spells it.
    pub field: &'static str,
    /// What the file said, as a human would read it back: a string's own text,
    /// anything else as the JSON it is. The record truncates it; a
    /// pathological file must not put a megabyte on one line.
    pub raw: String,
    /// The default that took its place, rendered from the default itself so
    /// the log cannot claim a value the run did not use.
    pub default: String,
}

impl Rejected {
    /// The `WARN` line this rejection earns.
    pub fn record(&self) -> Record {
        Record::settings_field_invalid(self.field, &self.raw, &self.default)
    }
}

/// The parse layer's verdict (spec §13), which is all-or-nothing. Both arms
/// name the outcome rather than the JSON: what the caller must know is whether
/// this file could be read, not what shape its root turned out to be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parsed {
    /// A parsable object root. Every known field has been read on its own;
    /// `rejected` names those that fell back, in the order the fields are
    /// defined rather than the order the file happens to list them.
    Readable {
        file: SettingsFile,
        rejected: Vec<Rejected>,
    },
    /// Unparsable JSON, or a root that is not an object. Nothing in the file
    /// is used and nothing in it is understood well enough to be preserved.
    Unreadable,
}

/// The settings this run uses, over the document they came from.
///
/// Both halves are load-bearing. The typed values are what the application
/// asks; the document is what the file said, unknown fields and rejected raw
/// values included, and it is what a rewrite starts from. They are kept in step
/// by construction: the setters are the only way either changes, so a raw value
/// the file kept can only ever be replaced by the user changing that setting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsFile {
    document: Map<String, Value>,
    language: LanguageChoice,
    max_backups: u32,
    window: Option<Window>,
    speak_filtered_count: bool,
    filtered_count_delay_ms: u32,
    search_escape_returns_focus: bool,
}

impl SettingsFile {
    /// The first run, and the fallback after an unreadable file: every default,
    /// over an empty document. Writing this creates `{}` — the file records
    /// choices, so defaults nobody chose do not materialise as choices somebody
    /// made.
    pub fn defaults() -> SettingsFile {
        SettingsFile {
            document: Map::new(),
            language: DEFAULT_LANGUAGE,
            max_backups: DEFAULT_MAX_BACKUPS,
            window: None,
            speak_filtered_count: DEFAULT_SPEAK_FILTERED_COUNT,
            filtered_count_delay_ms: DEFAULT_FILTERED_COUNT_DELAY_MS,
            search_escape_returns_focus: DEFAULT_SEARCH_ESCAPE_RETURNS_FOCUS,
        }
    }

    /// Reads the file's text through both layers.
    pub fn parse(text: &str) -> Parsed {
        // A UTF-8 BOM is what several Windows editors leave in front of an
        // otherwise perfectly good file, and JSON has no place for it. Dropping
        // it is not tolerance of malformed JSON — it is reading the text the
        // editor meant to write.
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let Ok(Value::Object(document)) = serde_json::from_str(text) else {
            return Parsed::Unreadable;
        };

        let mut rejected = Vec::new();
        let language = read_field(
            &document,
            LANGUAGE,
            DEFAULT_LANGUAGE.as_str(),
            &mut rejected,
            |value| value.as_str().and_then(LanguageChoice::parse),
        );
        let max_backups = read_field(
            &document,
            MAX_BACKUPS,
            &DEFAULT_MAX_BACKUPS.to_string(),
            &mut rejected,
            // Valid domain ≥ 1, and nothing outside it is nudged into it.
            |value| value.as_u64().and_then(in_backup_budget),
        );
        let window = read_field(
            &document,
            WINDOW,
            DEFAULT_WINDOW_SHOWN,
            &mut rejected,
            Window::read,
        );
        let speak_filtered_count = read_field(
            &document,
            SPEAK_FILTERED_COUNT,
            &DEFAULT_SPEAK_FILTERED_COUNT.to_string(),
            &mut rejected,
            Value::as_bool,
        );
        let filtered_count_delay_ms = read_field(
            &document,
            FILTERED_COUNT_DELAY_MS,
            &DEFAULT_FILTERED_COUNT_DELAY_MS.to_string(),
            &mut rejected,
            // 0–5000 whole; nothing outside the domain is nudged into it.
            |value| value.as_u64().and_then(in_count_delay),
        );
        let search_escape_returns_focus = read_field(
            &document,
            SEARCH_ESCAPE_RETURNS_FOCUS,
            &DEFAULT_SEARCH_ESCAPE_RETURNS_FOCUS.to_string(),
            &mut rejected,
            Value::as_bool,
        );

        Parsed::Readable {
            file: SettingsFile {
                document,
                language: language.unwrap_or(DEFAULT_LANGUAGE),
                max_backups: max_backups.unwrap_or(DEFAULT_MAX_BACKUPS),
                window,
                speak_filtered_count: speak_filtered_count.unwrap_or(DEFAULT_SPEAK_FILTERED_COUNT),
                filtered_count_delay_ms: filtered_count_delay_ms
                    .unwrap_or(DEFAULT_FILTERED_COUNT_DELAY_MS),
                search_escape_returns_focus: search_escape_returns_focus
                    .unwrap_or(DEFAULT_SEARCH_ESCAPE_RETURNS_FOCUS),
            },
            rejected,
        }
    }

    /// The Interface Language this run was asked for — the choice, not its
    /// outcome. [`crate::language::resolve`] turns it into a language.
    pub fn language(&self) -> LanguageChoice {
        self.language
    }

    /// The per-Scope backup budget, always ≥ 1.
    pub fn max_backups(&self) -> u32 {
        self.max_backups
    }

    /// The remembered geometry, or `None` when the file has none to give.
    pub fn window(&self) -> Option<Window> {
        self.window
    }

    /// Whether the filtered count Announcements — items 9, 10 and 11 — speak
    /// (v0.2.0 spec §15).
    pub fn speak_filtered_count(&self) -> bool {
        self.speak_filtered_count
    }

    /// The debounce before a filtered count speaks, in milliseconds — always
    /// within 0–5000 (v0.2.0 spec §15).
    pub fn filtered_count_delay_ms(&self) -> u32 {
        self.filtered_count_delay_ms
    }

    /// Whether ESC in the Search field returns focus to the list (v0.2.0 spec
    /// §15).
    pub fn search_escape_returns_focus(&self) -> bool {
        self.search_escape_returns_focus
    }

    /// The two settings the Settings dialog opens on — what this run is using,
    /// never the raw values the file kept for what it could not read. A
    /// `language` this version cannot do is not a language to show as done.
    pub fn choices(&self) -> Choices {
        Choices {
            language: self.language,
            max_backups: self.max_backups,
        }
    }

    /// Records what the Settings dialog answered with, and answers whether the
    /// document changed at all.
    ///
    /// **Only the settings the user changed are written**, and that is the
    /// choice-not-outcome rule doing its work at the one moment it can be got
    /// wrong. A `language` the file kept because this version could not read it
    /// stands in the document while the dialog shows the default that replaced
    /// it in memory; pressing OK over an untouched selector must therefore
    /// leave it standing, or every trip through this dialog would quietly
    /// downgrade a v0.2 file to what v0.1 happened to be doing.
    ///
    /// The comparison is of content and not a record that something happened —
    /// the same reading of [`Dirty`](crate::session::Session::is_dirty) the
    /// Editing Sessions take. A user who retypes the value that was already
    /// there has changed nothing, so nothing is written; the answer is what
    /// lets the caller leave a hand-edited file unreformatted and a first run
    /// without a `{}` nobody asked for.
    pub fn record_choices(&mut self, choices: Choices) -> bool {
        let mut changed = false;
        if choices.language != self.language {
            self.set_language(choices.language);
            changed = true;
        }
        if choices.max_backups != self.max_backups {
            self.set_max_backups(choices.max_backups);
            changed = true;
        }
        changed
    }

    /// Records a new Interface Language choice, in memory and in the document
    /// alike. This is what replaces a `language` value the file kept because
    /// this version could not read it: the user changed that setting, so the
    /// choice-not-outcome rule has nothing left to protect.
    pub fn set_language(&mut self, choice: LanguageChoice) {
        self.language = choice;
        self.document
            .insert(LANGUAGE.to_owned(), choice.as_str().into());
    }

    /// Records a new backup budget. Callers pass a value already in the valid
    /// domain (≥ 1) — the field layer's job is reading a file, not policing
    /// the dialog that writes one.
    pub fn set_max_backups(&mut self, max_backups: u32) {
        self.max_backups = max_backups;
        self.document
            .insert(MAX_BACKUPS.to_owned(), max_backups.into());
    }

    /// Records where the window was. Written on clean shutdown only (spec
    /// §12), which is why nothing here is time- or event-sensitive: it amends
    /// the document, and something else decides when the document is saved.
    pub fn set_window(&mut self, window: Window) {
        self.window = Some(window);
        let mut object = match self.document.get(WINDOW) {
            Some(Value::Object(existing)) => existing.clone(),
            _ => Map::new(),
        };
        window.write(&mut object);
        self.document.insert(WINDOW.to_owned(), object.into());
    }

    /// The document as the file should hold it: indented, one field per line,
    /// newline-terminated — this file is meant to be opened and edited, and
    /// what a hand wrote is what a rewrite hands back.
    pub fn to_json(&self) -> String {
        let mut text = serde_json::to_string_pretty(&self.document)
            .expect("a map of JSON values always serialises");
        text.push('\n');
        text
    }
}

/// Reads what was typed into the Settings dialog's backup-budget field
/// (spec §13), which is the same domain the file's own field layer accepts:
/// whole, and ≥ 1.
///
/// Both readings go through [`in_backup_budget`], so the dialog cannot come to
/// accept a budget the file would reject — which would be a value the user
/// chose, saw written, and lost on the next start with a `WARN` line as the
/// only trace.
///
/// Surrounding whitespace is not part of a number: this field holds a count,
/// not text that has to survive a round trip the way an Entry does. Everything
/// else the file would not accept is refused here too — a sign, a decimal
/// point, an exponent — because JSON does not accept them either.
pub fn parse_max_backups(typed: &str) -> Option<u32> {
    let typed = typed.trim();
    if typed.is_empty() || !typed.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    // A digit string too long for `u64` fails here rather than wrapping, and
    // is out of the domain either way.
    in_backup_budget(typed.parse().ok()?)
}

/// The backup budget this application accepts, wherever the number came from:
/// whole, and ≥ 1 (spec §13). Zero is outlawed rather than clamped — rotation
/// at zero deletes the pre-Apply safety net — and the ceiling is the type's.
fn in_backup_budget(n: u64) -> Option<u32> {
    u32::try_from(n).ok().filter(|n| *n >= 1)
}

/// The count delay this application accepts: whole, 0–5000 (v0.2.0 spec §15).
/// Zero is legal — speak on the next tick — and nothing past the ceiling is
/// clamped into it: a typo falls back to the default, visibly in the log.
fn in_count_delay(n: u64) -> Option<u32> {
    u32::try_from(n)
        .ok()
        .filter(|n| *n <= MAX_FILTERED_COUNT_DELAY_MS)
}

/// One field through the field layer: absent is not a rejection, a value
/// `read` cannot make sense of is. The default is passed in rendered rather
/// than spelled here, so the log cannot name a value the run did not use.
fn read_field<T>(
    document: &Map<String, Value>,
    field: &'static str,
    default: &str,
    rejected: &mut Vec<Rejected>,
    read: impl Fn(&Value) -> Option<T>,
) -> Option<T> {
    let value = document.get(field)?;
    read(value).or_else(|| {
        rejected.push(Rejected {
            field,
            raw: raw_text(value),
            default: default.to_owned(),
        });
        None
    })
}

/// A JSON number that is a whole `i32`. `2.5`, `1e9` and a quoted `"10"` are
/// all not one, which is the point: the file says what it says.
fn whole_number(value: &Value) -> Option<i32> {
    value.as_i64().and_then(|n| i32::try_from(n).ok())
}

/// A value as a human would read it back out of the file: a string's own text
/// (`fr`, not `"fr"`), anything else as the JSON it is (`-3`, `null`, `["uk"]`).
fn raw_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}
