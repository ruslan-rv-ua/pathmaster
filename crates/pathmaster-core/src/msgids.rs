//! The Catalogue's registry: every msgid the application looks up, in one list
//! (spec §11, ADR-0004).
//!
//! `translate()` is a function, not a macro, so nothing extracts the set of
//! msgids from the code and nothing checks that one exists. Naming every msgid
//! here is what makes the set knowable — and turns "one Catalogue" from a rule
//! someone must remember into a list a test can walk. The completeness gate
//! walks it; the shipped `.po` files are measured against it.
//!
//! A msgid is English source text, and that English is an API surface: `msgctxt`
//! is unbound at every level, so where two strings mean different things their
//! English must differ (ADR-0004). Placeholders are named braces — `%d` would be
//! indistinguishable from the `%VAR%` this application exists to edit.

/// One Catalogue entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CatalogueEntry {
    /// The English source text: the lookup key, and what a miss returns.
    pub msgid: &'static str,
    /// The plural msgid, for the entries looked up through `translate_plural`.
    /// The singular in `msgid` is the key both forms are found by.
    pub plural: Option<&'static str>,
    /// The menu this label belongs to, when its mnemonic must stay unique among
    /// that menu's siblings. Menus land with the tickets that build them.
    pub menu: Option<&'static str>,
}

impl CatalogueEntry {
    /// A string with one form: a label, a title, an Announcement.
    pub const fn text(msgid: &'static str) -> Self {
        CatalogueEntry {
            msgid,
            plural: None,
            menu: None,
        }
    }

    /// A string whose wording depends on a count.
    pub const fn plural(singular: &'static str, plural: &'static str) -> Self {
        CatalogueEntry {
            msgid: singular,
            plural: Some(plural),
            menu: None,
        }
    }

    /// A menu label, whose `&` mnemonic is gated against its siblings'.
    pub const fn menu_item(msgid: &'static str, menu: &'static str) -> Self {
        CatalogueEntry {
            msgid,
            plural: None,
            menu: Some(menu),
        }
    }
}

/// The Scope tab labels (spec §12).
pub const TAB_USER: &str = "User PATH";
pub const TAB_SYSTEM: &str = "System PATH";
pub const TAB_BACKUPS: &str = "Backups";

/// The two list columns every Scope tab shows (spec §7, §12).
pub const COLUMN_PATH: &str = "Path";
pub const COLUMN_STATUS: &str = "Status";

/// Announcement 1 (spec §10.1): the entry count on tab activation and Refresh.
/// The zero case is its own msgid rather than a plural form — Ukrainian's
/// `nplurals=3` has no zero form, and "no entries" is better speech than "0".
pub const ENTRIES_USER: &str = "User PATH: {n} entry";
pub const ENTRIES_USER_PLURAL: &str = "User PATH: {n} entries";
pub const ENTRIES_USER_NONE: &str = "User PATH: no entries";
pub const ENTRIES_SYSTEM: &str = "System PATH: {n} entry";
pub const ENTRIES_SYSTEM_PLURAL: &str = "System PATH: {n} entries";
pub const ENTRIES_SYSTEM_NONE: &str = "System PATH: no entries";

/// Every msgid the application looks up. Later tickets append their strings;
/// nothing is looked up that is not named here.
pub const REGISTRY: &[CatalogueEntry] = &[
    CatalogueEntry::text(TAB_USER),
    CatalogueEntry::text(TAB_SYSTEM),
    CatalogueEntry::text(TAB_BACKUPS),
    CatalogueEntry::text(COLUMN_PATH),
    CatalogueEntry::text(COLUMN_STATUS),
    CatalogueEntry::plural(ENTRIES_USER, ENTRIES_USER_PLURAL),
    CatalogueEntry::text(ENTRIES_USER_NONE),
    CatalogueEntry::plural(ENTRIES_SYSTEM, ENTRIES_SYSTEM_PLURAL),
    CatalogueEntry::text(ENTRIES_SYSTEM_NONE),
];

/// The placeholder names in `text`, in order of appearance.
///
/// A placeholder is `{name}` with `name` made of ASCII letters, digits and
/// underscores; anything else between braces is ordinary text, and `%VAR%` is
/// always data.
pub fn placeholders(text: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let mut rest = text;
    while let Some((_, close, name)) = next_braces(rest) {
        if is_placeholder_name(name) {
            names.push(name);
        }
        rest = &rest[close + 1..];
    }
    names
}

/// Substitutes `{name}` placeholders — the one substitution helper (spec §11).
///
/// Values are copied in verbatim and never rescanned, so Entry text carrying
/// braces or `%VAR%` cannot turn into a placeholder. A placeholder with no
/// value is left as it stands: the gate makes that unreachable in shipped text,
/// and a readable string beats a panic in an application NVDA is speaking.
pub fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some((open, close, name)) = next_braces(rest) {
        out.push_str(&rest[..open]);
        match values.iter().find(|(key, _)| *key == name) {
            Some((_, value)) if is_placeholder_name(name) => out.push_str(value),
            _ => out.push_str(&rest[open..=close]),
        }
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    out
}

/// The mnemonic letter of a label: the character after its first single `&`.
///
/// `&&` is an escaped ampersand, not a mnemonic. Ukrainian labels carry the
/// Latin letter in parentheses — `"Файл(&F)"` — so this answers `'F'` there too.
pub fn mnemonic(label: &str) -> Option<char> {
    let mut chars = label.chars();
    while let Some(c) = chars.next() {
        if c != '&' {
            continue;
        }
        match chars.next() {
            Some('&') => continue,
            other => return other,
        }
    }
    None
}

/// The first mnemonic letter two of `labels` share, compared case-insensitively
/// as Alt+F and Alt+f are the same keystroke. Labels without a mnemonic are
/// passed over — their absence is a separate defect.
pub fn duplicate_mnemonic<'a>(labels: impl IntoIterator<Item = &'a str>) -> Option<char> {
    let mut seen = std::collections::BTreeSet::new();
    for label in labels {
        if let Some(letter) = mnemonic(label) {
            let folded = letter.to_lowercase().next().unwrap_or(letter);
            if !seen.insert(folded) {
                return Some(folded);
            }
        }
    }
    None
}

/// The next `{...}` in `rest`: where its brace opens, where it closes, and what
/// stands between them — which is a placeholder name only if
/// [`is_placeholder_name`] says so. Both readers of the Catalogue's text scan it
/// through here, so "what counts as a placeholder" is answered in one place.
fn next_braces(rest: &str) -> Option<(usize, usize, &str)> {
    let open = rest.find('{')?;
    let close = open + 1 + rest[open + 1..].find('}')?;
    Some((open, close, &rest[open + 1..close]))
}

fn is_placeholder_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}
