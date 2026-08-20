# 13 — Apply end-to-end

**Spec:** [spec §5 (FR-apply), §4 (TC-wm-settingchange), §7 (over-length dialogs), §8 (FR-backup-auto), §9 (failure taxonomy), §10.1 items 2–3](../../pathmaster-v0-1-0/spec.md) · ADR-0001, ADR-0006, ADR-0008

**What to build:** Ctrl+S actually changes the machine, safely: an **Apply Run** re-reads the Scope, detects external edits, writes a Snapshot of what it just re-read, writes the registry preserving Value Type, broadcasts the change, and rotates the backups — with the full failure taxonomy where no failure mutates the Working Copy or moves the Baseline. The window moves the Baseline, re-runs diagnostics and announces the outcome afterwards, from what the run hands back.

The run is a function in `pathmaster-platform`, not a method on the window — [ADR-0008](../../../docs/adr/0008-apply-sequence-lives-in-platform.md) records why, and its consequences are the seven checkboxes below the spec's own.

**Blocked by:** 03 (registry adapter), 08 (Sessions in UI, announce), 09 (merged-length thresholds), 10 (Snapshot writing), [19](19-catalogue-lookup-seam.md) (the Catalogue seam).

19 blocks this one by number order rather than by dependency: Apply could be written without it. But this ticket adds Announcements 2 and 3 and the five taxonomy texts, and until the seam exists every one of them composes in the crate ADR-0007 leaves untested — so doing 19 first is the difference between landing them tested and moving them by hand afterwards.

**Status:** ready-for-agent

- [ ] Order fixed: re-read → compare `(vtype, bytes)` → (external-change dialog) → back up the re-read value, never the Baseline → write → move Baseline → re-run diagnostics; detection lives only in Apply — no watcher, no polling
- [ ] External-change dialog: title "PATH was modified externally since last refresh", buttons [Overwrite] (proceed; Undo stack survives) / [Refresh and discard my changes] (Working Copy and Baseline become the new value, stacks cleared, nothing written, no backup) / [Cancel] (nothing happens; Session stays dirty and knowingly stale)
- [ ] Over-length gates at Apply: past 8,191 post-write merged length → warning dialog (title per spec, [Apply Anyway] [Cancel]); at ≥ 32,767 → hard-cap dialog, single [Cancel], no proceed
- [ ] Snapshot written to `data\backups\` per the ticket-10 schema and filename rules, temp+rename with `.tmp` mid-write; rotation runs per-Scope after the write
- [ ] Registry write raw via the adapter; first Apply over an Absent Scope creates `REG_EXPAND_SZ`; zero Entries over Present writes an empty string
- [ ] Broadcast off the UI thread: `SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, 0, L"Environment", SMTO_ABORTIFHUNG, 1000–2000, …)`, lParam UTF-16LE NUL-terminated and outliving the call; a 0 return / timeout is not a failure — one `WARN` line, never surfaced
- [ ] Failure taxonomy with exact texts: snapshot-write failure → "Apply failed — could not write a backup, no changes were made."; registry-write failure → "Apply failed — {cause}"; no failure mutates the Working Copy or moves the Baseline; every failure lands one log record with the raw error code
- [ ] Success announces "User PATH applied" / "System PATH applied" (Announcement 2); focus stays on the current Entry; Apply disabled while clean; one `INFO apply:` audit line (Scope, entry count, chars, Value Type — no PATH text)
- [ ] Undo after Apply re-dirties the Session with the ", unsaved changes" suffix (barrier behaviour observable in the UI)
- [ ] Apply never consults the startup writability prediction — it verifies at write time (startup predicts, Apply verifies)

The spec's own requirements end there. These seven are [ADR-0008](../../../docs/adr/0008-apply-sequence-lives-in-platform.md)'s consequences, plus the two gaps the design pass found:

- [ ] The sequence is an **Apply Run** in `pathmaster-platform`, not a window method: Scopes in, one outcome out, no Editing Session held. The three questions arrive through a port with two adapters — the window's dialogs, scripted answers in the tests. `Timestamp` and the Data Directory (`DataDirState::dir()`, never a `Writable` path) are parameters
- [ ] Snapshot files get their own `pathmaster-platform` module: `data\backups\` spelled once, written through `datadir::write_replace`, and one listing serving both `SnapshotName::next` and `rotation::overflow`. Ticket 14 lists, reads and deletes through the same module
- [ ] `logfmt` gains the two records this needs — the Apply failure line carrying the raw error code, and the broadcast `WARN`. `Record::apply_written` is already there and gets its first caller
- [ ] Announcements 2–3 and the taxonomy's texts enter the Catalogue with Ukrainian; the completeness gate passes
- [ ] §9's fifth row is implemented: a re-read that fails takes the registry-write row's text
- [ ] `Command::Apply` with a menu home and Ctrl+S (ADR-0004: a shortcut can only live on a menu item's label), disabled while clean
- [ ] The window holds what the run needs: the Run's facts — `Logger` and Data Directory — as one struct built in `main`, and the last-read `RawValue` per Scope in `ScopeTab`, replaced from what each run hands back. The **backup budget is not one of them**: `maxBackups` changes while the application runs (ticket 16), so the window holds the current `SettingsFile` and each Apply Run reads the budget from it ([ADR-0010](../../../docs/adr/0010-run-properties-decided-in-one-place.md))
- [ ] Spec §17's `pathmaster-platform` module list gains this ticket's Snapshot-files module
- [ ] The over-length gate reads a length the run computes itself, from both Working Copies by spec §7's formula through the `Environment` port — never the last `Diagnosis`, which lags by a Timer tick and would be a second definition of the number the StatusBar already speaks
- [ ] Noted, not built here: once §9's fifth row exists, `refresh` moves onto it and stops failing silently (spec §5, FR-refresh)

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
