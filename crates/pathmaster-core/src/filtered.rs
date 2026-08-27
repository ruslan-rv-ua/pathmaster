//! The Filtered View's engine: the two narrowing axes and the membership they
//! compose (v0.2.0 spec §2, §3, §4).
//!
//! A **Filtered View** is an Editing Session's view of its Working Copy,
//! narrowed to the Entries matching that Scope's Search text **and** its
//! Filter, composed with AND (`CONTEXT.md`). Both axes live here — [`matches`]
//! is the Search half's one rule, [`Filter`] the diagnostic half's — and
//! [`Criteria`] is the pair, which is the thing a Scope actually holds.
//! Everything else about the view (per-Scope state, the focus rule, what is
//! spoken) is the window's, because it is about widgets and timing rather than
//! about text.
//!
//! The Search rule is deliberately small: **case-insensitive substring with
//! Unicode case folding, slash-folded (`/`→`\`), and nothing else** (spec §3).
//! Case and slash direction are foldings the domain already applies
//! everywhere; quote stripping, trailing-`\` trimming and `%VAR%` expansion
//! change *what text exists* and stay out — a search for `"` must find the
//! `Quoted` Entries. The query is never trimmed: whitespace is Entry content.

use crate::diagnostics::Issue;
use crate::msgids;

/// A Scope's Filter: which diagnostic statuses its list shows (v0.2.0 §4,
/// `CONTEXT.md`).
///
/// Seven exclusive states and no severity partition: v0.1.0's Issue types
/// share one consequence, so the PRD's "Errors / Warnings" split has nothing
/// to be a split *of*. Over-length is the sixth Issue and is **not** among
/// them — it flags a Scope rather than an Entry, and a state selecting it
/// would select every Entry or none.
///
/// [`Default`] is the state every Run opens each Scope in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Filter {
    /// Every Entry — the state that narrows nothing.
    #[default]
    All,
    /// The coarse axis: every Entry with a non-empty Status.
    WithIssues,
    /// The five per-type states, each admitting the Entries its type flagged.
    Missing,
    Relative,
    Quoted,
    Duplicate,
    Empty,
}

impl Filter {
    /// The seven states in submenu order (v0.2.0 §4): the two coarse states,
    /// then the five types in the rulebook's own severity order — the order
    /// the Status column joins them in, so the menu and the column cannot come
    /// to disagree about which Issue comes first.
    pub const ALL: [Filter; 7] = [
        Filter::All,
        Filter::WithIssues,
        Filter::Missing,
        Filter::Relative,
        Filter::Quoted,
        Filter::Duplicate,
        Filter::Empty,
    ];

    /// The Issue type this state selects, or `None` for the two coarse states,
    /// which are about the Issue set rather than about a member of it.
    pub fn issue(self) -> Option<Issue> {
        match self {
            Filter::All | Filter::WithIssues => None,
            Filter::Missing => Some(Issue::Missing),
            Filter::Relative => Some(Issue::Relative),
            Filter::Quoted => Some(Issue::Quoted),
            Filter::Duplicate => Some(Issue::Duplicate),
            Filter::Empty => Some(Issue::Empty),
        }
    }

    /// Whether an Entry with this Issue set is visible under this state
    /// (v0.2.0 §4).
    ///
    /// The set, not its first member: a `Missing`, `Duplicate` Entry is both,
    /// and either state shows it. `issues` is what the **last completed pass**
    /// found, so before one lands every set is empty and a narrowing state
    /// shows nothing — which is the honest answer for a question nothing has
    /// yet asked of the data (spec §7, FR-diag-async).
    pub fn admits(self, issues: &[Issue]) -> bool {
        match self.issue() {
            Some(issue) => issues.contains(&issue),
            None => match self {
                Filter::All => true,
                _ => !issues.is_empty(),
            },
        }
    }

    /// Whether this state narrows the view at all — half of Announcement 1's
    /// two-part condition (see [`Criteria::narrowing`]).
    pub fn narrows(self) -> bool {
        self != Filter::All
    }

    /// The state Ctrl+I leaves behind (v0.2.0 §4, §12): the coarse axis alone.
    ///
    /// `All` → `With issues`, and **any** narrowing state → `All`, so one
    /// keystroke is always a way back out. The five per-type states are
    /// menu-only: a toggle that cycled them would be a keystroke whose
    /// meaning depended on where it was pressed.
    pub fn toggled(self) -> Filter {
        match self {
            Filter::All => Filter::WithIssues,
            _ => Filter::All,
        }
    }

    /// This state's name — in the submenu, in Announcement 11, and in
    /// StatusBar field 0 (v0.2.0 §4, §13 item 11, §16).
    ///
    /// **The five type states reuse the Status column's own words**, which is
    /// why the Filter adds no msgid for a name: one Issue has one name, and a
    /// second English for it would be a second translation to keep in step
    /// (ADR-0004). None of the seven carries a mnemonic, for the same reason —
    /// three surfaces show these words and two of them would print the `&`.
    pub fn catalogue_msgid(self) -> &'static str {
        match self {
            Filter::All => msgids::FILTER_ALL,
            Filter::WithIssues => msgids::FILTER_WITH_ISSUES,
            Filter::Missing => Issue::Missing.catalogue_msgid(),
            Filter::Relative => Issue::Relative.catalogue_msgid(),
            Filter::Quoted => Issue::Quoted.catalogue_msgid(),
            Filter::Duplicate => Issue::Duplicate.catalogue_msgid(),
            Filter::Empty => Issue::Empty.catalogue_msgid(),
        }
    }
}

/// One Scope's narrowing: its Search text and its Filter, which is what a
/// Filtered View *is* (v0.2.0 §2).
///
/// The two travel together because every question worth asking is about both
/// of them — is this Scope narrowed, is this Entry visible, what does the
/// StatusBar say — and a pair held as two fields is a pair two callers can
/// compose differently. [`Default`] is the unnarrowed state every Run opens
/// every Scope in; nothing here persists.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Criteria {
    /// The Search text, exactly as typed — never trimmed, since whitespace is
    /// Entry content (spec §3).
    pub query: String,
    pub filter: Filter,
}

impl Criteria {
    /// The pair, for a caller that has both.
    pub fn new(query: impl Into<String>, filter: Filter) -> Criteria {
        Criteria {
            query: query.into(),
            filter,
        }
    }

    /// Whether a Filtered View is active: **an empty query and the `All`
    /// state is no view at all**.
    ///
    /// This is Announcement 1's condition, and the one place it is stated:
    /// §13 item 1 made it two-part when the Filter axis landed, and a
    /// condition spelled out at each call site is one that can be spelled out
    /// differently. It is also what closes Move Up, Move Down and Add (§2).
    pub fn narrowing(&self) -> bool {
        !self.query.is_empty() || self.filter.narrows()
    }

    /// Whether the Search half alone is narrowing. ESC answers to this one:
    /// it clears the text, and a view a Filter is still narrowing is still a
    /// Filtered View (spec §3).
    pub fn searching(&self) -> bool {
        !self.query.is_empty()
    }

    /// Whether one Entry is visible: **both** axes admit it (v0.2.0 §2).
    ///
    /// `rendering` is the text the list is showing — raw, or the expanded
    /// reading under Expansion Mode — so what the spoken count counts is
    /// exactly what the arrow keys will read (§3, §5).
    pub fn admits(&self, rendering: &str, issues: &[Issue]) -> bool {
        matches(rendering, &self.query) && self.filter.admits(issues)
    }
}

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

/// The visible set: the positions (0-based, Working-Copy order) of the Entries
/// the criteria admit. Order is preserved — a Filtered View shows fewer rows,
/// never reordered ones — and the positions are what keeps the `#` column
/// honest under any narrowing.
///
/// Each Entry arrives as the two things the criteria ask about: the text the
/// list is showing, and what the last completed pass found about it. The
/// rendering borrows or owns — raw mode hands over the Entries' own `&str`,
/// and expanded mode the `String`s one expansion pass produced (v0.2.0 §5) —
/// and the caller decides which, because that is the mode.
pub fn visible_indices<'a, S: AsRef<str>>(
    entries: impl IntoIterator<Item = (S, &'a [Issue])>,
    criteria: &Criteria,
) -> Vec<usize> {
    entries
        .into_iter()
        .enumerate()
        .filter(|(_, (rendering, issues))| criteria.admits(rendering.as_ref(), issues))
        .map(|(index, _)| index)
        .collect()
}

/// The one fold (spec §3): Unicode lowercase, and `/` read as `\`.
fn fold(text: &str) -> String {
    text.to_lowercase().replace('/', "\\")
}
