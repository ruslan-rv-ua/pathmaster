//! The Catalogue: the one lookup, injected, and everything composed out of it
//! (spec §11, §10.1, ADR-0003, ADR-0009).
//!
//! CONTEXT.md's **Catalogue** says there is exactly one, so that what is shown
//! and what is spoken cannot drift apart. At runtime that one is wx's, and
//! `translate()` is a wx call — which pinned every function that *composes* a
//! user-facing string to the crate no test links. Only the lookup was on the
//! wrong side: the msgids and [`fill`](crate::msgids::fill) were already here.
//! So the lookup becomes the [`Lookup`] interface, the composition moves down
//! beside the msgids it fills, and both adapters wrap the same single lookup
//! rather than adding a second one (ADR-0009).
//!
//! **The seam is only where there are rules.** A widget label is a bare lookup
//! with nothing to get wrong, and those stay in the binary, calling its
//! `catalog::translate` directly. What is here is what composes: the
//! Announcements, both StatusBar fields, the Status column's join, and
//! validation's rejection text.
//!
//! [`Announcement`] is the other half. ADR-0003 declared the Announcement
//! catalogue closed at seven and nothing enforced it, because `announce()`
//! took a `&str` and every string in the program is a `&str`. An enum whose
//! variants carry their own data is a value that can be built where the thing
//! happens, handed to the one thing that speaks, and counted by a test — so
//! the set is closed by the compiler.

use crate::diagnostics::Issue;
use crate::msgids::{self, fill};
use crate::path::Rejection;
use crate::session::{Scope, UndoOutcome};
use crate::thresholds::{self, Overlength};

/// The Catalogue's lookup — the one thing about a Catalogue that core cannot
/// do for itself.
///
/// Two adapters implement it and there will never be a third: the binary's,
/// which is `catalog::translate` and wx's installed global behind it; and the
/// tests', which answers with the msgid and picks `n == 1 ? singular : plural`
/// — wxdragon's own documented fallback when no catalogue answers, not an
/// invention for testing (ADR-0009).
pub trait Lookup {
    /// The Catalogue's text for `msgid`. A miss returns the msgid, which is
    /// English source text (ADR-0004).
    fn translate(&self, msgid: &str) -> String;

    /// The Catalogue's text for a string whose wording depends on a count.
    /// `singular` is the msgid both forms are found by; the catalogue's own
    /// `Plural-Forms` rule picks between them, so Ukrainian's three forms need
    /// nothing here.
    fn translate_plural(&self, singular: &str, plural: &str, n: u32) -> String;
}

/// One of §10.1's Announcements: the closed set of messages the application
/// speaks, as a type.
///
/// **Six variants for seven Announcements.** Item 5 is item 4's text with the
/// ", unsaved changes" suffix rather than a message of its own, and
/// [`UndoOutcome::crossed_apply`](crate::session::UndoOutcome::crossed_apply)
/// already models exactly that; a seventh variant would be a second route to
/// one sentence (ADR-0009).
///
/// **No platform type appears here.** A reason and a failure both live in
/// `pathmaster-platform`, which core cannot name without reversing the
/// dependency direction — so each contributes the msgid its own
/// `catalogue_msgid()` returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Announcement {
    /// **1** — a Scope tab was activated, or Refreshed: how many Entries it
    /// holds.
    EntryCount { scope: Scope, count: usize },
    /// **2** — a Scope's Working Copy reached the registry. Wired by impl
    /// ticket 13, which registers the two strings §10.1 gives it and picks
    /// between them by Scope.
    Applied { msgid: &'static str },
    /// **3** — an Apply did not complete: the §9 taxonomy text naming why.
    /// Wired by impl ticket 13, whose typed failure lives in
    /// `pathmaster-platform` and so contributes its `catalogue_msgid()`.
    ApplyFailed { msgid: &'static str },
    /// **4**, and **5** when the restored Checkpoint was taken before the last
    /// Apply: what was undone or redone. No path text — focus lands on the row
    /// and NVDA reads it for free.
    UndoRedo {
        step: UndoStep,
        outcome: UndoOutcome,
    },
    /// **6** — the Cancel command discarded a Working Copy back to its
    /// Baseline.
    ChangesDiscarded,
    /// **7** — this run cannot write its Data Directory, and why. `reason` is
    /// the msgid `ReadOnlyReason::catalogue_msgid()` returns: the reason is a
    /// platform type, its name is Catalogue text.
    ReadOnly { reason: &'static str },
}

/// Which of Announcement 4's two sentences a restored Checkpoint earns.
///
/// The direction travels as a named value rather than a `bool`, for the reason
/// every other command in this application does: a bare `true` at the call
/// site says nothing about which way it went.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndoStep {
    Undone,
    Redone,
}

/// What StatusBar field 0 reports about one Scope: how many Entries it holds
/// now, and how many findings the last pass made there.
///
/// The two numbers answer to different clocks on purpose — the count is the
/// screen's and updates with the edit, the issues are the last pass's and catch
/// up one Timer tick later, which is what §12's "updated after every diagnostic
/// pass" means. `issues` is `None` before any pass has run, which is not the
/// same as a pass that found nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScopeCounts {
    pub scope: Scope,
    pub entries: usize,
    pub issues: Option<usize>,
}

/// The Catalogue, holding the lookup it was built with.
///
/// Never a global. A global lookup is the trap this module exists to leave:
/// it is what made every composing function a wx caller, and a second one
/// would be a second Catalogue.
pub struct Catalogue {
    lookup: Box<dyn Lookup>,
}

impl Catalogue {
    /// Builds the Catalogue over a lookup. Exactly one is built per run.
    pub fn new(lookup: impl Lookup + 'static) -> Catalogue {
        Catalogue {
            lookup: Box::new(lookup),
        }
    }

    /// What an Announcement says (spec §10.1).
    pub fn announcement(&self, announcement: Announcement) -> String {
        match announcement {
            Announcement::EntryCount { scope, count } => self.entry_count(scope, count),
            Announcement::Applied { msgid } | Announcement::ApplyFailed { msgid } => {
                self.lookup.translate(msgid)
            }
            Announcement::UndoRedo { step, outcome } => self.undo_redo(step, outcome),
            Announcement::ChangesDiscarded => self.lookup.translate(msgids::CHANGES_DISCARDED),
            Announcement::ReadOnly { reason } => self.read_only(reason),
        }
    }

    /// The rejected edit's error dialog, whose title is the whole of the
    /// error — NVDA never speaks a `MessageDialog`'s body (spec §6, §10).
    pub fn rejection(&self, reason: Rejection) -> String {
        let message = self.lookup.translate(reason.catalogue_msgid());
        match reason {
            Rejection::Empty => message,
            Rejection::ForbiddenCharacter(character) => {
                fill(&message, &[("character", &character.to_string())])
            }
        }
    }

    /// StatusBar field 0, the general status (spec §12): each Scope's entry
    /// count and the issues the last pass found there — or, in a Read-only
    /// Data run, the mode and its reason in their place.
    ///
    /// The Scopes are composed in the order they are given, which is the order
    /// the tabs are in and not the runtime order a pass evaluates them in:
    /// this field is read on demand as one sentence (`NVDA+End`).
    ///
    /// The read-only substitution is Announcement 7's own text, composed
    /// through it rather than beside it — the mode and its reason are one
    /// sentence with one wording, spoken once at startup and shown here for
    /// the rest of the run (spec §10.1 item 7).
    pub fn general_status(
        &self,
        scopes: [ScopeCounts; 2],
        readonly: Option<&'static str>,
    ) -> String {
        match readonly {
            Some(reason) => self.announcement(Announcement::ReadOnly { reason }),
            None => scopes
                .iter()
                .map(|counts| self.counts(counts))
                .collect::<Vec<String>>()
                .join(" | "),
        }
    }

    /// StatusBar field 1 (spec §12, FR-diag-overlength): the merged length
    /// always, with the `cmd.exe` warning appended past 8,191.
    ///
    /// Over-length lives here and nowhere else — never in the Status column,
    /// never an Announcement — because no Entry is at fault for a length that
    /// only exists once both Scopes are merged. Empty only before the first
    /// pass has landed: the length is measured by the pass, and inventing a
    /// second place to compute it would be a second answer to the same
    /// question.
    pub fn merged_length(&self, length: Option<usize>) -> String {
        let Some(length) = length else {
            return String::new();
        };
        let mut text = fill(
            &self.lookup.translate_plural(
                msgids::MERGED_LENGTH,
                msgids::MERGED_LENGTH_PLURAL,
                length as u32,
            ),
            &[("n", &length.to_string())],
        );
        // Past the first threshold, which is the one this field names. The
        // hard cap is past it too, and has nothing further to say here — it
        // speaks at Apply.
        if thresholds::classify(length) != Overlength::Within {
            text.push_str(&self.lookup.translate(msgids::MERGED_LENGTH_EXCEEDS));
        }
        text
    }

    /// One Scope's half of StatusBar field 0.
    fn counts(&self, counts: &ScopeCounts) -> String {
        let entries = self.entry_count(counts.scope, counts.entries);
        match counts.issues {
            Some(issues) => entries + &self.issue_count(issues),
            None => entries,
        }
    }

    /// The issue half of one Scope's counts, a **suffix** because one gettext
    /// lookup selects its plural form on one number and that line carries two.
    ///
    /// Zero is shown like any other count once a pass has run — a Scope with
    /// no Entries provably has no Issues, and a fixed shape is easier to parse
    /// aurally than one that comes and goes. The Status column, where "never
    /// OK" applies, is a different surface.
    fn issue_count(&self, count: usize) -> String {
        fill(
            &self.lookup.translate_plural(
                msgids::ISSUES_SUFFIX,
                msgids::ISSUES_SUFFIX_PLURAL,
                count as u32,
            ),
            &[("m", &count.to_string())],
        )
    }

    /// The Status column's text: the flagged types' words, comma-joined, in
    /// the order the rulebook hands them over — most severe first (spec §7,
    /// FR-diag-status).
    ///
    /// A healthy Entry gets the empty string, and that empty column is the
    /// whole of the healthy state: never "OK", never a severity prefix, never
    /// an icon. NVDA then reads "{path}; Status: {types}" on a flagged row and
    /// the path alone on a clean one, for free, on every arrow key — which is
    /// why nothing is added here that a listener would have to hear past.
    pub fn status_column(&self, issues: &[Issue]) -> String {
        issues
            .iter()
            .map(|issue| self.lookup.translate(issue.catalogue_msgid()))
            .collect::<Vec<String>>()
            .join(", ")
    }

    /// Announcement 1: the Scope's entry count, with the zero case as its own
    /// msgid — "no entries" is better speech than "0", and Ukrainian's three
    /// plural forms have no zero form to give it (spec §10.1 item 1).
    fn entry_count(&self, scope: Scope, count: usize) -> String {
        let (none, singular, plural) = match scope {
            Scope::User => (
                msgids::ENTRIES_USER_NONE,
                msgids::ENTRIES_USER,
                msgids::ENTRIES_USER_PLURAL,
            ),
            Scope::System => (
                msgids::ENTRIES_SYSTEM_NONE,
                msgids::ENTRIES_SYSTEM,
                msgids::ENTRIES_SYSTEM_PLURAL,
            ),
        };
        if count == 0 {
            self.lookup.translate(none)
        } else {
            fill(
                &self.lookup.translate_plural(singular, plural, count as u32),
                &[("n", &count.to_string())],
            )
        }
    }

    /// Announcements 4 and 5: what was undone or redone, and — when the step
    /// took the Working Copy back across an Apply — that there are unsaved
    /// changes again (spec §10.1 items 4 and 5).
    ///
    /// The operation name is the one thing focus landing on a row cannot say,
    /// and it is Catalogue text of its own: translated first, then filled in,
    /// so the Ukrainian composes («Скасовано: додавання запису»).
    fn undo_redo(&self, step: UndoStep, outcome: UndoOutcome) -> String {
        let template = self.lookup.translate(match step {
            UndoStep::Undone => msgids::UNDONE,
            UndoStep::Redone => msgids::REDONE,
        });
        let operation = self.lookup.translate(outcome.operation.catalogue_msgid());
        let mut text = fill(&template, &[("operation", &operation)]);
        if outcome.crossed_apply {
            text.push_str(&self.lookup.translate(msgids::UNSAVED_CHANGES_SUFFIX));
        }
        text
    }

    /// Announcement 7, which is also StatusBar field 0 in a Read-only Data run
    /// — the mode and its reason, both halves Catalogue text (spec §10.1
    /// item 7, §12).
    fn read_only(&self, reason: &str) -> String {
        fill(
            &self.lookup.translate(msgids::READONLY),
            &[("reason", &self.lookup.translate(reason))],
        )
    }
}
