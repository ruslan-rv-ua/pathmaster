//! The Backups list: what one row says, what restoring it loads, and the order
//! the rows come in (spec §8, FR-backup-ui; ADR-0006).
//!
//! Every row here is built the way the tab builds one — a name paired with what
//! reading its file turned out to be — because the pairing is the rule under
//! test: the name is the one part of a Corrupted Snapshot that still speaks.

use pathmaster_core::backups::{self, Row};
use pathmaster_core::logfmt::Timestamp;
use pathmaster_core::session::{Scope, ValueType};
use pathmaster_core::snapshot::{Captured, Decoded, Snapshot, SnapshotName};

fn at(second: u8) -> Timestamp {
    Timestamp {
        year: 2026,
        month: 8,
        day: 19,
        hour: 14,
        minute: 32,
        second,
        offset_minutes: 180,
    }
}

/// The name that instant earns a Snapshot of `scope`, with no collision.
fn name(scope: Scope, second: u8) -> SnapshotName {
    SnapshotName::next(at(second), scope, &[])
}

/// One row, as the tab builds it: a name, and what its file decoded to.
fn row(name: SnapshotName, decoded: Decoded) -> Row {
    backups::rows([(name, decoded)])
        .pop()
        .expect("one name is one row")
}

fn valid(name: &SnapshotName, captured: Captured) -> Decoded {
    Decoded::Valid(Snapshot::under(name, captured))
}

fn present(entries: &[&str]) -> Captured {
    Captured::Present {
        value_type: ValueType::RegExpandSz,
        entries: entries.iter().map(|entry| (*entry).to_string()).collect(),
    }
}

#[test]
fn a_row_dates_a_snapshot_the_way_a_person_reads_a_date() {
    // The file name spells the instant `2026-08-19T14-32-07` because a Windows
    // file name cannot hold a colon. That is a fact about file names, not
    // something to read aloud.
    let name = name(Scope::User, 7);
    let row = row(name.clone(), valid(&name, present(&[r"C:\bin"])));

    assert_eq!(row.taken(), "2026-08-19 14:32:07");
}

#[test]
fn a_row_takes_its_scope_from_the_name_so_a_corrupted_file_still_has_one() {
    // The two things a Corrupted Snapshot still says are the two things its
    // name carries — which is why the list is built from the name and not from
    // the `scope` field inside a file that failed to parse.
    let name = name(Scope::System, 7);
    let row = row(name, Decoded::Corrupted);

    assert_eq!(row.scope(), Scope::System);
    assert_eq!(row.taken(), "2026-08-19 14:32:07");
}

#[test]
fn a_corrupted_snapshot_has_nothing_to_restore() {
    // Not "discouraged" — nothing to load at all, which is why its Restore is
    // a disabled control rather than one that fails when pressed.
    let name = name(Scope::User, 7);

    assert_eq!(row(name, Decoded::Corrupted).restores(), None);
}

#[test]
fn a_row_restores_the_entries_and_the_value_type_the_snapshot_captured() {
    let name = name(Scope::User, 7);
    let row = row(
        name.clone(),
        valid(
            &name,
            Captured::Present {
                value_type: ValueType::RegSz,
                entries: vec![r"C:\bin".to_string(), r"%JAVA_HOME%\bin".to_string()],
            },
        ),
    );

    let (entries, value_type) = row.restores().expect("a valid Snapshot restores");
    assert_eq!(entries, [r"C:\bin", r"%JAVA_HOME%\bin"]);
    // Carried, never assumed: reproducing the Value Type it captured is the
    // whole reason the schema records one (ADR-0006).
    assert_eq!(value_type, ValueType::RegSz);
}

#[test]
fn a_snapshot_of_an_absent_scope_restores_an_empty_working_copy() {
    // An Absent Scope had no value and so no Value Type to record. What comes
    // back is what a Session loaded from an Absent Scope already takes: no
    // Entries, typed as the first Apply will create the value (spec §4).
    let name = name(Scope::System, 7);
    let row = row(name.clone(), valid(&name, Captured::Absent));

    let (entries, value_type) = row.restores().expect("an Absent Snapshot restores");
    assert!(entries.is_empty());
    assert_eq!(value_type, ValueType::RegExpandSz);
}

#[test]
fn the_list_shows_the_newest_snapshot_first() {
    // The reverse of the order rotation reads the directory in, and
    // deliberately: rotation wants the oldest, and someone restoring wants the
    // backup they took last.
    let oldest = name(Scope::User, 1);
    let middle = name(Scope::System, 2);
    let newest = name(Scope::User, 3);

    let rows = backups::rows([
        (middle.clone(), Decoded::Corrupted),
        (oldest.clone(), Decoded::Corrupted),
        (newest.clone(), Decoded::Corrupted),
    ]);

    assert_eq!(
        rows.iter().map(Row::taken).collect::<Vec<String>>(),
        [
            "2026-08-19 14:32:03",
            "2026-08-19 14:32:02",
            "2026-08-19 14:32:01"
        ],
    );
    assert_eq!(
        rows.iter().map(Row::scope).collect::<Vec<Scope>>(),
        [Scope::User, Scope::System, Scope::User],
    );
}

#[test]
fn two_snapshots_of_one_second_come_back_in_the_order_their_suffixes_give_them() {
    // Within one second the collision suffix *is* the age (spec §8), so
    // newest-first has to read it as the number it is: `-10` is newer than
    // `-2`, which spelling alone would not give.
    let mut taken: Vec<SnapshotName> = Vec::new();
    for _ in 0..=10 {
        let next = SnapshotName::next(at(7), Scope::User, &taken);
        taken.push(next);
    }
    let content = |word: &str| present(&[word]);

    let rows = backups::rows([
        (taken[0].clone(), valid(&taken[0], content("first"))),
        (taken[10].clone(), valid(&taken[10], content("last"))),
        (taken[1].clone(), valid(&taken[1], content("second"))),
    ]);

    assert_eq!(
        rows.iter()
            .map(|row| row.restores().expect("a valid Snapshot").0[0].clone())
            .collect::<Vec<String>>(),
        ["last", "second", "first"],
    );
}
