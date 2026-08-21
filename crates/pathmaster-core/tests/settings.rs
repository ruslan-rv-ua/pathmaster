//! `settings.json` at the crate boundary (spec §13, ticket impl-07).
//!
//! Two layers, and the tests are grouped by them. The **parse layer** is
//! all-or-nothing: unparsable JSON or a root that is not an object means
//! nothing in the file is used. The **field layer** is per-field: an invalid
//! value of a known field falls back to its own default *in memory* while the
//! file keeps the raw value, so a choice this version cannot read is not
//! silently downgraded into one it can.

use pathmaster_core::language::LanguageChoice;
use pathmaster_core::logfmt::{line, Timestamp};
use pathmaster_core::settings::{
    parse_max_backups, Choices, Parsed, Rejected, SettingsFile, Window, DEFAULT_MAX_BACKUPS,
};

/// The successful half of [`SettingsFile::parse`], for the tests whose subject
/// is not the parse layer.
fn read(text: &str) -> (SettingsFile, Vec<Rejected>) {
    match SettingsFile::parse(text) {
        Parsed::Readable { file, rejected } => (file, rejected),
        Parsed::Unreadable => panic!("{text} was expected to parse"),
    }
}

/// The fields a rejection names, as the log will show them.
fn rejected_fields(rejected: &[Rejected]) -> Vec<&str> {
    rejected.iter().map(|r| r.field).collect()
}

// ------------------------------------------------------------- the parse layer

#[test]
fn an_object_root_with_no_fields_at_all_is_a_complete_set_of_defaults() {
    let (file, rejected) = read("{}");

    assert_eq!(file.language(), LanguageChoice::Auto);
    assert_eq!(file.max_backups(), 50);
    assert_eq!(file.window(), None);
    // A field the file does not mention is not a rejected field: absent,
    // unreadable and bad-field are three distinct states.
    assert!(rejected.is_empty());
}

#[test]
fn every_field_the_file_names_validly_is_the_value_this_run_uses() {
    let (file, rejected) = read(
        r#"{
             "language": "uk",
             "maxBackups": 7,
             "window": { "x": 10, "y": 20, "width": 900, "height": 650,
                         "maximised": true }
           }"#,
    );

    assert_eq!(file.language(), LanguageChoice::Ukrainian);
    assert_eq!(file.max_backups(), 7);
    assert_eq!(
        file.window(),
        Some(Window {
            x: 10,
            y: 20,
            width: 900,
            height: 650,
            maximised: true,
        })
    );
    assert!(rejected.is_empty());
}

#[test]
fn unparsable_json_is_unreadable_whole() {
    for text in [
        "",
        "   ",
        "not json at all",
        "{",
        r#"{"language": }"#,
        "{,}",
    ] {
        assert_eq!(
            SettingsFile::parse(text),
            Parsed::Unreadable,
            "{text:?} must not parse"
        );
    }
}

#[test]
fn a_root_that_is_not_an_object_is_unreadable_even_though_it_is_valid_json() {
    for text in [
        "[]",
        r#"["language", "uk"]"#,
        "42",
        "null",
        "true",
        r#""uk""#,
    ] {
        assert_eq!(
            SettingsFile::parse(text),
            Parsed::Unreadable,
            "{text:?} is not an object root"
        );
    }
}

#[test]
fn trailing_whitespace_and_a_newline_do_not_make_a_file_unreadable() {
    // What every editor leaves behind, and what this file's own writer emits.
    let (file, _) = read("{\n  \"maxBackups\": 3\n}\n");
    assert_eq!(file.max_backups(), 3);
}

#[test]
fn a_byte_order_mark_is_not_what_makes_a_file_unreadable() {
    // Several Windows editors save UTF-8 with one. The file the user wrote is
    // perfectly good JSON; setting it aside over an invisible character would
    // be the least explicable failure this application could produce.
    let (file, rejected) = read("\u{feff}{\"maxBackups\": 3}");
    assert_eq!(file.max_backups(), 3);
    assert!(rejected.is_empty());
}

// ------------------------------------------------------------- the field layer

#[test]
fn a_language_outside_the_stored_domain_falls_back_to_auto_and_is_reported() {
    // The v0.2 case: a value this version cannot read is not a broken file.
    let (file, rejected) = read(r#"{"language": "fr"}"#);

    assert_eq!(file.language(), LanguageChoice::Auto);
    assert_eq!(rejected_fields(&rejected), ["language"]);
    assert_eq!(rejected[0].raw, "fr");
    assert_eq!(rejected[0].default, "auto");
}

#[test]
fn a_language_that_is_not_a_string_is_reported_with_its_raw_value() {
    for (text, raw) in [
        (r#"{"language": 3}"#, "3"),
        (r#"{"language": null}"#, "null"),
        (r#"{"language": ["uk"]}"#, r#"["uk"]"#),
    ] {
        let (file, rejected) = read(text);
        assert_eq!(file.language(), LanguageChoice::Auto);
        assert_eq!(rejected_fields(&rejected), ["language"], "{text}");
        assert_eq!(rejected[0].raw, raw);
    }
}

#[test]
fn a_max_backups_below_one_falls_back_to_the_default_and_is_never_clamped() {
    // Clamping was rejected: -3 → 0 would mean "no backups", which would
    // silently delete the pre-Apply safety net this application exists for.
    for (text, raw) in [
        (r#"{"maxBackups": 0}"#, "0"),
        (r#"{"maxBackups": -3}"#, "-3"),
    ] {
        let (file, rejected) = read(text);
        assert_eq!(file.max_backups(), DEFAULT_MAX_BACKUPS, "{text}");
        assert_eq!(rejected_fields(&rejected), ["maxBackups"], "{text}");
        assert_eq!(rejected[0].raw, raw);
        assert_eq!(rejected[0].default, "50");
    }
}

#[test]
fn a_max_backups_that_is_not_a_whole_number_is_invalid() {
    for text in [
        r#"{"maxBackups": 2.5}"#,
        r#"{"maxBackups": "50"}"#,
        r#"{"maxBackups": true}"#,
        r#"{"maxBackups": null}"#,
        // Past u32: a budget nobody can reach is still not a budget we can hold.
        r#"{"maxBackups": 4294967296}"#,
    ] {
        let (file, rejected) = read(text);
        assert_eq!(file.max_backups(), DEFAULT_MAX_BACKUPS, "{text}");
        assert_eq!(rejected_fields(&rejected), ["maxBackups"], "{text}");
    }
}

#[test]
fn one_is_a_valid_backup_budget_and_the_boundary_is_not_off_by_one() {
    let (file, rejected) = read(r#"{"maxBackups": 1}"#);
    assert_eq!(file.max_backups(), 1);
    assert!(rejected.is_empty());
}

#[test]
fn a_window_record_is_whole_or_it_is_not_a_position() {
    // Geometry is one fact — where the window was. Half of it is not a place
    // to put a window, so the record falls back as a unit, under one name.
    for text in [
        r#"{"window": { "x": 10, "y": 20, "width": 900, "height": 650 }}"#,
        r#"{"window": { "x": 10, "y": 20, "width": 900, "maximised": false }}"#,
        r#"{"window": { "x": "10", "y": 20, "width": 900, "height": 650, "maximised": false }}"#,
        r#"{"window": { "x": 10, "y": 20, "width": 900, "height": 650, "maximised": 1 }}"#,
        r#"{"window": 900}"#,
        r#"{"window": null}"#,
    ] {
        let (file, rejected) = read(text);
        assert_eq!(file.window(), None, "{text}");
        assert_eq!(rejected_fields(&rejected), ["window"], "{text}");
        assert_eq!(rejected[0].default, "none", "{text}");
    }
}

#[test]
fn a_window_with_no_area_is_invalid_while_a_negative_position_is_not() {
    for text in [
        r#"{"window": { "x": 0, "y": 0, "width": 0, "height": 650, "maximised": false }}"#,
        r#"{"window": { "x": 0, "y": 0, "width": 900, "height": -1, "maximised": false }}"#,
    ] {
        let (file, rejected) = read(text);
        assert_eq!(file.window(), None, "{text}");
        assert_eq!(rejected_fields(&rejected), ["window"], "{text}");
    }

    // A second monitor left of or above the primary gives real negative
    // coordinates; clamping them to the work area is the restore's business.
    let (file, rejected) = read(
        r#"{"window": { "x": -1280, "y": -40, "width": 900, "height": 650,
                        "maximised": false }}"#,
    );
    assert_eq!(
        file.window(),
        Some(Window {
            x: -1280,
            y: -40,
            width: 900,
            height: 650,
            maximised: false,
        })
    );
    assert!(rejected.is_empty());
}

#[test]
fn every_bad_field_is_reported_once_and_the_good_ones_are_still_read() {
    let (file, rejected) = read(
        r#"{"language": "fr", "maxBackups": 0,
            "window": { "x": 10, "y": 20, "width": 900, "height": 650,
                        "maximised": false }}"#,
    );

    assert_eq!(file.language(), LanguageChoice::Auto);
    assert_eq!(file.max_backups(), DEFAULT_MAX_BACKUPS);
    assert!(file.window().is_some());
    assert_eq!(rejected_fields(&rejected), ["language", "maxBackups"]);
}

#[test]
fn a_rejected_value_reaches_the_log_as_the_warn_line_that_witnesses_it() {
    // The log is the only witness of a rejected field — no dialog, no
    // Announcement — so the wording is fixed here, not left to a caller.
    let (_, rejected) = read(r#"{"maxBackups": 0}"#);
    let stamp = Timestamp {
        year: 2026,
        month: 8,
        day: 19,
        hour: 15,
        minute: 36,
        second: 31,
        offset_minutes: 180,
    };
    assert_eq!(
        line(&stamp, &rejected[0].record()),
        "2026-08-19T15:36:31+03:00 WARN  settings: \
         field \"maxBackups\" invalid (raw: \"0\"), using default 50\n",
    );
}

// --------------------------------------------------- what a rewrite preserves

#[test]
fn an_unknown_field_rides_through_a_rewrite_untouched() {
    // v0.1 never deletes from the file what it does not understand.
    let (mut file, _) = read(r#"{"language": "uk", "futureSetting": {"depth": [1, 2]}}"#);
    file.set_max_backups(9);

    let rewritten = file.to_json();
    let (reread, rejected) = read(&rewritten);

    assert!(rejected.is_empty(), "{rewritten}");
    assert_eq!(reread.language(), LanguageChoice::Ukrainian);
    assert_eq!(reread.max_backups(), 9);
    assert!(
        rewritten.contains(r#""futureSetting""#) && rewritten.contains(r#""depth""#),
        "{rewritten}"
    );
}

#[test]
fn an_unknown_member_of_a_known_record_rides_through_too() {
    // A field a later version adds inside `window` is as unknown, and as
    // preserved, as a top-level one — writing geometry amends the record it
    // finds rather than replacing it.
    let (mut file, _) = read(
        r#"{"window": { "x": 1, "y": 2, "width": 900, "height": 650,
                        "maximised": false, "monitor": "primary" }}"#,
    );
    file.set_window(Window {
        x: 30,
        y: 40,
        width: 800,
        height: 600,
        maximised: true,
    });

    let rewritten = file.to_json();
    assert!(rewritten.contains(r#""monitor": "primary""#), "{rewritten}");
    assert_eq!(read(&rewritten).0.window(), file.window());
}

#[test]
fn a_hand_written_files_key_order_survives_a_rewrite() {
    let (mut file, _) = read(r#"{"zeta": 1, "maxBackups": 7, "alpha": 2}"#);
    file.set_max_backups(9);

    let rewritten = file.to_json();
    let order = |key: &str| rewritten.find(key).unwrap_or_else(|| panic!("{key}"));

    assert!(order("zeta") < order("maxBackups"));
    assert!(order("maxBackups") < order("alpha"));
}

#[test]
fn a_rejected_field_keeps_its_raw_value_until_the_user_changes_that_setting() {
    // The choice-not-outcome rule: a v0.2 value survives a v0.1 run.
    let (mut file, _) = read(r#"{"language": "fr", "maxBackups": 0}"#);
    file.set_max_backups(12);

    let rewritten = file.to_json();
    assert!(rewritten.contains(r#""fr""#), "{rewritten}");
    assert!(rewritten.contains("12"), "{rewritten}");

    // ...and changing that very setting is what replaces it.
    file.set_language(LanguageChoice::English);
    let rewritten = file.to_json();
    assert!(!rewritten.contains(r#""fr""#), "{rewritten}");
    assert!(rewritten.contains(r#""en""#), "{rewritten}");
}

#[test]
fn a_first_run_writes_only_what_it_was_actually_asked_to_store() {
    // The file records the choice, not its outcome — so defaults nobody chose
    // do not materialise as choices somebody made.
    let mut file = SettingsFile::defaults();
    assert_eq!(file.to_json(), "{}\n");

    file.set_window(Window {
        x: 10,
        y: 20,
        width: 900,
        height: 650,
        maximised: false,
    });
    let rewritten = file.to_json();
    assert!(rewritten.contains(r#""window""#), "{rewritten}");
    assert!(!rewritten.contains("language"), "{rewritten}");
    assert!(!rewritten.contains("maxBackups"), "{rewritten}");
}

#[test]
fn what_is_written_is_the_hand_editable_shape_the_file_promises() {
    let mut file = SettingsFile::defaults();
    file.set_language(LanguageChoice::Ukrainian);
    file.set_max_backups(50);
    file.set_window(Window {
        x: 10,
        y: 20,
        width: 900,
        height: 650,
        maximised: false,
    });

    assert_eq!(
        file.to_json(),
        "{\n  \
           \"language\": \"uk\",\n  \
           \"maxBackups\": 50,\n  \
           \"window\": {\n    \
             \"x\": 10,\n    \
             \"y\": 20,\n    \
             \"width\": 900,\n    \
             \"height\": 650,\n    \
             \"maximised\": false\n  \
           }\n\
         }\n",
    );
}

#[test]
fn every_value_this_run_holds_round_trips_through_the_file() {
    let mut file = SettingsFile::defaults();
    file.set_language(LanguageChoice::English);
    file.set_max_backups(1);
    file.set_window(Window {
        x: -3,
        y: 0,
        width: 1,
        height: 1,
        maximised: true,
    });

    let (reread, rejected) = read(&file.to_json());

    assert!(rejected.is_empty());
    assert_eq!(reread.language(), file.language());
    assert_eq!(reread.max_backups(), file.max_backups());
    assert_eq!(reread.window(), file.window());
}

#[test]
fn defaults_are_the_values_the_spec_names() {
    let file = SettingsFile::defaults();
    assert_eq!(file.language(), LanguageChoice::Auto);
    assert_eq!(file.max_backups(), 50);
    assert_eq!(DEFAULT_MAX_BACKUPS, 50);
    assert_eq!(file.window(), None);
}

// --------------------------------------- what the Settings dialog changes

#[test]
fn the_dialog_opens_on_the_settings_this_run_is_using() {
    let (file, _) = read(r#"{"language": "fr", "maxBackups": 7}"#);

    // The values in memory, never the raw ones the file kept: `fr` is not
    // something this version can do, so it is not something to show as done.
    assert_eq!(
        file.choices(),
        Choices {
            language: LanguageChoice::Auto,
            max_backups: 7,
        }
    );
}

#[test]
fn recording_the_answer_changes_only_the_settings_the_user_changed() {
    let (mut file, _) = read(r#"{"language": "fr", "maxBackups": 0}"#);

    assert!(file.record_choices(Choices {
        language: LanguageChoice::Auto,
        max_backups: 12,
    }));

    // The budget moves; the language the user left alone keeps the raw value
    // the file was holding for it — the choice-not-outcome rule, which is
    // about the setting the user changed and not about the dialog they opened.
    let rewritten = file.to_json();
    assert!(rewritten.contains(r#""fr""#), "{rewritten}");
    assert!(rewritten.contains("12"), "{rewritten}");
    assert_eq!(file.max_backups(), 12);
    assert_eq!(file.language(), LanguageChoice::Auto);
}

#[test]
fn changing_that_very_setting_is_what_replaces_a_value_the_file_kept() {
    let (mut file, _) = read(r#"{"language": "fr", "maxBackups": 7}"#);

    assert!(file.record_choices(Choices {
        language: LanguageChoice::Ukrainian,
        max_backups: 7,
    }));

    let rewritten = file.to_json();
    assert!(!rewritten.contains(r#""fr""#), "{rewritten}");
    assert!(rewritten.contains(r#""uk""#), "{rewritten}");
}

#[test]
fn an_answer_that_changes_nothing_leaves_the_document_exactly_as_it_was() {
    // Dirty is a comparison, not a record that something happened: an OK over
    // controls the user only looked at has nothing to write. So a hand-edited
    // file is not reformatted, a raw value it kept is not replaced, and a
    // first run does not gain a `{}` nobody asked for.
    let (mut file, _) = read(r#"{"language": "fr", "maxBackups": 0}"#);
    let before = file.to_json();
    let opened_on = file.choices();

    assert!(!file.record_choices(opened_on));
    assert_eq!(file.to_json(), before);

    let mut first_run = SettingsFile::defaults();
    let opened_on = first_run.choices();
    assert!(!first_run.record_choices(opened_on));
    assert_eq!(first_run.to_json(), "{}\n");
}

#[test]
fn a_first_run_records_the_setting_that_was_changed_and_not_the_one_that_was_not() {
    let mut file = SettingsFile::defaults();

    assert!(file.record_choices(Choices {
        language: LanguageChoice::Auto,
        max_backups: 10,
    }));

    let rewritten = file.to_json();
    assert!(rewritten.contains("maxBackups"), "{rewritten}");
    assert!(!rewritten.contains("language"), "{rewritten}");
}

#[test]
fn the_backup_budget_field_takes_a_whole_number_in_the_files_own_domain() {
    assert_eq!(parse_max_backups("1"), Some(1));
    assert_eq!(parse_max_backups("50"), Some(50));
    assert_eq!(parse_max_backups(&u32::MAX.to_string()), Some(u32::MAX));
    // Surrounding whitespace is not part of a number. This field holds a
    // count, not text that has to survive a round trip the way an Entry does.
    assert_eq!(parse_max_backups("  7  "), Some(7));
}

#[test]
fn the_backup_budget_field_rejects_everything_the_file_would_have_rejected() {
    for typed in [
        "",
        "   ",
        "0",
        "-3",
        "2.5",
        "abc",
        "7 backups",
        "1e3",
        "+5",
        // Past u32, and past u64 — a budget nobody can reach is still not one
        // this application can hold.
        "4294967296",
        "99999999999999999999999999",
    ] {
        assert_eq!(parse_max_backups(typed), None, "{typed:?}");
    }
}
