//! The Interface Language: the stored choice, and the one branch that resolves
//! it into the language this run speaks (spec §11).
//!
//! Resolution happens once at startup and never again while the application
//! runs. English is the **fallback**, not the default — that is why msgids are
//! English source text (ADR-0004).

/// What `settings.json` stores: the user's choice, never its outcome.
///
/// `Auto` is expressible explicitly so "follow the system again" is something
/// the file can say, rather than something only a deleted key can mean.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageChoice {
    /// Follow the system language.
    Auto,
    English,
    Ukrainian,
}

impl LanguageChoice {
    /// Every choice the Settings dialog's selector offers, in the order it
    /// offers them (spec §13).
    ///
    /// `Auto` leads because it is the default and the one a user undoing an
    /// explicit choice comes back to; the two languages follow in the order
    /// [`Language`] names them. The dialog reads its answer back by position,
    /// so this is the one place that order is written down.
    pub const SELECTABLE: [LanguageChoice; 3] = [
        LanguageChoice::Auto,
        LanguageChoice::English,
        LanguageChoice::Ukrainian,
    ];

    /// Reads a stored choice, or `None` when the value is not one of the three.
    ///
    /// An unrecognised value is a miss, not an `Auto`: the settings layer owes
    /// it a `WARN` line and keeps the raw value in the file.
    pub fn parse(stored: &str) -> Option<Self> {
        match stored {
            "auto" => Some(LanguageChoice::Auto),
            "en" => Some(LanguageChoice::English),
            "uk" => Some(LanguageChoice::Ukrainian),
            _ => None,
        }
    }

    /// The form written back to `settings.json`.
    pub fn as_str(self) -> &'static str {
        match self {
            LanguageChoice::Auto => "auto",
            LanguageChoice::English => "en",
            LanguageChoice::Ukrainian => "uk",
        }
    }

    /// Where this choice sits among the selector's items, which is how the
    /// dialog decides what to show as selected.
    ///
    /// `None` is unreachable — every choice is in
    /// [`SELECTABLE`](Self::SELECTABLE), and a test says so — and means "no
    /// selection", which is the one answer that cannot show a choice nobody
    /// made.
    pub fn selector_index(self) -> Option<usize> {
        Self::SELECTABLE
            .iter()
            .position(|selectable| *selectable == self)
    }

    /// The choice the selector's item at `index` stands for, or `None` for a
    /// position it has no item at.
    ///
    /// The other half of [`selector_index`](Self::selector_index), and here
    /// beside it: the selector answers by position, so the walk out to a
    /// position and the walk back from one are one rule and are tested as a
    /// round trip.
    pub fn at_selector_index(index: usize) -> Option<LanguageChoice> {
        Self::SELECTABLE.get(index).copied()
    }

    /// The language this choice names, when it names one — `None` for `Auto`,
    /// which names a question put to Windows rather than an answer.
    ///
    /// The one place the two enums are mapped onto each other, so
    /// [`resolve`] and the selector's own labels cannot come to disagree about
    /// which language `uk` is.
    pub fn language(self) -> Option<Language> {
        match self {
            LanguageChoice::Auto => None,
            LanguageChoice::English => Some(Language::English),
            LanguageChoice::Ukrainian => Some(Language::Ukrainian),
        }
    }
}

/// The languages that ship — one catalogue code each.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Ukrainian,
}

impl Language {
    /// The catalogue code: the `i18n/<code>.po` basename, and what
    /// `set_language_str` is given.
    pub fn code(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Ukrainian => "uk",
        }
    }

    /// The language's name in its own language, for the selector. Endonyms are
    /// deliberately outside the Catalogue: a user who cannot read the current
    /// Interface Language must still be able to find theirs.
    pub fn endonym(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Ukrainian => "Українська",
        }
    }
}

/// The system language, reduced to the only distinction that matters here.
///
/// wx knows one `Ukrainian`, so `uk` and `uk-UA` both land on it, and every
/// other language — `Unknown` included — is `Other`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemLanguage {
    Ukrainian,
    Other,
}

/// Resolves the Interface Language for this run.
///
/// Adding a third language means dropping `i18n/xx.po` in — the `.mo`, the
/// loader's table and `available_translations` all follow from that — and then
/// hand-editing this module: a variant on [`Language`] with its code and
/// endonym, one on [`LanguageChoice`] with its stored form, its place in
/// [`SELECTABLE`](LanguageChoice::SELECTABLE) and an arm in
/// [`language`](LanguageChoice::language), and one on [`SystemLanguage`] if the
/// system is to be followed into it. The catalogue is data; which languages
/// exist is not.
pub fn resolve(choice: LanguageChoice, system: SystemLanguage) -> Language {
    // A choice that names a language *is* the answer; only `Auto` asks.
    choice.language().unwrap_or(match system {
        SystemLanguage::Ukrainian => Language::Ukrainian,
        SystemLanguage::Other => Language::English,
    })
}
