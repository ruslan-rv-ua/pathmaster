//! The log line format (spec §14): what one record looks like, independent of
//! any file. English always, deliberately outside the Catalogue — the log is
//! written for a developer reading a machine they cannot see.
//!
//! Two absolute PII prohibitions are enforced here, at the line-building API:
//! no Entry/PATH text and no absolute filesystem paths appear in any record.
//! There is no constructor taking a free-form message — every record is built
//! from derived facts (counts, lengths, Value Type, Scope, data *state*), and
//! the two unavoidable free-text inlets are bounded: rejected settings values
//! are truncated, and the panic message is the panic's own payload.

use std::fmt::Write as _;

use crate::session::{Scope, ValueType};

/// Exactly three levels (spec §14), padded to five characters so columns
/// align. A record's level is fixed by its constructor, never chosen by the
/// caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// The healthy-run skeleton: startup, successful Apply, clean shutdown.
    Info,
    /// Anything the app survived by itself.
    Warn,
    /// A user-requested operation failed; a panic.
    Error,
}

impl Level {
    /// The five-character column form: `INFO ` / `WARN ` / `ERROR`.
    pub fn padded(self) -> &'static str {
        match self {
            Level::Info => "INFO ",
            Level::Warn => "WARN ",
            Level::Error => "ERROR",
        }
    }
}

/// A wall-clock instant in local time with its UTC offset — RFC 3339's local
/// form. Core never reads a clock (no I/O); the platform supplies one of
/// these per record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timestamp {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    /// Minutes east of UTC (`+03:00` is `180`); may be negative.
    pub offset_minutes: i32,
}

impl Timestamp {
    /// RFC 3339 with numeric offset: `2026-08-19T15:36:31+03:00`.
    pub fn rfc3339(&self) -> String {
        let sign = if self.offset_minutes < 0 { '-' } else { '+' };
        let offset = self.offset_minutes.unsigned_abs();
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{}{:02}:{:02}",
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
            sign,
            offset / 60,
            offset % 60,
        )
    }
}

/// The Data Directory fact the startup line carries: the state and, when
/// read-only, the named reason (spec §3 names it, never a bare "read-only")
/// — but never the location, which is PII prohibition #2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataState {
    Writable,
    ReadOnlyOwnLocationUnknown,
    ReadOnlyCannotCreate,
    ReadOnlyNotWritable,
}

impl DataState {
    fn describe(self) -> &'static str {
        match self {
            DataState::Writable => "writable",
            DataState::ReadOnlyOwnLocationUnknown => "read-only (own location unknown)",
            DataState::ReadOnlyCannotCreate => "read-only (cannot create data directory)",
            DataState::ReadOnlyNotWritable => "read-only (data directory not writable)",
        }
    }
}

/// The raw fact behind a failure — the OS error code, or the registry type
/// this application does not support — never a free-form message. The
/// platform's registry and file errors map onto this; core stays ignorant of
/// `io::Error`.
///
/// One enum for two callers on purpose: a startup read that failed and an
/// Apply that failed have the same two things to say, and §9's "every failure
/// lands one log record with the raw error code" is one rule, not two.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureCause {
    /// The call itself failed; the OS error code when the OS gave one.
    Io { os_error: Option<i32> },
    /// The value exists but is neither `REG_SZ` nor `REG_EXPAND_SZ`.
    UnsupportedType { vtype: u32 },
}

impl FailureCause {
    fn describe(self) -> String {
        match self {
            FailureCause::Io {
                os_error: Some(code),
            } => format!("os error {code}"),
            FailureCause::Io { os_error: None } => "io error".to_string(),
            FailureCause::UnsupportedType { vtype } => {
                format!("unsupported registry type {vtype}")
            }
        }
    }
}

/// Which step of FR-apply's fixed order failed — the whole of what an Apply
/// failure line has to say beyond the error code (spec §5, §9).
///
/// The three are the taxonomy's three failing rows. The fourth row is the
/// external-change dialog, which is a question rather than a failure, and the
/// fifth is the broadcast, which is not a failure at all
/// ([`Record::broadcast_timed_out`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyStep {
    /// The re-read that opens the sequence (§9's fifth row).
    ReRead,
    /// The Snapshot of what was just re-read, which is written before the
    /// registry is touched.
    Snapshot,
    /// The registry write itself.
    Write,
}

impl ApplyStep {
    fn describe(self) -> &'static str {
        match self {
            ApplyStep::ReRead => "re-read",
            ApplyStep::Snapshot => "backup",
            ApplyStep::Write => "registry write",
        }
    }
}

/// One log record: a level, an area, and a message built from derived facts.
/// Fields are private on purpose — these constructors are the whole way in,
/// which is what makes the PII prohibitions enforceable rather than advisory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    level: Level,
    area: &'static str,
    message: String,
}

impl Record {
    /// The startup line — version, elevation, data *state*, Interface
    /// Language. The one way a pasted log identifies its build.
    pub fn startup(version: &str, elevated: bool, data: DataState, language: &str) -> Self {
        Record {
            level: Level::Info,
            area: "startup",
            message: format!(
                "PathMaster {version}, elevated: {}, data: {}, language: {language}",
                if elevated { "yes" } else { "no" },
                data.describe(),
            ),
        }
    }

    /// The audit line Apply earns as the one system-mutating operation —
    /// derived facts only: Scope, Entry count, value length, Value Type.
    pub fn apply_written(
        scope: Scope,
        entries: usize,
        chars: usize,
        value_type: ValueType,
    ) -> Self {
        Record {
            level: Level::Info,
            area: "apply",
            message: format!(
                "{} scope written, {entries} entries, {chars} chars, {}",
                scope_name(scope),
                value_type_name(value_type),
            ),
        }
    }

    /// The Apply failure line (spec §9): which step of the fixed order failed
    /// and the raw code behind it. The taxonomy requires exactly one of these
    /// per failure, and it is `ERROR` because a user-requested operation did
    /// not happen.
    ///
    /// The Scope and the step are all the context there is: no path text, no
    /// entry count — nothing was written, so there is nothing to audit.
    pub fn apply_failed(scope: Scope, step: ApplyStep, cause: FailureCause) -> Self {
        Record {
            level: Level::Error,
            area: "apply",
            message: format!(
                "{} scope not applied, {} failed ({})",
                scope_name(scope),
                step.describe(),
                cause.describe(),
            ),
        }
    }

    /// The `WM_SETTINGCHANGE` broadcast that no window answered in time
    /// (spec §4, TC-wm-settingchange).
    ///
    /// Emphatically **not** an Apply failure: the write succeeded, and
    /// already-open shells never see the change regardless — only newly
    /// launched processes do. So it is never surfaced, and this line is its
    /// only trace. It is appended past the `Logger` by the thread that made
    /// the call, which may still be blocked when the Apply that started it has
    /// long since returned (ADR-0008).
    pub fn broadcast_timed_out() -> Self {
        Record {
            level: Level::Warn,
            area: "broadcast",
            message: "WM_SETTINGCHANGE timed out, already-open processes keep the old value"
                .to_string(),
        }
    }

    /// The clean-shutdown line. A killed process shows as this line's
    /// absence; a panic shows as the `ERROR panic:` line above it.
    pub fn shutdown_clean() -> Self {
        Record {
            level: Level::Info,
            area: "shutdown",
            message: "clean".to_string(),
        }
    }

    /// The logger's own recovery line, written on the first successful write
    /// after a run of dropped records — which is why the format never assumes
    /// a record originates in application code.
    pub fn records_lost(count: u64) -> Self {
        Record {
            level: Level::Warn,
            area: "log",
            message: format!("{count} records were lost"),
        }
    }

    /// The unreadable-`settings.json` line (spec §13). The user is told by a
    /// startup dialog; this is the developer's half — which of the two failure
    /// layers bit, and whether the file was set aside or is still where they
    /// left it. The two file names are names, not locations, so PII
    /// prohibition #2 is untouched.
    pub fn settings_unreadable(set_aside: bool) -> Self {
        Record {
            level: Level::Warn,
            area: "settings",
            message: format!(
                "settings.json could not be read, {}, using defaults",
                if set_aside {
                    "set aside as settings.json.bad"
                } else {
                    "left in place"
                },
            ),
        }
    }

    /// The per-field settings fallback line — the log is the *only* witness
    /// of a rejected value (spec §13), so the raw value is carried, but
    /// truncated to 100 characters with a marker: a pathological file must
    /// not put a megabyte on one line. This truncation is the only inlet for
    /// file-supplied text into any record.
    pub fn settings_field_invalid(field: &str, raw: &str, default: &str) -> Self {
        let shown = match raw.char_indices().nth(TRUNCATE_AT_CHARS) {
            Some((cut, _)) => format!("\"{}…\" [truncated]", &raw[..cut]),
            None => format!("\"{raw}\""),
        };
        Record {
            level: Level::Warn,
            area: "settings",
            message: format!("field \"{field}\" invalid (raw: {shown}), using default {default}"),
        }
    }

    /// A Scope whose startup read failed (spec never names this state, so the
    /// run takes the degraded road: an empty, non-writable Session — nothing
    /// can be written over a value that was never read). This line is the
    /// developer's only witness; the UI shows the consequence, not the cause.
    pub fn scope_read_failed(scope: Scope, cause: FailureCause) -> Self {
        Record {
            level: Level::Warn,
            area: "registry",
            message: format!(
                "{} scope could not be read ({}), treated as empty and non-writable",
                scope_name(scope),
                cause.describe(),
            ),
        }
    }

    /// The panic line: message plus `file:line`, no backtrace (the PDB is not
    /// shipped, so frames would be bare addresses). The platform hook formats
    /// and appends it past the logger; core supplies only the shape.
    pub fn panic(message: &str, file: &str, line: u32) -> Self {
        Record {
            level: Level::Error,
            area: "panic",
            message: format!("{message} ({file}:{line})"),
        }
    }
}

fn scope_name(scope: Scope) -> &'static str {
    match scope {
        Scope::User => "User",
        Scope::System => "System",
    }
}

fn value_type_name(value_type: ValueType) -> &'static str {
    match value_type {
        ValueType::RegSz => "REG_SZ",
        ValueType::RegExpandSz => "REG_EXPAND_SZ",
    }
}

/// Rejected settings values are cut after this many characters (spec §14's
/// "~100 chars").
const TRUNCATE_AT_CHARS: usize = 100;

/// Formats one record as its complete line, newline included — the writer
/// appends exactly this, so "one record per line" cannot drift. Any newline
/// smuggled in by a panic message or a rejected settings value becomes a
/// space here: the invariant is unconditional, not per-constructor.
pub fn line(timestamp: &Timestamp, record: &Record) -> String {
    let mut out = timestamp.rfc3339();
    let _ = write!(out, " {} {}: ", record.level.padded(), record.area);
    out.extend(
        record
            .message
            .chars()
            .map(|c| if c == '\n' || c == '\r' { ' ' } else { c }),
    );
    out.push('\n');
    out
}
