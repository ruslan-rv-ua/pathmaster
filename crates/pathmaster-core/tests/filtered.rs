//! The Filtered View's matching engine at the crate boundary (v0.2.0 spec §2,
//! §3; ticket impl 03).
//!
//! The rule under test is deliberately small: case-insensitive substring with
//! Unicode case folding, slash-folded (`/`→`\`), and nothing else. Quote
//! stripping, trailing-`\` trimming and `%VAR%` expansion change *what text
//! exists* and stay out — a search for `"` must find the `Quoted` Entries.

use pathmaster_core::filtered::{matches, visible_indices};

// ----------------------------------------------------------------- the fold

#[test]
fn a_substring_matches_wherever_it_stands() {
    assert!(matches(r"C:\Program Files\Git\cmd", "git"));
    assert!(matches(r"C:\Program Files\Git\cmd", r"C:\Program"));
    assert!(matches(r"C:\Program Files\Git\cmd", "cmd"));
    assert!(!matches(r"C:\Program Files\Git\cmd", "python"));
}

#[test]
fn case_is_folded_on_both_sides() {
    assert!(matches(r"C:\PROGRAM FILES\GIT", "git"));
    assert!(matches(r"c:\program files\git", "GIT"));
}

#[test]
fn the_fold_is_unicode_not_ascii() {
    // An ASCII fold would be silently case-sensitive for every Cyrillic path
    // (v0.2.0 spec §3).
    assert!(matches(r"C:\Users\Руслан\bin", "руслан"));
    assert!(matches(r"C:\Users\руслан\bin", "РУСЛАН"));
}

#[test]
fn slashes_are_folded_into_backslashes_on_both_sides() {
    assert!(matches(r"C:\Program Files\Git", "files/git"));
    assert!(matches(r"C:/Program Files/Git", r"files\git"));
}

#[test]
fn a_quote_is_searchable_text() {
    // Quote stripping stays out of the fold: a search for `"` finds the
    // Quoted Entries.
    assert!(matches(r#""C:\Program Files\Git""#, "\""));
    assert!(!matches(r"C:\Program Files\Git", "\""));
}

#[test]
fn the_query_is_never_trimmed() {
    // Whitespace is Entry content.
    assert!(matches(r"C:\Program Files\Git", "program f"));
    assert!(!matches(r"C:\ProgramFiles", "program f"));
    assert!(!matches(r"C:\Git", " git"));
}

#[test]
fn an_empty_query_matches_everything() {
    assert!(matches(r"C:\anything", ""));
    assert!(matches("", ""));
}

#[test]
fn an_empty_rendering_matches_only_the_empty_query() {
    assert!(!matches("", "git"));
}

// ----------------------------------------------------------- the visible set

#[test]
fn visible_indices_are_the_matching_positions_in_order() {
    let renderings = [r"C:\Git\cmd", r"C:\Python", r"C:\git\bin"];
    assert_eq!(
        visible_indices(renderings.iter().copied(), "git"),
        vec![0, 2]
    );
}

#[test]
fn an_empty_query_shows_every_entry() {
    let renderings = [r"C:\a", r"C:\b"];
    assert_eq!(visible_indices(renderings.iter().copied(), ""), vec![0, 1]);
}

#[test]
fn no_match_is_an_empty_set_not_an_error() {
    let renderings = [r"C:\a", r"C:\b"];
    assert_eq!(
        visible_indices(renderings.iter().copied(), "zz"),
        Vec::<usize>::new()
    );
}
