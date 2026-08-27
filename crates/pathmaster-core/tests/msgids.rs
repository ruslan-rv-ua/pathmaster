//! The Catalogue's registry and the pure checks its gate is built from
//! (spec §11, ADR-0004, ticket impl-06).
//!
//! The gate itself — the registry measured against the shipped `.po` files —
//! lives in `catalogue.rs`; this file fixes the behaviour that gate relies on,
//! including the negative cases real data is not expected to produce.

use pathmaster_core::msgids::{
    duplicate_mnemonic, fill, mnemonic, placeholders, CatalogueEntry, REGISTRY,
};

// ---------------------------------------------------------------- placeholders

#[test]
fn a_placeholder_is_a_named_brace() {
    assert_eq!(placeholders("User PATH: {n} entries"), vec!["n"]);
    assert_eq!(placeholders("Undone: {operation}"), vec!["operation"]);
    assert_eq!(placeholders("{a} then {b}"), vec!["a", "b"]);
    assert!(placeholders("Changes discarded").is_empty());
}

#[test]
fn percent_syntax_is_data_and_never_a_placeholder() {
    // The whole point of named braces: this application's domain is %VAR%.
    assert!(placeholders("%PATH% could not be read").is_empty());
    assert_eq!(placeholders("%SystemRoot% in {n} entries"), vec!["n"]);
}

#[test]
fn a_brace_that_is_not_a_placeholder_is_not_read_as_one() {
    assert!(placeholders("{unclosed").is_empty());
    assert!(placeholders("{}").is_empty());
    assert!(placeholders("{two words}").is_empty());
    assert!(placeholders("}{").is_empty());
}

// ------------------------------------------------------------------ filling in

#[test]
fn filling_substitutes_every_occurrence_by_name() {
    assert_eq!(
        fill("User PATH: {n} entries", &[("n", "14")]),
        "User PATH: 14 entries"
    );
    assert_eq!(fill("{n} of {n}", &[("n", "3")]), "3 of 3");
}

#[test]
fn a_substituted_value_is_never_rescanned() {
    // Entry text is user data and may contain anything at all.
    assert_eq!(fill("{path}", &[("path", "{n}"), ("n", "9")]), "{n}");
    assert_eq!(
        fill("{path}", &[("path", r"C:\dev\%VAR%")]),
        r"C:\dev\%VAR%"
    );
}

#[test]
fn an_unsupplied_placeholder_is_left_alone_rather_than_panicking() {
    // The gate makes this unreachable in shipped text; at runtime a missing
    // value must still leave something readable for a screen reader to speak.
    assert_eq!(fill("Undone: {operation}", &[]), "Undone: {operation}");
    assert_eq!(fill("{a} {b}", &[("b", "2")]), "{a} 2");
}

// ------------------------------------------------------------------- mnemonics

#[test]
fn the_mnemonic_is_the_letter_after_a_single_ampersand() {
    assert_eq!(mnemonic("&File"), Some('F'));
    assert_eq!(mnemonic("Move U&p"), Some('p'));
    // Ukrainian keeps the Latin letter in parentheses (ADR-0004).
    assert_eq!(mnemonic("Файл(&F)"), Some('F'));
    assert_eq!(mnemonic("Path"), None);
}

#[test]
fn an_escaped_ampersand_is_not_a_mnemonic() {
    assert_eq!(mnemonic("R&&D"), None);
    assert_eq!(mnemonic("R&&D &Tools"), Some('T'));
    assert_eq!(mnemonic("Trailing &"), None);
}

#[test]
fn duplicate_mnemonics_are_found_case_insensitively() {
    assert_eq!(duplicate_mnemonic(["&File", "&Edit"]), None);
    assert_eq!(duplicate_mnemonic(["&File", "&Find"]), Some('f'));
    assert_eq!(duplicate_mnemonic(["&file", "&File"]), Some('f'));
    // Labels with no mnemonic never collide with each other.
    assert_eq!(duplicate_mnemonic(["Path", "Status"]), None);
}

// -------------------------------------------------------------- the registry

#[test]
fn every_msgid_appears_once_so_two_meanings_can_never_share_one_english() {
    let mut seen = std::collections::BTreeSet::new();
    for entry in REGISTRY {
        assert!(
            seen.insert(entry.msgid),
            "msgid appears twice in the registry: {:?}",
            entry.msgid
        );
        if let Some(plural) = entry.plural {
            assert!(
                seen.insert(plural),
                "plural msgid appears twice in the registry: {plural:?}"
            );
        }
    }
}

#[test]
fn no_catalogue_entry_carries_an_accelerator() {
    // A `\t` inside a Catalogue entry is a defect: accelerators are appended
    // by the code, so a translator can never delete a shortcut (ADR-0004).
    for entry in REGISTRY {
        assert!(
            !entry.msgid.contains('\t'),
            "msgid carries an accelerator: {:?}",
            entry.msgid
        );
    }
}

#[test]
fn a_plural_entry_states_both_forms_and_they_agree_on_placeholders() {
    for entry in REGISTRY {
        let Some(plural) = entry.plural else { continue };
        assert_ne!(
            entry.msgid, plural,
            "plural entry repeats its singular: {plural:?}"
        );
        assert_eq!(
            placeholders(entry.msgid),
            placeholders(plural),
            "plural forms of {:?} disagree on placeholders",
            entry.msgid
        );
    }
}

#[test]
fn menu_labels_carry_a_mnemonic_that_is_unique_within_their_menu() {
    let menus: std::collections::BTreeSet<&str> =
        REGISTRY.iter().filter_map(|entry| entry.menu).collect();
    for menu in menus {
        let labels: Vec<&str> = REGISTRY
            .iter()
            .filter(|entry| entry.menu == Some(menu))
            .map(|entry| entry.msgid)
            .collect();
        for label in &labels {
            assert!(
                mnemonic(label).is_some(),
                "menu label without a mnemonic: {label:?}"
            );
        }
        assert_eq!(
            duplicate_mnemonic(labels.iter().copied()),
            None,
            "the {menu} menu reuses a mnemonic letter"
        );
    }
}

#[test]
fn the_registry_holds_the_strings_the_shell_shows() {
    for msgid in [
        pathmaster_core::msgids::TAB_USER,
        pathmaster_core::msgids::TAB_SYSTEM,
        pathmaster_core::msgids::TAB_BACKUPS,
        pathmaster_core::msgids::COLUMN_INDEX,
        pathmaster_core::msgids::COLUMN_PATH,
        pathmaster_core::msgids::COLUMN_STATUS,
        pathmaster_core::msgids::READONLY,
        pathmaster_core::msgids::READONLY_REASON_OWN_LOCATION_UNKNOWN,
        pathmaster_core::msgids::READONLY_REASON_CANNOT_CREATE,
        pathmaster_core::msgids::READONLY_REASON_NOT_WRITABLE,
        pathmaster_core::msgids::READONLY_REASON_OVERRIDE_UNUSABLE,
        pathmaster_core::msgids::DIALOG_UNKNOWN_ARGUMENT,
        pathmaster_core::msgids::DIALOG_COMMAND_LINE,
        pathmaster_core::msgids::USAGE,
    ] {
        assert!(
            REGISTRY
                .iter()
                .any(|entry: &CatalogueEntry| entry.msgid == msgid),
            "the shell shows {msgid:?} but the Catalogue does not hold it"
        );
    }
}
