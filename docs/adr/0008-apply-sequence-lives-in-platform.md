# The Apply sequence lives in `pathmaster-platform`, and never holds an Editing Session

Every editing command in PathMaster is a method on the window's `App`, which by
[ADR-0007](0007-crate-boundary-is-the-test-boundary.md) is the one crate no automated test reaches — the
GUI shell is covered by the Release Checklist alone. That has been the right trade for Add, Edit, Move
and the rest, whose rules live in `Session` and whose methods are a dozen lines of plumbing over it.
Apply is not that shape. It is the operation that writes the registry, and almost everything it must get
right is **ordering**: back up the value that was just re-read and never the Baseline, let no failure
move the Baseline, run rotation after the write and not before. Written as one more `App` method, none
of that would be testable, and what it guards against is the loss of the very data the application
exists to protect. So the sequence is a function in `pathmaster-platform`, and the window drives it.

**The seam is the crate boundary, because that is where the tests already are.** `pathmaster-platform`
holds the live-registry integration tests under a temporary `HKCU` key, `tempfile`, and the deny-ACL'd
directories the diagnostics tests already build — everything needed to make a write fail on purpose. The
tests worth having for Apply are not "which rule fired" but "did the order hold when the write failed":
was the Snapshot still there, was the Baseline still back, was the Working Copy untouched. Against a
mock those tests assert nothing. `pathmaster-core` was rejected for the same reason from the other side:
it would need three ports — registry, file, broadcast — invented for a single caller, where core's two
existing ports exist because the *rules* they serve are pure. Apply's order is not pure. It is entirely
about I/O.

**The sequence never holds an Editing Session.** It takes the Working Copy's Entries and Value Type by
value and answers with an outcome the window applies afterwards. Two things follow, and both were the
point. A Session is reached through an `Rc<RefCell<…>>`, and the diagnostic Timer ticks inside a modal
dialog's own event loop — a sequence holding `&mut Session` across the external-change dialog would meet
the pass's own borrow and panic, in a window NVDA is reading. And the taxonomy's first invariant, that
no failure moves the Baseline, stops being a rule to obey and becomes one the module has no means to
break: it is handed no Baseline to move.

**The user is asked through a port, not by returning control.** Three questions arrive mid-sequence —
the external-change dialog and the two over-length gates — and `pathmaster-platform` has no wx and will
not get any. Two adapters justify the port: the window's dialogs in the application, scripted answers in
the tests. A resumable state machine would buy the same testability and hand the control flow back to
the window, which is the thing being moved.

**A run covers Scopes, not a Scope.** "User first" and "partial failure aborts the close" (spec §5,
FR-close-confirm) are rules, and leaving them to the close-confirm's own loop would put them straight
back in the untested crate. So the sequence takes the Scopes to apply and owns both the order and the
stop, and Ctrl+S is a run of one — the **Apply Run** of `CONTEXT.md`. A user's [Cancel] stops the run
exactly as a failure does: to the close-confirm the consequence is identical, and the window stays open.

## Consequences

- **The window gains state it did not have**: the last-read `RawValue` per Scope, the `Logger`, the
  backup budget, and the Data Directory. They arrive as one struct of the run's facts, built in `main`,
  rather than as four more positional parameters — tickets 15 and 17 add two more.
- **The last-read `RawValue` cannot live in the Session.** External change is detected by comparing
  `(vtype, bytes)` (spec §4); decoding stops at the first NUL, so a decoded comparison would miss a real
  change. `RawValue` is a `pathmaster-platform` type that `Session` may not reach without reversing the
  dependency direction, so the window holds it. Three paths keep it current — startup, Refresh, and the
  external-change dialog's middle answer — and the outcome carries the fresh one, so each update is a
  hand-off rather than something to remember.
- **The failure taxonomy gains a fifth row.** A re-read that fails at Apply was named by neither §9 nor
  the ticket. It takes the write failure's own text, because the user's truth is the same either way —
  nothing was written. This is also the row that lets Refresh stop failing silently (spec §5,
  FR-refresh), which it has done since impl ticket 11 for want of a name.
- **The clock is a parameter.** A Snapshot name carries a second, and its collision suffix depends on
  what that second already holds; a test that cannot fix the clock cannot reach the rule that a freed
  suffix is never reissued — the rule that stops the rotation after an Apply deleting the backup that
  Apply just took.
- **Snapshot files get their own module in `pathmaster-platform`.** Apply writes and rotates, the
  Backups tab lists, reads and deletes — two callers, one place that spells `data\backups\`, and one
  listing serving both `SnapshotName::next` and `rotation::overflow`.
- **Broadcast logs itself.** `SendMessageTimeoutW` can block for two seconds, so it runs on its own
  thread and appends its `WARN` past the `Logger`, as the panic hook already does. Its record cannot
  ride an outcome that has already returned.
- **The Data Directory arrives as `DataDirState::dir()`, not as a `Writable` directory.** Taking the
  path only obtainable by matching `Writable` is the idiom `settings::write` uses, and it is the wrong
  one here: startup predicts, Apply verifies (ADR-0002). The sequence never asks whether it may write —
  it writes and reads the answer.
- **The other editing commands do not move.** Add, Edit, Delete, Move, Undo, Cancel and Refresh stay
  where they are. This decision is about one sequence, not about emptying the window.
