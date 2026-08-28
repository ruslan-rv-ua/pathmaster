//! The User Guide's file and its fallback address (v0.2.0 §9).
//!
//! What is asserted here is the two halves of the failure ladder's top rung:
//! the page lands under one name whatever language it is in, and it lands
//! **again** on every open — the rule that makes staleness structurally
//! impossible under scoop, which persists `data\` as a junction across
//! upgrades. The rungs below it are a browser and a network, and neither is
//! something a test may reach for.

#![cfg(windows)]

use std::fs;
use std::path::Path;

use pathmaster_core::language::Language;
use pathmaster_platform::help;

/// One writable directory per test, removed with it.
fn temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("a temp directory")
}

#[test]
fn the_page_lands_under_one_name_in_the_data_directory() {
    let dir = temp_dir();

    let written = help::write_page(dir.path(), b"<!doctype html>").expect("the page is written");

    assert_eq!(written, dir.path().join("help.html"));
    assert_eq!(fs::read(&written).unwrap(), b"<!doctype html>");
}

/// **No language suffix**, deliberately: a per-language name would leave an
/// orphan behind the first time the Interface Language changed, and the orphan
/// would still be a guide — readable, findable and wrong.
#[test]
fn changing_language_rewrites_the_one_file_and_leaves_no_orphan() {
    let dir = temp_dir();

    help::write_page(dir.path(), b"<html lang=\"en\">").expect("the English page");
    help::write_page(dir.path(), b"<html lang=\"uk\">").expect("the Ukrainian page");

    let names: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(names, [std::ffi::OsString::from("help.html")]);
    assert_eq!(
        fs::read(help::page_path(dir.path())).unwrap(),
        b"<html lang=\"uk\">"
    );
}

/// "Write only if missing" is poisoned here (§9), so the write is
/// unconditional — a page already on disk is replaced by this build's, not
/// left alone because something is there.
#[test]
fn an_older_page_is_overwritten_rather_than_left_alone() {
    let dir = temp_dir();
    fs::write(dir.path().join("help.html"), b"the guide from v0.1.0").unwrap();

    help::write_page(dir.path(), b"the guide from v0.2.0").expect("the page is written");

    assert_eq!(
        fs::read(help::page_path(dir.path())).unwrap(),
        b"the guide from v0.2.0"
    );
}

/// The second rung: a directory that cannot be written is a failure the caller
/// answers with the online copy, never an error shown to the user.
#[test]
fn a_directory_that_is_not_there_fails_rather_than_creating_one() {
    // The Data Directory is created once at startup and never here — a guide
    // that created it would be writing where the startup decision said it
    // could not (ADR-0002).
    let dir = temp_dir();
    let missing = dir.path().join("no-such-directory");

    assert!(help::write_page(&missing, b"<!doctype html>").is_err());
    assert!(!Path::new(&missing).exists());
}

#[test]
fn the_online_copy_is_pinned_to_the_build_that_opens_it() {
    assert_eq!(
        help::source_url("0.2.0", Language::Ukrainian),
        "https://github.com/ruslan-rv-ua/pathmaster/blob/v0.2.0/docs/help/uk.md"
    );
    // The tag carries a `v`, the way the releases do; the path under it is the
    // repository's own, which is what the heading-parity gate reads.
    assert_eq!(
        help::source_url("0.1.0", Language::English),
        "https://github.com/ruslan-rv-ua/pathmaster/blob/v0.1.0/docs/help/en.md"
    );
}
