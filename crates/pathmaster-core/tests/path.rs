//! Split/join at the crate boundary (spec §5, ticket impl-02).
//!
//! An Entry is the raw substring between `;` separators, byte-for-byte;
//! split-then-join reproduces the decoded value exactly.

use pathmaster_core::msgids;
use pathmaster_core::path::{join, rejection, split, Rejection};

#[test]
fn empty_value_decodes_to_zero_entries() {
    assert_eq!(split(""), Vec::<&str>::new());
}

#[test]
fn joining_zero_entries_yields_the_empty_string() {
    assert_eq!(join(&split("")), "");
}

#[test]
fn entries_are_raw_substrings_byte_for_byte() {
    let value = r#"C:\Windows; C:\spaced \;"C:\quoted";%SystemRoot%\System32"#;
    assert_eq!(
        split(value),
        vec![
            r"C:\Windows",
            r" C:\spaced \",
            r#""C:\quoted""#,
            r"%SystemRoot%\System32",
        ],
    );
}

#[test]
fn quotes_never_protect_a_separator() {
    // Splitting is naive, as the OS's own `CreateProcessW`/`SearchPathW`,
    // PowerShell and Python are (spec §7, FR-diag-split): a quoted `;` shows
    // as the two odd Entries those consumers also see.
    assert_eq!(
        split(r#""C:\semi;colon";C:\Windows"#),
        vec![r#""C:\semi"#, r#"colon""#, r"C:\Windows"],
    );
}

proptest::proptest! {
    #[test]
    fn split_then_join_is_byte_identity_for_any_value(value in ".*") {
        proptest::prop_assert_eq!(join(&split(&value)), value);
    }
}

#[test]
fn split_then_join_reproduces_the_value_exactly() {
    // A trailing `;` means the last Entry is empty; `;` alone is two empty
    // Entries; `;;` inside is an empty Entry between neighbours.
    for value in [
        r"C:\one;C:\two",
        r"C:\one;",
        ";",
        ";;",
        r"C:\one;;C:\two",
        r"lower;UPPER;Mixed\",
        "no separators at all",
    ] {
        assert_eq!(join(&split(value)), value, "round trip of {value:?}");
    }
}

// ---- What may be committed as an Entry (spec §6, FR-edit-f2) ----

#[test]
fn an_ordinary_path_is_accepted() {
    assert_eq!(rejection(r"C:\Program Files\Tool\bin"), None);
}

#[test]
fn the_length_zero_entry_is_rejected() {
    assert_eq!(rejection(""), Some(Rejection::Empty));
}

#[test]
fn whitespace_only_commits_verbatim() {
    // Blocking "   " would smuggle a trim into validation, and the editor
    // never trims or normalises (spec §6 D5). Whether it reads as an Empty
    // Entry is diagnostics' call, not the editor's.
    assert_eq!(rejection("   "), None);
    assert_eq!(rejection("\t"), None);
}

#[test]
fn each_forbidden_character_is_rejected_and_named() {
    for forbidden in ['<', '>', '|', '"'] {
        assert_eq!(
            rejection(&format!(r"C:\dir{forbidden}")),
            Some(Rejection::ForbiddenCharacter(forbidden)),
            "{forbidden:?} is forbidden in an Entry",
        );
    }
}

#[test]
fn the_separator_may_not_be_typed_into_an_entry() {
    // An Entry cannot contain the separator it is defined by: typing a second
    // path means a second Entry, not a character.
    assert_eq!(
        rejection(r"C:\one;C:\two"),
        Some(Rejection::ForbiddenCharacter(';')),
    );
}

#[test]
fn the_first_forbidden_character_in_the_text_is_the_one_reported() {
    assert_eq!(
        rejection(r#"C:\a|b<c"#),
        Some(Rejection::ForbiddenCharacter('|')),
    );
}

#[test]
fn validation_polices_characters_only_never_the_path_itself() {
    // A duplicate, a path that does not exist yet and a relative one all
    // commit legally — diagnostics flags them asynchronously (spec §6 D6).
    for text in [r"Z:\not\created\yet", "..", r"\\server\share", "tool"] {
        assert_eq!(rejection(text), None, "{text:?} is a legal Entry");
    }
}

#[test]
fn a_variable_reference_is_ordinary_text_to_validation() {
    assert_eq!(rejection(r"%SystemRoot%\System32"), None);
}

#[test]
fn each_rejection_names_the_catalogue_string_that_is_its_dialog() {
    // The message *is* the dialog's title — NVDA never speaks a body — so a
    // rejection with no Catalogue string would be a silent refusal.
    let registered: Vec<&str> = msgids::REGISTRY.iter().map(|entry| entry.msgid).collect();
    for rejection in [Rejection::Empty, Rejection::ForbiddenCharacter('<')] {
        let msgid = rejection.catalogue_msgid();
        assert!(
            registered.contains(&msgid),
            "{rejection:?} names {msgid:?}, which the Catalogue does not hold",
        );
    }
    assert_ne!(
        Rejection::Empty.catalogue_msgid(),
        Rejection::ForbiddenCharacter('<').catalogue_msgid(),
    );
    // The forbidden-character message names the character it rejected.
    assert_eq!(
        msgids::placeholders(Rejection::ForbiddenCharacter('<').catalogue_msgid()),
        vec!["character"],
    );
}
