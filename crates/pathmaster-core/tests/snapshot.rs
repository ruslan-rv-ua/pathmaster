//! The Snapshot file at the crate boundary (spec §8, ADR-0006, ticket impl-10).
//!
//! A Snapshot is a file a person can open: human-readable JSON recording the
//! Scope, its Value Type and either its Entries or that it was Absent. It is
//! trusted completely or not at all — every way the two validation layers can
//! fail lands on the same word, Corrupted.

use pathmaster_core::logfmt::Timestamp;
use pathmaster_core::session::{Scope, ValueType};
use pathmaster_core::snapshot::{self, Captured, Decoded, Snapshot, SnapshotName};

/// 2026-08-19T14-32-07 local time, the instant the spec's example prints.
const AT: Timestamp = Timestamp {
    year: 2026,
    month: 8,
    day: 19,
    hour: 14,
    minute: 32,
    second: 7,
    offset_minutes: 180,
};

/// The file names of `names`, which is how a test reads a listing back.
fn file_names(names: &[SnapshotName]) -> Vec<String> {
    names
        .iter()
        .map(|name| name.file_name().to_owned())
        .collect()
}

/// The valid half of [`Snapshot::decode`], for the tests whose subject is what
/// a good file says rather than what makes a bad one Corrupted.
fn decoded(text: &str) -> Snapshot {
    match Snapshot::decode(text) {
        Decoded::Valid(snapshot) => snapshot,
        Decoded::Corrupted => panic!("{text} was expected to decode"),
    }
}

// ------------------------------------------------------------------ the shape

#[test]
fn what_is_written_is_the_shape_the_spec_prints() {
    let snapshot = Snapshot {
        timestamp: "2026-08-19T14-32-07".to_owned(),
        scope: Scope::System,
        captured: Captured::Present {
            value_type: ValueType::RegExpandSz,
            entries: vec![r"C:\Windows".to_owned(), r"%JAVA_HOME%\bin".to_owned()],
        },
    };

    // The spec's own example, character for character — indented, one Entry
    // per line, newline-terminated: a file meant to be opened and diffed.
    assert_eq!(
        snapshot.encode(),
        r#"{
  "timestamp": "2026-08-19T14-32-07",
  "scope": "System",
  "valueType": "REG_EXPAND_SZ",
  "entries": [
    "C:\\Windows",
    "%JAVA_HOME%\\bin"
  ]
}
"#,
    );
}

#[test]
fn an_absent_scope_is_written_as_the_spec_prints_it_with_no_value_type() {
    // An Absent Scope has no Value Type — there is no value to have one — so
    // the field that carries it is not part of this shape (ADR-0006: the type
    // is "carried alongside the entries").
    let snapshot = Snapshot {
        timestamp: "2026-08-19T14-32-07".to_owned(),
        scope: Scope::System,
        captured: Captured::Absent,
    };

    assert_eq!(
        snapshot.encode(),
        r#"{
  "timestamp": "2026-08-19T14-32-07",
  "scope": "System",
  "absent": true
}
"#,
    );
}

#[test]
fn a_file_the_application_wrote_reads_back_as_what_it_captured() {
    for captured in [
        Captured::Absent,
        Captured::Present {
            value_type: ValueType::RegSz,
            entries: Vec::new(),
        },
        Captured::Present {
            value_type: ValueType::RegExpandSz,
            entries: vec![r"C:\Windows".to_owned(), String::new()],
        },
    ] {
        let snapshot = Snapshot {
            timestamp: "2026-08-19T14-32-07".to_owned(),
            scope: Scope::User,
            captured,
        };
        assert_eq!(decoded(&snapshot.encode()), snapshot);
    }
}

#[test]
fn absent_and_zero_entries_are_two_different_states_in_the_file() {
    // The distinction the schema exists to make (ticket 06): a Scope that did
    // not exist restores differently from one holding an empty value, so the
    // file says which it was rather than leaving it to be inferred.
    let absent = decoded(r#"{"timestamp": "t", "scope": "User", "absent": true}"#);
    let empty =
        decoded(r#"{"timestamp": "t", "scope": "User", "valueType": "REG_SZ", "entries": []}"#);

    assert_eq!(absent.captured, Captured::Absent);
    assert_eq!(
        empty.captured,
        Captured::Present {
            value_type: ValueType::RegSz,
            entries: Vec::new(),
        },
    );
}

#[test]
fn the_file_says_the_same_instant_and_scope_its_name_does() {
    // Not by two callers agreeing: the file is built from the name it will be
    // written under, so there is no second argument to get wrong.
    let name = SnapshotName::next(AT, Scope::System, &[]);
    let snapshot = Snapshot::under(&name, Captured::Absent);

    assert_eq!(snapshot.timestamp, name.timestamp());
    assert_eq!(snapshot.timestamp, "2026-08-19T14-32-07");
    assert_eq!(snapshot.scope, name.scope());
    assert_eq!(snapshot.scope, Scope::System);
}

// ------------------------------------------------------- layer one: it parses

#[test]
fn unparsable_json_is_corrupted() {
    for text in ["", "   ", "not json at all", "{", r#"{"scope": }"#, "{,}"] {
        assert_eq!(Snapshot::decode(text), Decoded::Corrupted, "{text:?}");
    }
}

#[test]
fn a_half_written_file_is_corrupted_rather_than_half_restorable() {
    // What an interrupted write leaves behind if the temp+rename ever fails
    // to protect the directory (ticket 07): a prefix of a good file. It must
    // not look restorable, and truncated JSON is exactly what layer one is.
    let whole =
        r#"{"timestamp": "t", "scope": "User", "valueType": "REG_SZ", "entries": ["C:\\a"]}"#;
    assert_eq!(
        Snapshot::decode(&whole[..whole.len() / 2]),
        Decoded::Corrupted,
    );
}

#[test]
fn a_root_that_is_not_an_object_is_corrupted_even_though_it_is_valid_json() {
    for text in [
        "[]",
        r#"["C:\\Windows"]"#,
        "42",
        "null",
        "true",
        r#""User""#,
    ] {
        assert_eq!(Snapshot::decode(text), Decoded::Corrupted, "{text:?}");
    }
}

#[test]
fn a_byte_order_mark_is_not_what_makes_a_snapshot_corrupted() {
    // The same reading `settings.json` gets: an invisible character several
    // Windows editors add is not a reason to lose a backup.
    let snapshot = decoded("\u{feff}{\"timestamp\": \"t\", \"scope\": \"User\", \"absent\": true}");
    assert_eq!(snapshot.scope, Scope::User);
}

// -------------------------------------------------- layer two: it has a shape

#[test]
fn a_timestamp_that_is_missing_or_not_a_string_is_corrupted() {
    for text in [
        r#"{"scope": "User", "absent": true}"#,
        r#"{"timestamp": 20260819, "scope": "User", "absent": true}"#,
        r#"{"timestamp": null, "scope": "User", "absent": true}"#,
    ] {
        assert_eq!(Snapshot::decode(text), Decoded::Corrupted, "{text:?}");
    }
}

#[test]
fn a_timestamp_is_only_ever_asked_to_be_a_string() {
    // Shape, not calendar: the Backups list dates a row from the file name,
    // so a hand-written stamp inside the file is not worth a second rulebook.
    let snapshot = decoded(r#"{"timestamp": "whenever", "scope": "User", "absent": true}"#);
    assert_eq!(snapshot.timestamp, "whenever");
}

#[test]
fn a_scope_that_is_not_one_of_the_two_words_is_corrupted() {
    for text in [
        r#"{"timestamp": "t", "absent": true}"#,
        r#"{"timestamp": "t", "scope": "system", "absent": true}"#,
        r#"{"timestamp": "t", "scope": "Machine", "absent": true}"#,
        r#"{"timestamp": "t", "scope": 0, "absent": true}"#,
    ] {
        assert_eq!(Snapshot::decode(text), Decoded::Corrupted, "{text:?}");
    }
}

#[test]
fn entries_without_a_readable_value_type_are_corrupted() {
    // H15, the hazard the schema exists to close: entries whose Value Type is
    // missing or unreadable cannot reproduce what the registry held, and a
    // guessed type is exactly the guess ADR-0006 refused to make.
    for text in [
        r#"{"timestamp": "t", "scope": "User", "entries": []}"#,
        r#"{"timestamp": "t", "scope": "User", "valueType": "REG_DWORD", "entries": []}"#,
        r#"{"timestamp": "t", "scope": "User", "valueType": "reg_sz", "entries": []}"#,
        r#"{"timestamp": "t", "scope": "User", "valueType": 1, "entries": []}"#,
    ] {
        assert_eq!(Snapshot::decode(text), Decoded::Corrupted, "{text:?}");
    }
}

#[test]
fn entries_that_are_not_an_array_of_strings_are_corrupted() {
    for text in [
        r#"{"timestamp": "t", "scope": "User", "valueType": "REG_SZ", "entries": "C:\\a"}"#,
        r#"{"timestamp": "t", "scope": "User", "valueType": "REG_SZ", "entries": {}}"#,
        r#"{"timestamp": "t", "scope": "User", "valueType": "REG_SZ", "entries": ["C:\\a", 2]}"#,
        r#"{"timestamp": "t", "scope": "User", "valueType": "REG_SZ", "entries": [null]}"#,
        r#"{"timestamp": "t", "scope": "User", "valueType": "REG_SZ", "entries": [["C:\\a"]]}"#,
    ] {
        assert_eq!(Snapshot::decode(text), Decoded::Corrupted, "{text:?}");
    }
}

#[test]
fn a_file_that_says_both_or_neither_says_nothing_coherent() {
    for text in [
        // Neither: no captured value at all.
        r#"{"timestamp": "t", "scope": "User", "valueType": "REG_SZ"}"#,
        // Both: an Absent Scope cannot also have held Entries. What counts is
        // that the file names both keys, not what the second one claims.
        r#"{"timestamp": "t", "scope": "User", "valueType": "REG_SZ", "entries": [], "absent": true}"#,
        r#"{"timestamp": "t", "scope": "User", "valueType": "REG_SZ", "entries": [], "absent": false}"#,
    ] {
        assert_eq!(Snapshot::decode(text), Decoded::Corrupted, "{text:?}");
    }
}

#[test]
fn absent_is_the_word_true_and_nothing_else() {
    for text in [
        r#"{"timestamp": "t", "scope": "User", "absent": false}"#,
        r#"{"timestamp": "t", "scope": "User", "absent": "true"}"#,
        r#"{"timestamp": "t", "scope": "User", "absent": 1}"#,
        r#"{"timestamp": "t", "scope": "User", "absent": null}"#,
    ] {
        assert_eq!(Snapshot::decode(text), Decoded::Corrupted, "{text:?}");
    }
}

#[test]
fn a_field_this_version_does_not_know_is_not_corruption() {
    // Forward compatibility in the direction that matters: a v0.2 field must
    // not make every v0.1 Snapshot unrestorable in the version that wrote it.
    let snapshot = decoded(
        r#"{"timestamp": "t", "scope": "User", "valueType": "REG_SZ",
             "entries": ["C:\\a"], "machineName": "somebody's PC"}"#,
    );
    assert_eq!(
        snapshot.captured,
        Captured::Present {
            value_type: ValueType::RegSz,
            entries: vec![r"C:\a".to_owned()],
        },
    );
}

// --------------------------------------------------------------- the file name

#[test]
fn the_name_carries_the_instant_and_the_scope_and_reads_back_as_itself() {
    // Local time, and the Scope in the name: rotation is a per-Scope budget,
    // so a file's Scope has to be readable without opening it.
    for (scope, expected) in [
        (Scope::System, "2026-08-19T14-32-07-System.json"),
        (Scope::User, "2026-08-19T14-32-07-User.json"),
    ] {
        let name = SnapshotName::next(AT, scope, &[]);
        assert_eq!(name.file_name(), expected);

        let parsed = SnapshotName::parse(expected).expect("the app's own name parses");
        assert_eq!(parsed, name);
        assert_eq!(parsed.timestamp(), "2026-08-19T14-32-07");
        assert_eq!(parsed.scope(), scope);
        assert_eq!(parsed.suffix(), None);
    }
}

#[test]
fn the_offset_is_not_in_the_name_because_the_time_is_local() {
    // One machine, one person, no syncing: UTC would buy nothing and read
    // worse. Two instants an hour apart in the same zone differ in the name;
    // the same wall clock in another zone does not.
    let elsewhere = Timestamp {
        offset_minutes: -480,
        ..AT
    };
    assert_eq!(
        SnapshotName::next(elsewhere, Scope::User, &[]).file_name(),
        "2026-08-19T14-32-07-User.json",
    );
}

#[test]
fn a_single_digit_field_is_padded_so_every_name_is_the_same_width() {
    // Fixed width is what makes "oldest first" readable in the directory and
    // sortable in the list.
    let early = Timestamp {
        year: 2026,
        month: 1,
        day: 2,
        hour: 3,
        minute: 4,
        second: 5,
        offset_minutes: 0,
    };
    assert_eq!(
        SnapshotName::next(early, Scope::System, &[]).file_name(),
        "2026-01-02T03-04-05-System.json",
    );
}

#[test]
fn a_second_snapshot_in_the_same_second_takes_the_lowest_free_suffix() {
    // Sub-second precision was rejected: it would put the clock's resolution
    // into a name a person reads, for nothing.
    let mut existing = Vec::new();
    for expected in [
        "2026-08-19T14-32-07-System.json",
        "2026-08-19T14-32-07-System-1.json",
        "2026-08-19T14-32-07-System-2.json",
    ] {
        let name = SnapshotName::next(AT, Scope::System, &existing);
        assert_eq!(name.file_name(), expected);
        existing.push(name);
    }
}

#[test]
fn the_two_scopes_never_collide_with_each_other() {
    // Two Applies in the same second, one per Scope: the Scope is part of the
    // name, so neither has to give way to the other.
    let system = SnapshotName::next(AT, Scope::System, &[]);
    let user = SnapshotName::next(AT, Scope::User, std::slice::from_ref(&system));

    assert_eq!(user.file_name(), "2026-08-19T14-32-07-User.json");
    assert_eq!(user.suffix(), None);
}

#[test]
fn a_gap_left_by_rotation_is_never_filled() {
    // Rotation deletes by age, not by suffix, so the plain name can be gone
    // while a suffixed one survives. The suffix still only climbs.
    let survivors = snapshot::listing(["2026-08-19T14-32-07-System-1.json"]);

    assert_eq!(
        SnapshotName::next(AT, Scope::System, &survivors).file_name(),
        "2026-08-19T14-32-07-System-2.json",
    );
}

#[test]
fn within_one_second_a_later_snapshot_always_sorts_later() {
    // Why the suffix may not be reused. Rotation deletes the oldest, so a
    // freed name is always an *old* name — reissuing it would make the newest
    // Snapshot of that second sort as its oldest, and the rotation that runs
    // straight after an Apply would delete the backup that Apply just took.
    let mut directory = snapshot::listing([
        "2026-08-19T14-32-07-System.json",
        "2026-08-19T14-32-07-System-1.json",
        "2026-08-19T14-32-07-System-2.json",
    ]);
    // Rotation at a budget of two took the oldest, the plain name.
    directory.remove(0);

    let newest = SnapshotName::next(AT, Scope::System, &directory);
    directory.push(newest.clone());
    directory.sort();

    assert_eq!(directory.last(), Some(&newest));
    assert_eq!(newest.file_name(), "2026-08-19T14-32-07-System-3.json");
}

#[test]
fn a_foreign_name_is_not_a_snapshot() {
    // Silently invisible, never Corrupted: Corrupted is reserved for a file
    // that looks like a Snapshot and fails validation, not for one that never
    // claimed to be one.
    for name in [
        // The atomic write's own temporary, mid-write by construction.
        "2026-08-19T14-32-07-System.tmp",
        "2026-08-19T14-32-07-System.json.tmp",
        "2026-08-19T14-32-07-System",
        // Not this application's file at all.
        "notes.json",
        "settings.json",
        "",
        ".json",
        // A stamp that is not the stamp.
        "2026-08-19-System.json",
        "2026-8-19T14-32-07-System.json",
        "2026-08-19T14:32:07-System.json",
        "yyyy-mm-ddThh-mm-ss-System.json",
        "2026-08-19T14-32-07System.json",
        // A Scope that is not a Scope.
        "2026-08-19T14-32-07-Machine.json",
        "2026-08-19T14-32-07-.json",
        "2026-08-19T14-32-07.json",
        // A suffix that is not a number.
        "2026-08-19T14-32-07-System-.json",
        "2026-08-19T14-32-07-System-x.json",
        "2026-08-19T14-32-07-System-+1.json",
        "2026-08-19T14-32-07-System-1-2.json",
        // Anything at all around the name.
        " 2026-08-19T14-32-07-System.json",
        "copy of 2026-08-19T14-32-07-System.json",
        "2026-08-19T14-32-07-System.json.bak",
    ] {
        assert_eq!(SnapshotName::parse(name), None, "{name:?}");
    }
}

#[test]
fn letter_case_in_the_scope_or_the_extension_does_not_hide_a_snapshot() {
    // Windows names one file either way, and a backup that vanishes over
    // letter case would be the least explicable thing this could do.
    for name in [
        "2026-08-19T14-32-07-system.json",
        "2026-08-19T14-32-07-SYSTEM.JSON",
        "2026-08-19T14-32-07-System.Json",
    ] {
        let parsed = SnapshotName::parse(name).unwrap_or_else(|| panic!("{name:?}"));
        assert_eq!(parsed.scope(), Scope::System);
    }
}

// ----------------------------------------------------------------- the listing

#[test]
fn a_listing_is_the_snapshots_among_a_directorys_files_oldest_first() {
    // What `read_dir` hands over, in whatever order it feels like: the
    // Snapshots come back in one order, and nothing else comes back at all.
    let listing = snapshot::listing([
        "2026-08-19T14-32-07-User.json",
        "settings.json",
        "2026-08-19T09-00-00-System.json",
        "2026-08-19T14-32-07-System.tmp",
        "2026-08-20T08-15-42-System.json",
    ]);

    assert_eq!(
        file_names(&listing),
        [
            "2026-08-19T09-00-00-System.json",
            "2026-08-19T14-32-07-User.json",
            "2026-08-20T08-15-42-System.json",
        ],
    );
}

#[test]
fn a_suffix_orders_as_the_number_it_is_not_as_the_text_it_looks_like() {
    // The one place spelling and age disagree: `-10` is newer than `-2`, and
    // rotation deletes by age.
    let listing = snapshot::listing([
        "2026-08-19T14-32-07-System-10.json",
        "2026-08-19T14-32-07-System-2.json",
        "2026-08-19T14-32-07-System.json",
    ]);

    assert_eq!(
        file_names(&listing),
        [
            "2026-08-19T14-32-07-System.json",
            "2026-08-19T14-32-07-System-2.json",
            "2026-08-19T14-32-07-System-10.json",
        ],
    );
}

#[test]
fn the_order_is_the_same_whatever_order_the_directory_was_read_in() {
    let names = [
        "2026-08-19T14-32-07-System.json",
        "2026-08-19T14-32-07-User.json",
        "2026-08-19T14-32-08-System.json",
    ];
    let mut backwards = names;
    backwards.reverse();

    assert_eq!(
        file_names(&snapshot::listing(names)),
        file_names(&snapshot::listing(backwards)),
    );
}

// ---------------------------------------------------------------- the property

proptest::proptest! {
    /// The one thing a Snapshot must do: give back exactly what it captured.
    /// Entries are raw substrings — quotes, backslashes, newlines and any
    /// letter case are all part of one, and all of them survive the file.
    #[test]
    fn any_captured_value_survives_the_round_trip(
        entries in proptest::collection::vec("(?s).*", 0..8),
        expands in proptest::bool::ANY,
        absent in proptest::bool::ANY,
    ) {
        let snapshot = Snapshot {
            timestamp: "2026-08-19T14-32-07".to_owned(),
            scope: Scope::System,
            captured: if absent {
                Captured::Absent
            } else {
                Captured::Present {
                    value_type: if expands { ValueType::RegExpandSz } else { ValueType::RegSz },
                    entries,
                }
            },
        };

        proptest::prop_assert_eq!(
            Snapshot::decode(&snapshot.encode()),
            Decoded::Valid(snapshot),
        );
    }
}
