//! The Snapshot file: what it holds, how it is named, and what makes it
//! Corrupted (spec §8, ADR-0006).

use std::cmp::Ordering;

use serde_json::{Map, Value};

use crate::logfmt::Timestamp;
use crate::session::{Scope, ValueType};

/// The field names, which are the file's API surface — never translated, and
/// never spelled twice.
const TIMESTAMP: &str = "timestamp";
const SCOPE: &str = "scope";
const VALUE_TYPE: &str = "valueType";
const ENTRIES: &str = "entries";
const ABSENT: &str = "absent";

/// What a Snapshot captured of a Scope's value: the Entries it held with the
/// Value Type they were stored under, or that the Scope was Absent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Captured {
    Absent,
    Present {
        value_type: ValueType,
        entries: Vec<String>,
    },
}

/// One Snapshot — the whole content of one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    /// The instant it was taken, in the file's own form
    /// (`2026-08-19T14-32-07`). Read back as whatever string the file holds:
    /// the Backups list dates a Snapshot from its file name, which is the one
    /// place a Corrupted file still speaks.
    pub timestamp: String,
    pub scope: Scope,
    pub captured: Captured,
}

/// What a file turned out to be. Both arms name the outcome rather than the
/// JSON: nothing downstream can do anything with *how* a Snapshot failed, and
/// [`Decoded::Corrupted`] is the word the user is shown (`CONTEXT.md`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decoded {
    Valid(Snapshot),
    /// Unparsable JSON, or a required field missing or of the wrong shape.
    /// There is no partial recovery and no guessed field: a Snapshot is
    /// trusted completely or not used at all.
    Corrupted,
}

impl Snapshot {
    /// The Snapshot the file `name` will hold. Its instant and its Scope come
    /// from that name rather than from a second reading of the clock, so what
    /// a Snapshot says and what it is called cannot disagree — there is no
    /// pair of arguments a caller could get out of step.
    pub fn under(name: &SnapshotName, captured: Captured) -> Snapshot {
        Snapshot {
            timestamp: name.timestamp().to_owned(),
            scope: name.scope(),
            captured,
        }
    }

    /// The JSON text the file holds: indented, one field per line,
    /// newline-terminated — this file is meant to be opened and read.
    pub fn encode(&self) -> String {
        let mut document = Map::new();
        document.insert(TIMESTAMP.to_owned(), self.timestamp.clone().into());
        document.insert(SCOPE.to_owned(), scope_name(self.scope).into());
        match &self.captured {
            Captured::Present {
                value_type,
                entries,
            } => {
                document.insert(VALUE_TYPE.to_owned(), value_type_name(*value_type).into());
                document.insert(ENTRIES.to_owned(), entries.as_slice().into());
            }
            Captured::Absent => {
                document.insert(ABSENT.to_owned(), true.into());
            }
        }
        let mut text = serde_json::to_string_pretty(&Value::Object(document))
            .expect("a map of JSON values always serialises");
        text.push('\n');
        text
    }

    /// Reads a file's text through both validation layers (spec §8): it parses
    /// as JSON, and then every field it must have is there with the right
    /// shape. Any failure at either layer is the same outcome — Corrupted.
    ///
    /// A field this version does not know is ignored rather than fatal: a
    /// later version's addition must not make today's Snapshots unrestorable.
    pub fn decode(text: &str) -> Decoded {
        // The same reading `settings.json` gets: a UTF-8 BOM is what several
        // Windows editors leave in front of otherwise perfectly good JSON, and
        // a Snapshot lost to an invisible character would be unexplainable.
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let Ok(Value::Object(document)) = serde_json::from_str::<Value>(text) else {
            return Decoded::Corrupted;
        };
        match read(&document) {
            Some(snapshot) => Decoded::Valid(snapshot),
            None => Decoded::Corrupted,
        }
    }
}

/// The shape layer: every question it asks has one answer, and any `None`
/// anywhere in it is the whole file's verdict.
fn read(document: &Map<String, Value>) -> Option<Snapshot> {
    let timestamp = document.get(TIMESTAMP)?.as_str()?.to_owned();
    let scope = scope_in_file(document.get(SCOPE)?.as_str()?)?;
    let captured = match (document.get(ENTRIES), document.get(ABSENT)) {
        // Exactly one of the two, never both and never neither: a file
        // claiming an Absent Scope *and* its Entries says nothing coherent.
        (Some(entries), None) => Captured::Present {
            value_type: value_type_in_file(document.get(VALUE_TYPE)?.as_str()?)?,
            entries: entries
                .as_array()?
                .iter()
                .map(|entry| Some(entry.as_str()?.to_owned()))
                .collect::<Option<Vec<String>>>()?,
        },
        // `absent` carries no Value Type, because an Absent Scope has none;
        // `absent: false` is not the other shape either — it is a file
        // claiming neither.
        (None, Some(Value::Bool(true))) => Captured::Absent,
        _ => return None,
    };
    Some(Snapshot {
        timestamp,
        scope,
        captured,
    })
}

/// A Snapshot's file name: `YYYY-MM-DDTHH-MM-SS-<Scope>.json`, local time,
/// with a numeric suffix when a second holds more than one Snapshot of a Scope
/// (`…-System-1.json`).
///
/// The name is the only part of a Snapshot that still speaks when its content
/// does not, which is why so much is asked of it: the Backups list dates a
/// Corrupted row from it, and rotation — a per-Scope budget — reads both the
/// Scope and the age of every file without opening one.
///
/// The name as the directory spells it is kept, never re-rendered: what a
/// caller deletes must be the file it was handed, letter for letter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotName {
    file_name: String,
    scope: Scope,
    suffix: Option<u32>,
}

impl SnapshotName {
    /// The name the next Snapshot of `scope` gets, given the Snapshots already
    /// in the directory: the plain name, or the suffix after the highest one
    /// that second already holds.
    ///
    /// Sub-second precision was rejected in favour of the suffix: the clock's
    /// resolution has no business in a name a person reads.
    ///
    /// The suffix only ever climbs, and a gap rotation left behind is never
    /// filled. That is not tidiness — it is what keeps the name's ordering
    /// honest. Within one second the suffix *is* the age, so reissuing a freed
    /// name would hand the newest Snapshot the oldest name, and the rotation
    /// that follows the Apply would delete the backup that Apply just took.
    pub fn next(at: Timestamp, scope: Scope, existing: &[SnapshotName]) -> SnapshotName {
        let timestamp = stamp(at);
        // Each Snapshot of this second rules out every suffix up to its own;
        // no Snapshot of it at all is what the plain name means.
        let suffix = existing
            .iter()
            .filter(|name| name.timestamp() == timestamp && name.scope == scope)
            .map(|name| name.suffix.map_or(1, |number| number.saturating_add(1)))
            .max();
        SnapshotName {
            file_name: format!(
                "{timestamp}-{}{}{EXTENSION}",
                scope_name(scope),
                match suffix {
                    Some(number) => format!("-{number}"),
                    None => String::new(),
                },
            ),
            scope,
            suffix,
        }
    }

    /// Reads one directory entry's file name. `None` means the file is not a
    /// Snapshot at all — a foreign file, or the atomic write's own `.tmp`
    /// temporary — and a file that is not a Snapshot is invisible rather than
    /// Corrupted: Corrupted is for a file that claims to be one and fails.
    pub fn parse(file_name: &str) -> Option<SnapshotName> {
        let name = strip_extension(file_name)?;
        let (timestamp, rest) = name.split_at_checked(STAMP_SHAPE.len())?;
        if !is_stamp(timestamp) {
            return None;
        }
        let rest = rest.strip_prefix('-')?;
        let (scope, suffix) = match rest.split_once('-') {
            Some((scope, suffix)) => (scope, Some(suffix_number(suffix)?)),
            None => (rest, None),
        };
        Some(SnapshotName {
            file_name: file_name.to_owned(),
            scope: scope_in_name(scope)?,
            suffix,
        })
    }

    /// The name as the directory spells it — what to open, and what to delete.
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// The instant in the name, in the file's own form: `2026-08-19T14-32-07`.
    /// Read off the name rather than stored beside it — both constructors put
    /// it there, and one copy cannot go stale against another.
    pub fn timestamp(&self) -> &str {
        &self.file_name[..STAMP_SHAPE.len()]
    }

    pub fn scope(&self) -> Scope {
        self.scope
    }

    /// The collision suffix, when a second held more than one Snapshot of this
    /// Scope.
    pub fn suffix(&self) -> Option<u32> {
        self.suffix
    }
}

/// Oldest first — which is the order rotation deletes in and the order the
/// Backups list is built from. The suffix is compared as the number it is:
/// `-10` follows `-2`, which spelling alone would not give.
impl Ord for SnapshotName {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp()
            .cmp(other.timestamp())
            .then(self.suffix.cmp(&other.suffix))
            // Same second, same suffix, different Scope: any order will do, so
            // long as it is the same one every run.
            .then_with(|| self.file_name.cmp(&other.file_name))
    }
}

impl PartialOrd for SnapshotName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The Snapshots among a directory's file names, oldest first.
///
/// Everything else is silently invisible — a foreign file, and the `.tmp` of a
/// write in progress, which is skipped by its extension rather than parsed,
/// because it is half a file by construction. The order is imposed here rather
/// than assumed of the directory: rotation deletes the oldest and the Backups
/// list shows the newest, and neither may depend on what `read_dir` felt like.
pub fn listing<'a>(file_names: impl IntoIterator<Item = &'a str>) -> Vec<SnapshotName> {
    let mut snapshots: Vec<SnapshotName> = file_names
        .into_iter()
        .filter_map(SnapshotName::parse)
        .collect();
    snapshots.sort();
    snapshots
}

/// The instant a Snapshot was taken, as both the file name and the file's
/// `timestamp` field spell it: local time, fixed width, and no UTC offset —
/// one machine, one person, nothing syncing.
fn stamp(at: Timestamp) -> String {
    format!(
        "{:04}-{:02}-{:02}T{:02}-{:02}-{:02}",
        at.year, at.month, at.day, at.hour, at.minute, at.second,
    )
}

/// The stamp's shape, spelled as the spec writes it: `d` is any ASCII digit,
/// everything else is itself. A shape, deliberately not a calendar — nothing
/// here needs the 31st of February to be a different kind of wrong.
const STAMP_SHAPE: &str = "dddd-dd-ddTdd-dd-dd";

/// The extension a Snapshot has, and the one thing the `.tmp` of a write in
/// progress does not.
const EXTENSION: &str = ".json";

fn is_stamp(text: &str) -> bool {
    text.len() == STAMP_SHAPE.len()
        && text
            .bytes()
            .zip(STAMP_SHAPE.bytes())
            .all(|(byte, shape)| match shape {
                b'd' => byte.is_ascii_digit(),
                literal => byte == literal,
            })
}

/// The name without its extension, case-insensitively — for the same reason
/// [`scope_in_name`] is. `None` for anything not ending in `.json`, which is
/// what keeps the `.tmp` of a write in progress out of every listing without
/// ever opening it.
fn strip_extension(file_name: &str) -> Option<&str> {
    let (name, extension) =
        file_name.split_at_checked(file_name.len().checked_sub(EXTENSION.len())?)?;
    extension.eq_ignore_ascii_case(EXTENSION).then_some(name)
}

/// A collision suffix is digits and nothing else. `+1` is a number to
/// `u32::from_str` and not a name this application ever writes, so the rule is
/// stated positively rather than left to what a parser happens to tolerate.
fn suffix_number(text: &str) -> Option<u32> {
    if !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// How the file spells a Scope — in its `scope` field and in its name alike.
/// This and [`value_type_name`] are the file's whole vocabulary: never
/// translated (spec §11), and spelled here rather than at each place that
/// reads one, so what is written and what is read cannot drift apart.
///
/// The log says these same two words ([`crate::logfmt`]) and deliberately says
/// them itself: a line a developer reads is a display detail, while this is a
/// format every Snapshot ever written is stored in. One must be free to change
/// without the other following it.
fn scope_name(scope: Scope) -> &'static str {
    match scope {
        Scope::User => "User",
        Scope::System => "System",
    }
}

/// How the file spells a Value Type — the registry's own two names.
fn value_type_name(value_type: ValueType) -> &'static str {
    match value_type {
        ValueType::RegSz => "REG_SZ",
        ValueType::RegExpandSz => "REG_EXPAND_SZ",
    }
}

/// The Scope a `scope` field names. Exactly, letter for letter: this is JSON
/// content, where `system` is simply a different string.
fn scope_in_file(word: &str) -> Option<Scope> {
    [Scope::System, Scope::User]
        .into_iter()
        .find(|scope| scope_name(*scope) == word)
}

/// The Scope a file name carries, case-insensitively — Windows names one file
/// either way, and a backup invisible over letter case would be the least
/// explicable thing this application could do.
fn scope_in_name(word: &str) -> Option<Scope> {
    [Scope::System, Scope::User]
        .into_iter()
        .find(|scope| word.eq_ignore_ascii_case(scope_name(*scope)))
}

/// The Value Type a `valueType` field names, exactly.
fn value_type_in_file(word: &str) -> Option<ValueType> {
    [ValueType::RegSz, ValueType::RegExpandSz]
        .into_iter()
        .find(|value_type| value_type_name(*value_type) == word)
}
