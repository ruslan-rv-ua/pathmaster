//! Snapshot files on disk: where they live, what is there, and taking one
//! away (spec §8, ADR-0006, ADR-0008).
//!
//! Two callers, one module. An Apply Run writes a Snapshot and rotates the
//! Scope's budget; the Backups tab lists, reads and deletes. So `data\backups\`
//! is spelled once, and — more usefully — **one listing serves both questions
//! the write asks of the directory**: what name the next Snapshot gets
//! ([`SnapshotName::next`]) and which files no longer fit the budget
//! ([`rotation::overflow`]). Those two must see the same set of files, because
//! the rule that keeps a fresh Snapshot from being rotated away by the very
//! Apply that took it — a suffix rotation freed is never reissued — is a rule
//! about one second's worth of names.
//!
//! Everything that decides is in `pathmaster-core`: the name's shape, the
//! file's schema, the budget's arithmetic. What is here is the filesystem.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use pathmaster_core::rotation;
use pathmaster_core::session::Scope;
use pathmaster_core::snapshot::{self, Snapshot, SnapshotName};

use crate::datadir;

/// The Snapshots' own directory, inside the Data Directory (TC-file-structure).
/// Never translated — a directory name is outside the Catalogue (spec §11).
pub const DIR_NAME: &str = "backups";

/// Where this run's Snapshots live. The Data Directory arrives as a path
/// rather than as a `Writable` state: startup predicts, Apply verifies
/// (ADR-0002, ADR-0008).
pub fn dir(data_dir: &Path) -> PathBuf {
    data_dir.join(DIR_NAME)
}

/// The Snapshots in `dir`, oldest first — the one reading of the directory
/// that both the next name and the rotation are answered from.
///
/// A directory that does not exist yet is an empty listing and not an error:
/// the first Apply of a run creates it, and until then "no Snapshots" is the
/// literal truth. Every other failure is reported, because a listing that
/// silently reads as empty would name a Snapshot over one already there and
/// rotate against a budget it cannot see.
///
/// Foreign files and the `.tmp` of a write in progress are invisible, which is
/// [`snapshot::listing`]'s rule and not this function's.
pub fn listing(dir: &Path) -> io::Result<Vec<SnapshotName>> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut file_names = Vec::new();
    for entry in entries {
        if let Some(name) = entry?.file_name().to_str() {
            file_names.push(name.to_owned());
        }
    }
    Ok(snapshot::listing(file_names.iter().map(String::as_str)))
}

/// Writes one Snapshot under the name it was built for, creating `data\backups\`
/// if this is the run's first.
///
/// Atomic, through [`datadir::write_replace`]: the content lands in a `.tmp`
/// beside the target and replaces it in one rename, so a reader — the Backups
/// tab, or the other instance — never sees half a file, and a `.tmp` that
/// survives a crash is invisible to every listing (spec §8).
pub fn write(dir: &Path, name: &SnapshotName, snapshot: &Snapshot) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    datadir::write_replace(&dir.join(name.file_name()), snapshot.encode().as_bytes())
}

/// Deletes the Snapshots of `scope` that no longer fit its budget — oldest
/// first, and only that Scope's, however old the other's are (spec §8,
/// FR-backup-rotation).
///
/// `listing` must be the whole directory **including the Snapshot just
/// written**, which is what stops a run from deleting the backup it has this
/// moment taken.
///
/// A file another instance has already deleted is success, not failure: two
/// instances are a designed state, and nothing about the budget depends on a
/// file surviving until it is deleted. So this answers nothing — a rotation
/// that could not delete has cost the user nothing, and the next one will find
/// the same file and try again.
pub fn rotate(dir: &Path, listing: &[SnapshotName], scope: Scope, max_backups: u32) {
    for name in rotation::overflow(listing, scope, max_backups) {
        let _ = fs::remove_file(dir.join(name.file_name()));
    }
}
