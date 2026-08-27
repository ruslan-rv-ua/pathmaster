//! The Filtered View's matching engine (v0.2.0 spec §2, §3).
//!
//! A **Filtered View** is an Editing Session's view of its Working Copy,
//! narrowed to the Entries matching that Scope's Search text and Filter,
//! composed with AND (`CONTEXT.md`). This module holds the Search half's one
//! rule — what matches — and the membership computation built on it; the
//! Filter axis composes in here when ticket 05 adds it. Everything else about
//! the view (per-Scope state, the focus rule, what is spoken) is the window's,
//! because it is about widgets and timing rather than about text.
//!
//! The rule is deliberately small: **case-insensitive substring with Unicode
//! case folding, slash-folded (`/`→`\`), and nothing else** (spec §3). Case
//! and slash direction are foldings the domain already applies everywhere;
//! quote stripping, trailing-`\` trimming and `%VAR%` expansion change *what
//! text exists* and stay out — a search for `"` must find the `Quoted`
//! Entries. The query is never trimmed: whitespace is Entry content.

/// Whether one Entry's displayed rendering matches the Search text.
///
/// Both sides go through the same fold — `str::to_lowercase`, which is the
/// Unicode fold, never an ASCII one that would be silently case-sensitive for
/// every Cyrillic path — and `/`→`\`, so a query typed with either slash finds
/// a path stored with the other. An empty query matches everything: no query
/// is no narrowing, not a search for nothing.
pub fn matches(rendering: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    fold(rendering).contains(&fold(query))
}

/// The visible set: the positions (0-based, Working-Copy order) of the
/// renderings the query matches. Order is preserved — a Filtered View shows
/// fewer rows, never reordered ones — and the positions are what keeps the
/// `#` column honest under any narrowing.
pub fn visible_indices<'a>(
    renderings: impl IntoIterator<Item = &'a str>,
    query: &str,
) -> Vec<usize> {
    renderings
        .into_iter()
        .enumerate()
        .filter(|(_, rendering)| matches(rendering, query))
        .map(|(index, _)| index)
        .collect()
}

/// The one fold (spec §3): Unicode lowercase, and `/` read as `\`.
fn fold(text: &str) -> String {
    text.to_lowercase().replace('/', "\\")
}
