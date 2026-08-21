# 13 — Apply end-to-end

**Spec:** [spec §5 (FR-apply), §4 (TC-wm-settingchange), §7 (over-length dialogs), §8 (FR-backup-auto), §9 (failure taxonomy), §10.1 items 2–3](../../pathmaster-v0-1-0/spec.md) · ADR-0001, ADR-0006, ADR-0008

**What to build:** Ctrl+S actually changes the machine, safely: an **Apply Run** re-reads the Scope, detects external edits, writes a Snapshot of what it just re-read, writes the registry preserving Value Type, broadcasts the change, and rotates the backups — with the full failure taxonomy where no failure mutates the Working Copy or moves the Baseline. The window moves the Baseline, re-runs diagnostics and announces the outcome afterwards, from what the run hands back.

The run is a function in `pathmaster-platform`, not a method on the window — [ADR-0008](../../../docs/adr/0008-apply-sequence-lives-in-platform.md) records why, and its consequences are the seven checkboxes below the spec's own.

**Blocked by:** 03 (registry adapter), 08 (Sessions in UI, announce), 09 (merged-length thresholds), 10 (Snapshot writing), [19](19-catalogue-lookup-seam.md) (the Catalogue seam).

19 blocks this one by number order rather than by dependency: Apply could be written without it. But this ticket adds Announcements 2 and 3 and the five taxonomy texts, and until the seam exists every one of them composes in the crate ADR-0007 leaves untested — so doing 19 first is the difference between landing them tested and moving them by hand afterwards.

**Status:** resolved

- [x] Order fixed: re-read → compare `(vtype, bytes)` → (external-change dialog) → back up the re-read value, never the Baseline → write → move Baseline → re-run diagnostics; detection lives only in Apply — no watcher, no polling
- [x] External-change dialog: title "PATH was modified externally since last refresh", buttons [Overwrite] (proceed; Undo stack survives) / [Refresh and discard my changes] (Working Copy and Baseline become the new value, stacks cleared, nothing written, no backup) / [Cancel] (nothing happens; Session stays dirty and knowingly stale)
- [x] Over-length gates at Apply: past 8,191 post-write merged length → warning dialog (title per spec, [Apply Anyway] [Cancel]); at ≥ 32,767 → hard-cap dialog, single [Cancel], no proceed
- [x] Snapshot written to `data\backups\` per the ticket-10 schema and filename rules, temp+rename with `.tmp` mid-write; rotation runs per-Scope after the write
- [x] Registry write raw via the adapter; first Apply over an Absent Scope creates `REG_EXPAND_SZ`; zero Entries over Present writes an empty string
- [x] Broadcast off the UI thread: `SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, 0, L"Environment", SMTO_ABORTIFHUNG, 1000–2000, …)`, lParam UTF-16LE NUL-terminated and outliving the call; a 0 return / timeout is not a failure — one `WARN` line, never surfaced
- [x] Failure taxonomy with exact texts: snapshot-write failure → "Apply failed — could not write a backup, no changes were made."; registry-write failure → "Apply failed — {cause}"; no failure mutates the Working Copy or moves the Baseline; every failure lands one log record with the raw error code
- [x] Success announces "User PATH applied" / "System PATH applied" (Announcement 2); focus stays on the current Entry; Apply disabled while clean; one `INFO apply:` audit line (Scope, entry count, chars, Value Type — no PATH text)
- [x] Undo after Apply re-dirties the Session with the ", unsaved changes" suffix (barrier behaviour observable in the UI)
- [x] Apply never consults the startup writability prediction — it verifies at write time (startup predicts, Apply verifies)

The spec's own requirements end there. These seven are [ADR-0008](../../../docs/adr/0008-apply-sequence-lives-in-platform.md)'s consequences, plus the two gaps the design pass found:

- [x] The sequence is an **Apply Run** in `pathmaster-platform`, not a window method: Scopes in, one outcome out, no Editing Session held. The three questions arrive through a port with two adapters — the window's dialogs, scripted answers in the tests. `Timestamp` and the Data Directory (`DataDirState::dir()`, never a `Writable` path) are parameters
- [x] Snapshot files get their own `pathmaster-platform` module: `data\backups\` spelled once, written through `datadir::write_replace`, and one listing serving both `SnapshotName::next` and `rotation::overflow`. Ticket 14 lists, reads and deletes through the same module
- [x] `logfmt` gains the two records this needs — the Apply failure line carrying the raw error code, and the broadcast `WARN`. `Record::apply_written` is already there and gets its first caller
- [x] Announcements 2–3 and the taxonomy's texts enter the Catalogue with Ukrainian; the completeness gate passes
- [x] §9's fifth row is implemented: a re-read that fails takes the registry-write row's text
- [x] `Command::Apply` with a menu home and Ctrl+S (ADR-0004: a shortcut can only live on a menu item's label), disabled while clean
- [x] The window holds what the run needs: the Run's facts — `Logger` and Data Directory — as one struct built in `main`, and the last-read `RawValue` per Scope in `ScopeTab`, replaced from what each run hands back. The **backup budget is not one of them**: `maxBackups` changes while the application runs (ticket 16), so the window holds the current `SettingsFile` and each Apply Run reads the budget from it ([ADR-0010](../../../docs/adr/0010-run-properties-decided-in-one-place.md))
- [x] Spec §17's `pathmaster-platform` module list gains this ticket's Snapshot-files module
- [x] The over-length gate reads a length the run computes itself, by spec §7's formula through the `Environment` port — never the last `Diagnosis`, which lags by a Timer tick and would be a second definition of the number the StatusBar already speaks. *Amended after the review: this line first said "from both Working Copies", which §7 contradicts for a Scope the run is not applying — see "The gate's number" below*
- [x] Noted, not built here: once §9's fifth row exists, `refresh` moves onto it and stops failing silently (spec §5, FR-refresh)

## Comments

Designed 2026-08-20, before any code, out of an architecture review of the hot spot — `ui/mod.rs` at 733
lines and seven of the last fifteen commits, in the crate ADR-0007 leaves with no automated tests. The
review's finding was not that the file is large. It was that eight tested core interfaces —
`mark_applied`, `restore`, `rotation::overflow`, `Snapshot::under`, `snapshot::listing`,
`Diagnosis::overlength`, `Overlength::may_proceed`, `settings::write` — have no production caller at all,
because they are the steps of this ticket and nothing yet owns the sequence that puts them in order.
Deciding where that sequence lives was cheap now and would not have been after 13, 14 and 15 had each
written into the window.

**The reasoning is in [ADR-0008](../../../docs/adr/0008-apply-sequence-lives-in-platform.md)** and is not
repeated here. What the design pass changed about *this ticket* is two gaps it turned up, neither of
which the checklist above had before:

- **Nothing keeps the last-read `RawValue`.** `ScopeKey::read()` hands one back and both existing callers
  — `main::load_session` and `App::refresh` — call `.decode()` and drop it. So the primitive spec §4
  fixes for external-change detection, comparing `(vtype, bytes)`, has nothing to compare against.
  `Session` cannot hold it: `RawValue` is a `pathmaster-platform` type and core may not reach it. The
  window holds it instead, and the run's outcome carries the fresh one so the update is a hand-off.
- **A failed re-read had no row in §9.** It is the first step of FR-apply's fixed order and the taxonomy
  named four failures, none of them this. §9 now has five; the fifth is the row §5's FR-refresh has been
  waiting on since impl ticket 11.

**Two smaller things that will bite if unnoticed.** `SendMessageTimeoutW` blocks for up to two seconds,
so the broadcast runs on its own thread and appends its `WARN` past the `Logger` the way `panic_hook`
already does — its record cannot ride an outcome that has already returned. And the `Timestamp` is a
parameter rather than a `logwriter::now()` call inside, because a Snapshot name's collision suffix
depends on what its second already holds: a test that cannot fix the clock cannot reach the rule that a
freed suffix is never reissued, which is the rule stopping the rotation after an Apply from deleting the
backup that Apply just took.

---

Implemented 2026-08-21 on `feature/catalogue-lookup-seam`. `pathmaster-platform` gains three
modules — `apply` (the Apply Run), `snapshots` (the files, `data\backups\` spelled once) and
`broadcast` — with 31 tests behind them, none of which link wxWidgets. `ui/mod.rs` grows by about
120 lines, all of it wiring: the run's inputs copied out, its outcome absorbed, and the three
dialogs behind the question port.

**The order is the whole of what the tests assert.** Not "which rule fired" but "did the order hold
when something went wrong": a re-read that fails leaves no Snapshot and no write; a backup that
cannot be written leaves the registry byte-for-byte as it was found; a registry write that fails
leaves the backup it had already taken, because that file is a true record of what the Scope held
and the one thing a user whose Apply just failed might want. The taxonomy's first invariant — no
failure moves the Baseline — needed no test at all: the run is handed no Baseline, so it has no
means to break it.

**Two shapes the ticket left open, both now closed the way ticket 19 predicted.**

- **`Announcement::Applied` carries a `Scope`, not a msgid.** Ticket 19 could not narrow it because
  §10.1's two strings were not registered and that ticket added no Catalogue text. This one
  registers them, so the variant narrows and the choice between "User PATH applied" and "System PATH
  applied" becomes a rule of the Catalogue rather than of the calling code. ADR-0009's msgid rule is
  untouched: it covers the Announcements whose cause is a *platform type*, and item 2's cause is a
  `Scope`, which is core's own.
- **`ApplyFailed` carries a cause, and the sentence around it moved down too.** §9's rows all say
  "Apply failed" and then name their own reason, so there is one frame — `"Apply failed — {cause}."`
  — and three cause phrases, filled in exactly as Announcement 7's `{reason}` already was. The frame
  carries the final stop, which is what makes the composed strings come out as §8 and §9 write them
  verbatim ("Apply failed — access denied.", "Apply failed — could not write a backup, no changes
  were made.") while leaving each cause a phrase a translator may reorder. The typed failure stays
  in `pathmaster-platform` and contributes only its `catalogue_msgid()`, which is ADR-0009's rule
  unchanged.

**Four decisions a reviewer would otherwise have to reconstruct:**

- **The over-length gates run once per run, before any Scope is touched.** The merged length is a
  fact about *both* Working Copies, so a two-Scope run asking twice would be asking twice about one
  number — and a gate opening after the first Scope had been written would be a warning about
  something that had already happened. A gate the user refuses stops the run at its first Scope,
  recorded as that Scope's `Cancelled`, so `Outcome::completed()` is one fold and the close-confirm
  ticket has one question to ask.
- **The three questions are three methods, and the hard cap answers nothing.** `Ask::hard_cap`
  returns `()`. §7's "single [Cancel] — no proceed" is then not a rule anyone has to remember; it is
  the only thing the signature can express.
- **`thresholds::merged_length_of` is new, and `diagnostics` now calls it.** The gate must compute
  the length itself — the last `Diagnosis` lags by a Timer tick, and the number in the dialog is the
  one the user is being asked to accept — but "compute it itself" must not mean a second definition
  of §7's formula. So the formula moved into `thresholds` whole, and both callers ask it.
- **One broadcast per run that wrote anything.** The `lParam` names the environment block rather
  than a variable, so two Scopes are still one change. It is spawned and its handle dropped; the
  handle exists at all so that ticket 15, which ends the process, has something to wait on.

**Three things this ticket touched that were not strictly its own**, each named so the diff does not
look wider than it is. `logfmt::ScopeReadCause` is now `FailureCause`: an Apply failure and a failed
startup read have the same two things to say — an OS error code, or a registry type we do not
support — and a near-duplicate enum is what the rename avoids. `question.rs` grew a `choose` over a
slice of labels, because the external-change dialog needs three buttons and the hard cap needs one;
`ask` is now two lines over it. And spec §17's platform module list gained `diagnostics` as well as
this ticket's two — it was missing, in the very list this ticket was amending, and the convention
that each module is named as it lands exists to stop exactly that.

**`refresh` still fails silently, deliberately.** §9's fifth row now exists, which is what
FR-refresh has been waiting on since impl ticket 11 — but moving Refresh onto it is a change to a
different command, and the ticket says "noted, not built here".

**Verified live, in Ukrainian, on this machine's real PATH.** Two runs of the debug build, driven
cross-process.

The first touched nothing: the File menu is there beside Edit («Файл(F)» / «Редагування(E)»), the
Apply button sits between Move Down and Cancel Changes as §15 orders it, and both read as disabled
while the Session is clean. One 9,000-character Entry and Ctrl+S raised «cmd.exe ігноруватиме PATH,
довший за 8 191 символ (11230 після цього застосування)» with [Усе одно застосувати] [Скасувати];
editing it to 40,000 raised «PATH не може перевищувати 32 767 символів (42230 після цього
застосування)» with one button. Both cancelled, and `HKCU\Environment\Path` came out of the run
byte-identical with `data\backups\` never created — the gate really does stop before anything is
written.

The second wrote, twice, and put the machine back exactly where it started (the original
`(vtype, bytes)` were captured first and compared by SHA-256 afterwards: 3,098 bytes in, 3,098 bytes
out, same hash). Adding `C:\PathMasterLiveCheck` and pressing Ctrl+S put «PATH користувача
застосовано» in the Banner and `INFO apply: User scope written, 43 entries, 1571 chars,
REG_EXPAND_SZ` in the log — derived facts, no PATH text. Ctrl+Z then read «Скасовано: Додавання
запису, незбережені зміни»: Announcement 5, the Apply barrier, observable exactly as the ticket
asks. The second Ctrl+S wrote the original value back and raised **no** external-change dialog,
which is the hand-off working — the first run's outcome had already replaced the tab's last-read
value, so the second run's re-read matched it. Two Snapshots landed in `data\backups\`, the second
larger than the first: each is of the value that was *re-read*, not of the one being written.

### The review

Both axes ran against `4fc6e9e`, the last of ticket 19. **Spec** confirmed the twenty checkboxes are
each satisfied by code and matched every Catalogue string against §7, §8, §9 and §10.1 character by
character, including that the frame-and-cause split composes §8's and §9's sentences verbatim.
**Standards** confirmed the three things most worth confirming: no `RefCell` is live across a modal
dialog, no new log record can carry PATH text or an absolute path, and the `FailureCause` rename
earns itself.

Eight findings applied:

- **`Run` was `CONTEXT.md`'s word for something else.** The glossary gives **Run** to one execution
  of the application and **Apply Run** to one pass of this sequence, and ADR-0010 exists to keep the
  two apart — yet `ui/mod.rs` imported a `Run` meaning the second and a `RunFacts` meaning the
  first. The struct is `ApplyRun` now, and its doc says why.
- **Two public routes to one formula.** `merged_length` (pre-expanded strings) and
  `merged_length_of` (Entries plus an `Environment`) differed by a suffix carrying none of the real
  difference. There is one `thresholds::merged_length` now, taking the Entries — so §7's formula
  cannot be asked for half-done, which was the whole reason it moved down here.
- **`absorb`'s `Applied` arm reached into `ScopeTab` three times**; it is `ScopeTab::applied` now,
  beside `adopt`, which it mirrors. `absorb` itself is `after_apply`, matching `after_edit`.
- **The Refresh shape was only half extracted.** `App::refresh` and the `Refreshed` arm still shared
  four lines around `adopt`. `adopt` now answers the **row** rather than the id and takes the focus
  reading itself, so both call sites are one line — which matters because its own doc says the
  halves must not come apart.
- **`Command::menu()` had a catch-all.** `_ => Edit` would have landed ticket 15's Exit in the wrong
  menu silently, in the one `match` over this enum that was not exhaustive. It is now.
- **Focus after Apply moved when it had no reason to.** `keep_focus` took the focus into the list
  unconditionally, which satisfies §10's "after Apply — stays on the current Entry" for a Ctrl+S
  pressed on a row and breaks its "focus never jumps without a reason" for one pressed on the Move
  Up button. It is `rescue_focus(session)` now and moves focus only when the control holding it is
  one this Apply has just disabled — which is the case it existed for, since Apply and Cancel both
  turn themselves off the moment the Session goes clean.
- **`broadcast`'s doc overclaimed.** It said the `JoinHandle` was there for "a caller about to end
  the process"; no such caller exists and `Outcome` does not carry the handle. It now says what is
  true: the handle is for this module's own test, and the Apply Run drops it.

**Two findings declined, and one deferred with its reasoning.**

- **"modified" in the external-change dialog's title** was flagged against **Dirty**'s `_Avoid_`
  list. That title is spec §5's text quoted verbatim and ADR-0004 makes Catalogue text load-bearing;
  the `_Avoid_` entry governs what this project *calls the Dirty concept*, and both "Modified" and
  the suggested "Changed" are on the same line of it. Not ours to reword.
- **`Diagnosis::overlength` still has no production caller** — and must not have one. This ticket's
  own checkbox forbids the gate from reading the last `Diagnosis`, which lags by a Timer tick.
- **`Overlength::may_proceed` still has no production caller either**, and that one is a fair hit:
  the ticket's Comments named it among the eight interfaces this sequence was to give a caller.
  `gate()` matches `classify()`'s three variants directly because the three behaviours are distinct
  — say nothing, ask, tell — and `may_proceed` collapses two of them into a `bool` the gate cannot
  use without classifying twice. So the rule "the warning is walkable, the cap is not" is now
  written in `may_proceed` and enforced by `Ask::hard_cap` returning nothing. **Deferred rather than
  papered over**: either `gate` should be restructured around it or `may_proceed` should be deleted,
  and choosing is not this ticket's to do on its last lap.

**One finding worth more than a fix, recorded here for the ticket that settles it.** The over-length
gate reads **both Working Copies**, which is what this ticket's checkbox says in as many words — but
§7 asks for the *post-write* merged length, and for a Scope the run is not applying those are
different numbers. In an elevated run with a large unapplied System edit, a perfectly legal User
Apply can be stopped by a length that would never have existed. The narrowness is real (the System
Session is non-writable unless elevated, so an unelevated run cannot reach it) and so is the
conflict, and resolving it means choosing between the ticket's wording and §7's. Left as the ticket
specifies it, named here rather than changed quietly.

### The Release Checklist

§10.2 makes the Checklist the whole of this application's GUI coverage, and this ticket added three
dialogs and an Announcement that had no step in it. It gains six, appended to section B the way
impl ticket 11 appended B4 and B5 — no section re-lettered, so a filled copy from an earlier release
still means what it said.

**B6-B8** are the external-change dialog's three answers, staged by editing
`HKCU\Environment\Path` in `regedit` while the app is open: that the dialog is spoken with all
three buttons and Escape answers with Cancel; that [Refresh and discard my changes] writes nothing
and leaves Ctrl+Z with nothing to take back, because the stacks were cleared; and that [Overwrite]
backs up **the value `regedit` left**, not the one the Session remembered — which is the one thing
about the order a reader cannot check by looking at the screen.

**B9 and B10** are the two over-length gates, and the StatusBar's own length field is what tells the
person running them when they have gone far enough. B10 names the count of buttons, because "no way
past" is the whole of what the hard cap is.

**B11 reaches the failure taxonomy from the keyboard**, which nothing else in the Checklist does: a
*file* named `backups` inside `data\` cannot be turned into the directory the Snapshot needs, so
Apply stops before it writes and says so. It is the only one of §9's rows an unelevated tester can
stage — access denied needs a System Scope, and a System Scope needs elevation.

**C6** is Announcement 2's other string. "System PATH applied" is unreachable unelevated by
construction, since a non-writable Session reads Apply as unavailable, so it belongs in the elevated
section or nowhere.

**A10 gained its second half.** It checked the speech and not the focus, though §10 fixes both
("after Apply — stays on the current Entry"); it now ends with the `NVDA+Tab` confirmation step 7
already carried. That is also the step that would catch a regression in `rescue_focus`.

### Heard, not only seen

The steps this ticket added were run on real NVDA by the user on 2026-08-21 and reported as
passing: **A10** and **A11** (the Apply Announcement and the ", unsaved changes" suffix an undo
across the barrier earns), **A14** (Apply and Cancel reading as unavailable on a clean Session), and
**B6–B11** — the external-change dialog's three answers, the two over-length gates, and the backup
failure that is the one row of §9's taxonomy an unelevated tester can stage.

That closes the gap this ticket's own verification could not. Everything recorded above was
*measured* — window titles read cross-process, the registry's bytes compared by hash, the log lines
read back off disk — and a value that is right and a screen reader that speaks it are different
claims. Only the second one is what this application is for.

**C6 was not run, and could not have been.** It asks for "System PATH applied" on an elevated
instance, and the Checklist's own route to one is Tools → Restart as Administrator, which impl
ticket 17 builds. It also requires an *installed* NVDA, portable being deaf to elevated windows. The
step stays in the Checklist with its box unticked, which is the honest state: Announcement 2's other
string is implemented and tested in `pathmaster-platform`, and has not yet been heard.

This is history, not a substitute for the release pass. §10.2 wants a filled copy naming the NVDA
used, produced before every release; that copy is ticket 18's, and it will run these steps again
against the shipped binary.

### The gate's number

The review found this ticket's own checkbox arguing with the spec, and the spec is right.

§7 asks the Apply gate for the **post-write** merged length; the checkbox said "from both Working
Copies". Those agree for a run over every dirty Scope and disagree for a run over one, and the
disagreement is not cosmetic: with a large unapplied System edit open, the old reading let a hard cap
block a perfectly legal User Apply on a length that would never exist. That is a lockout, not a
warning — the one outcome a gate must never produce.

**What settled it is what the limits are limits on.** Both numbers govern a *materialised
environment variable*, not an editor's buffer: 8,191 is the length past which `cmd.exe` drops an
inherited variable ([KB 830473](https://learn.microsoft.com/en-us/troubleshoot/windows-client/shell-experience/command-line-string-limitation)),
and 32,767 is the documented maximum size of one user-defined variable
([Environment Variables](https://learn.microsoft.com/en-us/windows/win32/procthread/environment-variables));
the environment *block* limit that Raymond Chen's
[2010 post](https://devblogs.microsoft.com/oldnewthing/20100203-00/?p=15083) describes was lifted at
Vista and is not a modern constraint. The merged `PATH` a process receives is one variable, composed
System first then User, out of **what the registry holds**. A Working Copy nobody has applied has
never been part of it and may never be. So it weighs nothing, and the gate now says so:
`after_this_run` takes each Scope's Working Copy when the run is applying it and its `last_read`
otherwise.

`last_read` rather than a fresh read, deliberately. It is already the value this application
believes the registry holds — the same value the external-change comparison is made against — and a
read here would add a failure mode to a step that has nothing to do when it fails.

**The StatusBar's number does not change, and is now allowed to differ.** §12 gives that field both
Working Copies because previewing what you are editing is its job. Two fields, two questions: "what
am I editing" and "what will this Apply leave behind". They part company only when the Scope you are
not applying has unsaved edits, and each is right about its own question. Nothing here should be
"fixed" later to make them agree.

**`Overlength::may_proceed` is deleted rather than called.** It was the review's other open item —
a tested core interface with no production caller, which is the exact smell ADR-0008 was written
about. It could not honestly get one: the gate does three different things with `classify`'s three
variants — say nothing, ask, tell — and a `bool` collapses two of them. Its sentence, "the warning is
walkable, the cap is not", now lives on `Ask::hard_cap`, which does not merely state the rule but has
no answer to give.

Two tests pin both halves: an unapplied Working Copy of 40,000 characters raises nothing at all, and
a System *registry value* of 9,000 raises the warning during a User Apply.
