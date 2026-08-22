//! Elevation detection (spec §9): `GetTokenInformation(TokenElevation)`,
//! checked against an independent oracle so the test is honest both on an
//! elevated CI runner and an unelevated developer shell — and the one
//! command-line argument the relaunch carries across the process boundary
//! (ADR-0005: only the active tab crosses; ticket 12 D5).

#![cfg(windows)]

use pathmaster_platform::elevation::{is_elevated, StartTab};

/// Oracle: the process token's mandatory integrity level, printed as raw SIDs
/// by `whoami /groups` in every locale. An elevated token runs at High
/// (`S-1-16-12288`) or System (`S-1-16-16384`); an unelevated one at Medium.
#[test]
fn elevation_matches_the_token_integrity_level() {
    let output = std::process::Command::new("whoami")
        .arg("/groups")
        .output()
        .unwrap();
    let groups = String::from_utf8_lossy(&output.stdout);
    let oracle = groups.contains("S-1-16-12288") || groups.contains("S-1-16-16384");

    assert_eq!(is_elevated(), oracle);
}

/// The writer and the reader of `--tab` are the same two functions, so what
/// the unelevated instance says is what the elevated one hears — for every
/// tab, the Backups tab included (it is a tab the user can leave, even though
/// it is not a Scope).
#[test]
fn the_tab_argument_round_trips_for_every_tab() {
    for tab in [StartTab::User, StartTab::System, StartTab::Backups] {
        let spawned = ["--tab".to_string(), tab.argument().to_string()];
        assert_eq!(StartTab::from_args(spawned), Some(tab));
    }
}

/// Anything that is not our own spawn reads as no request at all — a plain
/// launch, a foreign flag, a value nothing writes, or a `--tab` with nothing
/// after it. The degraded answer is None, never a guessed tab.
#[test]
fn a_foreign_or_missing_tab_argument_reads_as_none() {
    let cases: &[&[&str]] = &[
        &[],
        &["--tab"],
        &["--tab", "banana"],
        &["--elevated-write"],
        &["user"],
    ];
    for case in cases {
        let args = case.iter().map(|a| a.to_string());
        assert_eq!(StartTab::from_args(args), None, "args: {case:?}");
    }
}
