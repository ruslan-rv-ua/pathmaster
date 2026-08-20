//! The system UI language (spec §11), checked against an independent oracle so
//! the test is honest on a Ukrainian developer machine and an English CI runner
//! alike.

#![cfg(windows)]

use pathmaster_core::language::SystemLanguage;
use pathmaster_platform::locale::{from_langid, system_language};

#[test]
fn a_langid_is_read_by_its_primary_language_only() {
    // uk-UA and a neutral uk are the same language; the sublanguage half is not
    // ours to care about.
    assert_eq!(from_langid(0x0422), SystemLanguage::Ukrainian);
    assert_eq!(from_langid(0x0022), SystemLanguage::Ukrainian);
}

#[test]
fn every_other_language_is_other_including_its_neighbours() {
    // Russian is a different language, not a Ukrainian sublanguage.
    assert_eq!(from_langid(0x0419), SystemLanguage::Other);
    assert_eq!(from_langid(0x0409), SystemLanguage::Other);
    assert_eq!(from_langid(0x0000), SystemLanguage::Other);
}

/// Oracle: .NET's UI culture for a fresh process, which Windows derives from
/// the same user display-language setting `GetUserDefaultUILanguage` returns.
#[test]
fn the_system_language_matches_the_ui_culture_windows_reports() {
    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", "(Get-UICulture).Name"])
        .output()
        .unwrap();
    let name = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_lowercase();
    let oracle = if name == "uk" || name.starts_with("uk-") {
        SystemLanguage::Ukrainian
    } else {
        SystemLanguage::Other
    };

    assert_eq!(system_language(), oracle, "UI culture reported as {name:?}");
}
