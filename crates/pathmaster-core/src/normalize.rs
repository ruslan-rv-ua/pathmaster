//! Normalisation: the comparison-time reading of an Entry (spec §7,
//! FR-diag-normalise).
//!
//! It exists to answer one question — "are these two the same?" — and its
//! result is never stored, never written, and never handed to the registry.
//! It never consults the filesystem either: no 8.3 resolution, no symlinks, no
//! canonical casing. No surveyed tool does (research/13), and every one of
//! those would turn a comparison into I/O.
//!
//! The pipeline is fixed and ordered: strip one pair of surrounding `"` →
//! expand `%VAR%` → `/`→`\` → trim the trailing `\` unless that leaves a bare
//! root → fold case. The first two steps are also what the existence check
//! needs — the quote-stripped, expanded text is the path to probe — so they
//! are public in their own right and [`Normalised::of_expanded`] finishes the
//! job for a caller that already has them.
//!
//! The environment is injected rather than read: core takes no OS call, and a
//! rulebook whose answers depend on the machine it runs on is not testable.

/// The process environment as expansion reads it.
///
/// The lookup is case-insensitive, as `GetEnvironmentVariableW`'s own is —
/// `%systemroot%` and `%SystemRoot%` name the same variable. The adapter over
/// the real environment lands with the async pass (ticket 12).
pub trait Environment {
    /// The value of `name`, or `None` when this run does not define it.
    fn lookup(&self, name: &str) -> Option<String>;
}

/// What one expansion pass produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expansion {
    /// The expanded text. Unknown names are still in it, verbatim.
    pub text: String,
    /// Whether the text *begins* with a `%NAME%` this run does not define.
    ///
    /// The diagnostics rules read exactly this and nothing wider: whether a
    /// path is fully qualified is decided by its first characters, so an
    /// unresolved reference standing there is the one that makes the question
    /// unanswerable. One further along — `tools\%NOPE%` — leaves the shape
    /// perfectly legible (spec §7).
    pub starts_unresolved: bool,
}

/// An Entry's comparison key: two Entries are the same path exactly when their
/// `Normalised` values are equal.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Normalised(String);

impl Normalised {
    /// The whole pipeline over one raw Entry.
    pub fn of(entry: &str, env: &dyn Environment) -> Normalised {
        Normalised::of_expanded(&expand(strip_quotes(entry), env).text)
    }

    /// The pipeline's tail — slash direction, the trailing separator, the case
    /// fold — over text whose quotes are already stripped and whose `%VAR%`
    /// references are already expanded. The diagnostic pass needs that text for
    /// the filesystem probe anyway, and expanding twice would be a second
    /// answer to the same question.
    pub fn of_expanded(text: &str) -> Normalised {
        let backslashed = text.replace('/', "\\");
        Normalised(fold_case(trim_trailing_separators(&backslashed)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One pair of surrounding `"`, and only one.
///
/// Quotes are how a `;` was historically embedded in one Entry, and they still
/// arrive from the registry — but the OS's own path search does not strip them
/// (research/13), so the quoted spelling names a directory only for half the
/// ecosystem. Comparison and the existence check therefore read past them; the
/// raw Entry still round-trips untouched.
pub fn strip_quotes(entry: &str) -> &str {
    entry
        .strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
        .unwrap_or(entry)
}

/// Replaces `%NAME%` references with their values, `ExpandEnvironmentStringsW`'s
/// way (measured 2026-08-20; the runs are recorded in impl ticket 09):
///
/// * a defined name is replaced, the lookup ignoring case;
/// * an unknown name stays literal — and a failed reference gives its closing
///   `%` back to the scan, so `%NOPE%SystemRoot%` expands the second half;
///   an unknown name in the leading position is reported (`starts_unresolved`);
/// * `%%` is not a reference and resolves nothing;
/// * an unterminated `%` is ordinary text;
/// * the pass is single: a value that itself contains `%VAR%` is not expanded
///   again.
pub fn expand(text: &str, env: &dyn Environment) -> Expansion {
    let mut out = String::with_capacity(text.len());
    let mut starts_unresolved = false;
    let mut rest = text;
    while let Some(open) = rest.find('%') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        // A reference needs a closing `%` and a name between the two.
        let reference = after.find('%').filter(|close| *close > 0);
        let resolved =
            reference.and_then(|close| env.lookup(&after[..close]).map(|value| (value, close)));
        match resolved {
            Some((value, close)) => {
                out.push_str(&value);
                rest = &after[close + 1..];
            }
            // A failed reference emits its opening `%` and the scan resumes at
            // the very next character — the closing `%` is still in play.
            None => {
                starts_unresolved |= out.is_empty() && reference.is_some();
                out.push('%');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    Expansion {
        text: out,
        starts_unresolved,
    }
}

/// Whether `text` carries a `%NAME%` reference at all — the one question the
/// convert-or-keep dialog asks (spec §6).
///
/// A `REG_SZ` Scope stores such an Entry as literal text, so committing one
/// into a `REG_SZ` Scope is the single occasion the Value Type may change, and
/// only ever by asking. Whether the name resolves is a *different* question
/// and this does not ask it: the environment does not decide what a value type
/// expands. What counts as a reference is [`expand`]'s reading, walked the
/// same way — a closing `%` with a name between the two, `%%` resolving
/// nothing.
pub fn has_variable_reference(text: &str) -> bool {
    let mut rest = text;
    while let Some(open) = rest.find('%') {
        let after = &rest[open + 1..];
        if after.find('%').is_some_and(|close| close > 0) {
            return true;
        }
        // A failed reference gives its opening `%` back and the scan resumes
        // at the very next character, as expansion's does.
        rest = after;
    }
    false
}

/// Trims trailing separators, keeping one when trimming would leave a bare
/// root: `C:\` trimmed to `C:` names the current directory on that drive, and
/// `\` trimmed to nothing names nothing at all.
fn trim_trailing_separators(text: &str) -> &str {
    let trimmed = text.trim_end_matches('\\');
    if trimmed.len() < text.len() && (trimmed.is_empty() || trimmed.ends_with(':')) {
        &text[..trimmed.len() + 1]
    } else {
        trimmed
    }
}

/// Windows' ordinal case-insensitive comparison: a per-character uppercase
/// mapping, never a full case fold. `ß` stays `ß` rather than becoming `SS` —
/// the OS's upcase table maps one character to one character, and a comparison
/// that changed a string's length would not be one.
fn fold_case(text: &str) -> String {
    text.chars()
        .map(|c| {
            let mut upper = c.to_uppercase();
            match (upper.next(), upper.next()) {
                (Some(single), None) => single,
                _ => c,
            }
        })
        .collect()
}
