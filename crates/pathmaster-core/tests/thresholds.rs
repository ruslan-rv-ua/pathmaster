//! The merged-length thresholds at the crate boundary (spec §7
//! FR-diag-overlength, ticket impl-09).
//!
//! Over-length is scope-level, never per-entry: one length over both Working
//! Copies, and two numbers that mean different things — 8,191 is a warning a
//! user may walk past, 32,767 is a wall.

use pathmaster_core::thresholds::{self, Overlength, CMD_LIMIT, HARD_CAP};

// ---- The length ----

#[test]
fn the_merged_length_counts_both_scopes_and_the_separator() {
    // `len(System + ";" + User)`: 3 + 1 + 4.
    assert_eq!(thresholds::merged_length("abc", "defg"), 8);
}

#[test]
fn the_separator_counts_even_when_a_scope_is_empty() {
    // The spec's formula is literal, and so is this: an empty Scope still
    // contributes the `;` Windows joins with.
    assert_eq!(thresholds::merged_length("", ""), 1);
    assert_eq!(thresholds::merged_length("abc", ""), 4);
}

#[test]
fn the_unit_is_utf16_code_units_not_bytes_or_characters() {
    // Cyrillic is two bytes each and one code unit each; a non-BMP character
    // is one `char` and two code units — the number Windows counts.
    assert_eq!(thresholds::merged_length("Програми", ""), 9);
    assert_eq!(thresholds::merged_length("\u{1D11E}", ""), 3);
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
