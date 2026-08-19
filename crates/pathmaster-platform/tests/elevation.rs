//! Elevation detection (spec §9): `GetTokenInformation(TokenElevation)`,
//! checked against an independent oracle so the test is honest both on an
//! elevated CI runner and an unelevated developer shell.

#![cfg(windows)]

use pathmaster_platform::elevation::is_elevated;

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
