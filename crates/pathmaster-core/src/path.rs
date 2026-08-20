//! Splitting a raw `PATH` value into Entries, joining them back, and what may
//! be typed into one (spec §5, §6).
//!
//! An Entry is the raw substring between `;` separators, byte-for-byte —
//! whitespace, case, quotes and trailing `\` all preserved. Quotes never
//! protect a separator. The round-trip invariant: `join(&split(v)) == v`.
//!
//! The editor's validation lives here because it is the same fact read from
//! the other end: the separator an Entry is defined by is the separator that
//! cannot be typed into one.

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

/// The characters an Entry may not contain: Windows' own illegal path
/// characters, plus the `;` that separates one Entry from the next.
///
/// `*` and `?` are deliberately absent — they are illegal in a *file name*,
/// but a `PATH` entry naming a directory with one is the user's business, and
/// `:` is how a drive is spelled.
const FORBIDDEN: [char; 5] = ['<', '>', '|', '"', ';'];

/// Why the editor may not commit a text as an Entry (spec §6, FR-edit-f2).
///
/// Each variant names the Catalogue string that *is* the error dialog: NVDA
/// never speaks a `MessageDialog`'s body, so the message is its title
/// (spec §10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rejection {
    /// The field was empty. Length-zero is the only emptiness validation
    /// knows: whitespace-only is a legal Entry.
    Empty,
    /// The text carries a character an Entry may not contain.
    ForbiddenCharacter(char),
}

impl Rejection {
    /// The msgid whose text is the error dialog's title. The
    /// [`ForbiddenCharacter`](Rejection::ForbiddenCharacter) string carries a
    /// `{character}` placeholder the caller fills with the character itself.
    pub fn catalogue_msgid(&self) -> &'static str {
        match self {
            Rejection::Empty => crate::msgids::REJECTED_EMPTY,
            Rejection::ForbiddenCharacter(_) => crate::msgids::REJECTED_FORBIDDEN_CHARACTER,
        }
    }
}

/// Why `text` may not be committed as an Entry, or `None` when it may.
///
/// Validation polices characters and nothing else. **Whitespace-only commits
/// verbatim** — blocking `"   "` would smuggle a trim into validation, and the
/// editor never trims or normalises. Duplicates, relative paths and paths that
/// do not exist yet all commit legally too: those are Issues diagnostics
/// reports, never input errors (spec §6, D5 and D6).
pub fn rejection(text: &str) -> Option<Rejection> {
    if text.is_empty() {
        return Some(Rejection::Empty);
    }
    text.chars()
        .find(|c| FORBIDDEN.contains(c))
        .map(Rejection::ForbiddenCharacter)
}
