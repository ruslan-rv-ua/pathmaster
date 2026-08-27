//! Fix Issues' rulebook: which Entries the repair surface offers, what it
//! proposes to do to each, which of them start checked, and what carrying the
//! chosen ones out does to a Session (v0.2.0 spec §7, `CONTEXT.md`
//! **Fix Issues**).
//!
//! A **Fix Issues** is a modal, per-Scope repair surface. What lives here is
//! everything about it a test can hold: the dialog's own behaviour is the
//! window's, and the words every cell shows are the Catalogue's
//! ([`Catalogue::status_column`] for the Issue column,
//! [`Action::catalogue_msgid`] for the Action one). This module is pure, takes
//! no Expansion Mode, and cannot be given one — which is the whole of "the Path
//! column is always the raw text, whatever the mode shows".
//!
//! The rules, each of which a test fixes:
//!
//! * **Fixable is three deletions and one repair.** Missing, Duplicate and
//!   Empty propose deleting the Entry; Quoted proposes removing its quotes.
//!   **Relative proposes nothing** — qualifying a path needs a base directory
//!   only the user knows — so an Entry flagged only Relative earns no row at
//!   all, and neither does a healthy one. Over-length never reaches here: it
//!   flags a Scope rather than an Entry ([`crate::thresholds`]) and has its own
//!   surface.
//! * **One row per Entry, one computed action.** The action is a deletion when
//!   any of the three deletion types is flagged — deleting cures Quoted too —
//!   and removing the quotes otherwise. A row never does both.
//! * **The defaults are Disk Cleanup's principle**: what is checked on arrival
//!   is what is safe to do without looking. Removing quotes is
//!   behaviour-preserving by construction, and a Duplicate or an Empty Entry is
//!   redundant whatever the disk says, so all three arrive checked. A deletion
//!   the *Missing* flag alone earns is the one that arrives unchecked wherever
//!   its absence is not evidence: a `%VAR%` this run does not define, or a root
//!   that is not a fixed disk. Network roots are never probed and never flag
//!   Missing (spec §7, FR-diag-missing), so no row of theirs is ever this one.
//! * **Carrying the chosen rows out is one Checkpoint**, however many Entries
//!   it touched, and it resolves them **by identity** ([`repair`]).
//!
//! [`Catalogue::status_column`]: crate::catalogue::Catalogue::status_column

use crate::diagnostics::Issue;
use crate::msgids;
use crate::normalize::{has_variable_reference, strip_quotes};
use crate::session::{EntryId, Operation, Session};

/// The one thing a row proposes to do to its Entry — `CONTEXT.md`'s "one
/// proposed action", and what the Action column names.
///
/// Two, and closed: the fixable Issue types are four, and the three that say
/// the Entry has no business in a `PATH` all cure the same way. A third action
/// would need a fifth fixable type, which spec §7 does not have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Delete the Entry. What Missing, Duplicate and Empty all propose — and
    /// what cures a Quoted Entry that is also one of them.
    Delete,
    /// Remove **every** `"` in the Entry. `"` is illegal in a Windows file
    /// name, so no quote anywhere in the text can be path content — which is
    /// what makes this the one repair guaranteed to preserve behaviour, and so
    /// the one §7 lets a tool propose without the user reading the path first.
    RemoveQuotes,
}

impl Action {
    /// What a flagged set proposes, or `None` for an Entry Fix Issues takes no
    /// part in — a healthy one, or one flagged only Relative.
    ///
    /// The single home of "is this Entry fixable?": the menu item's enablement
    /// counts what this answers ([`fixable`]) and the dialog shows what it
    /// answers, so a row can never exist that the menu says is not there.
    pub fn proposed(issues: &[Issue]) -> Option<Action> {
        if issues.iter().any(Self::deletes) {
            Some(Action::Delete)
        } else if issues.contains(&Issue::Quoted) {
            Some(Action::RemoveQuotes)
        } else {
            None
        }
    }

    /// The Catalogue string the Action column shows for this action (v0.2.0
    /// §14).
    ///
    /// The deletion **reuses Announcement 4's operation msgid**: same meaning,
    /// same English, so one operation has one translation (ADR-0004).
    pub fn catalogue_msgid(self) -> &'static str {
        match self {
            Action::Delete => msgids::OPERATION_DELETE,
            Action::RemoveQuotes => msgids::FIX_REMOVE_QUOTES,
        }
    }

    /// The Entry text this action leaves behind — `None` where it leaves none,
    /// which is what a deletion is.
    ///
    /// Carrying a row out is exactly this answer handed to the Session, so the
    /// difference between the two actions is stated once rather than at every
    /// call site.
    pub fn leaves(self, raw: &str) -> Option<String> {
        match self {
            Action::Delete => None,
            Action::RemoveQuotes => Some(raw.replace('"', "")),
        }
    }

    /// Whether this Issue proposes deleting the Entry: the two the text alone
    /// decides, and Missing, which only a probe can.
    fn deletes(issue: &Issue) -> bool {
        Self::decided_by_text(issue) || matches!(issue, Issue::Missing)
    }

    /// Whether this Issue was decided from the Entry's **text alone**, as
    /// Duplicate and Empty are and Missing is not.
    ///
    /// One cascade, read by both rules that need it: it is half of what makes a
    /// deletion proposed at all, and — because a finding no filesystem made
    /// cannot be wrong for want of a disk — it is also what takes a row past
    /// the cautious default in [`checked_on_arrival`].
    fn decided_by_text(issue: &Issue) -> bool {
        matches!(issue, Issue::Duplicate | Issue::Empty)
    }
}

/// The one machine fact the defaults ask for, injected like every other
/// (spec §7's `DriveType=Fixed`).
///
/// It is deliberately **not** a third question on
/// [`Filesystem`](crate::diagnostics::Filesystem): the diagnostic rulebook does
/// not care whether a disk is fixed — it asks only whether a root may be probed
/// — and a trait grows a method for the rule that reads it, not for the machine
/// that could answer it.
pub trait DriveTypes {
    /// Whether `path`'s root is a fixed local disk: the machine's own storage,
    /// which is always there, so a path missing from it is stale rather than
    /// merely absent.
    ///
    /// Everything else answers `false` — removable media, an optical drive, a
    /// network root, and text with no root at all — because what they share is
    /// that not finding a path under them proves nothing about the Entry.
    fn is_fixed_root(&self, path: &str) -> bool;
}

/// One row of the dialog: one Entry, one action, and the state its checkbox
/// arrives in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// The Entry this row acts on, by identity. The point of the row: carrying
    /// it out resolves back to the Entry through this and never through a
    /// position, which is the one thing that survives a duplicate.
    pub id: EntryId,
    /// The Entry's 1-based position in the Working Copy — §2.1's `#`, the same
    /// number the main list's own column carries, and never this row's place
    /// among its siblings.
    pub position: usize,
    /// The Entry exactly as stored: what the Path column shows, whatever the
    /// Expansion Mode, because it is what will be deleted or repaired.
    pub raw: String,
    /// What the last completed pass found about this Entry, most-severe-first
    /// — what the Issue column joins, in the Status column's own words.
    pub issues: Vec<Issue>,
    pub action: Action,
    /// Whether the checkbox arrives checked. A default and nothing more: the
    /// user's own answer is read back off the native list when the dialog
    /// closes.
    pub checked: bool,
}

/// One Scope's fixable Entries, in Working-Copy order.
///
/// Built once and never updated: the dialog it feeds is modal, and modality is
/// what keeps the Working Copy still underneath it (v0.2.0 §7).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Plan {
    rows: Vec<Row>,
}

impl Plan {
    /// Plans over one Scope's **whole Working Copy, in its own order** — the
    /// contract, not a convenience: each Entry's `#` is read off its place in
    /// what is handed over, so a caller passing a subset would label every row
    /// with a position the main list does not agree with.
    ///
    /// It is the whole Working Copy rather than the Filtered View for the same
    /// reason: the surface is per Scope (`CONTEXT.md`), and a repair pass
    /// narrowed by what the user happened to be searching for would leave half
    /// the work undone and out of sight.
    ///
    /// Each Entry arrives as the three things a row needs: its identity, its
    /// raw text, and what the last completed pass found about it. The drives
    /// are injected like the environment is elsewhere — core takes no OS call.
    pub fn of<'a>(
        entries: impl IntoIterator<Item = (EntryId, &'a str, &'a [Issue])>,
        drives: &dyn DriveTypes,
    ) -> Plan {
        Plan {
            rows: entries
                .into_iter()
                .enumerate()
                .filter_map(|(index, (id, raw, issues))| {
                    let action = Action::proposed(issues)?;
                    Some(Row {
                        id,
                        position: index + 1,
                        raw: raw.to_string(),
                        issues: issues.to_vec(),
                        action,
                        checked: checked_on_arrival(action, issues, raw, drives),
                    })
                })
                .collect(),
        }
    }

    /// Every row, in the order the dialog lists them.
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// Whether this Scope has nothing for the surface to do — the state the
    /// menu item's enablement exists to keep the user from ever reaching.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// How many of these Entries Fix Issues can act on: the menu item's whole
/// enablement question (v0.2.0 §7).
///
/// Counted from the flagged sets alone, without the drives: what a row's
/// checkbox *starts* as changes nothing about whether the row exists, and the
/// menu is re-synced after every operation — a mount-table lookup per Missing
/// Entry on each of those would be work spent to answer a question that does
/// not depend on it.
pub fn fixable<'a>(issues: impl IntoIterator<Item = &'a [Issue]>) -> usize {
    issues
        .into_iter()
        .filter(|flagged| Action::proposed(flagged).is_some())
        .count()
}

/// Carries the chosen rows out on `session` — **one Checkpoint**, however many
/// Entries they touched — and answers how many Entries actually changed, which
/// is the number Announcement 12 speaks (v0.2.0 §7, §13 item 12).
///
/// One Checkpoint is the point: the batch is what makes a single Ctrl+Z restore
/// every Entry the dialog repaired, and it is why the operation has a name of
/// its own for the undo to speak.
///
/// **By identity, never by position.** Each row names an Entry, and the text
/// each action is computed over is the Working Copy's own — not the text the
/// plan remembered — so an Entry an earlier row's deletion has taken away is
/// passed over rather than counted, and none of the deletions can shift a later
/// row onto the wrong Entry. Zero changed is not an operation and leaves no
/// Checkpoint, which is also what a non-writable Session answers.
///
/// The Checkpoint's focus hint is **the first surviving neighbour**: the Entry
/// left standing where the first repaired row stood, clamped to the last row —
/// Delete's own law, asked of the whole batch rather than of one Entry. It is
/// what an undo of "Fixing issues" lands on, and it is `None` only over a Scope
/// the repair emptied, where there is no row to land on at all.
pub fn repair(session: &mut Session, chosen: &[(EntryId, Action)]) -> usize {
    let mut fixed = 0;
    session.batch(Operation::FixIssues, |working| {
        // Read before anything moves: the place the first repaired row stood
        // in is the place the hint is read back out of afterwards.
        let first = chosen.first().and_then(|(id, _)| position_of(working, *id));
        for (id, action) in chosen {
            let Some(raw) = raw_of(working, *id) else {
                continue;
            };
            let done = match action.leaves(&raw) {
                Some(repaired) => working.edit(*id, repaired),
                None => working.delete(*id),
            };
            fixed += usize::from(done);
        }
        surviving_neighbour(working, first?)
    });
    fixed
}

/// The Entry now standing at `index`, clamped to the last row — `None` only
/// over an emptied Working Copy.
fn surviving_neighbour(session: &Session, index: usize) -> Option<EntryId> {
    let last = session.entries().len().checked_sub(1)?;
    Some(session.entries()[index.min(last)].id())
}

/// Where the Entry `id` names now stands, or `None` for one this Session no
/// longer holds.
fn position_of(session: &Session, id: EntryId) -> Option<usize> {
    session.entries().iter().position(|entry| entry.id() == id)
}

/// The raw text of the Entry `id` names, or `None` for one this Session no
/// longer holds.
fn raw_of(session: &Session, id: EntryId) -> Option<String> {
    session
        .entries()
        .iter()
        .find(|entry| entry.id() == id)
        .map(|entry| entry.raw().to_string())
}

/// The Disk-Cleanup rule: ON for what is safe to do without looking.
fn checked_on_arrival(
    action: Action,
    issues: &[Issue],
    raw: &str,
    drives: &dyn DriveTypes,
) -> bool {
    match action {
        // Behaviour-preserving by construction, whatever the Entry names.
        Action::RemoveQuotes => true,
        // A redundant or unusable Entry is redundant or unusable whether or
        // not the path it spells is on the disk today, so the cautious rule
        // below is about a deletion the *Missing* flag alone earns.
        Action::Delete if issues.iter().any(Action::decided_by_text) => true,
        // Missing alone: checked only where the absence is evidence. A `%VAR%`
        // this run does not define may name a directory another run does — and
        // the raw text is what the row shows, so the rule is visible in the
        // row it judges. A root that is not a fixed disk may simply not be in
        // the drive.
        Action::Delete => !has_variable_reference(raw) && drives.is_fixed_root(strip_quotes(raw)),
    }
}
