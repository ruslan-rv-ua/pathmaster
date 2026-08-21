# 14 — Backups tab and Restore

**Spec:** [spec §8 (FR-backup-ui), §15 (Open Backups Folder)](../../pathmaster-v0-1-0/spec.md) · ADR-0006

**What to build:** The Backups tab lists every Snapshot on disk and Restore brings one back — into the Working Copy as one ordinary, undoable Checkpoint, never straight to the registry. Demoable with hand-placed Snapshot files before Apply exists in a build, and with real ones after.

**Blocked by:** 08 (UI shell + Sessions), 10 (schema + validation).

**Status:** resolved

- [x] The Backups tab lists Snapshots: date/time, Scope, entry count; files failing the two-layer validation show `[Corrupted]` as passive list text — never an Announcement — with Restore disabled per-row; foreign files (wrong name pattern, `.tmp`) are silently invisible
- [x] Restore loads the chosen Snapshot's Entries and Value Type into the target Scope's Working Copy as one ordinary Checkpoint — it never writes the registry directly; no confirm dialog (undo is the safety net)
- [x] After Restore the target Scope's tab is activated with focus on the restored list, so the operation is heard through focus; undo of a Restore announces "Undone: Restore snapshot"
- [x] Restore to a non-writable Session (System unelevated; Read-only Data) is a disabled control that reads as disabled
- [x] Tools → Open Backups Folder opens the directory via a shell invocation, not a file dialog
- [x] New strings in the Catalogue with Ukrainian translations

## Comments

Implemented 2026-08-21 on `feature/backups-tab-and-restore`.

`pathmaster-core` gains **`backups`** — a Snapshot's name married to what reading its file turned
out to be — and `pathmaster-platform`'s `snapshots` gains the two questions the tab asks the
filesystem: read them all, and show the user where they live. The binary gains `ui/backups_page`
and the four lines of wiring that make Restore an edit like any other. Twenty new tests, none of
which link wxWidgets.

**A `SnapshotFile` is a name plus a verdict, and that pairing is the whole module.** The name is the
one part of a Corrupted Snapshot that still speaks, so a file that fails validation is still dated
and still shows its Scope; what it has is nothing to restore. That makes `SnapshotFile::restores()`
— `Option<(&[String], ValueType)>` — answer both of the tab's questions at once: what Restore loads,
and whether the button is worth anything. `None` *is* Corrupted, so the two cannot drift.

**Four rules the ticket left open, all four now in spec §8:**

- **`[Corrupted]` stands in the entry-count column**, where a count would. It answers the same
  question for a file that cannot be read — how many Entries would restoring load — and a fourth
  column would be a fourth heading saying the same thing. §8 names three columns; it now has three.
- **A file that cannot be *read* is Corrupted too.** Unparsable JSON, a mistyped field and a file
  the OS will not open are one thing to the person on that row. Passing it over instead would make
  the list disagree with the directory, where the file still occupies its Scope's rotation budget.
- **Newest first** — the reverse of `snapshot::listing`'s order, which is rotation's. Rotation wants
  the oldest; someone restoring wants the backup they took last. Both are the one `SnapshotName`
  ordering, so the suffix separating two Snapshots of a single second is read as the number it is
  at either end (`-10` after `-2`, tested).
- **Restoring an Absent Snapshot loads no Entries, typed `REG_EXPAND_SZ`**, and its count column
  reads `0`. An Absent Scope recorded no Value Type, and a Working Copy has no Absent state to
  restore into — so the answer is the one a Session loaded from an Absent Scope already takes. That
  rule now has one home, `session::ABSENT_VALUE_TYPE`, and `Session::new`'s decode and
  `Row::restores` are its two callers.

**Restore is a button and nothing else, deliberately.** §15 gives it no menu item and no
accelerator — "the Backups tab covers it" — so it is *not* a `Command`. Putting it in that enum
would have cost three generalisations to hold one command sharing nothing with the other ten:
`menu()` would return an `Option` in the one `match` whose exhaustiveness ticket 13 fixed,
`button_label()` would need a notion of *which page* a button belongs to (today every command with
a label lands on both Scope tabs), and `enabled()` would have to take a row. It has exactly one
route, so the rule that enum exists to enforce — one answer to "what is available", because a
shortcut can only live on a menu item's label — has nothing to enforce for it.

**Open Backups Folder is a `Command`, and it is why `enabled` grew an `Availability`.** It is the
first command that is not about a Scope: it is available on the Backups tab, where there is no
Session at all, and its answer comes from the Run. `Availability { session, data_dir }` is what a
command's availability is decided from, and tickets 16 and 17 add their Tools items to the same
shape. A Scope page's buttons are synced against **that page's** Session rather than the active
one, which is what the old signature already did and what the struct now says out loud.

**Two smaller decisions worth naming.**

- **The directory is read when the tab is activated, not held.** The other instance writes
  Snapshots into it too — two instances are a designed state — and this tab is the only place they
  are shown. A rebuild leaves focus exactly where it is: nothing here is an operation, so there is
  no row an operation points at.
- **`ensure_folder` creates `data\backups\` when it can** and falls back to the Data Directory
  when it cannot. That is not a side effect smuggled onto a menu item: it is the directory this
  application writes its own backups into, and the next Apply creates it anyway. What the fallback
  buys is that a menu item reading as available opens *something*.

**One silence, deliberately.** A backups directory that cannot be read lists as no Snapshots and
says nothing about it — the Announcement catalogue is closed at seven, none of them is about a
list, and the only run that reaches it is one whose Data Directory has gone out from under it. It
is the same silence `refresh` keeps for the same reason.

**Three columns and no new pixel constant.** §12 D2 gives the application exactly one deliberate
pixel constant and one explicit `FromDIP` call, both spent on the Scope list's Status column. These
three are sized with `wxLIST_AUTOSIZE_USEHEADER`, which measures content and header and — on the
last column — takes whatever width is left. `focused_row` moved out of `ScopePage` into `ui::list`
on the way, because it is one rule with a subtraction in it that must not happen twice.

### Verified live, in Ukrainian, against hand-placed Snapshots

A portable copy of the debug build in a scratch directory, its `data\backups\` holding eight files:
two valid User Snapshots of one second (suffixed and not), a valid User Snapshot of three Entries,
a valid System Snapshot of an Absent Scope, one file of unparsable JSON, one that parses with
`entries: [42]`, a `notes.txt` and a `.json.999.tmp`.

The tab listed **six** rows, newest first — «Дата й час» / «Область» / «Записи» — with
`2026-08-21 10:00:00` twice in suffix order, «[Пошкоджено]» on both bad files, `0` on the Absent
one, and neither the `notes.txt` nor the `.tmp` anywhere. Arrowing down the six, «Відновити» read
enabled, DISABLED, DISABLED, enabled, DISABLED — valid User, the two Corrupted, valid User, and the
valid **System** one, which an unelevated run cannot write. That is all four of this ticket's
enabled-state rules in one pass.

Restore on the three-Entry User Snapshot opened no dialog, activated the User tab, put its three
Entries in the list with focus on the first, spoke «PATH користувача: 3 записи», and left
«Застосувати» and «Відхилити зміни» enabled — a dirty Session, and a registry nothing had touched.
Ctrl+Z answered «Скасовано: Відновлення знімка» and the Session went clean again; Ctrl+Y,
«Повторено: Відновлення знімка».

Tools → Open Backups Folder («Інструменти(T)») opened Explorer at exactly
`…\live\data\backups` — confirmed through the shell's own window list, not from the screen. And a
Snapshot written into the directory while the app was running appeared at the top of the list the
next time the tab was activated.

### The review

Both axes ran against `a117d8e`, the last of ticket 20. **Spec** confirmed all six checkboxes and
walked each of the seven behaviours most worth doubting — that `[Corrupted]` is composed only into a
column and never reaches `announce()`, that foreign files are filtered by name before a file is ever
opened, that `Session::restore` touches nothing but the Working Copy, and that Read-only Data both
lists and refuses. **Standards** read the diff against `CONTEXT.md`, the ADRs and the borrow rule.

Seven findings applied:

- **`Row` was a word the glossary reserves.** `CONTEXT.md`'s **Entry** puts *Row* on its `_Avoid_`
  list, and a `pub` type in the pure core is the strongest form of that drift — ticket 20's review
  renamed `Startup` for exactly this reason. It is **`SnapshotFile`** now, which needs no new
  glossary concept: it is the file a Snapshot lives in, plus what reading it gave.
  `backups::rows` is `newest_first`, which names the rule it exists for, and
  `Catalogue::backup_row` is `snapshot_columns`.
- **The `App` doc comment had gone stale**, and it is the repo's stated home of the "no borrow across
  a call that runs someone else's code" rule. It said *two* kinds of call can re-enter; this ticket
  added two more, and unlike `render`'s these are **bound**: `BackupsPage::show` rebuilds a list under
  a live `on_item_focused` whose handler reads every Session *and* the page's own cell, and
  `Notebook::set_selection` runs the page-changed handler synchronously. Both were already safe —
  `show` replaces its cell last, `restore` copies out before it borrows — and the comment now says so,
  because that comment is what the next author checks.
- **`Command::enabled` wrote `available.data_dir` twice**, the second arm unreachable. It is one
  `match` over two kinds of command now — the one that answers to the Run, and the ten that answer to
  a Scope through `over()` — still exhaustive, so the next command added has to say which it is.
- **`restore_target` and `restore_payload` shared their two opening lines.** Both go through one
  private `focused()` now, which is also the only place the page's cell is borrowed for reading —
  worth more than the deduplication, since both callers run where a list event can arrive.
- **`folder_to_open` read as a pure query and ran `create_dir_all`.** It is `ensure_folder`, and its
  doc leads with the creation.
- **`CONTEXT.md`'s Corrupted was not amended**, though this ticket widened it to files the OS will not
  open — ticket 10 set the precedent by amending the glossary alongside §8. Done, and it now also
  says what Corrupted is *not*: a file that never claimed to be a Snapshot is invisible instead.
- **Nothing covered the Read-only Data half of checkbox 4.** The System-unelevated half is exercised
  live and by Checklist step 23; the other half is now step 25, continuing step 17's staging — the
  list still shows every Snapshot and Restore is unavailable on all of them.

**Three findings declined, with the reasoning, and one recorded rather than fixed.**

- **`(entries, ValueType)` as a Data Clump** — the pair the glossary already calls a **Working Copy**.
  True, but that is `Session::restore`'s signature, which predates this ticket; giving it a type is a
  change to the editing model, not to this tab.
- **`ScopePage::focused_row` is now a one-line forwarder** to `ui::list`. Cutting it would trade a
  Middle Man for a Message Chain — three call sites in the window would reach `tab.page.list`
  themselves — so it stays.
- **`sync_button` could disable Restore while Restore holds focus**, the case `ScopePage::rescue_focus`
  exists for. Traced, and unreachable: the only thing that changes the answer is which row is focused,
  and every way of reaching another row moves focus onto the list first. The one path that clears the
  list without the user touching it — `show()` — runs only from the page-changed handler, which fires
  after focus has already left the page, and it disables the button *before* wx restores focus into
  it, which Windows will not do to a disabled control. Named here so it is not re-raised from scratch.
- **Restoring an Absent Snapshot is a lossy round trip.** The sharpest finding: design ticket 14 added
  `absent: true` because a Snapshot "needs to distinguish an Absent Scope from a present-but-empty
  one… otherwise restore cannot reproduce what it saved" — and restoring one produces an empty Working
  Copy, which Apply writes as a present, empty value. It is the same loss *reading* an Absent Scope
  already takes (spec §4), so it is not this tab's to introduce or to close: closing it needs an Absent
  state in the Working Copy and a delete path in Apply, neither of which v0.1.0 has. **Recorded rather
  than papered over** — spec §8 and ADR-0006's Consequences both now say the file keeps the
  distinction and the Restore does not.

**Open Backups Folder's create-and-fall-back was flagged as scope creep and kept.** §15 asked for "a
shell invocation, not a file dialog" and left open what happens when the folder is not there; something
must. Of the four answers — do nothing, disable the item on a filesystem check in every sync, create,
or create and fall back — the last is the only one where a menu item that reads as available always
opens something, and the fallback fires only when there are no Snapshots at all. It is now written into
§15 rather than living in the code alone, which is the half of the finding that was right.
