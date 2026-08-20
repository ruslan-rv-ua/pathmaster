//! Interface Language resolution at the crate boundary (spec §11, ticket impl-06).
//!
//! The stored choice is `auto`/`en`/`uk` and the run resolves it once, by a
//! two-way branch: Ukrainian system → `uk`, everything else → `en`.

use pathmaster_core::language::{resolve, Language, LanguageChoice, SystemLanguage};

#[test]
fn the_stored_choice_domain_is_auto_en_uk() {
    assert_eq!(LanguageChoice::parse("auto"), Some(LanguageChoice::Auto));
    assert_eq!(LanguageChoice::parse("en"), Some(LanguageChoice::English));
    assert_eq!(LanguageChoice::parse("uk"), Some(LanguageChoice::Ukrainian));
}

#[test]
fn an_unrecognised_choice_is_not_silently_read_as_auto() {
    // The caller (the settings ticket) owes a WARN line and keeps the raw
    // value, so parsing reports the miss instead of swallowing it.
    assert_eq!(LanguageChoice::parse("uk-UA"), None);
    assert_eq!(LanguageChoice::parse("EN"), None);
    assert_eq!(LanguageChoice::parse(""), None);
}

#[test]
fn a_choice_round_trips_through_its_stored_form() {
    for choice in [
        LanguageChoice::Auto,
        LanguageChoice::English,
        LanguageChoice::Ukrainian,
    ] {
        assert_eq!(LanguageChoice::parse(choice.as_str()), Some(choice));
    }
}

#[test]
fn auto_follows_a_ukrainian_system_and_falls_back_to_english_otherwise() {
    assert_eq!(
        resolve(LanguageChoice::Auto, SystemLanguage::Ukrainian),
        Language::Ukrainian
    );
    assert_eq!(
        resolve(LanguageChoice::Auto, SystemLanguage::Other),
        Language::English
    );
}

#[test]
fn an_explicit_choice_overrides_the_system_in_both_directions() {
    assert_eq!(
        resolve(LanguageChoice::English, SystemLanguage::Ukrainian),
        Language::English
    );
    assert_eq!(
        resolve(LanguageChoice::Ukrainian, SystemLanguage::Other),
        Language::Ukrainian
    );
}

#[test]
fn a_language_carries_its_catalogue_code_and_its_endonym() {
    assert_eq!(Language::English.code(), "en");
    assert_eq!(Language::Ukrainian.code(), "uk");
    // Listed in its own language, so a user who cannot read the current
    // interface language can still find theirs (spec §11).
    assert_eq!(Language::English.endonym(), "English");
    assert_eq!(Language::Ukrainian.endonym(), "Українська");
}
