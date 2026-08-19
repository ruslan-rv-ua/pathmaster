//! Splitting a raw `PATH` value into Entries and joining them back (spec §5).
//!
//! An Entry is the raw substring between `;` separators, byte-for-byte —
//! whitespace, case, quotes and trailing `\` all preserved. Quotes never
//! protect a separator. The round-trip invariant: `join(&split(v)) == v`.

/// Splits a decoded `PATH` value into raw Entry substrings.
///
/// An empty value decodes to zero Entries, not one empty Entry; a trailing
/// `;` means the last Entry is empty.
pub fn split(value: &str) -> Vec<&str> {
    if value.is_empty() {
        return Vec::new();
    }
    value.split(';').collect()
}

/// Joins raw Entry substrings back into a `PATH` value.
pub fn join<S: AsRef<str>>(entries: &[S]) -> String {
    entries
        .iter()
        .map(AsRef::as_ref)
        .collect::<Vec<_>>()
        .join(";")
}
