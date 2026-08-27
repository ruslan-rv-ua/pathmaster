# 09 — Fix Issues dialog

**Spec:** [delta-spec §7, §13 (item 12), §14 (strings)](../../pathmaster-v0-2-0/spec.md)

**What to build:** Edit → "Fix Issues…" opens a modal, per-Scope repair surface: one checkbox row per fixable Entry with one computed action (Delete entry, or Remove quotes), Disk-Cleanup defaults, and one Checkpoint on [Fix selected] — "Fixed {n} entries" spoken after focus lands. Nothing reaches the registry; only the Working Copy changes.

**Blocked by:** 01 (retrofit), 02 (the `#` column meaning the dialog reuses).

**Status:** done — driven live in both languages; the **Space toggle alone could not be re-measured in this session** (no synthetic keyboard reaches the app here) and stands on wayfinder ticket 16 probe 7, see Comments

- [x] Fixable = three deletions + one repair: Missing, Duplicate, Empty propose **Delete entry**; Quoted proposes **Remove quotes — every `"` in the Entry**; Relative gets no repair and Relative-only Entries are excluded; Over-length is excluded entirely — no row, no reminder text
- [x] One row per Entry, one computed action: the Issue column carries the comma-joined Status string; the action is Delete entry when any of Missing/Duplicate/Empty is flagged (deletion cures Quoted too), else Remove quotes
- [x] Columns # / Path / Issue / Action; `#` is the original position (§2.1 convention); Path is always the raw text, whatever the Expansion Mode
- [x] Checkboxes are native `LVS_EX_CHECKBOXES` through the raw-`LVM_*` hatch; check state is read once, by `LVM_GETITEMSTATE` at apply time — no check events; Space toggles with the change announced in place (rides the native path, measured in wayfinder ticket 16)
- [x] Defaults — the Disk Cleanup principle: ON for Remove quotes, Delete via Duplicate or Empty, and Delete via Missing on a `DriveType=Fixed` local root with no `%VAR%` in the raw text; OFF for Delete via Missing when the raw text contains `%VAR%` or the root is a non-Fixed drive; network roots are never probed, never flag, and have no row
- [x] Buttons [Fix selected] [Cancel] — "Apply" nowhere in a label; title names the Scope; initial focus on the first row; [Cancel] keeps default and Escape; no Select-all/Clear-all; zero rows checked at activation = Cancel — no Checkpoint, no Announcement, button never dynamically disabled
- [x] Applying = one Checkpoint in the active Session, operation name "Fixing issues" («Виправлення проблем»); focus first (Delete's law: same index clamped to the new last row), then Announcement 12 "Fixed {n} entries", plural by {n}, both languages; one Ctrl+Z restores every fixed Entry; re-diagnosis follows the existing recompute-after-every-change law
- [x] Enablement: Edit → "Fix Issues…" (after the Move pair, before the history block, **no accelerator**), disabled on the Backups tab; enabled iff the active Scope has ≥ 1 fixable row AND its Session is writable (System unelevated and Read-only Data disable it); menu enablement is the only indicator
- [x] Staleness, both halves: at open, the dialog builds only from a diagnostic pass whose generation stamp equals the current Working-Copy generation — if none exists yet, the command waits for the outstanding pass (< 1 s budget, no spinner); after open, modality is the fence — apply resolves checked rows to Entries by id, never by index, and asserts the generation unchanged
- [x] Catalogue strings shipped in both languages: the two titles, "Fix selected", "Issue", "Action", "Remove quotes", the "Fix Issues…" menu item; "Delete entry" reuses Announcement 4's operation msgid; Cancel, Path, `#` reuse existing msgids; i18n gate green
- [x] No `settings.json` field — nothing about the dialog persists

## Comments

**2026-08-27 (implementation)** — The rulebook is a new pure module, `pathmaster_core::fix`, and like
`tree` it **takes no Expansion Mode and cannot be given one** — which is §7's "the Path column is
always the raw text" said structurally. `Plan::of` takes `(EntryId, &str, &[Issue])` per Entry of the
whole Working Copy plus an injected `DriveTypes` and hands back the rows the dialog fills from;
`fix::repair` takes the rows the user checked and carries them out on a Session. **The enum is
`Action`, not `Repair`** — `CONTEXT.md` calls it "one proposed action" and the column header is
"Action", and §7 spends "repair" on the *quote* half ("three deletions + one repair"), so naming the
whole enum after one of its two arms would be a second vocabulary. 31 tests fix every rule §7 states.

**Everything a test can hold lives in core** (ADR-0007). `fix::repair` owns the apply: the batch, the
by-identity resolution, the count Announcement 12 speaks, the Checkpoint's focus hint, and the
non-writable Session. `App::fix_issues` is left with what only a window can do — the staleness, the
dialog, the focus and the voice — which is the tier rule applied rather than quoted.

**Five readings §7 left to implementation, each with its reason.**

1. **"Delete via Missing" means Missing *alone*.** §7 lists Duplicate and Empty as ON and Missing as
   conditional, and says nothing about an Entry flagged both. A second copy of an Entry is redundant
   whether or not the path it spells is on the disk today, so the cautious rule is read as being about
   a deletion the Missing flag alone earns: `Missing, Duplicate` arrives **checked**. Reading it the
   other way would leave the commonest safe row — a duplicate of a stale path — unchecked, which is
   the opposite of the Disk-Cleanup principle it comes from.
2. **`DriveType=Fixed` is a trait of its own, not a third `Filesystem` method.** The diagnostic
   rulebook asks only whether a root may be *probed* (`RootKind::Network`); it never asks whether a
   disk is fixed, and adding the question to its trait would make every fake in
   `core/tests/diagnostics.rs` implement a method the rules never call. `fix::DriveTypes` is one
   question with one implementor, and `LocalFilesystem` answers both through the one private
   `drive_type` helper — so `root_kind` and `is_fixed_root` cannot come to disagree about a drive.
   Everything with no drive letter to classify answers `false`: a UNC path, a device-namespace path,
   an unresolved `%VAR%`, a relative name. Each of those is already OFF for its own reason, so the
   answer is never load-bearing there — it is simply the honest one.
3. **The generation stamp already existed.** §7 asks for "a pass stamped with the Working-Copy
   generation it read"; `diagnostics::Worker` has carried exactly that counter since ticket 12 —
   `sent` is bumped by every request, and every Working-Copy change requests one — so
   `Worker::outstanding()` **is** the comparison, and `Pump::outstanding()` is all this ticket added.
   A second counter beside it could only be a second answer to the same question. It also means the
   at-open half and the after-open assert are literally the same expression, which is what §7's
   "an invariant named so no implementation unmakes it silently" wanted.
4. **The wait blocks the UI thread rather than pumping it.** §7 says "< 1 s budget, no spinner", and
   the pass crosses back over an `mpsc` channel, so nothing has to be pumped for it to land: the
   command polls `collect_pass()` every 10 ms for at most a second. Pumping messages instead would
   re-enter the very commands this is deciding on behalf of. A budget that expires opens nothing at
   all — the honest reading of "builds only from a pass whose stamp equals the current generation".
   Measured unreachable in practice: a pass over 45 entries lands inside one poll.
5. **The Checkpoint's hint is Delete's law asked of a batch.** The contract fixes it — "the
   Checkpoint's hint is the first surviving neighbour" — and `fix::repair` reads the position of the
   **first** chosen row before anything moves, then hands back whichever Entry is standing there when
   the batch is done, clamped to the new last row. Over a Scope the repair emptied it answers `None`,
   which is the one case with no row to land on. So `Undone: Fixing issues` lands on the Entry that
   took the first repaired row's place — a row that exists in both states, rather than a position
   whose meaning changed underneath it.

**The generation assert and the expired budget are both silent, deliberately.** Neither can be spoken:
§13 closes the Announcement set at fourteen and gives this command exactly one item, the summary. Both
are guards on states that cannot occur — nothing may request a pass while a modal is up, and a pass
over a real `PATH` lands inside one 10 ms poll — so what they buy is that a *future* change which
broke either invariant would do nothing rather than something wrong. Menu enablement remains the only
indicator §7 gives this command, in the good case and in these.

**A pass landing inside that wait can also remove the last fixable row**, so the plan's emptiness is
re-checked after the wait rather than trusted from the menu's enablement. The menu is synced from the
*last completed* pass, which is the same clock the Status column runs on; the wait moves that clock
forward, and a command that opened an empty dialog because the menu had been right a moment ago would
be the one thing §7's enablement rule exists to prevent.

**The plan's inputs are copied out of the scoped access before it is built**, not read through it:
building a plan asks the machine which kind of drive a root is, and an OS call has no more business
inside a `Scoped::with` closure than a dialog does — ADR-0011's own owned-values-out rule, applied at
the same seam. `ScopeTab::fixable`, which the menu re-syncs after every operation, asks the machine
nothing and stays inside.

**`from_dip` moved to `ui::list`.** It was `scope_page`'s private helper and this dialog needed the
same conversion — `ListCtrl` column widths are the one size wxdragon does *not* scale across the FFI
boundary. `ui::list` already existed for exactly this ("the one question both tabs ask a `ListCtrl`"),
so it is one rule with one home rather than two copies, and both column-width constants now read
through it.

**The hatch is thirty lines and all of it is in one module.** `wxdragon` exposes neither
`LVS_EX_CHECKBOXES` nor a check event, so `ui::fix_dialog` sends `LVM_SETEXTENDEDLISTVIEWSTYLE`,
`LVM_SETITEMSTATE` and `LVM_GETITEMSTATE` to the list's own `SysListView32` handle — **in-process**,
which is the whole of why the pointer-carrying `LVM_SETITEMSTATE` is safe here and fatal from a probe.
The style is set **after** the items exist: comctl32 gives an item its state image when the style
arrives, so enabling it first would leave rows whose box is drawn from a state nobody wrote. The
states are read exactly once, after `door::show` returns and before `destroy` — the moment the user's
answer exists anywhere.

**Cancel is the default button, and the first build had it the wrong way round.** §7 and the contract
both say "[Cancel] keeps default **and** Escape", which inverts every other dialog in the application
— ticket 07's Tree View makes its commit button the default, and this one must not. It is the one
dialog whose commit deletes Entries in bulk while the user's whole gesture is Space on a row, so Enter
is bound to the way out and [Fix selected] costs a Tab or a click. Caught by the spec review of this
change; the ring is on Cancel in both screenshots below.

**Live verification** (staged copies with private Data Directories; the Working Copy staged by
hand-placing a Snapshot in `data\backups\` and pressing Restore — the Add dialog **rejects a `"`
outright and refuses empty text**, so neither a Quoted nor an Empty Entry can be put in front of this
dialog any other way; menus, widget tree, checkbox states and the Banner all read cross-process;
nothing was ever applied):

- Staged Working Copy, in order: `C:\Windows\PathMasterProbeA` (Missing) · `"C:\Program Files"`
  (Quoted) · `%PATHMASTER_NOPE%\bin` (Missing, `%VAR%`) · `C:\Windows\PathMasterProbeA`
  (Missing + Duplicate) · `   ` (Empty) · `tools\bin` (Relative) · `C:\Users` (healthy) ·
  `Q:\pathmaster-probe` (Missing on an unmounted letter).
- **The plan** — six rows, `#` = 1, 2, 3, 4, 5, 8: the Relative-only Entry and the healthy one have no
  row, and the numbers are the Working Copy's positions rather than the dialog's. Path is the raw text
  in every row, quotes and `%VAR%` visible. Issue reads `Missing, Duplicate` on row 4 — the Status
  column's own join. Action reads `Delete entry` on five rows and `Remove quotes` on the Quoted one.
- **The defaults** — `True, True, False, True, True, False`: the `%VAR%`-carrying Missing row and the
  unmounted-drive one arrive unchecked, the `Missing + Duplicate` row arrives checked. The one that
  reads against §7's literal OFF clause is that last one, so it is worth stating what it costs: two
  copies of one `%VAR%` Entry give a **Missing** row (unchecked) and a **Missing, Duplicate** row
  (checked), so the checked default removes the redundant copy and keeps the one the user may still
  want. Reading it the other way would leave every duplicate of a stale path unchecked.
- **Menu** — `Fix &Issues…`, id 6007, index 6 in Edit, between `Mo&ve Down` and `&Undo`, **no
  accelerator**; greyed on the Backups tab with every other Edit item, and greyed on the **unelevated
  System tab** while `Co&py` and `Re&fresh` stay live beside it — the writability line, read exactly
  where §7 puts it.
- **Buttons** — `Fix selected` and `Cancel`, both always enabled; the **default ring is on Cancel**.
- **Apply** — [Fix selected] with the four defaults: Banner `Fixed 4 entries`, 8 rows → 5. One Ctrl+Z:
  `Undone: Fixing issues`, back to 8 — one Checkpoint for the whole apply. With **one** row checked
  (the quote repair alone): `Fixed 1 entry`, and 8 rows still, because a repair deletes nothing.
- **Zero checked = Cancel** — every box cleared, then [Fix selected]: the Banner keeps its previous
  text, the Working Copy keeps its 8 rows, and the dialog closes. Nothing to undo.
- **Re-diagnosis** — reopened after the quote repair, the dialog shows five rows: the repaired Entry
  is healthy and has none. The existing recompute-after-every-change law, and the staleness rule
  working, in one reading.
- **Ukrainian** — `Виправити проблеми(&I)…` (Latin mnemonic in parentheses, unique in the Edit menu
  beside A, E, D, P, M, V, U, R, C, F); title `Виправлення проблем — PATH користувача`; buttons
  `Виправити позначені` / `Скасувати`; columns `#` / `Шлях` / `Проблема` / `Дія`; cells `Прибрати
  лапки` and `Видалення запису`; Banner `Виправлено 4 записи`, `Скасовано: Виправлення проблем`.

**One consequence of §14 worth recording, not a defect.** The Action column's deletion cell **reuses
Announcement 4's `Delete entry` msgid**, which §14 requires — so in Ukrainian it reads
«Видалення запису», the verbal noun that composition needs («Скасовано: видалення запису»), beside
«Прибрати лапки», which §14 fixes as an imperative. A noun and an imperative under one «Дія» header.
Both strings are the spec's own; changing either would break the reuse §14 asks for or contradict the
Ukrainian it fixes, so the mix is recorded rather than resolved.

**Space could not be re-measured in this session, and is the one criterion standing on an earlier
measurement.** Synthetic keyboard input reaches nothing from this harness — neither `keybd_event` nor
`SendInput`, with the window confirmed foreground and `GetGUIThreadInfo` confirming the list holds the
focus; a Down arrow does not move the focused row of the **main window's own list** either, so the
finding is the harness's and not the dialog's (the standing rule: confirm against a known-good surface
first). What *was* measured is the half that matters for the code: the boxes are live and toggling one
by mouse changes what `LVM_GETITEMSTATE` reads back at apply time, so "the state the user leaves is
the state that is applied" is proven end to end. Space itself is comctl32's own `LVS_EX_CHECKBOXES`
behaviour with no code of ours in the path, measured against real NVDA in wayfinder ticket 16 probe 7,
which is what §7 cites. The Release Checklist's Fix Issues step (folded in by ticket 12) is where it
is proven again with NVDA at the keyboard.

**No `settings.json` field and no state kept**: a fresh `Plan` and a fresh `ListCtrl` per open, so
"nothing about the dialog persists" is a property of the code rather than a rule kept by hand.
