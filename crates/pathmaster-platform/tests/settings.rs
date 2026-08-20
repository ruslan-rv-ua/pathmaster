//! `settings.json` on a real disk (spec §13, ticket impl-07): the three states
//! the file can be found in, and the set-aside that only Writable Data performs.
//!
//! No mocks — the failure this ticket exists for is a hand-edited file, and the
//! set-aside is a rename that must survive a `.bad` copy already being there.

#![cfg(windows)]

use std::fs::{self, OpenOptions};
use std::os::windows::fs::OpenOptionsExt as _;
use std::path::{Path, PathBuf};

use pathmaster_core::language::LanguageChoice;
use pathmaster_core::settings::{SettingsFile, Window};
use pathmaster_platform::datadir::{DataDirState, ReadOnlyReason};
use pathmaster_platform::settings::{self, Source, BAD_FILE_NAME, FILE_NAME};

/// A Data Directory that exists and may be written — the ordinary run.
fn writable(dir: &Path) -> DataDirState {
    DataDirState::Writable(dir.to_path_buf())
}

/// Read-only Data over a directory that exists: settings are still readable
/// there, which is exactly why the reason carries the path.
fn read_only(dir: &Path) -> DataDirState {
    DataDirState::ReadOnly(ReadOnlyReason::NotWritable(dir.to_path_buf()))
}

fn put(dir: &Path, contents: &str) -> PathBuf {
    let file = dir.join(FILE_NAME);
    fs::write(&file, contents).unwrap();
    file
}

fn names(dir: &Path) -> Vec<String> {
    let mut found: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    found.sort();
    found
}

// ------------------------------------------------------------- absent: no file

#[test]
fn an_absent_file_is_a_first_run_and_leaves_the_directory_empty() {
    let dir = tempfile::tempdir().unwrap();

    let loaded = settings::read(&writable(dir.path()));

    // Defaults, and nothing to report: no dialog, no log line, no file. The
    // file appears on the first natural write, not at startup.
    assert_eq!(loaded.source, Source::Absent);
    assert_eq!(loaded.file, SettingsFile::defaults());
    assert!(names(dir.path()).is_empty());
}

#[test]
fn a_run_that_does_not_know_its_own_location_has_no_file_to_read() {
    let loaded = settings::read(&DataDirState::ReadOnly(ReadOnlyReason::OwnLocationUnknown));

    assert_eq!(loaded.source, Source::Absent);
    assert_eq!(loaded.file, SettingsFile::defaults());
}

// ------------------------------------------------------- readable: the file wins

#[test]
fn a_readable_file_is_what_the_run_uses() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), r#"{"language": "uk", "maxBackups": 3}"#);

    let loaded = settings::read(&writable(dir.path()));

    assert_eq!(loaded.source, Source::Read(Vec::new()));
    assert_eq!(loaded.file.language(), LanguageChoice::Ukrainian);
    assert_eq!(loaded.file.max_backups(), 3);
}

#[test]
fn a_bad_field_comes_back_for_the_log_and_leaves_the_file_alone() {
    let dir = tempfile::tempdir().unwrap();
    let original = r#"{"language": "fr"}"#;
    put(dir.path(), original);

    let loaded = settings::read(&writable(dir.path()));

    let Source::Read(rejected) = &loaded.source else {
        panic!("{:?} was expected to be a read file", loaded.source);
    };
    assert_eq!(rejected.len(), 1);
    assert_eq!(rejected[0].field, "language");
    assert_eq!(loaded.file.language(), LanguageChoice::Auto);
    // A bad field is not an unreadable file: nothing is set aside, and the raw
    // value stays until the user changes that setting.
    assert_eq!(names(dir.path()), [FILE_NAME]);
    assert_eq!(
        fs::read_to_string(dir.path().join(FILE_NAME)).unwrap(),
        original
    );
}

#[test]
fn settings_are_read_in_read_only_data_too() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), r#"{"language": "uk"}"#);

    let loaded = settings::read(&read_only(dir.path()));

    assert_eq!(loaded.file.language(), LanguageChoice::Ukrainian);
}

// ------------------------------------------------- unreadable: set aside as .bad

#[test]
fn an_unparsable_file_is_set_aside_whole_and_the_run_uses_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let original = "{ this was hand-edited badly";
    put(dir.path(), original);

    let loaded = settings::read(&writable(dir.path()));

    assert_eq!(loaded.source, Source::Unreadable { set_aside: true });
    assert_eq!(loaded.file, SettingsFile::defaults());
    // Set aside, never overwritten: the user's text is still on disk, and the
    // name they edited is free for a fresh file to take.
    assert_eq!(names(dir.path()), [BAD_FILE_NAME]);
    assert_eq!(
        fs::read_to_string(dir.path().join(BAD_FILE_NAME)).unwrap(),
        original
    );
}

#[test]
fn a_root_that_is_not_an_object_is_set_aside_like_any_other_unreadable_file() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), r#"["language", "uk"]"#);

    let loaded = settings::read(&writable(dir.path()));

    assert_eq!(loaded.source, Source::Unreadable { set_aside: true });
    assert_eq!(names(dir.path()), [BAD_FILE_NAME]);
}

#[test]
fn a_file_that_is_not_even_text_is_unreadable_rather_than_a_crash() {
    let dir = tempfile::tempdir().unwrap();
    // What PowerShell's `>` leaves behind: UTF-16LE, which is not UTF-8.
    fs::write(
        dir.path().join(FILE_NAME),
        [0xff, 0xfe, 0x7b, 0x00, 0x7d, 0x00],
    )
    .unwrap();

    let loaded = settings::read(&writable(dir.path()));

    assert_eq!(loaded.source, Source::Unreadable { set_aside: true });
    assert_eq!(loaded.file, SettingsFile::defaults());
}

#[test]
fn the_set_aside_copy_is_single_and_the_next_incident_overwrites_it() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(BAD_FILE_NAME), "the previous incident").unwrap();
    put(dir.path(), "this one is bad too");

    settings::read(&writable(dir.path()));

    assert_eq!(names(dir.path()), [BAD_FILE_NAME]);
    assert_eq!(
        fs::read_to_string(dir.path().join(BAD_FILE_NAME)).unwrap(),
        "this one is bad too"
    );
}

#[test]
fn a_file_this_run_cannot_get_at_is_never_the_file_it_sets_aside() {
    // Two instances are a designed state (spec §3): there is no single-instance
    // lock, so a perfectly good settings.json can be momentarily unreadable.
    // The run falls back and the user is told — but moving a good file onto the
    // single `.bad` copy would destroy exactly what the set-aside protects.
    let dir = tempfile::tempdir().unwrap();
    let original = r#"{"language": "uk"}"#;
    put(dir.path(), original);
    let exclusive = OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(dir.path().join(FILE_NAME))
        .unwrap();

    let loaded = settings::read(&writable(dir.path()));
    // Held across the read and no longer: the assertions below read the file
    // themselves, and the temp directory cannot delete an open handle either.
    drop(exclusive);

    assert_eq!(loaded.source, Source::Unreadable { set_aside: false });
    assert_eq!(loaded.file, SettingsFile::defaults());
    assert_eq!(names(dir.path()), [FILE_NAME]);
    assert_eq!(
        fs::read_to_string(dir.path().join(FILE_NAME)).unwrap(),
        original
    );
}

#[test]
fn read_only_data_reads_the_bad_file_but_never_moves_it() {
    let dir = tempfile::tempdir().unwrap();
    let original = "{ still not JSON";
    put(dir.path(), original);

    let loaded = settings::read(&read_only(dir.path()));

    // The dialog still shows — that is the caller's business — but nothing is
    // written, which includes not renaming.
    assert_eq!(loaded.source, Source::Unreadable { set_aside: false });
    assert_eq!(loaded.file, SettingsFile::defaults());
    assert_eq!(names(dir.path()), [FILE_NAME]);
    assert_eq!(
        fs::read_to_string(dir.path().join(FILE_NAME)).unwrap(),
        original
    );
}

// ------------------------------------------------------------------- the write

#[test]
fn writing_creates_the_file_and_leaves_no_temp_behind() {
    let dir = tempfile::tempdir().unwrap();
    let mut file = SettingsFile::defaults();
    file.set_max_backups(7);

    settings::write(dir.path(), &file).unwrap();

    assert_eq!(names(dir.path()), [FILE_NAME]);
    assert_eq!(
        fs::read_to_string(dir.path().join(FILE_NAME)).unwrap(),
        file.to_json()
    );
}

#[test]
fn the_file_one_run_writes_is_the_file_the_next_run_reads() {
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), r#"{"language": "fr", "futureSetting": 7}"#);

    // A run that read the file, changed one setting, and shut down cleanly.
    let mut loaded = settings::read(&writable(dir.path())).file;
    loaded.set_window(Window {
        x: 10,
        y: 20,
        width: 900,
        height: 650,
        maximised: false,
    });
    settings::write(dir.path(), &loaded).unwrap();

    let next = settings::read(&writable(dir.path()));

    assert_eq!(next.file.window(), loaded.window());
    // The unknown field and the raw value of the setting nobody changed both
    // survived the round trip; the language is still reported as rejected.
    assert!(matches!(&next.source, Source::Read(rejected) if rejected.len() == 1));
    let text = fs::read_to_string(dir.path().join(FILE_NAME)).unwrap();
    assert!(
        text.contains(r#""fr""#) && text.contains("futureSetting"),
        "{text}"
    );
}
