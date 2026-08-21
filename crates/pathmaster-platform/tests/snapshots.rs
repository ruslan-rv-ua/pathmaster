//! The Snapshot files on disk (spec §8, ticket impl-13).
//!
//! What the module decides is nothing — the name's shape, the schema and the
//! budget all live in `pathmaster-core`. What it owns is the directory, so
//! these tests are about the filesystem: a directory that is not there yet, a
//! `.tmp` a listing must not see, and a rotation that deletes one Scope's
//! oldest and leaves the other Scope alone however old its files are.

#![cfg(windows)]

use std::fs;
use std::path::Path;

use pathmaster_core::logfmt::Timestamp;
use pathmaster_core::session::{Scope, ValueType};
use pathmaster_core::snapshot::{Captured, Snapshot, SnapshotName};
use pathmaster_platform::snapshots;

fn at(second: u8) -> Timestamp {
    Timestamp {
        year: 2026,
        month: 8,
        day: 21,
        hour: 14,
        minute: 32,
        second,
        offset_minutes: 180,
    }
}

/// One Snapshot of `scope`, written under the name that instant earns it.
fn take(dir: &Path, scope: Scope, second: u8) -> SnapshotName {
    let existing = snapshots::listing(dir).expect("a listing");
    let name = SnapshotName::next(at(second), scope, &existing);
    let snapshot = Snapshot::under(
        &name,
        Captured::Present {
            value_type: ValueType::RegExpandSz,
            entries: vec![r"C:\bin".to_string()],
        },
    );
    snapshots::write(dir, &name, &snapshot).expect("a written Snapshot");
    name
}

fn file_names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .expect("the directory")
        .map(|entry| {
            entry
                .expect("an entry")
                .file_name()
                .to_string_lossy()
                .into()
        })
        .collect();
    names.sort();
    names
}

#[test]
fn the_snapshots_live_in_backups_under_the_data_directory() {
    // Spelled once, here, so the two callers cannot disagree about where a
    // backup is (spec §8, TC-file-structure).
    assert_eq!(
        snapshots::dir(Path::new(r"C:\tools\data")),
        Path::new(r"C:\tools\data\backups"),
    );
}

#[test]
fn a_directory_that_does_not_exist_yet_lists_as_empty_and_not_as_a_failure() {
    // The first Apply of a first run finds no `data\backups\` at all, and "no
    // Snapshots" is the literal truth rather than something to report.
    let temp = tempfile::tempdir().unwrap();
    let dir = snapshots::dir(temp.path());

    assert_eq!(
        snapshots::listing(&dir).expect("an empty listing"),
        Vec::new()
    );
    assert!(!dir.exists(), "listing must not create anything");
}

#[test]
fn writing_creates_the_directory_and_leaves_no_temporary_behind() {
    let temp = tempfile::tempdir().unwrap();
    let dir = snapshots::dir(temp.path());

    let name = take(&dir, Scope::User, 7);

    assert_eq!(file_names(&dir), vec![name.file_name().to_string()]);
    assert_eq!(
        Snapshot::decode(&fs::read_to_string(dir.join(name.file_name())).unwrap()),
        pathmaster_core::snapshot::Decoded::Valid(Snapshot::under(
            &name,
            Captured::Present {
                value_type: ValueType::RegExpandSz,
                entries: vec![r"C:\bin".to_string()],
            },
        )),
    );
}

#[test]
fn a_listing_sees_snapshots_oldest_first_and_nothing_else_at_all() {
    let temp = tempfile::tempdir().unwrap();
    let dir = snapshots::dir(temp.path());
    let older = take(&dir, Scope::User, 7);
    let newer = take(&dir, Scope::System, 9);
    // A foreign file and the `.tmp` of a write in progress: invisible rather
    // than Corrupted, because neither claims to be a Snapshot.
    fs::write(dir.join("notes.txt"), b"hello").unwrap();
    fs::write(dir.join("2026-08-21T14-32-08-User.json.4242.tmp"), b"{}").unwrap();

    let listing = snapshots::listing(&dir).expect("a listing");

    assert_eq!(
        listing
            .iter()
            .map(SnapshotName::file_name)
            .collect::<Vec<&str>>(),
        vec![older.file_name(), newer.file_name()],
    );
}

#[test]
fn rotation_deletes_the_oldest_of_one_scope_and_never_the_others() {
    // `maxBackups` is an independent per-Scope budget: fifty User Applies must
    // not silently wipe every System Snapshot (spec §8, FR-backup-rotation).
    let temp = tempfile::tempdir().unwrap();
    let dir = snapshots::dir(temp.path());
    let oldest_user = take(&dir, Scope::User, 1);
    let newer_user = take(&dir, Scope::User, 2);
    let lone_system = take(&dir, Scope::System, 3);

    let listing = snapshots::listing(&dir).expect("a listing");
    snapshots::rotate(&dir, &listing, Scope::User, 1);

    assert_eq!(
        file_names(&dir),
        {
            let mut kept = vec![
                newer_user.file_name().to_string(),
                lone_system.file_name().to_string(),
            ];
            kept.sort();
            kept
        },
        "only the User Scope's oldest goes"
    );
    assert!(!dir.join(oldest_user.file_name()).exists());
}

#[test]
fn rotation_tolerates_a_file_another_instance_has_already_deleted() {
    // Two instances are a designed state — there is no single-instance lock —
    // and nothing about the budget depends on a file surviving until it is
    // deleted (spec §8).
    let temp = tempfile::tempdir().unwrap();
    let dir = snapshots::dir(temp.path());
    let oldest = take(&dir, Scope::User, 1);
    let newest = take(&dir, Scope::User, 2);
    let listing = snapshots::listing(&dir).expect("a listing");
    fs::remove_file(dir.join(oldest.file_name())).unwrap();

    snapshots::rotate(&dir, &listing, Scope::User, 1);

    assert_eq!(file_names(&dir), vec![newest.file_name().to_string()]);
}
