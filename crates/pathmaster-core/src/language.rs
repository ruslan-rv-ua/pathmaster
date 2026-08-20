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
/// endonym, one on [`LanguageChoice`] with its stored form, one on
/// [`SystemLanguage`] if the system is to be followed into it, and an arm here.
/// The catalogue is data; which languages exist is not.
pub fn resolve(choice: LanguageChoice, system: SystemLanguage) -> Language {
    match choice {
        LanguageChoice::English => Language::English,
        LanguageChoice::Ukrainian => Language::Ukrainian,
        LanguageChoice::Auto => match system {
            SystemLanguage::Ukrainian => Language::Ukrainian,
            SystemLanguage::Other => Language::English,
        },
    }
}
