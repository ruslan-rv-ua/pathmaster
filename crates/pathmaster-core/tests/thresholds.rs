//! The merged-length thresholds at the crate boundary (spec §7
//! FR-diag-overlength, ticket impl-09).
//!
//! Over-length is scope-level, never per-entry: one length over both Working
//! Copies, and two numbers that mean different things — 8,191 is a warning a
//! user may walk past, 32,767 is a wall.

use pathmaster_core::normalize::Environment;
use pathmaster_core::thresholds::{self, Overlength, CMD_LIMIT, HARD_CAP};

/// One defined variable, so the formula's expansion step is visible in the
/// number it produces.
struct Env;

impl Environment for Env {
    fn lookup(&self, name: &str) -> Option<String> {
        (name.eq_ignore_ascii_case("SystemRoot")).then(|| r"C:\Windows".to_string())
    }
}

/// No Scope's Entries at all — the shape a Scope that is Absent or empty
/// decodes to.
const NONE: [&str; 0] = [];

// ---- The length ----

#[test]
fn the_merged_length_counts_both_scopes_and_the_separator() {
    // `len(System + ";" + User)`: 3 + 1 + 4.
    assert_eq!(thresholds::merged_length(&["abc"], &["defg"], &Env), 8);
}

#[test]
fn the_separator_counts_even_when_a_scope_is_empty() {
    // The spec's formula is literal, and so is this: an empty Scope still
    // contributes the `;` Windows joins with.
    assert_eq!(thresholds::merged_length(&NONE, &NONE, &Env), 1);
    assert_eq!(thresholds::merged_length(&["abc"], &NONE, &Env), 4);
}

#[test]
fn the_unit_is_utf16_code_units_not_bytes_or_characters() {
    // Cyrillic is two bytes each and one code unit each; a non-BMP character
    // is one `char` and two code units — the number Windows counts.
    assert_eq!(thresholds::merged_length(&["Програми"], &NONE, &Env), 9);
    assert_eq!(thresholds::merged_length(&["\u{1D11E}"], &NONE, &Env), 3);
}

#[test]
fn each_scopes_entries_are_joined_with_the_semicolon_windows_joins_them_with() {
    // The Entries are what the formula takes, not a value someone joined on
    // its behalf: `C:\a;C:\b` is 9, plus the separator, plus an empty User.
    assert_eq!(
        thresholds::merged_length(&[r"C:\a", r"C:\b"], &NONE, &Env),
        10
    );
}

#[test]
fn the_formula_expands_each_scope_once_before_it_counts() {
    // `len(expand(System WC) + ";" + expand(User WC))` — the whole of spec
    // §7's formula, which the diagnostic pass and an Apply Run both ask for
    // and must get one answer to. There is no way to ask for half of it.
    //
    // System expands and joins to `C:\Windows;C:\a`, which is 15; User is
    // `C:\b`, which is 4; and the `;` Windows joins the two with is the
    // twentieth character.
    assert_eq!(
        thresholds::merged_length(&[r"%SystemRoot%", r"C:\a"], &[r"C:\b"], &Env),
        20
    );
}

#[test]
fn an_undefined_reference_is_counted_as_the_literal_text_it_stays() {
    // Expansion leaves an unknown name alone, so the length counts what
    // Windows would actually materialise — a `PATH` with `%NOPE%` still in
    // it: six characters, plus the separator.
    assert_eq!(thresholds::merged_length(&[r"%NOPE%"], &NONE, &Env), 7);
}

// ---- The two numbers ----

#[test]
fn the_thresholds_are_the_measured_ones() {
    assert_eq!(CMD_LIMIT, 8_191);
    assert_eq!(HARD_CAP, 32_767);
}

#[test]
fn a_length_within_every_threshold_is_within() {
    assert_eq!(thresholds::classify(0), Overlength::Within);
    assert_eq!(thresholds::classify(CMD_LIMIT), Overlength::Within);
}

#[test]
fn past_8191_cmd_ignores_the_variable_and_apply_may_still_proceed() {
    // KB 830473: cmd.exe drops an inherited variable longer than this — a real
    // consequence, and a legal thing to choose.
    assert_eq!(thresholds::classify(CMD_LIMIT + 1), Overlength::CmdLimit);
    assert_eq!(thresholds::classify(HARD_CAP - 1), Overlength::CmdLimit);
    assert!(thresholds::classify(CMD_LIMIT + 1).may_proceed());
}

#[test]
fn at_32767_the_cap_is_hard_and_nothing_may_proceed() {
    // The value cannot be materialised into any process environment, so there
    // is no proceed button to offer.
    assert_eq!(thresholds::classify(HARD_CAP), Overlength::HardCap);
    assert_eq!(thresholds::classify(HARD_CAP + 1), Overlength::HardCap);
    assert!(!thresholds::classify(HARD_CAP).may_proceed());
}

#[test]
fn the_healthy_lengths_may_proceed() {
    assert!(thresholds::classify(0).may_proceed());
    assert!(thresholds::classify(CMD_LIMIT).may_proceed());
}

#[test]
fn there_is_no_2047_threshold() {
    // Vista-era folklore, hotfixed a decade ago (research/13).
    assert_eq!(thresholds::classify(2_047), Overlength::Within);
    assert_eq!(thresholds::classify(2_048), Overlength::Within);
}
