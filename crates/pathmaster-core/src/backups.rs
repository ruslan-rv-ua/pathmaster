//! The Backups list: one line per Snapshot file in the directory, what each one
//! says, and what restoring it loads (spec §8, FR-backup-ui; ADR-0006).
//!
//! A [`SnapshotFile`] is a Snapshot's **name** married to what reading it turned
//! out to be, and that pairing is the whole of the module. The name is the one
//! part of a Corrupted Snapshot that still speaks, so a file failing validation
//! is still dated and still shows its Scope; what it has is nothing to restore.
//!
//! Restoring loads a Snapshot into a Working Copy and never into the registry,
//! so what one hands over is exactly what one ordinary Checkpoint captures: the
//! Entries, and the Value Type they were stored under.
//!
//! Reading the files is the caller's, in the imperative shell. This module is
//! handed what a directory turned out to hold; it never opens one.

use crate::session::{Scope, ValueType, ABSENT_VALUE_TYPE};
use crate::snapshot::{Captured, Decoded, SnapshotName};

/// One Snapshot file as the Backups tab shows it: its name, and what reading it
/// turned out to be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotFile {
    name: SnapshotName,
    decoded: Decoded,
}

impl SnapshotFile {
    /// The Scope this Snapshot holds, and so the Scope a Restore targets — a
    /// Snapshot goes back into the Scope it was taken from and nowhere else.
    ///
    /// Read from the name, which is what lets a Corrupted file answer at all.
    pub fn scope(&self) -> Scope {
        self.name.scope()
    }

    /// The instant it was taken, as the list shows it: `2026-08-19 14:32:07`.
    ///
    /// The file name spells that instant `2026-08-19T14-32-07` because a
    /// Windows file name cannot hold a colon (spec §8) — a fact about file
    /// names, and not one to read aloud. Taken from the name rather than from
    /// the file's own `timestamp` field, for the same reason
    /// [`scope`](Self::scope) is.
    pub fn taken(&self) -> String {
        let stamp = self.name.timestamp();
        match stamp.split_once('T') {
            Some((date, time)) => format!("{date} {}", time.replace('-', ":")),
            // Unreachable — a `SnapshotName` exists only for a name that
            // matched the stamp's shape — and answered rather than panicked:
            // a list of backups is not a place to stop being able to speak.
            None => stamp.to_owned(),
        }
    }

    /// What restoring this Snapshot loads into the target Scope's Working
    /// Copy: its Entries, and the Value Type they were stored under.
    ///
    /// `None` for a Corrupted file, which has nothing to load — which is why
    /// its Restore is a disabled control rather than one that fails when
    /// pressed.
    ///
    /// A Snapshot of an **Absent** Scope answers with no Entries and
    /// [`ABSENT_VALUE_TYPE`]: it recorded no Value Type, because an Absent
    /// Scope has none (ADR-0006), and a Working Copy has no Absent state to
    /// restore it into — so what comes back is what a Session loaded from an
    /// Absent Scope already takes. Applying it therefore creates a present and
    /// empty value rather than an Absent Scope; the file keeps the distinction
    /// the Working Copy cannot (spec §8).
    pub fn restores(&self) -> Option<(&[String], ValueType)> {
        match &self.decoded {
            Decoded::Corrupted => None,
            Decoded::Valid(snapshot) => Some(match &snapshot.captured {
                Captured::Present {
                    value_type,
                    entries,
                } => (entries.as_slice(), *value_type),
                Captured::Absent => (&[][..], ABSENT_VALUE_TYPE),
            }),
        }
    }
}

/// The Snapshot files a directory turned out to hold, **newest first**.
///
/// That is the reverse of [`snapshot::listing`](crate::snapshot::listing)'s
/// order, and deliberately: rotation asks the directory for its oldest, and
/// someone restoring wants the backup they took last. Both orders are the one
/// [`SnapshotName`] ordering, so the suffix that separates two Snapshots of a
/// single second is read as the number it is at either end.
pub fn newest_first(read: impl IntoIterator<Item = (SnapshotName, Decoded)>) -> Vec<SnapshotFile> {
    let mut files: Vec<SnapshotFile> = read
        .into_iter()
        .map(|(name, decoded)| SnapshotFile { name, decoded })
        .collect();
    files.sort_by(|file, other| other.name.cmp(&file.name));
    files
}
