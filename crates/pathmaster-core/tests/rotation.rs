//! The backup budget at the crate boundary (spec §8 FR-backup-rotation,
//! ticket impl-10).
//!
//! `maxBackups` is an independent per-Scope budget, never a count pooled
//! across the directory: fifty User Applies must not silently wipe every
//! System Snapshot. Rotation works from file names alone — it never opens a
//! file, which is why a Corrupted Snapshot rotates exactly like a good one.

use pathmaster_core::logfmt::Timestamp;
use pathmaster_core::rotation;
use pathmaster_core::session::Scope;
use pathmaster_core::snapshot::{self, Decoded, Snapshot, SnapshotName};

/// A directory holding `count` Snapshots of `scope`, a second apart, the
/// oldest first — enough to make "the oldest of that Scope" mean something.
fn taken_every_second(scope: Scope, count: u32) -> Vec<String> {
    (0..count)
        .map(|second| {
            format!(
                "2026-08-19T14-32-{second:02}-{}.json",
                match scope {
                    Scope::System => "System",
                    Scope::User => "User",
                }
            )
        })
        .collect()
}

/// The file names rotation would have the caller delete.
fn overflowing(file_names: &[String], scope: Scope, max_backups: u32) -> Vec<String> {
    let listing = snapshot::listing(file_names.iter().map(String::as_str));
    rotation::overflow(&listing, scope, max_backups)
        .iter()
        .map(|name| name.file_name().to_owned())
        .collect()
}

#[test]
fn a_directory_within_its_budget_has_nothing_to_rotate() {
    let names = taken_every_second(Scope::System, 3);

    assert!(overflowing(&names, Scope::System, 50).is_empty());
    assert!(overflowing(&names, Scope::System, 3).is_empty());
}

#[test]
fn overflow_is_the_oldest_of_that_scope_and_the_newest_always_survive() {
    let names = taken_every_second(Scope::User, 5);

    assert_eq!(
        overflowing(&names, Scope::User, 3),
        [
            "2026-08-19T14-32-00-User.json",
            "2026-08-19T14-32-01-User.json",
        ],
    );
}

#[test]
fn the_two_scopes_have_independent_budgets() {
    // The failure this decision exists to avoid: a pooled count lets one
    // Scope's Applies starve the other's backups out of existence.
    let mut names = taken_every_second(Scope::User, 50);
    names.extend(taken_every_second(Scope::System, 2));

    assert_eq!(overflowing(&names, Scope::System, 5), Vec::<String>::new());
    assert_eq!(overflowing(&names, Scope::User, 50), Vec::<String>::new());
    // And when User does overflow, only User files are named.
    assert!(overflowing(&names, Scope::User, 10)
        .iter()
        .all(|name| name.ends_with("-User.json")));
}

#[test]
fn a_budget_of_one_keeps_the_newest_snapshot_and_nothing_else() {
    let names = taken_every_second(Scope::System, 3);

    assert_eq!(
        overflowing(&names, Scope::System, 1),
        [
            "2026-08-19T14-32-00-System.json",
            "2026-08-19T14-32-01-System.json",
        ],
    );
}

#[test]
fn a_budget_lowered_since_the_last_apply_rotates_the_whole_backlog_at_once() {
    // `maxBackups` is a budget, not an increment: what fits is what is kept,
    // however far over the directory has drifted.
    let names = taken_every_second(Scope::User, 12);

    assert_eq!(overflowing(&names, Scope::User, 2).len(), 10);
}

#[test]
fn a_corrupted_snapshot_counts_toward_its_scopes_budget_and_rotates_like_any_other() {
    // Rotation is given names, never content — it has no way to treat a
    // Corrupted file differently, which is exactly the decision: the Scope is
    // readable from the name whatever happened to the content, and exempting
    // corrupted files would let them accumulate outside the budget rotation
    // exists to enforce.
    let directory = [
        ("2026-08-19T14-32-00-System.json", "half a fi"),
        ("2026-08-19T14-32-01-System.json", r#"{"scope": "System"}"#),
        (
            "2026-08-19T14-32-02-System.json",
            r#"{"timestamp": "t", "scope": "System", "absent": true}"#,
        ),
    ];
    assert_eq!(Snapshot::decode(directory[0].1), Decoded::Corrupted);
    assert_eq!(Snapshot::decode(directory[1].1), Decoded::Corrupted);

    let names: Vec<String> = directory
        .iter()
        .map(|(name, _)| (*name).to_owned())
        .collect();

    // Budget 2: three Snapshots of this Scope, so the oldest goes — and it is
    // a Corrupted one, deleted exactly as a good one would be.
    assert_eq!(
        overflowing(&names, Scope::System, 2),
        ["2026-08-19T14-32-00-System.json"],
    );
    // Budget 1: the survivor is the newest, not the newest *readable* one.
    assert_eq!(
        overflowing(&names, Scope::System, 1),
        [
            "2026-08-19T14-32-00-System.json",
            "2026-08-19T14-32-01-System.json",
        ],
    );
}

#[test]
fn a_file_another_instance_already_deleted_is_simply_not_in_the_listing() {
    // Two instances rotate the same directory. Rotation names files from the
    // listing it was given, so a file that has since gone is not named twice —
    // and the caller's delete treats not-found as success for the rest.
    let mut names = taken_every_second(Scope::System, 5);
    names.remove(0);

    assert_eq!(
        overflowing(&names, Scope::System, 3),
        ["2026-08-19T14-32-01-System.json"],
    );
}

#[test]
fn rotating_a_directory_that_has_already_been_rotated_deletes_nothing() {
    let names = taken_every_second(Scope::System, 5);
    let deleted = overflowing(&names, Scope::System, 3);
    let survivors: Vec<String> = names
        .into_iter()
        .filter(|name| !deleted.contains(name))
        .collect();

    assert_eq!(survivors.len(), 3);
    assert!(overflowing(&survivors, Scope::System, 3).is_empty());
}

#[test]
fn a_budget_of_zero_still_keeps_one_snapshot() {
    // Zero cannot reach here from the file — `settings.json` rejects it
    // (spec §13) — so this is the floor under a caller's mistake, at the one
    // step that deletes: obeying a zero budget would delete the Snapshot the
    // Apply in progress has just taken.
    let names = taken_every_second(Scope::System, 2);

    assert_eq!(
        overflowing(&names, Scope::System, 0),
        ["2026-08-19T14-32-00-System.json"],
    );
}

#[test]
fn an_empty_directory_rotates_to_nothing() {
    assert!(overflowing(&[], Scope::System, 50).is_empty());
}

#[test]
fn a_snapshot_of_the_other_scope_is_never_named_even_when_it_is_the_oldest() {
    let names: Vec<String> = ["2026-08-19T14-32-00-User.json"]
        .into_iter()
        .map(str::to_owned)
        .chain(taken_every_second(Scope::System, 3))
        .collect();

    assert_eq!(
        overflowing(&names, Scope::System, 1),
        [
            "2026-08-19T14-32-00-System.json",
            "2026-08-19T14-32-01-System.json",
        ],
    );
}

#[test]
fn the_names_to_delete_come_back_oldest_first() {
    // The order a caller deletes in, and the order a log would read in.
    let names = taken_every_second(Scope::User, 4);
    let deleted = overflowing(&names, Scope::User, 1);
    let mut oldest_first = deleted.clone();
    oldest_first.sort();

    assert_eq!(deleted, oldest_first);
}

#[test]
fn a_foreign_file_is_not_a_snapshot_and_is_never_rotated() {
    // Rotation deletes files. Anything that is not a Snapshot is out of its
    // reach entirely — including the `.tmp` of a write in progress, which is
    // some other instance's half-written file, not this one's backlog.
    let names: Vec<String> = [
        "settings.json",
        "2026-08-19T14-32-00-System.tmp",
        "PathMaster.log",
    ]
    .into_iter()
    .map(str::to_owned)
    .chain(taken_every_second(Scope::System, 2))
    .collect();

    assert_eq!(
        overflowing(&names, Scope::System, 1),
        ["2026-08-19T14-32-00-System.json"],
    );
}

#[test]
fn the_snapshot_an_apply_has_just_taken_is_never_the_one_rotation_deletes() {
    // The whole loop, in the one second where the two modules can disagree:
    // three Applies, a rotation that takes the oldest, then a fourth Apply in
    // that same second. Names are how rotation tells age, so the fourth
    // Snapshot must not be handed a name that reads older than the survivors —
    // rotation runs straight after the write, and would delete the backup the
    // write took.
    let at = Timestamp {
        year: 2026,
        month: 8,
        day: 19,
        hour: 14,
        minute: 32,
        second: 7,
        offset_minutes: 180,
    };
    let mut directory: Vec<String> = Vec::new();
    for _ in 0..3 {
        let listing = snapshot::listing(directory.iter().map(String::as_str));
        directory.push(
            SnapshotName::next(at, Scope::System, &listing)
                .file_name()
                .to_owned(),
        );
    }

    let rotated = overflowing(&directory, Scope::System, 2);
    directory.retain(|name| !rotated.contains(name));

    let listing = snapshot::listing(directory.iter().map(String::as_str));
    let fresh = SnapshotName::next(at, Scope::System, &listing)
        .file_name()
        .to_owned();
    directory.push(fresh.clone());

    assert!(
        !overflowing(&directory, Scope::System, 2).contains(&fresh),
        "rotation named the Snapshot the Apply had just taken: {directory:?}",
    );
}
