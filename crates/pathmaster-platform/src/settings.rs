//! `settings.json` in the Data Directory (spec §13): finding it, reading it,
//! and setting it aside when it cannot be read.
//!
//! The rules this file exists to hold are all about *which* state the file was
//! found in, because three of them look alike from a distance and behave
//! differently: **absent** is a first run (defaults, no dialog, no log line, no
//! file until something naturally writes one), **read** may still carry
//! per-field rejections for the log, and **unreadable** costs the user one
//! startup dialog and costs the file its name.
//!
//! Read in both data modes; written only in Writable Data — and that asymmetry
//! is in the signatures. [`read`] takes the whole [`DataDirState`] because it
//! behaves differently in each, while [`write`] takes a directory, which the
//! caller can only get by matching [`DataDirState::Writable`].

use std::fs;
use std::io;
use std::path::Path;

use pathmaster_core::settings::{Parsed, Rejected, SettingsFile};

use crate::datadir::{self, DataDirState};

/// The file itself (TC-file-structure). Never translated — file names are
/// outside the Catalogue (spec §11).
pub const FILE_NAME: &str = "settings.json";

/// The single set-aside copy (TC-file-structure). One name, not a series: a
/// growing pile of `.bad.1`, `.bad.2` would be litter in a directory whose
/// contents the spec enumerates exactly.
pub const BAD_FILE_NAME: &str = "settings.json.bad";

/// Which of the three states `settings.json` was found in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// No file at all — a first run, and not a failure.
    Absent,
    /// A parsable object root. `rejected` names the known fields whose values
    /// were invalid; each is owed one `WARN` line, and nothing else.
    Read(Vec<Rejected>),
    /// Unparsable JSON, a root that is not an object, or a file that could not
    /// be read as text at all. The run uses full defaults and owes the user one
    /// startup dialog. `set_aside` reports whether the file was moved to
    /// `settings.json.bad` — false in Read-only Data, where nothing is written,
    /// and false if the move itself failed.
    Unreadable { set_aside: bool },
}

/// The settings this run uses, and where they came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Load {
    pub file: SettingsFile,
    pub source: Source,
}

/// Reads `settings.json` out of the Data Directory.
///
/// Read in both data modes: a run that cannot write its directory can still
/// obey the language the user chose. Only the set-aside is withheld in
/// Read-only Data — it is a write, and Read-only Data performs none.
pub fn read(data: &DataDirState) -> Load {
    let Some(dir) = data.dir() else {
        return absent();
    };
    let target = dir.join(FILE_NAME);
    let bytes = match fs::read(&target) {
        Ok(bytes) => bytes,
        // Absent is its own state, and a first run is not a failure.
        Err(error) if error.kind() == io::ErrorKind::NotFound => return absent(),
        // The file exists and this run cannot get at it: a lock held by the
        // other instance (which is a designed state — there is no
        // single-instance lock), a denied ACL, a directory wearing the name.
        // The run is on defaults and the user is owed the dialog, but the file
        // is **not** set aside: nothing is known to be wrong with its contents,
        // and moving a good file over the single `.bad` copy would destroy
        // exactly what the set-aside exists to preserve.
        Err(_) => {
            return Load {
                file: SettingsFile::defaults(),
                source: Source::Unreadable { set_aside: false },
            }
        }
    };
    // From here the bytes are in hand, so every remaining failure is the file's
    // own content — text that is not UTF-8 is as unreadable as JSON that does
    // not parse, and both earn the set-aside.
    let parsed = match String::from_utf8(bytes) {
        Ok(text) => SettingsFile::parse(&text),
        Err(_) => Parsed::Unreadable,
    };
    match parsed {
        Parsed::Readable { file, rejected } => Load {
            file,
            source: Source::Read(rejected),
        },
        Parsed::Unreadable => set_aside(&target, data.is_writable()),
    }
}

/// First run, or a run with nowhere to look: full defaults and nothing to say.
fn absent() -> Load {
    Load {
        file: SettingsFile::defaults(),
        source: Source::Absent,
    }
}

/// Replaces `settings.json` with what `file` now says, atomically — a reader
/// (the other instance, which is a designed state) never sees a half-written
/// file, and a failed write leaves the previous one intact.
///
/// Taking the directory rather than the [`DataDirState`] is the point: the only
/// way to get one is to match [`DataDirState::Writable`], so "written only in
/// Writable Data" is visible at the call site instead of trusted.
pub fn write(dir: &Path, file: &SettingsFile) -> io::Result<()> {
    datadir::write_replace(&dir.join(FILE_NAME), file.to_json().as_bytes())
}

/// Sets a file whose contents cannot be read aside, and answers with full
/// defaults.
///
/// The move is one atomic replace onto the single `.bad` name, so the previous
/// incident's copy is overwritten rather than accumulating. Set aside, never
/// overwritten in place: the dominant cause of an unparsable file is a hand
/// edit, and the edit is the one thing the user cannot get back. Read-only Data
/// performs no writes at all, a rename included — the user still gets the
/// dialog, and their file stays exactly where they left it.
fn set_aside(target: &Path, writable: bool) -> Load {
    let moved = writable
        && datadir::rename_replacing(target, &target.with_file_name(BAD_FILE_NAME)).is_ok();
    Load {
        file: SettingsFile::defaults(),
        source: Source::Unreadable { set_aside: moved },
    }
}
