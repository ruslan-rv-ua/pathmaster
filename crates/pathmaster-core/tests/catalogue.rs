//! The completeness gate: the msgid registry measured against every shipped
//! `.po` (spec §11, ADR-0004, ticket impl-06).
//!
//! It runs where wxWidgets is not linked, so it reads the `.po` sources rather
//! than the `.mo` files wx loads. `build.rs` drops untranslated and fuzzy
//! messages on the way, exactly as `msgfmt` does, and two of the checks below
//! close that gap instead of trusting it: every registered msgid must be usable,
//! and a catalogue may hold nothing the registry does not name — between them no
//! `.po` entry can exist that the build's filter and this file would read
//! differently. The binary's smoke test closes the loop on the wx side.
//!
//! English ships a catalogue too, so both languages take one path through wx
//! rather than English riding the miss-returns-the-msgid fallback; being the
//! source language, its catalogue is gated as the identity of the registry.
//!
//! Reaching into `../pathmaster/i18n` is a test-time path, not a dependency
//! edge: §17 puts the gate in core and the `.po` files with the binary that
//! embeds them, and `cargo test -p pathmaster-core` still links no wx and still
//! builds on any OS. The `.po` enumeration is deliberately repeated from
//! `build.rs` — sharing it would mean either giving the pure core a filesystem
//! or letting a build script depend on a test.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use polib::catalog::Catalog;
use polib::message::MessageView;

use pathmaster_core::filtered::Filter;
use pathmaster_core::msgids::{
    duplicate_mnemonic, mnemonic, placeholders, CatalogueEntry, REGISTRY,
};

/// The language the msgids are written in (ADR-0004). Its catalogue is the
/// identity — see `the_english_catalogue_repeats_the_msgids_it_is_made_of`.
const SOURCE_LANGUAGE: &str = "en";

/// The catalogues live with the binary that embeds them (spec §17).
fn i18n_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../pathmaster/i18n")
}

/// Every `<code>.po` under `i18n/`, by language code, so a language that ships
/// is a language that is gated.
fn catalogues() -> BTreeMap<String, Catalog> {
    let mut found = BTreeMap::new();
    for entry in std::fs::read_dir(i18n_dir()).expect("i18n directory") {
        let path = entry.expect("directory entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("po") {
            continue;
        }
        let code = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("language code")
            .to_string();
        let catalog = polib::po_file::parse(&path)
            .unwrap_or_else(|e| panic!("{} does not parse: {e}", path.display()));
        found.insert(code, catalog);
    }
    assert!(!found.is_empty(), "no catalogue found to gate");
    found
}

/// What the compiled `.mo` will contain: a message wx can actually answer with.
/// Fuzzy is gettext's "guessed after the source changed" and reads as missing.
fn is_usable(message: &dyn MessageView) -> bool {
    message.is_translated() && !message.is_fuzzy()
}

/// Every translated form of a message — one string, or one per plural form.
fn translations_of(message: &dyn MessageView) -> Vec<&str> {
    match message.msgstr_plural() {
        Ok(forms) => forms.iter().map(String::as_str).collect(),
        Err(_) => vec![message.msgstr().expect("a singular message")],
    }
}

/// Runs `check` over every (language, registry entry, message) the catalogues
/// carry. An entry a catalogue lacks is passed over here: absence is
/// `every_msgid_is_present_and_usable_in_every_language`'s report, and one
/// missing string should fail one test rather than every test.
fn each_message(mut check: impl FnMut(&str, &CatalogueEntry, &dyn MessageView)) {
    for (code, catalog) in catalogues() {
        for entry in REGISTRY {
            if let Some(message) = catalog.find_message(None, entry.msgid, entry.plural) {
                check(&code, entry, message);
            }
        }
    }
}

#[test]
fn every_language_ships_a_catalogue_that_names_itself() {
    let catalogues = catalogues();
    for code in [SOURCE_LANGUAGE, "uk"] {
        assert!(
            catalogues.contains_key(code),
            "{code} is one of the two languages that ship"
        );
    }
    for (code, catalog) in &catalogues {
        assert_eq!(
            &catalog.metadata.language, code,
            "{code}.po declares a different Language: header"
        );
        assert!(
            catalog.metadata.content_type.contains("charset=UTF-8"),
            "{code}.po must declare charset=UTF-8"
        );
    }
}

#[test]
fn the_english_catalogue_repeats_the_msgids_it_is_made_of() {
    // English has no translation to hold: the msgid *is* its text. So this
    // catalogue exists to be identical, and saying so here is what keeps it
    // from becoming a second, quietly diverging source of the English — an
    // edit here that the msgid constant does not share fails the build.
    let catalogues = catalogues();
    let english = catalogues.get(SOURCE_LANGUAGE).expect("en.po");
    for entry in REGISTRY {
        let Some(message) = english.find_message(None, entry.msgid, entry.plural) else {
            continue; // presence is a separate test's report
        };
        let expected: Vec<&str> = match entry.plural {
            Some(plural) => vec![entry.msgid, plural],
            None => vec![entry.msgid],
        };
        assert_eq!(
            translations_of(message),
            expected,
            "en.po says something the msgid {:?} does not",
            entry.msgid
        );
    }
}

#[test]
fn ukrainian_declares_three_plural_forms() {
    let catalogues = catalogues();
    let uk = catalogues.get("uk").expect("uk.po");
    assert_eq!(uk.metadata.plural_rules.nplurals, 3);
}

#[test]
fn every_msgid_is_present_and_usable_in_every_language() {
    for (code, catalog) in catalogues() {
        for entry in REGISTRY {
            let message = catalog
                .find_message(None, entry.msgid, entry.plural)
                .unwrap_or_else(|| panic!("{code}.po is missing {:?}", entry.msgid));
            assert!(
                is_usable(message),
                "{code}.po leaves {:?} untranslated or fuzzy",
                entry.msgid
            );
        }
    }
}

#[test]
fn a_msgid_that_does_not_exist_is_reported_missing() {
    // Self-sensitivity: the presence check above must be able to fail. Absence
    // is asked the same way — never by comparing a translation to its msgid,
    // which would flag every string whose Ukrainian equals its English.
    for (code, catalog) in catalogues() {
        assert!(
            catalog
                .find_message(None, "There is no such string in PathMaster", None)
                .is_none(),
            "{code}.po answers for a msgid that was never registered"
        );
    }
}

#[test]
fn a_plural_entry_carries_one_form_per_declared_plural() {
    let nplurals: BTreeMap<String, usize> = catalogues()
        .iter()
        .map(|(code, catalog)| (code.clone(), catalog.metadata.plural_rules.nplurals))
        .collect();
    each_message(|code, entry, message| {
        if entry.plural.is_none() {
            return;
        }
        let expected = nplurals[code];
        let forms = message.msgstr_plural().expect("a plural message");
        assert_eq!(
            forms.len(),
            expected,
            "{code}.po gives {:?} {} forms, not {expected}",
            entry.msgid,
            forms.len()
        );
    });
}

#[test]
fn a_translation_carries_exactly_the_placeholders_its_msgid_does() {
    each_message(|code, entry, message| {
        let expected: BTreeSet<&str> = placeholders(entry.msgid).into_iter().collect();
        for translation in translations_of(message) {
            let actual: BTreeSet<&str> = placeholders(translation).into_iter().collect();
            assert_eq!(
                actual, expected,
                "{code}.po changes the placeholders of {:?} in {translation:?}",
                entry.msgid
            );
        }
    });
}

#[test]
fn no_translation_carries_an_accelerator() {
    // A tab in a Catalogue entry is a defect: it would register a shortcut the
    // code never appended, and delete the one it did (ADR-0004).
    each_message(|code, _, message| {
        for translation in translations_of(message) {
            assert!(
                !translation.contains('\t'),
                "{code}.po carries an accelerator in {translation:?}"
            );
        }
    });
}

#[test]
fn no_filter_state_name_carries_a_mnemonic_in_any_language() {
    // The seven Filter states name themselves in three places — the View
    // submenu, Announcement 11 and StatusBar field 0 — and only one of the
    // three swallows an `&` (v0.2.0 spec §4). A translator writing «Усі(&A)»
    // in the shape every *other* menu label here takes would put a literal
    // ampersand into the StatusBar and into what NVDA speaks, so the rule that
    // these are not menu labels is gated rather than trusted.
    //
    // It is checked over the **translations**: the English side is a `const`
    // a reviewer reads, while the `.po` is where this can go wrong unseen.
    let names: BTreeSet<&str> = Filter::ALL
        .into_iter()
        .map(Filter::catalogue_msgid)
        .collect();
    let mut checked = 0;
    each_message(|code, entry, message| {
        if !names.contains(entry.msgid) {
            return;
        }
        checked += 1;
        for translation in translations_of(message) {
            assert!(
                mnemonic(translation).is_none(),
                "{code}.po gives the Filter state {:?} a mnemonic: {translation:?}",
                entry.msgid
            );
        }
    });
    // Self-sensitivity: seven names in each of the two shipped languages, or
    // the loop above passed by proving nothing.
    assert_eq!(checked, names.len() * 2);
}

#[test]
fn a_translated_menu_keeps_one_mnemonic_letter_per_item() {
    // A translator can break this without the text looking wrong — and unlike
    // the English side, no reviewer of the Ukrainian would notice.
    //
    // No menu has landed yet: the first is the Edit menu of the entry-editing
    // ticket, and each menu's labels arrive with the menu. Until then this walks
    // an empty set — the logic it walks it with is what `msgids.rs` pins against
    // fixtures, so the first `menu_item` entry to appear is gated on arrival.
    for (code, catalog) in catalogues() {
        let menus: BTreeSet<&str> = REGISTRY.iter().filter_map(|entry| entry.menu).collect();
        for menu in menus {
            let mut labels = Vec::new();
            for entry in REGISTRY.iter().filter(|entry| entry.menu == Some(menu)) {
                let Some(message) = catalog.find_message(None, entry.msgid, None) else {
                    continue;
                };
                let label = message.msgstr().expect("a singular message");
                assert!(
                    mnemonic(label).is_some(),
                    "{code}.po drops the mnemonic from {label:?}"
                );
                labels.push(label);
            }
            assert_eq!(
                duplicate_mnemonic(labels.iter().copied()),
                None,
                "{code}.po reuses a mnemonic letter within the {menu} menu"
            );
        }
    }
}

#[test]
fn a_catalogue_holds_nothing_the_registry_does_not_name() {
    // A renamed msgid leaves its old translation behind; it would sit there
    // translated, gated, and never looked up again.
    let registered: BTreeSet<&str> = REGISTRY.iter().map(|entry| entry.msgid).collect();
    for (code, catalog) in catalogues() {
        for message in catalog.messages() {
            assert!(
                registered.contains(message.msgid()),
                "{code}.po translates {:?}, which no msgid constant names",
                message.msgid()
            );
        }
    }
}
