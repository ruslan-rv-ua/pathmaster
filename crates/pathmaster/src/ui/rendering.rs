//! Expansion Mode as the window holds it (v0.2.0 §5): the one app-wide flag,
//! and the environment it expands against.
//!
//! The mode is **one cell for the application** — both Scope tabs render alike
//! — so it is shared by `Rc` rather than passed down: every rendering path
//! reads it (the visible set, the rows, the counts, the menu's check mark),
//! and a mode threaded through as an argument is a mode two callers can
//! disagree about. It dies with the Run: no `settings.json` field, nothing
//! persisted, every Run opening raw.
//!
//! The environment travels with it because the two are never apart — a
//! rendering is a mode *and* what `%VAR%` resolves against — and it is the
//! same [`ProcessEnvironment`] the diagnostic pass reads, so what is shown can
//! never disagree with what is diagnosed.
//!
//! `Cell`, not [`Scoped`]: ADR-0011 is about borrows escaping into a dispatch,
//! and a `Copy` mode is read out by value with no borrow to escape — the same
//! shape `merged_length` and `relaunched` already have.
//!
//! [`Scoped`]: crate::scoped::Scoped

use std::borrow::Cow;
use std::cell::Cell;

use pathmaster_core::expansion::Mode;
use pathmaster_core::normalize::Environment;
use pathmaster_platform::diagnostics::ProcessEnvironment;

/// How the application is rendering Entries right now.
pub struct Rendering {
    mode: Cell<Mode>,
    env: ProcessEnvironment,
}

impl Rendering {
    /// The rendering every Run opens in: raw, over this process's environment.
    pub fn new() -> Rendering {
        Rendering {
            mode: Cell::new(Mode::default()),
            env: ProcessEnvironment,
        }
    }

    /// The mode now in force — what the menu's check mark reads, and what the
    /// lists are showing.
    pub fn mode(&self) -> Mode {
        self.mode.get()
    }

    /// Flips the mode and answers the one now in force, which is the one the
    /// Announcement names: the flip and what is said about it come from a
    /// single read, so they cannot describe different modes.
    ///
    /// **Nothing about the Working Copy is touched** — this is derived view
    /// state, no Checkpoint, invisible to Undo and Redo both ways (v0.2.0 §5).
    pub fn toggle(&self) -> Mode {
        let mode = self.mode.get().toggled();
        self.mode.set(mode);
        mode
    }

    /// One Entry as the list shows it now — the text the `Path` cell carries
    /// and the text Search matches, which are the same text by construction
    /// (v0.2.0 §3).
    pub fn render<'a>(&self, raw: &'a str) -> Cow<'a, str> {
        self.mode.get().render(raw, &self.env)
    }

    /// The environment `%VAR%` resolves against, for the one reading that is
    /// **not** a rendering: the Tree View's shape is the expanded reading
    /// whatever the mode says (v0.2.0 §6).
    ///
    /// Handed out from here rather than built afresh at the call site so the
    /// window keeps one environment — the same one the diagnostic pass reads,
    /// which is what stops a tree from placing an Entry somewhere its Status
    /// column disagrees with.
    pub fn environment(&self) -> &dyn Environment {
        &self.env
    }
}
