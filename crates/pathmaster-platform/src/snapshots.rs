//! Snapshot files on disk: where they live, what is there, and taking one
//! away (spec §8, ADR-0006, ADR-0008).
//!
//! Two callers, one module. An Apply Run writes a Snapshot and rotates the
//! Scope's budget; the Backups tab reads them all and shows the user where they
//! live. So `data\backups\` is spelled once, and — more usefully —
//! **one listing serves both questions
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

use pathmaster_core::backups::{self, SnapshotFile};
use pathmaster_core::rotation;
use pathmaster_core::session::Scope;
use pathmaster_core::snapshot::{self, Decoded, Snapshot, SnapshotName};

use crate::datadir;
use crate::shell;

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

/// Every Snapshot in `dir`, read and validated — the whole content of the
/// Backups list, newest first (spec §8, FR-backup-ui).
///
/// The listing decides what is a Snapshot at all; this decides what each one
/// turned out to be. **A file that cannot be read is Corrupted**, exactly as
/// one that fails to parse: both are a file claiming to be a Snapshot that
/// nothing can be restored from, which is the whole of what the row and its
/// disabled Restore have to say. Passing it over instead would make the list
/// disagree with the directory — the file is there, and it counts toward its
/// Scope's rotation budget either way.
pub fn load(dir: &Path) -> io::Result<Vec<SnapshotFile>> {
    let names = listing(dir)?;
    Ok(backups::newest_first(names.into_iter().map(|name| {
        let decoded = match fs::read_to_string(dir.join(name.file_name())) {
            Ok(text) => Snapshot::decode(&text),
            Err(_) => Decoded::Corrupted,
        };
        (name, decoded)
    })))
}

/// Hands the Snapshots' own directory to the shell — Tools → Open Backups
/// Folder (spec §15). An open, never a file dialog: this shows a folder, it
/// does not ask for one.
///
/// A shell that will not open it is silence, and the answer is dropped here on
/// purpose. There is no Announcement for it — the catalogue is closed at
/// fourteen — and none to give: the only run this can happen in is one whose
/// Data Directory does not exist either.
pub fn open_folder(data_dir: &Path) {
    let _ = shell::open(ensure_folder(data_dir).as_os_str());
}

/// **Creates** `data\backups\` if it is not there, and answers the folder
/// [`open_folder`] then shows: that directory, or the Data Directory itself
/// when it could not be created.
///
/// The creation is why this is not the pure query its answer looks like. It is
/// not a side effect smuggled onto a menu item, though: this is the directory
/// the application writes its own backups into, and the next Apply creates it
/// anyway ([`write`]). What the fallback buys is that a menu item reading as
/// available opens *something* — a Read-only Data run cannot create it, and the
/// directory it would have lived in says more than nothing at all.
///
/// Both this and [`open_folder`] take the **Data** Directory rather than the
/// backups directory the rest of the module takes, because choosing between the
/// two is the question they answer.
pub fn ensure_folder(data_dir: &Path) -> PathBuf {
    let backups = dir(data_dir);
    match fs::create_dir_all(&backups) {
        Ok(()) => backups,
        Err(_) => data_dir.to_owned(),
    }
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
