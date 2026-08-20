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

/// Oracle, asked of Windows rather than of a process that has a culture of its
/// own: the display language Windows itself is set to — the user's override
/// when they have set one, and the language Windows was installed in when they
/// have not. That pair is what `GetUserDefaultUILanguage` answers from.
///
/// It is deliberately **not** `(Get-UICulture).Name`, which the first version
/// of this test asked and which is wrong for this purpose: that is the *host
/// process's* thread UI culture, and .NET Framework — which Windows PowerShell
/// 5.1 runs on — falls back to `en-US` for a console app whose code page
/// cannot render the OS language. Measured on a Ukrainian machine
/// (2026-08-20): `GetUserDefaultUILanguage` said `0x0422`, the override said
/// `uk`, the installed culture said `uk-UA`, the registry's
/// `PreferredUILanguages` said `uk-UA` — and `Get-UICulture` alone said
/// `en-US`. An oracle that disagrees with every other reading of the setting
/// is measuring the wrong thing.
#[test]
fn the_system_language_matches_the_display_language_windows_is_set_to() {
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            // The `try` is not decoration: an image without the International
            // module raises a *terminating* CommandNotFoundException, and an
            // oracle that dies is worse than one that falls back — the
            // installed culture is the right answer whenever no override
            // exists, which is the same branch.
            "$o = $null; try { $o = (Get-WinUILanguageOverride).Name } catch {}; \
             if ($o) { $o } else { [Globalization.CultureInfo]::InstalledUICulture.Name }",
        ])
        .output()
        .unwrap();
    let name = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_lowercase();
    assert!(!name.is_empty(), "the oracle answered nothing");
    let oracle = if name == "uk" || name.starts_with("uk-") {
        SystemLanguage::Ukrainian
    } else {
        SystemLanguage::Other
    };

    assert_eq!(
        system_language(),
        oracle,
        "Windows displays itself in {name:?}"
    );
}
