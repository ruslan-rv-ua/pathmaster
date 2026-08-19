//! Split/join at the crate boundary (spec §5, ticket impl-02).
//!
//! An Entry is the raw substring between `;` separators, byte-for-byte;
//! split-then-join reproduces the decoded value exactly.

use pathmaster_core::path::{join, split};

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
