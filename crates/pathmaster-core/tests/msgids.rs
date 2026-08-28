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

/// The declared menus and the filled ones are one set.
///
/// `menu_labels_carry_a_mnemonic_that_is_unique_within_their_menu` derives its
/// groups from the registry, so what it walks is whatever the entries happen to
/// say. Read from inside that gate, two things are invisible: a `MENU_GROUP_`
/// constant no entry uses — a menu declared and never wired — and a group the
/// entries name that no constant does, which is a menu nobody decided on. This
/// holds the two lists equal, in both directions.
///
/// The size rule is the other half. A group of one is trivially unique, so it
/// is the one size in which a misfiled label can sit and still read as green.
#[test]
fn every_declared_menu_group_is_one_the_registry_fills() {
    use pathmaster_core::msgids::{
        MENU_GROUP_BAR, MENU_GROUP_EDIT, MENU_GROUP_FILE, MENU_GROUP_HELP, MENU_GROUP_TOOLS,
        MENU_GROUP_VIEW,
    };

    let declared: std::collections::BTreeSet<&str> = [
        MENU_GROUP_BAR,
        MENU_GROUP_FILE,
        MENU_GROUP_EDIT,
        MENU_GROUP_VIEW,
        MENU_GROUP_TOOLS,
        MENU_GROUP_HELP,
    ]
    .into_iter()
    .collect();
    let filled: std::collections::BTreeSet<&str> =
        REGISTRY.iter().filter_map(|entry| entry.menu).collect();

    assert_eq!(declared, filled);
    for menu in &declared {
        // A group of one cannot collide, so a group of one is where a typo
        // hides. Every menu this application has holds at least two items.
        let held = REGISTRY.iter().filter(|e| e.menu == Some(*menu)).count();
        assert!(
            held >= 2,
            "the {menu} menu group holds only {held} label(s)"
        );
    }
}

/// v0.2.0 §12 proposed **S, F, I, E, T** for the new View menu and left the set
/// "for the gate to confirm" — this is the confirmation, written down.
///
/// The uniqueness check above would pass just as happily on any other five
/// distinct letters, so it cannot be the record of *which* set was chosen. The
/// menu grew all at once and the letters were reasoned about all at once; this
/// is where a later edit that quietly re-letters one of them has to argue with
/// the decision rather than slip past it.
#[test]
fn the_view_menu_carries_the_letters_the_spec_proposed() {
    use pathmaster_core::msgids::{
        MENU_EXPANDED_VALUES, MENU_FILTER, MENU_GROUP_VIEW, MENU_PATH_TREE, MENU_SEARCH,
        MENU_TOGGLE_ISSUES_FILTER,
    };

    let proposed = [
        (MENU_SEARCH, 'S'),
        (MENU_FILTER, 'F'),
        (MENU_TOGGLE_ISSUES_FILTER, 'I'),
        (MENU_EXPANDED_VALUES, 'E'),
        (MENU_PATH_TREE, 'T'),
    ];

    // These five labels are the View menu, and the View menu is these five
    // labels. Without this the letters below would still be right about the
    // constants and say nothing about the menu — a sixth item, or one of the
    // five moved elsewhere, would leave the set unchecked and this test green.
    let filled: std::collections::BTreeSet<&str> = REGISTRY
        .iter()
        .filter(|entry| entry.menu == Some(MENU_GROUP_VIEW))
        .map(|entry| entry.msgid)
        .collect();
    let named: std::collections::BTreeSet<&str> =
        proposed.iter().map(|(label, _)| *label).collect();
    assert_eq!(filled, named);

    for (label, letter) in proposed {
        assert_eq!(mnemonic(label), Some(letter), "the mnemonic of {label:?}");
    }
}

/// The v0.2.0 catalogue audit: every string §14 adds beyond the Announcements
/// is one the registry names.
///
/// A specification cross-reference rather than a behaviour — §14 is a prose
/// list, and prose is where a string goes missing without anything failing.
/// Once a msgid is here, `catalogue.rs` carries it the rest of the way: present
/// and usable in both languages, plural forms complete, placeholders intact.
/// The small overlap with `the_registry_holds_the_strings_the_shell_shows` is
/// deliberate: the two lists are kept for different reasons, and neither is the
/// other's summary.
#[test]
fn every_string_the_v0_2_0_delta_adds_is_in_the_registry() {
    use pathmaster_core::msgids::{
        BUTTON_FIX_SELECTED, BUTTON_GO_TO_ENTRY, COLUMN_ACTION, COLUMN_INDEX, COLUMN_ISSUE,
        DIALOG_COMMAND_LINE, DIALOG_FIX_SYSTEM, DIALOG_FIX_USER, DIALOG_TREE_SYSTEM,
        DIALOG_TREE_USER, DIALOG_UNKNOWN_ARGUMENT, FILTER_ALL, FILTER_WITH_ISSUES,
        FIX_REMOVE_QUOTES, MENU_COPY, MENU_EXPANDED_VALUES, MENU_FILTER, MENU_FIX_ISSUES,
        MENU_PATH_TREE, MENU_SEARCH, MENU_TITLE_VIEW, MENU_TOGGLE_ISSUES_FILTER, MENU_USER_GUIDE,
        OPERATION_DELETE, OPERATION_FIX_ISSUES, SEARCH_LABEL, TREE_RELATIVE_ENTRIES,
        TREE_UNRESOLVED_VARIABLES, USAGE,
    };

    let expected = [
        // Search (§14), and the main list's new column (§2.1).
        SEARCH_LABEL,
        COLUMN_INDEX,
        // Tree View (§6, §14) — Cancel beside [Go to entry] is the existing
        // BUTTON_DIALOG_CANCEL, which v0.1.0 already registered.
        DIALOG_TREE_USER,
        DIALOG_TREE_SYSTEM,
        BUTTON_GO_TO_ENTRY,
        TREE_UNRESOLVED_VARIABLES,
        TREE_RELATIVE_ENTRIES,
        // Fix Issues (§7, §14). The deletion's Action cell is deliberately
        // absent: it reuses OPERATION_DELETE, the same operation under the
        // same English (ADR-0004), and a second msgid for it would be the
        // defect rather than the omission.
        DIALOG_FIX_USER,
        DIALOG_FIX_SYSTEM,
        BUTTON_FIX_SELECTED,
        COLUMN_ISSUE,
        COLUMN_ACTION,
        OPERATION_DELETE,
        FIX_REMOVE_QUOTES,
        OPERATION_FIX_ISSUES,
        // Command line (§10, §14).
        DIALOG_UNKNOWN_ARGUMENT,
        DIALOG_COMMAND_LINE,
        USAGE,
        // The menus §12 grew: a whole new bar title with its five items, plus
        // one item each in Edit and Help. The Filter submenu's other five
        // states reuse the Status column's Issue words.
        MENU_TITLE_VIEW,
        MENU_SEARCH,
        MENU_FILTER,
        FILTER_ALL,
        FILTER_WITH_ISSUES,
        MENU_TOGGLE_ISSUES_FILTER,
        MENU_EXPANDED_VALUES,
        MENU_PATH_TREE,
        MENU_COPY,
        MENU_FIX_ISSUES,
        MENU_USER_GUIDE,
    ];

    for msgid in expected {
        assert!(
            REGISTRY.iter().any(|entry| entry.msgid == msgid),
            "§14 names {msgid:?} and the Catalogue does not hold it"
        );
    }
}
