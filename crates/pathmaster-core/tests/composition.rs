//! The Catalogue's composition rules at the crate boundary (spec §11, §10.1,
//! §12, §7; ticket impl-19, ADR-0009).
//!
//! Everything here runs through the identity adapter and links no wxWidgets.
//! What composition can get wrong is language-independent — whether a
//! placeholder was filled, whether a suffix was appended, whether zero Entries
//! took their own msgid rather than a plural form — so a test that asserted
//! real Ukrainian would be asserting the `.po` gate's job with wx's plural
//! selection replaced by someone else's (ADR-0009). The real translations are
//! gated in `catalogue.rs`, and one composed sentence is asserted end-to-end
//! in the binary's wx smoke test.
//!
//! (`tests/catalogue.rs` is the completeness gate, which is about the strings
//! themselves; this file is about what is built out of them.)

use pathmaster_core::backups::{self, SnapshotFile};
use pathmaster_core::catalogue::{Announcement, Catalogue, Lookup, ScopeCounts, UndoDirection};
use pathmaster_core::diagnostics::Issue;
use pathmaster_core::logfmt::Timestamp;
use pathmaster_core::msgids;
use pathmaster_core::path::Rejection;
use pathmaster_core::session::{Operation, Scope, UndoOutcome, ValueType};
use pathmaster_core::snapshot::{Captured, Decoded, Snapshot, SnapshotName};
use pathmaster_core::thresholds;

/// The tests' adapter: it answers with the msgid and picks
/// `n == 1 ? singular : plural`. That is not a convenience — it is precisely
/// wxdragon's documented fallback when no catalogue answers, so composition is
/// exercised against a lookup the product itself can have (ADR-0009).
struct Untranslated;

impl Lookup for Untranslated {
    fn translate(&self, msgid: &str) -> String {
        msgid.to_string()
    }

    fn translate_plural(&self, singular: &str, plural: &str, n: u32) -> String {
        if n == 1 {
            singular.to_string()
        } else {
            plural.to_string()
        }
    }
}

/// The one Catalogue every test here composes through.
fn the_catalogue() -> Catalogue {
    Catalogue::new(Untranslated)
}

// ---- Announcement 1: the entry count (spec §10.1 item 1) ----

#[test]
fn the_entry_count_fills_the_number_it_counted() {
    assert_eq!(
        the_catalogue().announcement(Announcement::EntryCount {
            scope: Scope::User,
            count: 5,
        }),
        "User PATH: 5 entries"
    );
}

#[test]
fn one_entry_takes_the_singular_form() {
    assert_eq!(
        the_catalogue().announcement(Announcement::EntryCount {
            scope: Scope::User,
            count: 1,
        }),
        "User PATH: 1 entry"
    );
}

#[test]
fn no_entries_takes_its_own_msgid_rather_than_a_plural_form() {
    // Ukrainian's `nplurals=3` has no zero form, and "no entries" is better
    // speech than "0" — so zero is a msgid of its own, and never a count
    // filled into one (spec §10.1 item 1).
    assert_eq!(
        the_catalogue().announcement(Announcement::EntryCount {
            scope: Scope::User,
            count: 0,
        }),
        "User PATH: no entries"
    );
    assert_eq!(
        the_catalogue().announcement(Announcement::EntryCount {
            scope: Scope::System,
            count: 0,
        }),
        "System PATH: no entries"
    );
}

#[test]
fn each_scope_counts_under_its_own_name() {
    assert_eq!(
        the_catalogue().announcement(Announcement::EntryCount {
            scope: Scope::System,
            count: 42,
        }),
        "System PATH: 42 entries"
    );
}

// ---- Announcements 4 and 5: undo and redo (spec §10.1 items 4 and 5) ----

/// One restored Checkpoint, as `Session::undo` would report it. `focus` is not
/// composed into anything — the row is where NVDA reads the path for free.
fn outcome(operation: Operation, crossed_apply: bool) -> UndoOutcome {
    UndoOutcome {
        focus: None,
        operation,
        crossed_apply,
    }
}

#[test]
fn an_undo_names_the_operation_it_took_back() {
    // `{operation}` is itself Catalogue text — the operation's own msgid,
    // translated before it is filled in (spec §10.1 item 4).
    assert_eq!(
        the_catalogue().announcement(Announcement::UndoRedo {
            direction: UndoDirection::Undo,
            outcome: outcome(Operation::Delete, false),
        }),
        "Undone: Delete entry"
    );
}

#[test]
fn a_redo_names_the_same_operation_in_its_own_sentence() {
    assert_eq!(
        the_catalogue().announcement(Announcement::UndoRedo {
            direction: UndoDirection::Redo,
            outcome: outcome(Operation::Move, false),
        }),
        "Redone: Move entry"
    );
}

#[test]
fn the_unsaved_changes_suffix_is_appended_only_across_the_apply_barrier() {
    // Announcement 5 is item 4's text plus the suffix, which is why it is not
    // a variant of its own: `crossed_apply` is the whole of the difference.
    assert_eq!(
        the_catalogue().announcement(Announcement::UndoRedo {
            direction: UndoDirection::Undo,
            outcome: outcome(Operation::Add, true),
        }),
        "Undone: Add entry, unsaved changes"
    );
    assert_eq!(
        the_catalogue().announcement(Announcement::UndoRedo {
            direction: UndoDirection::Redo,
            outcome: outcome(Operation::Add, true),
        }),
        "Redone: Add entry, unsaved changes"
    );
    assert!(!the_catalogue()
        .announcement(Announcement::UndoRedo {
            direction: UndoDirection::Undo,
            outcome: outcome(Operation::Add, false),
        })
        .contains("unsaved"));
}

#[test]
fn every_operation_name_reaches_the_sentence_that_undoes_it() {
    // A seventh Operation with no msgid of its own would leave `{operation}`
    // standing in the spoken text; walking the enum is what makes that visible
    // here rather than in NVDA.
    for operation in [
        Operation::Add,
        Operation::Edit,
        Operation::Delete,
        Operation::Move,
        Operation::Cancel,
        Operation::ChangeValueType,
        Operation::Restore,
    ] {
        let spoken = the_catalogue().announcement(Announcement::UndoRedo {
            direction: UndoDirection::Undo,
            outcome: outcome(operation, false),
        });
        assert_eq!(
            spoken,
            format!("Undone: {}", operation.catalogue_msgid()),
            "{operation:?}"
        );
        assert!(!spoken.contains("{operation}"), "{operation:?}");
    }
}

// ---- The rest of the closed seven (spec §10.1 items 2, 3, 6, 7) ----

#[test]
fn a_cancel_speaks_one_string_and_composes_nothing() {
    assert_eq!(
        the_catalogue().announcement(Announcement::ChangesDiscarded),
        "Changes discarded"
    );
}

#[test]
fn a_read_only_run_names_its_reason_in_the_slot_kept_for_it() {
    // Both halves are Catalogue text: the reason is translated before it is
    // filled in, and it arrives as the msgid `ReadOnlyReason::catalogue_msgid`
    // returns (spec §10.1 item 7).
    assert_eq!(
        the_catalogue().announcement(Announcement::ReadOnly {
            reason: msgids::READONLY_REASON_NOT_WRITABLE,
        }),
        "Read-only: the data directory is not writable"
    );
}

#[test]
fn a_successful_apply_names_the_scope_it_wrote() {
    // §10.1 item 2 is two strings, and choosing between them is the
    // Catalogue's rule rather than the caller's: the Announcement carries a
    // Scope, which is core's own type, so nothing outside can pick the wrong
    // sentence for the Scope it just wrote.
    assert_eq!(
        the_catalogue().announcement(Announcement::Applied { scope: Scope::User }),
        "User PATH applied"
    );
    assert_eq!(
        the_catalogue().announcement(Announcement::Applied {
            scope: Scope::System
        }),
        "System PATH applied"
    );
}

#[test]
fn a_failed_apply_fills_the_cause_into_the_one_sentence_the_taxonomy_speaks() {
    // §9's rows share a sentence and differ only in the phrase filled into it,
    // so `{cause}` is Catalogue text translated before it is filled in — the
    // shape Announcement 7's `{reason}` already has. The cause arrives as the
    // msgid `pathmaster-platform`'s typed failure returns, because core cannot
    // name a platform type (ADR-0009).
    assert_eq!(
        the_catalogue().announcement(Announcement::ApplyFailed {
            cause: msgids::APPLY_FAILED_ACCESS_DENIED,
        }),
        "Apply failed — access denied."
    );
    assert_eq!(
        the_catalogue().announcement(Announcement::ApplyFailed {
            cause: msgids::APPLY_FAILED_BACKUP,
        }),
        "Apply failed — could not write a backup, no changes were made."
    );
    assert_eq!(
        the_catalogue().announcement(Announcement::ApplyFailed {
            cause: msgids::APPLY_FAILED_REGISTRY,
        }),
        "Apply failed — the registry could not be written."
    );
}

#[test]
fn the_announcement_catalogue_is_the_specs_seven_and_nothing_else() {
    // ADR-0003 closed the catalogue at seven and nothing enforced it. This is
    // the enforcement: six variants for seven items, because item 5 is item
    // 4's text with a suffix and `crossed_apply` already models it.
    let catalogue = every_announcement();
    assert_eq!(catalogue.len(), 6);
    assert_eq!(
        catalogue.iter().map(|(item, _)| *item).collect::<Vec<u8>>(),
        [1, 2, 3, 4, 6, 7]
    );
    // Item 5 is the one with no variant of its own — and it is reachable, or
    // the count above would be hiding a message rather than sharing one.
    assert!(the_catalogue()
        .announcement(Announcement::UndoRedo {
            direction: UndoDirection::Undo,
            outcome: outcome(Operation::Delete, true),
        })
        .ends_with(", unsaved changes"));
    for (item, announcement) in catalogue {
        let spoken = the_catalogue().announcement(announcement);
        assert!(!spoken.is_empty(), "Announcement {item} says nothing");
        assert!(
            pathmaster_core::msgids::placeholders(&spoken).is_empty(),
            "Announcement {item} speaks an unfilled placeholder: {spoken:?}"
        );
    }
}

/// One value per variant, each labelled with the §10.1 item it stands for.
///
/// **The `match` is what closes the set**: an eighth Announcement cannot be
/// added to the enum without failing to compile here, which is the whole point
/// of the type. The list is what says which spec item each variant is.
fn every_announcement() -> Vec<(u8, Announcement)> {
    let catalogue = vec![
        (
            1,
            Announcement::EntryCount {
                scope: Scope::User,
                count: 3,
            },
        ),
        (2, Announcement::Applied { scope: Scope::User }),
        (
            3,
            Announcement::ApplyFailed {
                cause: msgids::APPLY_FAILED_REGISTRY,
            },
        ),
        (
            4,
            Announcement::UndoRedo {
                direction: UndoDirection::Undo,
                outcome: outcome(Operation::Delete, false),
            },
        ),
        (6, Announcement::ChangesDiscarded),
        (
            7,
            Announcement::ReadOnly {
                reason: msgids::READONLY_REASON_CANNOT_CREATE,
            },
        ),
    ];
    for (_, announcement) in &catalogue {
        match announcement {
            Announcement::EntryCount { .. }
            | Announcement::Applied { .. }
            | Announcement::ApplyFailed { .. }
            | Announcement::UndoRedo { .. }
            | Announcement::ChangesDiscarded
            | Announcement::ReadOnly { .. } => {}
        }
    }
    catalogue
}

// ---- The Apply-time over-length dialog (spec §7, FR-diag-overlength) ----

#[test]
fn each_over_length_gate_names_its_own_threshold_and_the_length_after_this_apply() {
    // Two sentences and not one with the threshold filled in: each threshold
    // is a measured constant of the OS, and the one fact each warning exists
    // to carry must not be droppable by a translation.
    let catalogue = the_catalogue();
    assert_eq!(
        catalogue.cmd_limit_dialog(9_000),
        "cmd.exe will ignore a PATH longer than 8,191 characters (9000 after this Apply)"
    );
    assert_eq!(
        catalogue.hard_cap_dialog(40_000),
        "PATH cannot exceed 32,767 characters (40000 after this Apply)"
    );
}

#[test]
fn both_over_length_titles_speak_the_length_they_were_given() {
    // The number is the merged length this Apply would leave behind, and a
    // title that dropped it would be a warning about nothing in particular.
    let catalogue = the_catalogue();
    for length in [thresholds::CMD_LIMIT + 1, thresholds::HARD_CAP, 123_456] {
        for title in [
            catalogue.cmd_limit_dialog(length),
            catalogue.hard_cap_dialog(length),
        ] {
            assert!(title.contains(&length.to_string()), "{length}: {title:?}");
            assert!(msgids::placeholders(&title).is_empty(), "{title:?}");
        }
    }
}

// ---- The Status column (spec §7, FR-diag-status) ----

#[test]
fn the_status_column_joins_the_flagged_words_most_severe_first() {
    // The rulebook hands its findings over already in severity order, and the
    // column keeps that order rather than sorting again — one source for
    // "which comes first", which is `Issue::SEVERITY`.
    assert_eq!(
        the_catalogue().status_column(&Issue::SEVERITY),
        "Missing, Relative, Quoted, Duplicate, Empty"
    );
    assert_eq!(
        the_catalogue().status_column(&[Issue::Quoted, Issue::Duplicate]),
        "Quoted, Duplicate"
    );
}

#[test]
fn a_healthy_entry_gets_an_empty_column_and_never_a_word_for_it() {
    // An empty column is the only healthy state: never "OK", never a severity
    // prefix, never an icon (spec §7).
    assert_eq!(the_catalogue().status_column(&[]), "");
}

// ---- StatusBar field 0: the general status (spec §12) ----

/// One Scope's numbers as the window reports them: `issues` is `None` until a
/// pass has looked.
fn counts(scope: Scope, entries: usize, issues: Option<usize>) -> ScopeCounts {
    ScopeCounts {
        scope,
        entries,
        issues,
    }
}

#[test]
fn field_zero_reads_user_first_then_system() {
    // One sentence, read on demand (`NVDA+End`), in the order the tabs are in
    // — not the runtime order a pass evaluates the Scopes in.
    assert_eq!(
        the_catalogue().general_status(
            [
                counts(Scope::User, 2, Some(1)),
                counts(Scope::System, 9, Some(0)),
            ],
            None
        ),
        "User PATH: 2 entries (1 issue) | System PATH: 9 entries (0 issues)"
    );
}

#[test]
fn field_zero_reads_user_first_whichever_order_it_is_handed() {
    // A pass evaluates System first, because that is the order Windows merges
    // the two Scopes in — and this field is not a pass, it is the tab order
    // read aloud as one sentence. So the ordering is the Catalogue's rule, not
    // the caller's, and a caller reaching for a pass's own order cannot
    // reverse the sentence.
    assert_eq!(
        the_catalogue().general_status(
            [
                counts(Scope::System, 9, Some(0)),
                counts(Scope::User, 2, Some(1)),
            ],
            None
        ),
        "User PATH: 2 entries (1 issue) | System PATH: 9 entries (0 issues)"
    );
}

#[test]
fn the_issue_half_is_absent_until_a_pass_has_looked() {
    // "(0 issues)" would be a claim about a Scope nothing has read. Zero after
    // a pass is shown like any other count — the Status column is where "never
    // OK" applies, and it is a different surface.
    assert_eq!(
        the_catalogue().general_status(
            [counts(Scope::User, 1, None), counts(Scope::System, 0, None),],
            None
        ),
        "User PATH: 1 entry | System PATH: no entries"
    );
}

#[test]
fn read_only_data_puts_the_mode_and_its_reason_where_the_counts_would_be() {
    let field = the_catalogue().general_status(
        [
            counts(Scope::User, 2, Some(1)),
            counts(Scope::System, 9, Some(0)),
        ],
        Some(msgids::READONLY_REASON_NOT_WRITABLE),
    );
    assert_eq!(field, "Read-only: the data directory is not writable");
    assert!(!field.contains("entries"));
}

// ---- StatusBar field 1: the merged length (spec §12, FR-diag-overlength) ----

#[test]
fn field_one_shows_the_length_the_pass_measured() {
    assert_eq!(
        the_catalogue().merged_length(Some(120)),
        "Merged PATH: 120 chars"
    );
    assert_eq!(
        the_catalogue().merged_length(Some(1)),
        "Merged PATH: 1 char"
    );
}

#[test]
fn field_one_is_blank_only_before_the_first_pass_has_landed() {
    assert_eq!(the_catalogue().merged_length(None), "");
}

#[test]
fn the_threshold_warning_is_appended_only_past_the_cmd_limit() {
    // 8,191 is the last length `cmd.exe` still honours, so the warning starts
    // one character later. The hard cap is past this too and has nothing
    // further to say here — it speaks at Apply.
    assert_eq!(
        the_catalogue().merged_length(Some(thresholds::CMD_LIMIT)),
        "Merged PATH: 8191 chars"
    );
    assert_eq!(
        the_catalogue().merged_length(Some(thresholds::CMD_LIMIT + 1)),
        "Merged PATH: 8192 chars — exceeds 8,191 (cmd.exe limit)"
    );
    assert!(the_catalogue()
        .merged_length(Some(thresholds::HARD_CAP))
        .ends_with(" — exceeds 8,191 (cmd.exe limit)"));
}

// ---- Validation's rejection text (spec §6, §10) ----

#[test]
fn a_rejected_entry_names_the_character_that_rejected_it() {
    // The dialog's title *is* the error — NVDA never speaks a `MessageDialog`
    // body — so the character has to reach the title or the message names no
    // fault at all.
    assert_eq!(
        the_catalogue().rejection(Rejection::ForbiddenCharacter(';')),
        "The entry contains a forbidden character: ;"
    );
}

#[test]
fn the_empty_rejection_has_nothing_to_fill_and_fills_nothing() {
    assert_eq!(
        the_catalogue().rejection(Rejection::Empty),
        "The entry cannot be empty"
    );
}

// ---- The Backups list's three columns (spec §8, FR-backup-ui) ----

/// One Snapshot file, as the Backups tab builds one: a name, and what reading
/// it turned out to be.
fn snapshot_file(scope: Scope, captured: Option<Captured>) -> SnapshotFile {
    let name = SnapshotName::next(
        Timestamp {
            year: 2026,
            month: 8,
            day: 19,
            hour: 14,
            minute: 32,
            second: 7,
            offset_minutes: 180,
        },
        scope,
        &[],
    );
    let decoded = match captured {
        Some(captured) => Decoded::Valid(Snapshot::under(&name, captured)),
        None => Decoded::Corrupted,
    };
    backups::newest_first([(name, decoded)])
        .pop()
        .expect("one file")
}

fn holding(entries: &[&str]) -> Option<Captured> {
    Some(Captured::Present {
        value_type: ValueType::RegExpandSz,
        entries: entries.iter().map(|entry| (*entry).to_string()).collect(),
    })
}

#[test]
fn a_row_reads_as_when_it_was_taken_which_scope_and_how_many_entries() {
    assert_eq!(
        the_catalogue().snapshot_columns(&snapshot_file(Scope::User, holding(&["one", "two"]))),
        [
            "2026-08-19 14:32:07".to_string(),
            "User PATH".to_string(),
            "2".to_string(),
        ],
    );
}

#[test]
fn a_scope_is_named_in_the_backups_list_exactly_as_its_own_tab_names_it() {
    // One name per Scope: a second English for it would be a second
    // translation to keep in step (ADR-0004).
    assert_eq!(
        the_catalogue().snapshot_columns(&snapshot_file(Scope::System, holding(&[])))[1],
        msgids::TAB_SYSTEM,
    );
}

#[test]
fn a_corrupted_snapshot_says_so_where_its_entry_count_would_stand() {
    // Passive list text in the column that answers "what would restoring this
    // load" — never an Announcement (`CONTEXT.md`, **Corrupted**).
    let row = the_catalogue().snapshot_columns(&snapshot_file(Scope::User, None));

    assert_eq!(row[0], "2026-08-19 14:32:07");
    assert_eq!(row[1], "User PATH");
    assert_eq!(row[2], "[Corrupted]");
}

#[test]
fn a_snapshot_of_an_absent_scope_counts_the_entries_restoring_it_would_load() {
    // None — which is what a Working Copy restored from an Absent Scope holds,
    // there being no Absent state to restore it into (ADR-0006).
    assert_eq!(
        the_catalogue().snapshot_columns(&snapshot_file(Scope::System, Some(Captured::Absent)))[2],
        "0",
    );
}
