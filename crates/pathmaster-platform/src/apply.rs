//! The Apply Run: the one sequence that changes the machine (spec §5
//! FR-apply, §4, §7, §8, §9; ADR-0008).
//!
//! Almost everything Apply must get right is **ordering** — back up the value
//! that was just re-read and never the Baseline, let no failure move the
//! Baseline, rotate after the write and not before — and ordering is exactly
//! what a method on the window could not have a test for. So the sequence
//! lives here, where the live-registry tests, `tempfile` and the deny-ACL'd
//! directories already are, and the window drives it (ADR-0008).
//!
//! **It never holds an Editing Session.** Working Copies arrive by value and
//! an [`Outcome`] goes back for the window to apply. The taxonomy's first
//! invariant — no failure moves the Baseline — therefore stops being a rule to
//! obey and becomes one this module has no means to break: it is handed no
//! Baseline to move. The second reason is sharper still: a Session is reached
//! through an `Rc<RefCell<…>>`, the diagnostic Timer ticks inside a modal
//! dialog's own event loop, and a sequence holding `&mut Session` across the
//! external-change dialog would meet the pass's own borrow and panic.
//!
//! **The user is asked through [`Ask`], not by returning control.** Three
//! questions arrive mid-sequence and this crate has no wx: the window's
//! dialogs answer them in the application, scripted answers in the tests.
//!
//! **The clock and the Data Directory are parameters.** A Snapshot name's
//! collision suffix depends on what its second already holds, so a test that
//! cannot fix the clock cannot reach the rule that a freed suffix is never
//! reissued — the rule that stops the rotation after an Apply from deleting
//! the backup that Apply just took. And the directory arrives as a path rather
//! than as a `Writable` state, because startup predicts and Apply verifies
//! (ADR-0002): this sequence never asks whether it may write, it writes and
//! reads the answer.

use std::io;
use std::path::Path;

use pathmaster_core::logfmt::{ApplyStep, FailureCause, Record, Timestamp};
use pathmaster_core::msgids;
use pathmaster_core::normalize::Environment;
use pathmaster_core::path::{join, split};
use pathmaster_core::session::{Scope, ScopeValue, ValueType};
use pathmaster_core::snapshot::{Captured, Snapshot, SnapshotName};
use pathmaster_core::thresholds::{self, Overlength};
use windows_sys::Win32::Foundation::ERROR_ACCESS_DENIED;

use crate::broadcast;
use crate::registry::{RawValue, RegistryError, ScopeKey};
use crate::snapshots;

/// One Scope, as a run is handed it: which value to write where, and what that
/// value looked like when it was last read.
#[derive(Debug, Clone)]
pub struct ScopeInput {
    pub scope: Scope,
    /// Where the value lives. A constructor parameter all the way down, so a
    /// test aims the whole sequence at a temporary key.
    pub key: ScopeKey,
    /// The Working Copy's Entries, raw and in list order.
    pub entries: Vec<String>,
    /// The Working Copy's Value Type, written back with it and never changed
    /// here (spec §4).
    pub value_type: ValueType,
    /// What the registry held the last time this Scope was read — at startup,
    /// at a Refresh, or at the previous Apply.
    ///
    /// Comparing the re-read against **this** is the whole of external-change
    /// detection (spec §4). It is a `RawValue` and not a decoded one because
    /// decoding stops at the first NUL: a change past one would be invisible.
    pub last_read: RawValue,
}

/// One Apply Run, as it is handed over: what to write, where the files go,
/// and the two facts a test must be able to fix.
///
/// It is `ApplyRun` and not `Run` because `CONTEXT.md` gives those two words to
/// different things — a **Run** is one execution of the application, and this
/// is one pass of the Apply sequence over one or more Scopes. The window holds
/// both, and one of them being called `Run` there would be the collision
/// [ADR-0010](../../../docs/adr/0010-run-properties-decided-in-one-place.md)
/// exists to prevent.
///
/// Both Scopes arrive however few are being applied, because the merged length
/// is a fact about the pair (spec §7) and the gate has to know it before any
/// Scope is touched.
pub struct ApplyRun<'a> {
    pub scopes: [ScopeInput; 2],
    /// The Scopes to apply, in the order the run takes them, stopping at the
    /// first that does not complete. Ctrl+S is a run of one; the close-confirm's
    /// Save is one over every dirty Scope, User first (spec §5).
    pub order: &'a [Scope],
    /// `DataDirState::dir()` — never a path only obtainable by matching
    /// `Writable`.
    pub data_dir: &'a Path,
    /// Where the broadcast's own `WARN` goes, or `None` in a run without a
    /// log. Nothing else here writes to it: the rest of what a run logs is
    /// returned in the [`Outcome`].
    pub log_path: Option<&'a Path>,
    /// The instant the run happens, which is the Snapshot's name and its
    /// `timestamp` field alike.
    pub at: Timestamp,
    /// The backup budget, read from the `SettingsFile` the window holds at the
    /// moment the run starts — deliberately **not** a property of the Run, as
    /// `maxBackups` changes while the application is running (ADR-0010).
    pub max_backups: u32,
}

/// The three questions a run asks mid-sequence, and the seam that lets it ask
/// them from a crate with no wx (ADR-0008).
///
/// Two adapters justify the port and there will be no third: the window's
/// dialogs, and scripted answers in the tests.
pub trait Ask {
    /// The value moved under the Session between the last read and now
    /// (spec §5, FR-apply). All three answers are legal.
    fn external_change(&self, scope: Scope) -> ExternalChange;

    /// Past 8,191 the merged `PATH` is one `cmd.exe` ignores entirely — a real
    /// consequence, and a legal thing to choose knowingly (spec §7). `true`
    /// proceeds.
    fn cmd_limit(&self, length: usize) -> bool;

    /// At 32,767 the value cannot be materialised into any process environment
    /// at all. **It answers nothing**, because there is nothing to answer: the
    /// dialog's only button is Cancel, and a hard cap that could return `true`
    /// would be a rule somebody has to remember rather than one the signature
    /// keeps (spec §7).
    fn hard_cap(&self, length: usize);
}

/// What the user chose when told the value had moved (spec §5, FR-apply).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalChange {
    /// Write anyway, over what was just found. The Undo stack survives.
    Overwrite,
    /// Adopt what was just found: Working Copy and Baseline both become it,
    /// the stacks clear, nothing is written and no backup is taken. The Scope
    /// **completes** — it simply was not applied (`CONTEXT.md`, Apply Run).
    RefreshAndDiscard,
    /// Nothing happens. The Session stays dirty and knowingly stale, and the
    /// run stops exactly as a failure would.
    Cancel,
}

/// Why a Scope was not applied — §9's taxonomy, as a type.
///
/// Each variant knows two things and no more: the Catalogue phrase the user
/// hears, and the raw code the log records. It carries no message of its own,
/// which is what keeps a `Display` string from reaching either.
#[derive(Debug)]
pub enum Failure {
    /// The re-read that opens the sequence failed (§9's fifth row). It takes
    /// the registry-write row's own cause, because the user's truth is the
    /// same either way — nothing was written.
    ReRead(RegistryError),
    /// The Snapshot of what was just re-read could not be written, so the
    /// registry was never touched.
    Snapshot(io::Error),
    /// The registry write failed.
    Write(RegistryError),
}

impl Failure {
    /// The msgid Announcement 3 fills its `{cause}` with (spec §9, §10.1
    /// item 3). A platform type may not appear in an Announcement, so it
    /// contributes this instead (ADR-0009).
    pub fn catalogue_msgid(&self) -> &'static str {
        match self {
            Failure::Snapshot(_) => msgids::APPLY_FAILED_BACKUP,
            Failure::ReRead(error) | Failure::Write(error) => match error {
                RegistryError::Io(error) if is_access_denied(error) => {
                    msgids::APPLY_FAILED_ACCESS_DENIED
                }
                _ => msgids::APPLY_FAILED_REGISTRY,
            },
        }
    }

    /// Which step of the fixed order this was.
    fn step(&self) -> ApplyStep {
        match self {
            Failure::ReRead(_) => ApplyStep::ReRead,
            Failure::Snapshot(_) => ApplyStep::Snapshot,
            Failure::Write(_) => ApplyStep::Write,
        }
    }

    /// The raw code the one log record carries (spec §9's invariant).
    fn log_cause(&self) -> FailureCause {
        match self {
            Failure::ReRead(error) | Failure::Write(error) => error.log_cause(),
            Failure::Snapshot(error) => FailureCause::Io {
                os_error: error.raw_os_error(),
            },
        }
    }
}

/// What one Scope's pass through the sequence did.
#[derive(Debug)]
pub enum ScopeOutcome {
    /// Written. The Baseline moves onto the Working Copy, and `stored` is what
    /// the registry now holds — the value the *next* run compares against.
    Applied { stored: RawValue },
    /// The external-change dialog's middle answer: Working Copy and Baseline
    /// both become `found`, the stacks clear, and nothing was written.
    Refreshed { found: RawValue },
    /// The user stopped it — an over-length gate, or the external-change
    /// dialog's [Cancel]. Nothing happened and the Session stays dirty.
    Cancelled,
    /// §9's taxonomy: what the user is told, and what the log records.
    Failed(Failure),
}

impl ScopeOutcome {
    /// Whether this Scope completed. A Scope a run completes is not
    /// necessarily one it applied: the external-change dialog's middle answer
    /// adopts the value that was just read and writes nothing (`CONTEXT.md`).
    pub fn completed(&self) -> bool {
        matches!(
            self,
            ScopeOutcome::Applied { .. } | ScopeOutcome::Refreshed { .. }
        )
    }
}

/// What a run did, for the window to apply afterwards.
#[derive(Debug)]
pub struct Outcome {
    /// One per Scope the run reached, in the order it reached them. The last
    /// is the one that stopped the run, when one did.
    pub scopes: Vec<(Scope, ScopeOutcome)>,
    /// The records the run earned, in order. **Returned rather than written**,
    /// so what a run logs is assertable without a filesystem (ADR-0008); the
    /// one exception is the broadcast's `WARN`, which cannot ride an outcome
    /// that has already returned and appends itself (spec §4).
    pub records: Vec<Record>,
}

impl Outcome {
    /// Whether every Scope the run reached completed. The close-confirm's
    /// "partial failure aborts the close" is this question (spec §5,
    /// FR-close-confirm), and a user's [Cancel] answers it exactly as a
    /// failure does: the window stays open either way.
    pub fn completed(&self) -> bool {
        self.scopes.iter().all(|(_, outcome)| outcome.completed())
    }
}

/// Runs the Apply sequence over `run.order`, in that order, stopping at the
/// first Scope that does not complete.
///
/// The fixed order, per Scope (spec §5, FR-apply): **re-read → compare
/// `(vtype, bytes)` → (external-change dialog) → back up the re-read value,
/// never the Baseline → write → rotate**. Detection lives only here — there is
/// no watcher and no polling. What the window does afterwards, from the
/// [`Outcome`], is the rest of the order: move the Baseline, re-run
/// diagnostics, announce.
///
/// The over-length gates run **once, before any Scope is touched**. The merged
/// length is a fact about both Working Copies, not about the Scope being
/// written, so asking twice in a two-Scope run would be asking twice about one
/// number; and a gate that opened after the first Scope had already been
/// written would be a warning about something that had happened.
pub fn apply(run: ApplyRun<'_>, env: &dyn Environment, ask: &dyn Ask) -> Outcome {
    let mut outcome = Outcome {
        scopes: Vec::new(),
        records: Vec::new(),
    };
    let Some(first) = run.order.first() else {
        return outcome;
    };
    if !gate(&run, env, ask) {
        // The run stops at its first Scope, exactly as that Scope's own
        // [Cancel] would stop it.
        outcome.scopes.push((*first, ScopeOutcome::Cancelled));
        return outcome;
    }

    let backups = snapshots::dir(run.data_dir);
    let mut wrote = false;
    for scope in run.order {
        let input = input_for(&run.scopes, *scope);
        let done = one_scope(input, &backups, &run, ask, &mut outcome.records);
        wrote |= matches!(done, ScopeOutcome::Applied { .. });
        let completed = done.completed();
        outcome.scopes.push((*scope, done));
        if !completed {
            break;
        }
    }
    // One broadcast per run that wrote anything: the `lParam` names the
    // environment block rather than a variable, so two Scopes are still one
    // change (spec §4). Its handle is dropped, which is what "off the UI
    // thread" is worth — the run returns while the call is still blocking.
    if wrote {
        drop(broadcast::environment_changed(
            run.log_path.map(Path::to_path_buf),
        ));
    }
    outcome
}

/// The over-length gates (spec §7, FR-diag-overlength). `false` stops the run.
///
/// The length is computed here rather than read off the last `Diagnosis`: that
/// one lags by a Timer tick, and the number in the dialog is the one the user
/// is being asked to accept. Both readings go through the same formula, which
/// is why the formula is in `pathmaster-core` and not at either of them.
fn gate(run: &ApplyRun<'_>, env: &dyn Environment, ask: &dyn Ask) -> bool {
    let system = &input_for(&run.scopes, Scope::System).entries;
    let user = &input_for(&run.scopes, Scope::User).entries;
    let length = thresholds::merged_length(system, user, env);
    match thresholds::classify(length) {
        Overlength::Within => true,
        Overlength::CmdLimit => ask.cmd_limit(length),
        Overlength::HardCap => {
            ask.hard_cap(length);
            false
        }
    }
}

/// One Scope through the fixed order.
fn one_scope(
    input: &ScopeInput,
    backups: &Path,
    run: &ApplyRun<'_>,
    ask: &dyn Ask,
    records: &mut Vec<Record>,
) -> ScopeOutcome {
    // 1. Re-read. A failure here is §9's fifth row: the comparison cannot be
    //    made, and proceeding without it is precisely the case where an
    //    external change is overwritten with no dialog.
    let found = match input.key.read() {
        Ok(found) => found,
        Err(error) => return failed(input.scope, Failure::ReRead(error), records),
    };

    // 2. Compare `(vtype, bytes)` — never the key's timestamp (spec §4).
    if found != input.last_read {
        match ask.external_change(input.scope) {
            ExternalChange::Cancel => return ScopeOutcome::Cancelled,
            ExternalChange::RefreshAndDiscard => return ScopeOutcome::Refreshed { found },
            ExternalChange::Overwrite => {}
        }
    }

    // 3. Back up **what was just re-read**, never the Baseline — the Baseline
    //    is what the Session thought it had, and after an external edit that
    //    is precisely not what is about to be overwritten (spec §8).
    //
    //    One listing answers both questions the write asks of the directory:
    //    what this Snapshot is called, and — with its own name added — which
    //    files no longer fit the budget.
    let listing = match snapshots::listing(backups) {
        Ok(listing) => listing,
        Err(error) => return failed(input.scope, Failure::Snapshot(error), records),
    };
    let name = SnapshotName::next(run.at, input.scope, &listing);
    let snapshot = Snapshot::under(&name, captured(&found));
    if let Err(error) = snapshots::write(backups, &name, &snapshot) {
        return failed(input.scope, Failure::Snapshot(error), records);
    }

    // 4. Write, raw and typed, through the adapter. An Absent Scope's Working
    //    Copy is typed `REG_EXPAND_SZ` at load, so a first Apply creates it as
    //    that; zero Entries join to an empty string, which is written and
    //    never deleted (spec §4).
    let value = join(&input.entries);
    if let Err(error) = input.key.write(input.value_type, &value) {
        return failed(input.scope, Failure::Write(error), records);
    }
    records.push(Record::apply_written(
        input.scope,
        input.entries.len(),
        // UTF-16 code units, the unit Windows stores a value in and the unit
        // every other length in this application is counted in.
        value.encode_utf16().count(),
        input.value_type,
    ));

    // 5. Rotate, per-Scope and **after** the write — with the Snapshot just
    //    taken in the listing, which is what stops the rotation from deleting
    //    the backup this Apply took (spec §8).
    let mut after = listing;
    after.push(name);
    snapshots::rotate(backups, &after, input.scope, run.max_backups);

    ScopeOutcome::Applied {
        stored: RawValue::written(input.value_type, &value),
    }
}

/// One failure: one log record with the raw error code, and the outcome that
/// carries the phrase the user hears (spec §9's invariants).
fn failed(scope: Scope, failure: Failure, records: &mut Vec<Record>) -> ScopeOutcome {
    records.push(Record::apply_failed(
        scope,
        failure.step(),
        failure.log_cause(),
    ));
    ScopeOutcome::Failed(failure)
}

/// The Snapshot's content: the value **decoded**, because a Snapshot records
/// Entries and a Value Type rather than bytes (ADR-0006). An Absent Scope has
/// no value and so no Value Type, which is the schema's other shape.
fn captured(found: &RawValue) -> Captured {
    match found.decode() {
        ScopeValue::Absent => Captured::Absent,
        ScopeValue::Present { value_type, raw } => Captured::Present {
            value_type,
            entries: split(&raw).into_iter().map(str::to_owned).collect(),
        },
    }
}

/// The input for one Scope. Both Scopes are always present, so this is total —
/// and it is by Scope rather than by array position, so no caller can hand the
/// run one Scope's Entries under the other's name.
fn input_for(scopes: &[ScopeInput; 2], scope: Scope) -> &ScopeInput {
    scopes
        .iter()
        .find(|input| input.scope == scope)
        .expect("a run is handed both Scopes")
}

fn is_access_denied(error: &io::Error) -> bool {
    error.raw_os_error() == Some(ERROR_ACCESS_DENIED as i32)
}
