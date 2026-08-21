//! The log line format at the crate boundary (spec §14, ticket impl-05).
//!
//! One record per line: `<RFC 3339 local+offset> <LEVEL> <area>: <message>`.
//! Expected strings come from the spec's own examples, not recomputed.

use pathmaster_core::logfmt::{line, DataState, Record, Timestamp};
use pathmaster_core::session::{Scope, ValueType};

fn spec_timestamp() -> Timestamp {
    Timestamp {
        year: 2026,
        month: 8,
        day: 19,
        hour: 15,
        minute: 36,
        second: 31,
        offset_minutes: 180,
    }
}

#[test]
fn startup_line_matches_the_spec_example_byte_for_byte() {
    let record = Record::startup("0.1.0", false, DataState::Writable, "uk");
    assert_eq!(
        line(&spec_timestamp(), &record),
        "2026-08-19T15:36:31+03:00 INFO  startup: \
         PathMaster 0.1.0, elevated: no, data: writable, language: uk\n",
    );
}

#[test]
fn apply_audit_line_matches_the_spec_example() {
    let record = Record::apply_written(Scope::User, 14, 512, ValueType::RegExpandSz);
    assert_eq!(
        line(&spec_timestamp(), &record),
        "2026-08-19T15:36:31+03:00 INFO  apply: \
         User scope written, 14 entries, 512 chars, REG_EXPAND_SZ\n",
    );
}

#[test]
fn clean_shutdown_line_matches_the_spec() {
    assert_eq!(
        line(&spec_timestamp(), &Record::shutdown_clean()),
        "2026-08-19T15:36:31+03:00 INFO  shutdown: clean\n",
    );
}

#[test]
fn lost_records_line_is_a_warn_from_the_log_area() {
    assert_eq!(
        line(&spec_timestamp(), &Record::records_lost(3)),
        "2026-08-19T15:36:31+03:00 WARN  log: 3 records were lost\n",
    );
}

#[test]
fn panic_line_carries_message_and_file_line_only() {
    let record = Record::panic("boom", r"crates\pathmaster\src\main.rs", 42);
    assert_eq!(
        line(&spec_timestamp(), &record),
        "2026-08-19T15:36:31+03:00 ERROR panic: \
         boom (crates\\pathmaster\\src\\main.rs:42)\n",
    );
}

#[test]
fn all_three_levels_pad_to_five_chars_so_columns_align() {
    let ts = spec_timestamp();
    // The area starts at the same byte column on every level: the level
    // field is exactly five characters wide, one space either side.
    assert_eq!(
        line(&ts, &Record::shutdown_clean()).find("shutdown:"),
        Some(32)
    );
    assert_eq!(line(&ts, &Record::records_lost(1)).find("log:"), Some(32));
    assert_eq!(
        line(&ts, &Record::panic("x", "y.rs", 1)).find("panic:"),
        Some(32)
    );
}

#[test]
fn an_unreadable_settings_file_says_in_the_log_where_it_went() {
    // The dialog tells the user their edit did not take; this line tells a
    // developer reading the log which file to go and look at. File names are
    // not paths — PII prohibition #2 is about locations, and there is none here.
    assert_eq!(
        line(&spec_timestamp(), &Record::settings_unreadable(true)),
        "2026-08-19T15:36:31+03:00 WARN  settings: \
         settings.json could not be read, set aside as settings.json.bad, using defaults\n",
    );
    assert_eq!(
        line(&spec_timestamp(), &Record::settings_unreadable(false)),
        "2026-08-19T15:36:31+03:00 WARN  settings: \
         settings.json could not be read, left in place, using defaults\n",
    );
}

#[test]
fn rejected_settings_value_is_logged_verbatim_when_short() {
    let record = Record::settings_field_invalid("maxBackups", "0", "50");
    assert_eq!(
        line(&spec_timestamp(), &record),
        "2026-08-19T15:36:31+03:00 WARN  settings: \
         field \"maxBackups\" invalid (raw: \"0\"), using default 50\n",
    );
}

#[test]
fn rejected_settings_value_over_100_chars_is_truncated_with_a_marker() {
    let raw = "x".repeat(300);
    let record = Record::settings_field_invalid("language", &raw, "auto");
    let text = line(&spec_timestamp(), &record);
    assert!(
        text.contains(&format!("(raw: \"{}…\" [truncated])", "x".repeat(100))),
        "{text:?}",
    );
    assert!(!text.contains(&"x".repeat(101)), "{text:?}");
}

#[test]
fn truncation_cuts_at_a_character_boundary_not_mid_codepoint() {
    let raw = "й".repeat(300);
    let record = Record::settings_field_invalid("language", &raw, "auto");
    let text = line(&spec_timestamp(), &record);
    assert!(
        text.contains(&format!("\"{}…\"", "й".repeat(100))),
        "{text:?}"
    );
}

#[test]
fn a_raw_value_with_newlines_still_yields_one_record_per_line() {
    let record = Record::settings_field_invalid("language", "a\r\nb\nc", "auto");
    let text = line(&spec_timestamp(), &record);
    assert_eq!(text.matches('\n').count(), 1, "{text:?}");
    assert!(text.ends_with('\n'), "{text:?}");
}

#[test]
fn a_panic_message_with_newlines_still_yields_one_record_per_line() {
    let record = Record::panic("assertion failed:\nleft != right", "y.rs", 7);
    let text = line(&spec_timestamp(), &record);
    assert_eq!(text.matches('\n').count(), 1, "{text:?}");
}

#[test]
fn offsets_format_negative_zero_and_half_hour() {
    let mut ts = spec_timestamp();
    ts.offset_minutes = -300;
    assert!(ts.rfc3339().ends_with("-05:00"));
    ts.offset_minutes = 0;
    assert!(ts.rfc3339().ends_with("+00:00"));
    ts.offset_minutes = 330;
    assert!(ts.rfc3339().ends_with("+05:30"));
}

#[test]
fn a_scope_that_could_not_be_read_logs_the_raw_cause_and_what_the_run_did() {
    // Startup reads a Scope it cannot decode into a Session; the run survives
    // (empty, non-writable) and this line is the developer's only witness.
    use pathmaster_core::logfmt::FailureCause;
    assert_eq!(
        line(
            &spec_timestamp(),
            &Record::scope_read_failed(Scope::System, FailureCause::Io { os_error: Some(5) }),
        ),
        "2026-08-19T15:36:31+03:00 WARN  registry: \
         System scope could not be read (os error 5), treated as empty and non-writable\n",
    );
    assert_eq!(
        line(
            &spec_timestamp(),
            &Record::scope_read_failed(Scope::User, FailureCause::Io { os_error: None }),
        ),
        "2026-08-19T15:36:31+03:00 WARN  registry: \
         User scope could not be read (io error), treated as empty and non-writable\n",
    );
    assert_eq!(
        line(
            &spec_timestamp(),
            &Record::scope_read_failed(Scope::User, FailureCause::UnsupportedType { vtype: 3 }),
        ),
        "2026-08-19T15:36:31+03:00 WARN  registry: \
         User scope could not be read (unsupported registry type 3), treated as empty and non-writable\n",
    );
}

#[test]
fn read_only_startup_lines_name_the_reason_never_a_location() {
    for (state, expected) in [
        (
            DataState::ReadOnlyOwnLocationUnknown,
            "data: read-only (own location unknown)",
        ),
        (
            DataState::ReadOnlyCannotCreate,
            "data: read-only (cannot create data directory)",
        ),
        (
            DataState::ReadOnlyNotWritable,
            "data: read-only (data directory not writable)",
        ),
    ] {
        let text = line(
            &spec_timestamp(),
            &Record::startup("0.1.0", true, state, "en"),
        );
        assert!(text.contains(expected), "{text:?}");
        assert!(text.contains("elevated: yes"), "{text:?}");
    }
}

#[test]
fn an_apply_that_failed_names_the_step_and_carries_the_raw_code() {
    // §9's invariant: every failure lands one log record with the raw error
    // code. The three failing rows are three steps of one fixed order, and the
    // line says which — "nothing was written" reads the same in all three, and
    // a developer needs to know where it stopped.
    use pathmaster_core::logfmt::{ApplyStep, FailureCause};
    for (step, cause, expected) in [
        (
            ApplyStep::ReRead,
            FailureCause::UnsupportedType { vtype: 3 },
            "System scope not applied, re-read failed (unsupported registry type 3)",
        ),
        (
            ApplyStep::Snapshot,
            FailureCause::Io { os_error: Some(5) },
            "System scope not applied, backup failed (os error 5)",
        ),
        (
            ApplyStep::Write,
            FailureCause::Io { os_error: Some(5) },
            "System scope not applied, registry write failed (os error 5)",
        ),
    ] {
        assert_eq!(
            line(
                &spec_timestamp(),
                &Record::apply_failed(Scope::System, step, cause),
            ),
            format!("2026-08-19T15:36:31+03:00 ERROR apply: {expected}\n"),
        );
    }
}

#[test]
fn a_broadcast_that_timed_out_is_a_warn_and_never_an_error() {
    // The write succeeded; only the notification did not land, and
    // already-open shells never see it regardless (spec §4). ERROR is reserved
    // for a user-requested operation that failed, and this one did not.
    assert_eq!(
        line(&spec_timestamp(), &Record::broadcast_timed_out()),
        "2026-08-19T15:36:31+03:00 WARN  broadcast: \
         WM_SETTINGCHANGE timed out, already-open processes keep the old value\n",
    );
}
