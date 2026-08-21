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

**A row is a name plus a verdict, and that pairing is the whole module.** The name is the one part
of a Corrupted Snapshot that still speaks, so a file that fails validation is still dated and still
shows its Scope; what it has is nothing to restore. That makes `Row::restores()` — `Option<(&[String],
ValueType)>` — answer both of the tab's questions at once: what Restore loads, and whether the
button is worth anything. `None` *is* Corrupted, so the two cannot drift.

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
- **`folder_to_open` creates `data\backups\` when it can** and falls back to the Data Directory
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
