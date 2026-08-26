# Fix Issues dialog contract

Type: grilling
Status: resolved (2026-08-26)
Blocked by: 01, 07

## Question

FR-fix-issues: a preview dialog listing every current Issue with a checkbox, apply the checked fixes
as one operation. Widget facts (01: list checkboxes) and the severity/filter decision (07: the
vocabulary the dialog speaks) are in. Specify:

- **What is fixable?** The PRD's proposed action is only ever "delete the entry". Of the six types:
  Duplicate, Empty, Missing → delete; but Relative and Quoted have *repairs* (qualify? strip
  quotes?) — does v0.2.0 offer repairs, or deletions only? Quoted's fix is trivial
  (strip the quotes) and was called "trivial fix" in v0.1.0's spec — decide whether that trivial
  fix finally exists here.
- Default check state: PRD says duplicates and empties on, missing-on-network/variable roots off,
  missing-on-fixed-local on. v0.1.0 never flags network-rooted entries at all — reconcile the
  defaults with what diagnostics actually produce.
- One user-visible operation → **one Checkpoint** (v0.1.0's undo law) covering every checked fix
  across… one Scope or both? The dialog lists Issues from which Working Copies — active Scope only,
  or both (cross-scope duplicates span them)?
- Scope-level Issues (overlength) are not per-entry and have no checkbox fix — confirm the dialog
  simply excludes them, and say so where?
- Dialog anatomy through the Catalogue: list columns, check-state announcement (01 says what NVDA
  gets for free), the apply button's label, what is announced on apply (count wording, both
  languages), and the automatic re-diagnosis after.
- Enablement: the command is live only when Issues exist (PRD) — where that state is surfaced given
  no toolbar (menu-item enablement is NVDA-readable for free).
- The stale-pass hazard: Issues are recomputed async; the dialog must not apply fixes computed
  against an older Working Copy. State the staleness rule it obeys.

## Resolution (2026-08-26)

Researched first: [research/09-fix-issues-best-practices.md](../research/09-fix-issues-best-practices.md),
per the map's standing directive 7. Decisions:

1. **Fixable = three deletions + one repair.** Missing, Duplicate and Empty propose "delete the
   entry"; Quoted's trivial fix finally exists: **remove every `"` in the Entry** — not just a
   surrounding pair. `"` is illegal in Windows file names, so no quote can be path content, and
   v0.1.0's own measurement ("the quoted spelling is dead" for `CreateProcessW`/PowerShell/`where`/
   Python) covers interior quotes too — the repair passes the one respected auto-fix criterion
   (ESLint: guaranteed not to change behaviour). **Relative gets no repair**: qualification needs a
   base directory only the user knows — the depends-on-intent case every auto-fixer refuses
   (RapidEE, the closest product precedent, highlights and leaves it manual).
2. **One row per Entry, one computed action** (the PRD's row-per-problem is amended). The Issue
   column carries the comma-joined types — the main list's Status string; the action is
   **Delete entry** when any of Missing/Duplicate/Empty is flagged (deletion cures Quoted too),
   else **Remove quotes**. No repeated path rows for NVDA, no conflicting checks on one Entry;
   bespoke intents ("keep the duplicate but fix its quotes") belong to Edit — Fix Issues is a bulk
   convenience, not an editor.
3. **Relative-only Entries are excluded.** A row that can fix nothing is noise in a fix dialog
   (the Disk Cleanup model: unfixable ≠ listable). The delta-spec names the exclusion explicitly
   and points at Edit and the Filter's `Relative` state as where those Entries are found and cured.
4. **Active Scope only; one Checkpoint** in that Session's history, operation name "Fixing issues"
   (uk «Виправлення проблем»). The undo machinery is per-Session by construction — a cross-scope
   operation would mean two Checkpoints in two stacks, undoable only in halves; and cross-scope
   duplicates always flag the **User** copy (System evaluates first, spec §7 FR-diag-duplicate), so
   the dialog opened on the User tab lists them. Every adjacent v0.1.0 command (Cancel, Refresh,
   Restore) is already per-Scope; Disk Cleanup is likewise per-drive.
5. **Defaults — the Disk Cleanup principle** (guaranteed-safe pre-checked, "might still be wanted"
   not). **ON**: Remove quotes; Delete via Duplicate or Empty (the canonical copy survives / the
   Entry is empty anyway — deliberately wins over `%VAR%`); Delete via Missing on a `DriveType=
   Fixed` local root with no `%VAR%` in the raw text. **OFF**: Delete via Missing when the raw text
   contains `%VAR%` (dead for this process, possibly alive for the ones that matter) or the root is
   a non-Fixed drive (an unplugged stick comes back). The PRD's network row reconciles to
   **nothing**: network roots are never probed and never flag (spec §7 FR-diag-missing).
6. **Material and columns**: native report-mode ListCtrl with `LVS_EX_CHECKBOXES` through 01's
   raw-`LVM_*` hatch; check state is read once, by `LVM_GETITEMSTATE` at apply time — check
   *events* (unreceivable through wxdragon) are never needed. Columns **# / Path / Issue /
   Action**, `#` the original position (03's convention), the checkbox riding column 0's state
   image. **Path is always the raw text**, whatever the Expansion Mode — the dialog shows what
   will be deleted or repaired, the quotes repair is only visible raw, and decision 5's `%VAR%`
   rule must be visible in the row it judges. NVDA reading of checkbox state → ticket 16.
7. **Buttons and focus**: **[Fix selected] [Cancel]** (uk «Виправити позначені» / «Скасувати») —
   "Apply" is banned from the label: it is this product's reserved word for the registry write,
   and Windows convention reads Apply as commit-and-stay besides. The title names the Scope
   ("Fix issues — User PATH"). Initial focus on the first row (the work is reviewing checks);
   [Cancel] keeps default and Escape — 08's precedent of a work-inside dialog ceding only focus
   from §10's discipline. No Select-all/Clear-all (Disk Cleanup and VS Preview Changes have none;
   Space on a row is the whole mechanism). Exact msgids, mnemonics, accelerator → assembly (15).
8. **The Announcement catalogue grows to twelve**: "Fixed {n} entries" (uk «Виправлено {n}
   записів»), plural by {n}, counting applied rows — after decision 2 a row *is* an Entry, and one
   number is honest where counting co-occurring types would not be. Order is the v0.1.0 law: focus
   first, Announcement second — last heard is the summary. **Zero rows checked at activation =
   Cancel**: the dialog closes with no Checkpoint and no Announcement (the button is never
   dynamically disabled — no check events exist to drive it). Post-close focus follows Delete's
   law: same index, clamped to the new last row; the Checkpoint's hint is the first surviving
   neighbour. **Re-diagnosis is the existing §7 law** — recompute after every Working-Copy
   change — so the PRD's "diagnostics run after" costs the delta-spec one cross-reference.
9. **Enablement**: **Edit → "Fix issues…"** (Working-Copy commands live in Edit, 02; the PRD's
   toolbar button is already a recorded deviation), disabled on the Backups tab. Enabled iff the
   active Scope has **≥ 1 fixable row** (decisions 2–3 — not merely "Issues exist": all-Relative
   or Over-length-only would open empty) **and** its Session is writable (the Restore precedent,
   spec §8 — System unelevated and Read-only Data disable it). Menu enablement is the only
   surface — NVDA-readable for free, computed when the menu opens; no separate indicator, because
   the Status column and StatusBar's "({k} issues)" already say there is work.
10. **The staleness rule, two halves.** *(a) At open:* every diagnostic pass is stamped with the
    Working-Copy generation it read; the dialog builds only from a pass whose stamp equals the
    current generation — if none exists yet, the command waits for the outstanding pass (< 1 s
    budget, spec §7; no spinner, no menu flicker). *(b) After open:* the dialog is modal and the
    Working Copy mutates only through the UI, so modality is the fence; apply resolves checked
    rows to Entries **by id** (ids survive Move/Edit), never by index, and asserts the generation
    unchanged — an invariant named so no implementation unmakes it silently (e.g. by going
    modeless). This is LSP's versioned-edit discipline (the cure VS Code names for its stale
    Problems-window fixes) minus the concurrent-editing half modality removes.
11. **Over-length is excluded entirely** — no row, no reminder text in the dialog. The delta-spec
    states it beside decision 3's exclusion, 07's gesture ("Over-length is Scope-level and takes
    no part"), and points at its whole existing surface: the StatusBar length field and the two
    Apply gates.

Downstream: NVDA listview-checkbox proof → ticket 16 (item 7); dialog strings, accelerator,
mnemonics and Catalogue numbering → assembly (15); `CONTEXT.md` gains **Fix Issues**; no
`settings.json` field — nothing about the dialog persists.
