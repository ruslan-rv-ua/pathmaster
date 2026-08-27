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

use crate::backups::SnapshotFile;
use crate::diagnostics::Issue;
use crate::expansion::Mode;
use crate::filtered::Filter;
use crate::language::LanguageChoice;
use crate::msgids::{self, fill};
use crate::path::Rejection;
use crate::session::{Scope, UndoOutcome};
use crate::thresholds::{self, Overlength};
use crate::tree::Node;

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

/// One of the Announcements: the closed set of messages the application
/// speaks, as a type (spec §10.1; v0.2.0 §13, growing the set toward
/// fourteen as the tickets land their variants).
///
/// Item 5 is item 4's text with the
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
    /// **2** — a Scope's Working Copy reached the registry. §10.1 gives it two
    /// strings, one per Scope, and which is spoken is decided here rather than
    /// by the caller: the Scope is core's own type, so the choice is a rule the
    /// Catalogue can hold.
    Applied { scope: Scope },
    /// **3** — an Apply did not complete: the §9 taxonomy's sentence, naming
    /// why. `cause` is the msgid of the phrase filled into it — the typed
    /// failure lives in `pathmaster-platform`, which core cannot name, so it
    /// contributes its `catalogue_msgid()` exactly as a Read-only reason does.
    ApplyFailed { cause: &'static str },
    /// **4**, and **5** when the restored Checkpoint was taken before the last
    /// Apply: what was undone or redone. No path text — focus lands on the row
    /// and NVDA reads it for free.
    UndoRedo {
        direction: UndoDirection,
        outcome: UndoOutcome,
    },
    /// **6** — the Cancel command discarded a Working Copy back to its
    /// Baseline.
    ChangesDiscarded,
    /// **7** — this run cannot write its Data Directory, and why. `reason` is
    /// the msgid `ReadOnlyReason::catalogue_msgid()` returns: the reason is a
    /// platform type, its name is Catalogue text.
    ReadOnly { reason: &'static str },
    /// **8** (v0.2.0) — Expansion Mode was toggled: which rendering both Scope
    /// lists now show. The toggle is not an edit and leaves no Checkpoint, so
    /// this sentence and the re-rendered rows are the whole of what happened —
    /// and which of its two strings is spoken is decided here, from the
    /// [`Mode`] the Announcement carries, because the mode is core's own type.
    ExpansionMode { mode: Mode },
    /// **9** (v0.2.0) — the filtered count on a view-criteria change: `shown`
    /// visible of `total` in the Scope. Spoken debounced on typing pauses and
    /// never on Working-Copy changes, which recompute membership silently.
    FilteredCount { shown: usize, total: usize },
    /// **10** (v0.2.0) — the Scope-named filtered count, on tab activation and
    /// Refresh while that Scope has a Filtered View. The same composition is
    /// StatusBar field 0's fragment for a narrowed Scope whose Filter is `All`
    /// (v0.2.0 §16). It does **not** name the Filter: an arrival says which
    /// Scope and how much of it, and which state narrowed it is what the
    /// submenu's own radio mark says.
    ScopeFilteredCount {
        scope: Scope,
        shown: usize,
        total: usize,
    },
    /// **11** (v0.2.0) — the Filter was changed to a narrowing state: the
    /// **already-composed** Search∧Filter count, named by the state that
    /// produced it. One announcement and never two — the Filter is a discrete
    /// gesture, so it speaks the whole of what it did rather than a message
    /// and then a debounced count the way the Expansion toggle does.
    FilterCount {
        filter: Filter,
        shown: usize,
        total: usize,
    },
    /// **12** (v0.2.0) — the Fix Issues dialog applied the rows the user had
    /// checked. Spoken **after focus has landed**, so the summary is the last
    /// thing heard: the row focus lands on is what NVDA reads first, and this
    /// says how much of the list it stands for.
    ///
    /// There is no zero: nothing checked is a Cancel, which leaves no
    /// Checkpoint and speaks nothing (v0.2.0 §7).
    FixedEntries { count: usize },
    /// **13** (v0.2.0) — the focused Entry's displayed rendering reached the
    /// clipboard. Fixed text with no placeholder: it never echoes what it
    /// copied, because focus has just read the row and Entries run long
    /// (v0.2.0 §8).
    CopiedToClipboard,
    /// **14** (v0.2.0) — the clipboard write failed, spoken immediately and
    /// with no retry.
    ///
    /// Its own variant rather than a flag on [`CopiedToClipboard`]: these are
    /// two items of the closed set, not one item with a mood, and a `bool` at
    /// the call site would say nothing about which sentence it chose. NVDA
    /// speaks nothing of its own for an application-side copy, so a swallowed
    /// failure is indistinguishable from a missed keystroke.
    CopyFailed,
}

/// Which way the undo history was walked, and so which of Announcement 4's two
/// sentences is spoken.
///
/// It travels as a named value rather than a `bool`, for the reason every
/// command in this application does: a bare `true` at the call site says
/// nothing about which way it went. It is named for the walk and not for the
/// Checkpoint it lands on — CONTEXT.md's **Checkpoint** keeps "undo step" on
/// its `_Avoid_` list, and this is not one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndoDirection {
    Undo,
    Redo,
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
    /// How many Entries the Scope's Filtered View shows — `Some` only while
    /// one is active, and then the fragment reads "{n} of {m}" (v0.2.0 §16).
    /// `None` is no narrowing, which is not the same as `Some(entries)`: a
    /// view that happens to match everything still reads as a view.
    pub visible: Option<usize>,
    /// The Scope's Filter, which the fragment **names** while it is not `All`
    /// (v0.2.0 §16). Read only under a narrowing, and it cannot be otherwise:
    /// a narrowing state is itself a Filtered View, so `All` is the only state
    /// `visible: None` can arrive with.
    pub filter: Filter,
    pub issues: Option<usize>,
}

/// The product's own name, deliberately **outside** the Catalogue: it is what
/// this application is called in every language, and a translated one would
/// name a program the user could not then find in Alt+Tab or the taskbar. It
/// is the same string the exe's `VERSIONINFO` carries as `ProductName`
/// (spec §16), which `tests/versioninfo.rs` reads back off the built binary.
///
/// It lives here rather than at the window because the rule that picks
/// between it and the elevated title is composition, and composition lives
/// beside the msgids it fills (ADR-0009).
const PRODUCT_NAME: &str = "PathMaster";

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
            Announcement::Applied { scope } => self.lookup.translate(match scope {
                Scope::User => msgids::APPLIED_USER,
                Scope::System => msgids::APPLIED_SYSTEM,
            }),
            Announcement::ApplyFailed { cause } => fill(
                &self.lookup.translate(msgids::APPLY_FAILED),
                &[("cause", &self.lookup.translate(cause))],
            ),
            Announcement::UndoRedo { direction, outcome } => self.undo_redo(direction, outcome),
            Announcement::ChangesDiscarded => self.lookup.translate(msgids::CHANGES_DISCARDED),
            Announcement::ReadOnly { reason } => self.read_only(reason),
            Announcement::ExpansionMode { mode } => self.lookup.translate(match mode {
                Mode::Expanded => msgids::SHOWING_EXPANDED_VALUES,
                Mode::Raw => msgids::SHOWING_RAW_VALUES,
            }),
            Announcement::FilteredCount { shown, total } => self.filtered_count(shown, total),
            Announcement::ScopeFilteredCount {
                scope,
                shown,
                total,
            } => self.scope_filtered_count(scope, shown, total),
            Announcement::FilterCount {
                filter,
                shown,
                total,
            } => self.filter_count(filter, shown, total),
            Announcement::FixedEntries { count } => self.fixed_entries(count),
            Announcement::CopiedToClipboard => self.lookup.translate(msgids::COPIED_TO_CLIPBOARD),
            Announcement::CopyFailed => self.lookup.translate(msgids::COPY_FAILED),
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

    /// The close-confirm's title, which is the whole of that dialog: the
    /// Scopes whose Sessions are dirty, named in one sentence (spec §5,
    /// FR-close-confirm; §10's dialog discipline).
    ///
    /// **The order is the caller's**, deliberately unlike
    /// [`general_status`](Self::general_status)'s. That field reads both
    /// Scopes whatever order it is handed, because it is a reading of the two
    /// tabs; this title reads the very list the Apply Run is about to take,
    /// User first. One reading of which Sessions are dirty therefore feeds the
    /// sentence and the sequence alike, and a second ordering rule here could
    /// only promise an order the run does not keep.
    ///
    /// An empty list is not a state this composes for: a run with nothing
    /// dirty closes with no dialog at all.
    pub fn close_confirm_dialog(&self, dirty: &[Scope]) -> String {
        let scopes: Vec<String> = dirty
            .iter()
            .map(|scope| self.lookup.translate(tab_msgid(*scope)))
            .collect();
        fill(
            &self.lookup.translate(msgids::DIALOG_CLOSE_CONFIRM),
            &[("scopes", &scopes.join(", "))],
        )
    }

    /// The Settings dialog's language selector, one item per choice it offers,
    /// in [`LanguageChoice::SELECTABLE`]'s order (spec §11, §13).
    ///
    /// **Only the auto choice is looked up.** It names a rule — follow the
    /// system — rather than a language, so it is Catalogue text like any other
    /// sentence. The languages beside it are their own endonyms and
    /// deliberately outside the Catalogue: a user who cannot read the current
    /// Interface Language must still be able to find theirs, and a translated
    /// "English" would be the one item in this list they could not.
    ///
    /// Composed here rather than in the dialog because the pairing is a rule:
    /// the selector answers by position, so a list of labels that could differ
    /// in length or order from the choices they stand for would be a selector
    /// answering with a language nobody picked.
    pub fn language_items(&self) -> Vec<String> {
        LanguageChoice::SELECTABLE
            .iter()
            .map(|choice| match choice.language() {
                Some(language) => language.endonym().to_owned(),
                None => self
                    .lookup
                    .translate(msgids::SETTINGS_LANGUAGE_FOLLOWS_SYSTEM),
            })
            .collect()
    }

    /// StatusBar field 0, the general status (spec §12): each Scope's entry
    /// count and the issues the last pass found there — or, in a Read-only
    /// Data run, the mode and its reason in their place.
    ///
    /// **User first, then System**, whichever order the two arrive in: this
    /// field is read on demand as one sentence (`NVDA+End`), and the order is
    /// the one the tabs are in — deliberately not the runtime order a pass
    /// evaluates the Scopes in, which is System first and would silently
    /// reverse the sentence if a caller passed a pass's own order. Each
    /// [`ScopeCounts`] carries the Scope its numbers belong to, so ordering
    /// here cannot mispair a count with a name.
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
            None => {
                let mut ordered = scopes;
                ordered.sort_by_key(|counts| match counts.scope {
                    Scope::User => 0,
                    Scope::System => 1,
                });
                ordered
                    .iter()
                    .map(|counts| self.counts(counts))
                    .collect::<Vec<String>>()
                    .join(" | ")
            }
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

    /// The warning an Apply past 8,191 earns, which is the whole of that
    /// dialog — NVDA never speaks a `MessageDialog`'s body (spec §7,
    /// FR-diag-overlength; §10).
    ///
    /// `length` is the merged length **this Apply would leave behind**, which
    /// is what the title says and why it is passed rather than read off a
    /// field. Which of the two titles a length earns is not decided here: the
    /// run has already classified it, and classifying again would be a second
    /// answer to the same question.
    pub fn cmd_limit_dialog(&self, length: usize) -> String {
        self.overlength(msgids::DIALOG_OVER_CMD_LIMIT, length)
    }

    /// The hard cap's dialog at 32,767 — the same shape as
    /// [`cmd_limit_dialog`](Self::cmd_limit_dialog) and a different sentence,
    /// because this one has no way past it.
    pub fn hard_cap_dialog(&self, length: usize) -> String {
        self.overlength(msgids::DIALOG_OVER_HARD_CAP, length)
    }

    /// The main window's title: which of the two instances the user is in
    /// (spec §9, ticket 12 D11).
    ///
    /// Alt+Tab speaks a window's title first, which makes this the cheapest
    /// always-available answer to "am I in the elevated one?" — so the
    /// elevated title is Catalogue text and carries the cmd.exe convention,
    /// while the unelevated one is [`PRODUCT_NAME`], which no language
    /// varies.
    ///
    /// Composed here rather than at the window for the reason
    /// [`language_items`](Self::language_items) is: **choosing between a
    /// translated string and a deliberately untranslated one is a rule**, and
    /// a rule pinned to the wx-linking crate is a rule no test can reach
    /// (ADR-0009). `elevated` is handed in because it is a fact about the
    /// process, which this crate is pure of.
    pub fn window_title(&self, elevated: bool) -> String {
        match elevated {
            true => self.lookup.translate(msgids::WINDOW_TITLE_ELEVATED),
            false => PRODUCT_NAME.to_string(),
        }
    }

    /// View → "PATH Tree…": the title of the modal opened over `scope`
    /// (v0.2.0 §6, §14).
    ///
    /// Two whole strings picked between rather than one frame with a Scope
    /// name dropped in, which is §11's rule and [`entry_count`](Self::entry_count)'s
    /// reason: «PATH користувача» has to agree with the sentence it stands in,
    /// and a frame cannot make it. The rule lives here because choosing
    /// between two msgids by a domain value is exactly what this crate holds
    /// (ADR-0009).
    pub fn tree_title(&self, scope: Scope) -> String {
        self.lookup.translate(match scope {
            Scope::User => msgids::DIALOG_TREE_USER,
            Scope::System => msgids::DIALOG_TREE_SYSTEM,
        })
    }

    /// Edit → "Fix Issues…": the title of the modal opened over `scope`
    /// (v0.2.0 §7, §14) — two whole strings picked between, for
    /// [`tree_title`](Self::tree_title)'s reason and by the same rule.
    pub fn fix_title(&self, scope: Scope) -> String {
        self.lookup.translate(match scope {
            Scope::User => msgids::DIALOG_FIX_USER,
            Scope::System => msgids::DIALOG_FIX_SYSTEM,
        })
    }

    /// Help → About: what this build is, in the one line NVDA speaks of a
    /// dialog (spec §15, §16).
    ///
    /// `version` is handed in rather than read here because this crate is pure
    /// — the version is the *binary's*, taken from Cargo at compile time and
    /// gated against the exe's `VERSIONINFO`. The name and the licence are in
    /// the msgid itself, so this composes exactly one thing and cannot compose
    /// it wrong.
    pub fn about_dialog(&self, version: &str) -> String {
        fill(
            &self.lookup.translate(msgids::DIALOG_ABOUT),
            &[("version", version)],
        )
    }

    /// The dialog an argument this application does not recognise earns
    /// (v0.2.0 §10) — the title only; its body is the usage line, which is one
    /// plain lookup and composes nothing.
    ///
    /// `argument` is the user's own text and is filled in verbatim, like every
    /// other value [`fill`] substitutes: braces or a `%VAR%` inside it cannot
    /// turn into a placeholder, because nothing rescans what was filled in.
    pub fn unknown_argument_dialog(&self, argument: &str) -> String {
        fill(
            &self.lookup.translate(msgids::DIALOG_UNKNOWN_ARGUMENT),
            &[("arg", argument)],
        )
    }

    /// The number both over-length titles name, filled into whichever of them
    /// is being spoken.
    fn overlength(&self, msgid: &str, length: usize) -> String {
        fill(&self.lookup.translate(msgid), &[("n", &length.to_string())])
    }

    /// One Scope's half of StatusBar field 0. A narrowed Scope reads "{n} of
    /// {m}" through Announcement 10's own composition — one wording for one
    /// meaning — while the issue parenthetical never changes: it counts the
    /// Scope's Issues, not the view's (v0.2.0 §16).
    fn counts(&self, counts: &ScopeCounts) -> String {
        let entries = match (counts.visible, counts.filter.narrows()) {
            (Some(shown), true) => {
                self.named_filtered_count(counts.scope, counts.filter, shown, counts.entries)
            }
            (Some(shown), false) => self.scope_filtered_count(counts.scope, shown, counts.entries),
            (None, _) => self.entry_count(counts.scope, counts.entries),
        };
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

    /// Announcement 12: how many Entries the Fix Issues dialog repaired
    /// (v0.2.0 §13 item 12).
    ///
    /// Plural by `{n}`, which is also the number filled in — unlike the
    /// filtered counts, whose two numbers made the choice worth writing down.
    /// No zero case: nothing checked never reaches here.
    fn fixed_entries(&self, count: usize) -> String {
        fill(
            &self.lookup.translate_plural(
                msgids::FIXED_ENTRIES,
                msgids::FIXED_ENTRIES_PLURAL,
                count as u32,
            ),
            &[("n", &count.to_string())],
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

    /// One Tree View node's label — and for a leaf, **the whole audible
    /// payload** (v0.2.0 §6).
    ///
    /// A native tree item has no columns and no description, and NVDA spends
    /// `accValue` on the level, so everything a leaf has to say has to be in
    /// its text. Three parts, in this order and joined here:
    ///
    /// * the segment, or the chain a compression joined;
    /// * **the raw Entry in parentheses, only when it differs** from the
    ///   expansion the tree was built over — a leaf that reads the same both
    ///   ways would otherwise say it twice;
    /// * **the Issue suffix in the exact Status-column words**, only when the
    ///   snapshot found an Issue. It is [`status_column`](Self::status_column)
    ///   itself, so one Issue has one name wherever it is read (ADR-0004).
    ///
    /// **A branch and a group carry no suffix.** Status belongs to the Entry,
    /// not to the prefix it shares or the group it was gathered into — a
    /// parent that repeated its children's findings would be a node the user
    /// has to hear past on the way down.
    pub fn tree_label(&self, node: &Node) -> String {
        let (chain, entry) = match node {
            Node::Branch { chain, .. } => return chain.clone(),
            Node::Group { group, .. } => return self.lookup.translate(group.catalogue_msgid()),
            Node::Leaf { chain, entry } => (chain, entry),
        };
        let mut label = chain.clone();
        if entry.raw != entry.expanded {
            label.push_str(&format!(" ({})", entry.raw));
        }
        let status = self.status_column(&entry.issues);
        if !status.is_empty() {
            label.push_str(" — ");
            label.push_str(&status);
        }
        label
    }

    /// One Snapshot file in the Backups tab's three columns: when it was
    /// taken, the Scope it holds, and how many Entries restoring it would load
    /// (spec §8, FR-backup-ui).
    ///
    /// A Corrupted file's third column is `[Corrupted]` where the count would
    /// stand, because that is the answer to the same question for a file that
    /// cannot be read. It is passive list text and never an Announcement — the
    /// same free ride the Status column already takes, read when the row gets
    /// focus and never spoken unprompted (`CONTEXT.md`, **Corrupted**).
    ///
    /// The Scope is named with its tab's own label: a Scope has one name, and
    /// a second English for it would be a second translation to keep in step
    /// (ADR-0004).
    pub fn snapshot_columns(&self, file: &SnapshotFile) -> [String; 3] {
        [
            file.taken(),
            self.lookup.translate(tab_msgid(file.scope())),
            match file.restores() {
                Some((entries, _)) => entries.len().to_string(),
                None => self.lookup.translate(msgids::SNAPSHOT_CORRUPTED),
            },
        ]
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

    /// Announcement 9 (v0.2.0 §13 item 9): the filtered count. The plural form
    /// is selected by **the total**, `{m}` — the number whose word the
    /// sentence ends on — and zero visible takes its own worded msgid rather
    /// than a count.
    fn filtered_count(&self, shown: usize, total: usize) -> String {
        if shown == 0 {
            return self.lookup.translate(msgids::FILTERED_COUNT_NONE);
        }
        self.shown_of_total(
            msgids::FILTERED_COUNT,
            msgids::FILTERED_COUNT_PLURAL,
            shown,
            total,
        )
    }

    /// Announcement 10 (v0.2.0 §13 item 10), and the narrowed half of
    /// [`counts`](Self::counts): the Scope-named filtered count, whole strings
    /// per Scope like every Scope-named sentence here. Plural by `{m}`; the
    /// zero cases are their own msgids, one per Scope.
    fn scope_filtered_count(&self, scope: Scope, shown: usize, total: usize) -> String {
        let (none, singular, plural) = match scope {
            Scope::User => (
                msgids::FILTERED_USER_NONE,
                msgids::FILTERED_USER,
                msgids::FILTERED_USER_PLURAL,
            ),
            Scope::System => (
                msgids::FILTERED_SYSTEM_NONE,
                msgids::FILTERED_SYSTEM,
                msgids::FILTERED_SYSTEM_PLURAL,
            ),
        };
        if shown == 0 {
            return self.lookup.translate(none);
        }
        self.shown_of_total(singular, plural, shown, total)
    }

    /// Announcement 11 (v0.2.0 §13 item 11): the composed Search∧Filter count,
    /// named by the state that produced it.
    ///
    /// The name is Catalogue text of its own — the Status column's word, for
    /// the five type states — translated first and then filled in, the way an
    /// Apply failure's cause is, so the Ukrainian composes.
    fn filter_count(&self, filter: Filter, shown: usize, total: usize) -> String {
        self.named_count(
            (
                msgids::FILTER_COUNT_NONE,
                msgids::FILTER_COUNT,
                msgids::FILTER_COUNT_PLURAL,
            ),
            filter,
            shown,
            total,
        )
    }

    /// StatusBar field 0's fragment for a Scope whose Filter is not `All`
    /// (v0.2.0 §16): the state named inside the Scope's own sentence.
    ///
    /// Whole strings per Scope like [`scope_filtered_count`], and a separate
    /// set rather than a wrapper around it: the name lands **between** the
    /// Scope and the count, where no prefix or suffix can put it.
    ///
    /// [`scope_filtered_count`]: Self::scope_filtered_count
    fn named_filtered_count(
        &self,
        scope: Scope,
        filter: Filter,
        shown: usize,
        total: usize,
    ) -> String {
        let forms = match scope {
            Scope::User => (
                msgids::FILTERED_USER_NAMED_NONE,
                msgids::FILTERED_USER_NAMED,
                msgids::FILTERED_USER_NAMED_PLURAL,
            ),
            Scope::System => (
                msgids::FILTERED_SYSTEM_NAMED_NONE,
                msgids::FILTERED_SYSTEM_NAMED,
                msgids::FILTERED_SYSTEM_NAMED_PLURAL,
            ),
        };
        self.named_count(forms, filter, shown, total)
    }

    /// The one composition every Filter-naming count shares: the state's name
    /// filled into whichever of its `(none, singular, plural)` forms the count
    /// earns.
    ///
    /// The name is Catalogue text of its own — the Status column's word, for
    /// the five type states — translated first and then filled in, the way an
    /// Apply failure's cause is, so the Ukrainian composes. The zero case has
    /// no numbers to fill and still names the state: "which filter found
    /// nothing" is the whole of what comes back from it.
    fn named_count(
        &self,
        (none, singular, plural): (&str, &str, &str),
        filter: Filter,
        shown: usize,
        total: usize,
    ) -> String {
        let name = self.lookup.translate(filter.catalogue_msgid());
        if shown == 0 {
            return fill(&self.lookup.translate(none), &[("filter", &name)]);
        }
        fill(
            &self.lookup.translate_plural(singular, plural, total as u32),
            &[
                ("filter", &name),
                ("n", &shown.to_string()),
                ("m", &total.to_string()),
            ],
        )
    }

    /// The "{n} of {m}" frame both unnamed filtered counts fill: one lookup
    /// whose plural form `{m}` selects, then both numbers filled in.
    fn shown_of_total(&self, singular: &str, plural: &str, shown: usize, total: usize) -> String {
        fill(
            &self.lookup.translate_plural(singular, plural, total as u32),
            &[("n", &shown.to_string()), ("m", &total.to_string())],
        )
    }

    /// Announcements 4 and 5: what was undone or redone, and — when the step
    /// took the Working Copy back across an Apply — that there are unsaved
    /// changes again (spec §10.1 items 4 and 5).
    ///
    /// The operation name is the one thing focus landing on a row cannot say,
    /// and it is Catalogue text of its own: translated first, then filled in,
    /// so the Ukrainian composes («Скасовано: додавання запису»).
    fn undo_redo(&self, direction: UndoDirection, outcome: UndoOutcome) -> String {
        // The direction is Undo or Redo; the sentence it earns is "Undone" or
        // "Redone" — a verbal noun follows in the Ukrainian, which is why the
        // two are separate msgids rather than one with a filled-in word.
        let template = self.lookup.translate(match direction {
            UndoDirection::Undo => msgids::UNDONE,
            UndoDirection::Redo => msgids::REDONE,
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

/// A Scope's one name: the label its own tab already carries (ADR-0004). Both
/// surfaces that name a Scope in prose — the Backups list's Scope column and
/// the close-confirm's title — ask here, so a second English for one Scope
/// would be a second translation to keep in step.
fn tab_msgid(scope: Scope) -> &'static str {
    match scope {
        Scope::User => msgids::TAB_USER,
        Scope::System => msgids::TAB_SYSTEM,
    }
}
