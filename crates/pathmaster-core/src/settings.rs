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

/// How the geometry default reads in the log — the only default with no value
/// to render, because having none *is* the default (spec §12: first run is
/// 900×650 centred, which the window decides, not the file).
const DEFAULT_WINDOW_SHOWN: &str = "none";

/// The field names, which are the file's API surface — never translated
/// (spec §11) and never spelled twice.
const LANGUAGE: &str = "language";
const MAX_BACKUPS: &str = "maxBackups";
const WINDOW: &str = "window";

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
            |value| {
                value
                    .as_u64()
                    .and_then(|n| u32::try_from(n).ok())
                    .filter(|n| *n >= 1)
            },
        );
        let window = read_field(
            &document,
            WINDOW,
            DEFAULT_WINDOW_SHOWN,
            &mut rejected,
            Window::read,
        );

        Parsed::Readable {
            file: SettingsFile {
                document,
                language: language.unwrap_or(DEFAULT_LANGUAGE),
                max_backups: max_backups.unwrap_or(DEFAULT_MAX_BACKUPS),
                window,
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
