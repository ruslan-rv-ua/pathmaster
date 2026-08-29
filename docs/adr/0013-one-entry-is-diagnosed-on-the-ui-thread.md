# One Entry is diagnosed on the UI thread, and only where the answer is complete

A diagnostic pass never runs on the UI thread, and the reason is not taste: a `PATH` of a few hundred
Entries is a few hundred `GetFileAttributesW` calls, and a stalled message pump is a window NVDA cannot
read. So a pass goes to a worker thread, comes back over a channel, and is drained from a Timer that polls
every 100 ms (spec §7, FR-diag-async). Findings are held by `(id, raw)`, so an Entry whose text has just
changed carries none until the new pass lands — deliberately, because reading the old pass by row would put
a stale "Missing" on the path the user has this second corrected.

That leaves one hole, and it is exactly where the application is least able to afford one. Add and Edit end
by landing focus on the Entry the user just typed, and NVDA reads that row on arrival — position, path, and
the Status column, which at that instant is **empty**. Empty is also how a perfectly healthy Entry reads.
So the user who adds `C:\toosl` hears nothing at all about it, and finds out only by arrowing off the row
and back after the pass has quietly landed. The window between the two is small in milliseconds and total
in effect: it covers the whole of the one moment the row is spoken.

**The row is made to say it, rather than something else being made to say it for the row.** Three other
answers were on the table. A fifteenth Announcement would change a set closed by decision, and would speak
after the pass — by which time focus may be somewhere else entirely — to say what the row itself is
perfectly capable of saying. Landing focus only once the pass has arrived trades a silent fault for an
audible one: the list is read first and the row a fifth of a second later, which is a worse sound than the
one being fixed. Doing nothing leaves not one fault but two, the second being that an Entry with no findings
matches no narrowing Filter: editing one broken path into another, in a list filtered to broken paths, made
the row **vanish** from under the focus that was about to land on it. Add cannot reach that second fault —
a narrowed Scope closes it, because it appends at a position the view may be hiding — but Edit stays open
throughout, and it is where the fault lives.

**So one Entry — the concerned one — is diagnosed synchronously, and the boundary is what makes that
affordable.** The rulebook is asked what it would probe for that Entry, and the machine is asked whether
answering is cheap here:

- an Entry the existence check never reaches — Empty, or Relative — costs nothing at all, and those two are
  the flags a typo most often earns;
- a network root is never probed by anyone, here or on the worker, under the standing rule that a dead UNC
  blocks 20–60 uncancellable seconds;
- a fixed disk is the machine's own storage, so `GetFileAttributesW` answers without spinning anything up;
- **everything else declines** — removable media, an optical drive, a root this run cannot classify. That
  is where a probe raises the OS's own "There is no disk in the drive" box, and the worker thread suppresses
  it with a thread-scoped `SEM_FAILCRITICALERRORS` precisely because it has no window to own it. On the UI
  thread it would have one.

Setting that error mode on the UI thread instead was rejected: it is thread-scoped, so it would be a
property of the whole run (ADR-0010), and it would silence device errors for the native folder picker
Browse opens as well — which is somebody else's dialog, answering somebody else's question.

**A refusal writes nothing, never a partial reading.** An Entry diagnosed on everything but its existence
would read "Duplicate" on arrival and grow "Missing" a moment later: the row would have told NVDA something
false. A blank Status has told it nothing, which is the contract the row already lives under.

**The seam is the crate boundary, for [ADR-0007](0007-crate-boundary-is-the-test-boundary.md)'s reason.**
`pathmaster-core` answers what one Entry's Issues are — the same `diagnose_entry` a pass runs, over the same
duplicate set built from the same runtime order, so the two cannot come to disagree. `pathmaster-platform`
holds the boundary above, because which roots are cheap is a fact about the machine and not a diagnostic
rule — the distinction `DriveTypes` already carries in its own doc comment. The window is left with one
`if let`. Rules in the GUI crate would be rules no automated test can reach.

## Consequences

- **The concerned Entry of an Add or an Edit is never Undiagnosed** — the invariant this decision exists to
  establish, and the one a later change must not quietly drop. `CONTEXT.md` names the state.
- **A test asserts the two answers agree**, for every index of both Working Copies over a fixture built to
  make the prefix walk matter. It is the load-bearing test: a stamp is only ever right because it is the
  pass's own rulebook run over one Entry, so nothing else in this decision survives the two diverging.
- **`Findings` stops meaning "the last pass"** and starts meaning "the last pass, plus what has been stamped
  since". StatusBar field 0 counts a stamped finding, which the field's own doc comment used to rule out —
  it now says so. `None` still means no pass has run, and still may not be read as "no Issues".
- **The Filtered View becomes honest at the moment of the edit.** An Entry edited under a narrowing Filter
  is now matched on what it has just become, so one that still belongs in the view stays in it, visible and
  focused, instead of dropping out until the pass puts it back.
- **Startup is deliberately untouched.** No pass has landed, so `Findings` is `None` and nothing is stamped:
  the first row is still read without its Status (the Release Checklist's step A1 says so, and stays true).
  Stamping into an empty `Findings` would have StatusBar field 0 claim a count for a Scope nothing has
  looked at.
- **Move, Undo, Redo, Restore and Fix Issues are untouched.** Move changes neither an Entry's text nor its
  id, so its row already carries its findings. The other four change many Entries at once — one stamp would
  be an arbitrary choice among them — and all four already speak.
- **The rulebook answers a second question in public**: `probe_target` says what the existence check would
  read for an Entry, or nothing for one it never reaches. It exists so a caller can find out whether a probe
  is coming *before* paying for it, and it shares the branch it describes with the rule itself.
- **The Release Checklist gains three steps**, because what changed is what NVDA says, and the Checklist is
  what gates that: an Add that lands on a missing folder, an Add under a narrowing Filter, and an Edit that
  makes a healthy Entry missing.
