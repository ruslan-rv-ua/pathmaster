//! The system's UI language — one of the two facts the Interface Language is
//! resolved from (spec §11).
//!
//! Asked of Windows rather than of wx. wxdragon's `Language` enum mirrors
//! wxWidgets 3.2's `wxLanguage`, which stops at 234, while the vendored
//! wxWidgets 3.3.3 renumbered that enum to roughly nine hundred entries:
//! `wxLocale::GetSystemLanguage()` returns 3.3's ordinal, which 3.2's table
//! either fails to map — every Ukrainian machine reads as `Unknown` — or maps to
//! the wrong language outright. Measured on a `uk-UA` machine (ticket impl-06).
//! `set_language_str("uk")` is unaffected: it is a string, not an ordinal.

use pathmaster_core::language::SystemLanguage;
use windows_sys::Win32::Globalization::GetUserDefaultUILanguage;

/// `LANG_UKRAINIAN`, the primary language identifier Windows gives Ukrainian.
const LANG_UKRAINIAN: u16 = 0x22;

/// The language Windows shows its own interface in, reduced to the one
/// distinction the Interface Language cares about.
///
/// This is the display language, not the formatting locale, so a Ukrainian who
/// formats dates the American way still gets a Ukrainian interface. wx reads
/// the same setting one step wider — `GetUserPreferredUILanguages` returns the
/// whole MUI preference list, of which this is the first entry; with a two-way
/// branch and no language negotiation, the two can only differ for a user whose
/// preferred list leads with a language Windows is not displaying.
pub fn system_language() -> SystemLanguage {
    // SAFETY: a Win32 call taking no arguments and returning a LANGID by value.
    from_langid(unsafe { GetUserDefaultUILanguage() })
}

/// Reads a Windows LANGID: only its primary half names the language, so
/// `uk-UA` (0x0422) and a neutral `uk` (0x0022) are the same answer.
pub fn from_langid(langid: u16) -> SystemLanguage {
    if langid & 0x3ff == LANG_UKRAINIAN {
        SystemLanguage::Ukrainian
    } else {
        SystemLanguage::Other
    }
}
