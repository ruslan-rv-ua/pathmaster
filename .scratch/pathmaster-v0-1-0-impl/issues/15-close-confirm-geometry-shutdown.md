# 15 — Close-confirm, geometry persistence, clean shutdown

**Spec:** [spec §5 (FR-close-confirm), §12 (geometry), §14 (shutdown line)](../../pathmaster-v0-1-0/spec.md)

**What to build:** Closing the application is safe and remembered: dirty Sessions raise one dialog naming them, Save routes through the full Apply path with partial failure aborting the close, and a clean shutdown persists window geometry and leaves its log line. Reopening restores the window where it was, clamped to real monitors.

**Blocked by:** 07 (settings write path), 13 (Save goes through Apply), [20](20-startup-decisions-module.md) (so geometry clamping lands in a tested module).

**Status:** resolved

- [x] One dialog for the application, title naming the dirty Scopes ("Unsaved changes in: User PATH, System PATH — save before closing?"), buttons [Save] [Discard] [Cancel]; clean Sessions close with no dialog
- [x] Save applies each dirty Session in turn, User first, each through the full Apply path (external-change detection, backup, taxonomy included)
- [x] Partial failure aborts the close: window stays open, focus moves to the failed tab, the reason is announced
- [x] Geometry (position, size, maximised state) written to settings.json on clean shutdown only, via the atomic-replace helper, preserving unknown fields; not written in Read-only Data
- [x] On startup geometry is restored clamped to the connected monitors' work area; fully off-screen → default size centred on primary
- [x] Clean shutdown logs `INFO shutdown: clean`
- [x] Dialog strings in the Catalogue with Ukrainian translations

## Comments

Implemented 2026-08-21 on `feature/close-confirm-geometry-shutdown`.

`pathmaster-platform` gains **`geometry`** — where the window opens — and the binary gains one
close path that every route out of the application arrives on. Nineteen new tests, none of which
link wxWidgets.

**One list of dirty Scopes, read once.** It is what the dialog's title names and what the Apply Run
is handed, in tab order, which is User first. Two readings, or a second ordering rule inside the
Catalogue, could only ever promise an order the sequence does not keep — so
`Catalogue::close_confirm_dialog` deliberately names the Scopes in the order it is given, unlike
`general_status`, which sorts because it reads the two tabs rather than a run. Both name a Scope
through one `tab_msgid`, the label its own tab already carries.

**"Did the run complete" and "which Scope failed" are two questions.** `Outcome::completed()`
already decided whether the close proceeds, and a [Cancel] inside the run — the external-change
dialog's, or either over-length gate — answers it exactly as a failure does: the window stays open.
But a Cancel is not a failure. The user chose it, so there is nothing to announce and no tab to be
sent to; folding the two into one answer would send them to a tab they had just declined to write,
with an empty Banner to explain it. Hence `Outcome::failed_scope()`, which is the *focus* question
and nothing else. The failed tab is activated **before** the outcome is applied, because activating
a tab speaks its entry count (§10.1 item 1) while the failure speaks Announcement 3 — last spoken
is what is heard, and the reason is what the user needs.

**Save is `apply_scopes`, the same call Ctrl+S makes.** Ctrl+S was a run of one Scope inlined in
`apply`; it is now a run over a slice, and the close-confirm passes every dirty Scope. Nothing
about the sequence is special-cased for closing — external-change detection, the backup, the
taxonomy and the audit line all happen exactly as they do for Ctrl+S, which is what "each through
the full Apply path" has to mean if the close is to be trusted.

**`geometry` is one module with the seam drawn through it.** `work_areas()` is the one
`EnumDisplayMonitors` call a test cannot make fail; `place(remembered, &work_areas)` is arithmetic,
and it is where every rule lives (ADR-0010's reason for arriving before this ticket). Ten tests
arrange a second monitor left of the primary, a monitor unplugged since the window was last closed,
a window merely touching an edge, and a run that can see no monitors at all — none of which this
machine has.

**Five rules the spec left to the implementation, all five now in §12.** The clamp is to *one*
monitor — the one showing most of the window — never to the union, because the union has holes: two
monitors of different heights leave a region inside the bounding box and on no screen. Sharing an
edge is being off-screen. The **work area**, never the monitor's full rectangle, or a restored
window's title bar can end up under the taskbar. A run that can see no monitors takes the default
place, which is also what a failed enumeration reads as. And the round trip is physical pixels at
both ends: wxdragon routes a *builder's* size through its implicit `FromDIP` and
`set_size_with_pos`/`get_position`/`get_size` through nothing, so the restore sets the geometry on
the built frame rather than handing it to the builder — the builder would scale it once more per
run.

**A maximised window records the rectangle wx reports while maximised, beside the flag.** Restoring
sets that geometry and maximises over it, so an un-maximise afterwards lands on a window the size
of the screen rather than on nothing at all. It is the one place the round trip is deliberately
lossy, and the loss is a window that is too big rather than one that is not there.

**Writing the file needed its own two rules (now in §13, whose taxonomy is about reading).** The
write is `settings::write` — one atomic replace of the amended document — so a hand edit's unknown
fields, nested ones included, and its key order all survive. A write that fails earns one
`WARN settings:` line and nothing else: no dialog, because on this path the window is already going
and a dialog would outlive what it is about; no Announcement, because the catalogue is closed at
seven. Without the line a setting that silently never persists would have no witness at all.

**"Not written in Read-only Data" is visible at the call site.** `Run::data_dir()` is
`DataDirState::dir()`, which a Read-only run has too — so `App::writable_data_dir()` reads the
`readonly` reason the UI already holds, which is `None` in exactly one case. Apply deliberately
does not ask this question (startup predicts, Apply verifies — ADR-0002); geometry is the other
side of that rule, because nobody asked for it and a run that could only find out by failing should
not try.

**File → Exit, and one close event.** §15's last File item lands here: Alt+F4 is Windows' own
gesture given a menu home, which does not create the shortcut but makes the item *read* as the
shortcut it already is (ADR-0004). The item is available in every state — an application a dirty
Session could disable the way out of is one the user has to kill. The title bar's [X], Alt+F4,
File → Exit and the taskbar's Close all arrive as one `EVT_CLOSE`, so the dialog is asked once and
in one place; a close that proceeds skips the event on to wx's own handler, and one that does not
vetoes it, which is also what answers a Windows session end.

### Verified live against a portable debug build

A copy of the debug build in a scratch directory, its `data\` beside it, driven by
`GetWindowRect`/`MoveWindow`, `WM_COMMAND` and `WM_CLOSE`. Nothing was applied — the real `PATH`
was compared before and after and is unchanged.

- **Geometry round trip.** Moved to `320,110 1000×700`, closed: `settings.json` records exactly
  that, and the log's last line is `INFO  shutdown: clean`. Reopened at `320,110 1000×700`.
- **The clamp.** A remembered `1000×1070` on a work area of `1680×1002` reopened at `320,0
  1000×1002` — the height cut to the work area, the top moved up by the least that fits.
- **Maximised.** Maximised, closed (`"maximised": true` recorded beside the maximised rectangle),
  reopened maximised.
- **Off-screen.** `settings.json` hand-edited to `x: 9000, y: 9000`: reopened at `390,176 900×650`
  — the default size, centred on the primary monitor.
- **Preservation.** A hand-written file with `language`, `maxBackups`, an unknown `futureField` and
  an unknown member *inside* `window` came through the rewrite with all four intact and its key
  order unchanged.
- **First run.** No `settings.json` until the first clean shutdown, which creates it holding
  `window` alone — defaults nobody chose do not materialise as choices somebody made.
- **The dialog.** With the User Session dirty, closing raised a dialog titled
  `Unsaved changes in: User PATH — save before closing?` whose three buttons read `Save`,
  `Discard`, `Cancel`. [Cancel] left the window open with the Session still dirty and **no**
  `shutdown: clean` line; closing again and answering [Discard] closed the application with the
  registry unchanged and no new file in `data\backups\`.
- **Routes.** File → Exit (`id=6012`, label `E&xit\tAlt+F4`, enabled while Apply read as disabled
  on a clean Session) raised the same dialog as `WM_CLOSE`.
- **Read-only Data.** A run whose `data\` carried a deny-write ACE still **read** its
  `settings.json` and opened at the remembered `250,90 950×680`; after a clean close the file was
  byte-identical, and the run had no log at all — which is §14's rule one, not a missing shutdown
  line.

[Save] was not exercised live: it writes the real `HKCU\Environment\Path`. It is covered by the
Apply Run's own integration tests against a temporary key — including the multi-Scope order, the
stop at the first Scope that does not complete, and `failed_scope` — and by Release Checklist steps
28 and 29.

### The review, applied

Two axes over `develop...HEAD`. Five findings taken, one declined.

- **A catch-all over `Command`** had crept into `run()`'s dispatch — the one non-exhaustive match
  over that enum in the tree, against a rule this very diff writes twice in `command.rs`. Arms
  spelled out.
- **"Focus moves to the failed tab" was half done.** `set_selection` activates a tab; it does not
  land the keyboard focus in it, and this file treats the two as separate steps everywhere else. A
  row focused in a control that is not focused is silent, which for this application is the same as
  not having happened — so `save_then_close` now activates *and* focuses, and
  `ScopePage::focus_list` is the one home for "focus into the list, on the row the user was on"
  that `rescue_focus` already needed.
- **A minimised window silently lost the geometry.** Windows parks one far off every monitor and
  reports that as its position, with `is_maximized` reading `false` whatever it was — so closing
  from the taskbar while minimised recorded coordinates the next start could only read as
  off-screen, and the window came back centred. A minimised window is now not written at all: the
  file keeps the last place the user could actually see it. §12 says so.
- **`WorkArea::overlap` overflowed `i32`.** `window.x + window.width` comes out of a hand-editable
  file that §13 deliberately does not clamp, so `x: 2147483647` panicked before the window was
  shown — a hand edit turning into an application that will not start. The arithmetic is `i64` now,
  with the far edge of the type as an ordinary answer (no overlap), and two tests pin it.
- **One comment claimed more than the code does.** Vetoing `EVT_CLOSE` does not answer a Windows
  log-off: that is `wxEVT_QUERY_END_SESSION` on the application object, which nothing implements.
  The claim is gone and the limit is named in §5 instead — accepted for v0.1.0, since what a
  log-off loses is edits that had never reached the machine.
- **§15's amendment now names what it overrides**: "every menu item's enabled state reflects the
  active Session". Exit is the second item that does not (Open Backups Folder was the first,
  following the Run) and the only one that follows nothing at all.

**Declined: extracting a shared `Rect`.** `WorkArea` and `settings::Window` are the same rectangle
wearing two names, and the smell is real — but `Window` is ticket 07's settled record of what the
file holds, with its own read/write pair, its own field-layer rules and its own tests. Re-cutting it
around a `Rect` would rewrite a public core type and its serialisation for no behavioural gain, in
a ticket that is about closing a window. Recorded here so the next person meets a decision rather
than an oversight.

**Verified live afterwards**, same portable build, and the real `PATH` unchanged before and after:

- Closed at `260,140 1040×660`, reopened, **minimised**, closed from the taskbar: `settings.json`
  still says `260,140 1040×660`, and the run still logged `INFO  shutdown: clean`.
- Partial failure, arranged so the registry is never reached — a *file* where `data\backups\`
  must be a directory, so the Apply Run fails at the backup step. In Ukrainian, [Зберегти] left the
  window open with the Banner reading «Не вдалося застосувати — не вдалося створити резервну
  копію, зміни не внесено.», the keyboard focus on a `SysListView32`, the registry byte-identical,
  `ERROR apply: User scope not applied, backup failed (os error 267)` in the log and **no**
  `shutdown: clean` line under it.
- The Ukrainian dialog reads «Незбережені зміни: PATH користувача — зберегти перед закриттям?»
  with [Зберегти] [Відхилити] [Скасувати], and File → `Вихід(&X)` carries `Alt+F4` — the Latin
  mnemonic in parentheses, as ADR-0004 requires.

### Heard, not only seen

The steps this ticket added were run on real NVDA by the user on 2026-08-21 and reported as
passing: **A26–A30** — the close-confirm answered with [Cancel] leaving the window open and the log
without its shutdown line, [Discard] closing with the registry untouched, a two-Scope [Save]
applying User first, a Save whose backup step fails leaving the window open on the failed tab with
the reason spoken, and File → Exit and Alt+F4 reaching the same dialog as the title bar's [X] — and
**L3–L7**: the window reopening where it was left, maximised reopening maximised, a remembered place
that no longer exists falling back to centred, a hand edit's unknown fields surviving the rewrite,
the Read-only Data run that still reads its geometry and writes nothing back, and a minimised close
that leaves the remembered place alone.

That closes the gap this ticket's own verification could not. Everything recorded above was
*measured* — window rectangles through `GetWindowRect`, the dialog's title and buttons through the
window list, the enabled states off the live menu bar, the Banner's text and the focused control's
class cross-process — and a window that is in the right place and a screen reader that says what
just happened are different claims. Only the second one is what this application is for.

**A28's elevated half still has no in-app route.** Section C elevates through Tools → Restart as
Administrator, which impl ticket 17 builds; until it exists the only way to a two-Scope Save is
launching the exe elevated by hand, and an *installed* NVDA — a portable one being deaf to elevated
windows. However it was reached this time, the release pass will reach it through the Checklist's
own route.

This is history, not a substitute for the release pass: §10.2 wants a filled copy naming the NVDA
used, produced before every release, and that copy is ticket 18's.
