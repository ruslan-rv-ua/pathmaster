//! Expansion Mode: how the application is rendering Entries right now
//! (v0.2.0 spec §5, `CONTEXT.md`).
//!
//! One mode for the whole application — both Scope tabs render alike, because
//! Search and Filter are queries against data while this is how the user is
//! reading paths right now. It is **derived view state**: it reads the Working
//! Copy and never changes it, sits outside the Undo history, and every Run
//! starts raw with nothing persisted.
//!
//! What lives here is the rule, which is small: raw mode shows the stored text
//! untouched, and expanded mode shows **Normalisation's own reading** of it —
//! [`normalize::expand`] over the process environment, so what is shown can
//! never disagree with what is diagnosed. Nothing else of the Normalisation
//! pipeline applies: quote stripping, the trailing separator and the case fold
//! answer "are these the same path?", and a rendering is not a comparison key.
//! An undefined `%VAR%` therefore stays literal in place, with no new Issue
//! type and no inline marker — the Status column's natural `Missing` already
//! answers "why" — and the Value Type conditions none of it.
//!
//! The environment is injected, like every other reading of it: core takes no
//! OS call.
//!
//! [`normalize::expand`]: crate::normalize::expand

use std::borrow::Cow;

use crate::normalize::{expand, Environment};

/// Which rendering the lists are showing.
///
/// A value rather than a `bool` for the reason every direction in this
/// application is one: a bare `true` at a call site says nothing about which
/// way it went. [`Default`] is the mode every Run opens in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Entries as stored — `%JAVA_HOME%\bin`.
    #[default]
    Raw,
    /// Entries with their `%VAR%` references expanded — `C:\jdk21\bin`.
    Expanded,
}

impl Mode {
    /// The mode the toggle command leaves behind.
    pub fn toggled(self) -> Mode {
        match self {
            Mode::Raw => Mode::Expanded,
            Mode::Expanded => Mode::Raw,
        }
    }

    /// Whether values are being shown expanded — which is what the View menu's
    /// `wxITEM_CHECK` item marks, and the one place that reading is made, so
    /// the mark and the rendering cannot disagree (v0.2.0 §5).
    pub fn expanded(self) -> bool {
        self == Mode::Expanded
    }

    /// One Entry as the list shows it under this mode.
    ///
    /// Raw borrows: the default mode has nothing to do, and every rebuild
    /// renders every visible row. Expanded owns what one expansion pass
    /// produced.
    pub fn render<'a>(self, raw: &'a str, env: &dyn Environment) -> Cow<'a, str> {
        match self {
            Mode::Raw => Cow::Borrowed(raw),
            Mode::Expanded => Cow::Owned(expand(raw, env).text),
        }
    }
}
