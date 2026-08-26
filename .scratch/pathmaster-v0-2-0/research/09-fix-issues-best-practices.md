# Research: Fix Issues dialog contract (supports ticket 09)

Web research gathered 2026-08-26, before grilling, per the map's standing directive 7. Structured as
recommendation-per-question with sources; "no direct guidance found" is stated where true. Count
wording, WCAG 4.1.3 status-message phrasing and menu check-state mechanics are cited from
[04-live-filter-best-practices.md](04-live-filter-best-practices.md),
[05-var-expansion-best-practices.md](05-var-expansion-best-practices.md) and
[07-filter-bar-best-practices.md](07-filter-bar-best-practices.md) rather than restated.

## Q1. Repairs, or deletions only?

**Recommendation: deletions for Missing / Duplicate / Empty, plus exactly one repair — Quoted →
remove the quotes — because it passes the only respected auto-fix criterion: guaranteed not to
change intended behaviour. Relative gets no repair: qualifying requires inventing a base directory
the tool cannot know — the textbook "depends on intent" case every auto-fixer refuses.**

- ESLint's core criterion is the cleanest statement in the field: core rules "only autofix problems
  if they can guarantee that the fixes will never break the code"; fixes that could alter behaviour
  become *suggestions* the user applies deliberately
  ([Custom Rules](https://eslint.org/docs/latest/extend/custom-rules),
  [issue #7873](https://github.com/eslint/eslint/issues/7873) — the unsafe-fix debate). ESLint has
  *removed* shipped autofixes on discovering they could change meaning
  ([PR #12157](https://github.com/eslint/eslint/pull/12157), `no-unsafe-negation`) — the precedent
  cuts both ways: a fix once offered is hard to retract.
- Quoted passes the criterion on v0.1.0's own measurement (spec §7, FR-diag-quoted): the quoted
  spelling is already dead for `CreateProcessW`/`SearchPathW`, PowerShell, `where`, Python —
  removing the quotes restores the one spelling that works everywhere and cannot break a consumer
  the quotes hadn't already broken. v0.1.0's spec itself calls it "silent breakage, trivial fix".
- Relative fails it: a qualification needs a base (`.` relative to what — the app's cwd? the
  user's profile? the directory it "obviously" meant?). No auto-fixer found that guesses a base;
  Rapid Environment Editor — the closest product precedent — *highlights* invalid entries in red
  and leaves the choice to "either fix it or delete the wrong item" manually
  ([rapidee.com, Path variable](https://www.rapidee.com/en/path-variable),
  [Cleanup Paths walkthrough](https://codeyarns.com/tech/2012-03-06-rapid-environment-editor-cleanup-paths.html) —
  its bulk "Cleanup paths" removes only empty and duplicate values, never repairs).
- The preview-with-checkboxes shape itself is well-precedented: Visual Studio's Preview Changes
  window lists each pending change with a checkbox and applies the checked subset
  ([Preview code changes](https://learn.microsoft.com/en-us/visualstudio/ide/preview-changes)).

## Q2. Default check state

**Recommendation: Disk Cleanup's principle — checked by default exactly the categories whose
removal is guaranteed-safe, unchecked anything that could destroy something the user still wants.
Applied to what v0.1.0's diagnostics actually produce: Duplicate, Empty and the Quoted repair (if
adopted) default on; Missing defaults on only for plainly-local roots; Missing on an Entry whose
root came through `%VAR%` defaults off; the PRD's "network-rooted Missing off" row reconciles to
nothing — v0.1.0 never probes network roots, so that category cannot exist.**

- Disk Cleanup ships with the safe categories pre-checked and the risky ones (Downloads,
  Recycle Bin) unchecked; the user opts *in* to the destructive rows
  ([tenforums tutorial on its check-state defaults](https://www.tenforums.com/tutorials/102375-check-uncheck-all-items-disk-cleanup-default-windows-10-a.html),
  [7datarecovery on which rows are safe](https://blog.7datarecovery.com/does-disk-cleanup-delete-files/)).
- Same split in lint land: `--fix` applies only the guaranteed-safe class by default; everything
  else waits for a deliberate act ([ESLint Custom Rules](https://eslint.org/docs/latest/extend/custom-rules)).
- Why %VAR%-rooted Missing is the off-by-default row: an undefined or differently-defined variable
  makes the Entry "missing" *for this process* while it may be alive for the processes that matter
  (v0.1.0 spec §7: expansion reads the *process* environment; an undefined `%VAR%` flags Missing
  naturally). That is exactly the "may still be wanted" shape Disk Cleanup leaves unchecked. The
  PRD itself grouped `%`-paths with network roots as default-off — the reconciliation keeps its
  intent and drops only the empty network row.

## Q3. One Scope or both; the one-Checkpoint law

**Recommendation: active Scope only. v0.1.0's undo machinery is per-Session (per-Scope) by
construction — one cross-scope operation would need two Checkpoints in two stacks and could only be
half-undone; every destructive-adjacent v0.1.0 command (Cancel, Refresh, Restore) already acts on
the active Scope only. Cross-scope duplicates are reachable anyway: the *later* copy carries the
flag (spec §7 FR-diag-duplicate — the User copy), so the dialog opened on the User tab lists them.
No direct external guidance found — this is the product's own law; the nearest precedent,
Disk Cleanup, is likewise scoped to one drive at a time.**

## Q4. Scope-level Issues are excluded

**Recommendation: yes — the dialog is defined over Entry-level Issues (checkbox = Entry-level
action), and Over-length already has a complete, separate surface: always-visible StatusBar length
field plus the two Apply gates (v0.1.0 spec §7 FR-diag-overlength). Ticket 07 set the precedent
("Over-length is Scope-level and takes no part" in the Filter). State the exclusion in the
delta-spec's Fix Issues section, as 07 did for the Filter — not in the dialog itself. No external
guidance found; note VS's Error List similarly mixes only listable, per-location items and leaves
build-level state elsewhere.**

## Q5. Anatomy: columns, check-state announcement, the button, the closing announcement

**Recommendation: native `LVS_EX_CHECKBOXES` report-mode list (the raw-`LVM_*` hatch from 01);
columns Path / Issue / Action (plus `#` for the original position, ticket 03's convention);
commit button a specific verb — NOT the PRD's "Apply Selected", because "Apply" is this product's
reserved word for writing the registry and these fixes only edit the Working Copy; on close, one
count announcement through the existing mechanism, and re-diagnosis is already automatic.**

- Check-state under NVDA: comctl32 exposes a `LVS_EX_CHECKBOXES` item's state via MSAA, and both
  NVDA failures found are non-native controls — CCleaner's DirectUI list
  ([nvaccess/nvda#6887](https://github.com/nvaccess/nvda/issues/6887), closed works-for-me /
  app-specific) and a web date-picker
  ([#7136](https://github.com/nvaccess/nvda/issues/7136), duplicate); a Win10 shell dialog case
  ([#5653](https://github.com/nvaccess/nvda/issues/5653)) is also not a listview. Nothing found
  against native SysListView32 checkboxes — same verdict as 05/07's menu items: expected to work,
  needs the live-NVDA proof (→ ticket 16). The sharp edge stays 01's: check *events* are
  unreceivable through wxdragon — read state via `LVM_GETITEMSTATE` at apply time instead of
  tracking toggles.
- Button label: "start commit button labels with a verb… specific labels that make sense on their
  own" ([Command Buttons, Windows UX Guide](https://learn.microsoft.com/en-us/windows/win32/uxguide/ctrl-command-buttons));
  and in Windows convention "Apply" specifically means "commit but keep the dialog open" on
  property sheets ([same guide](https://learn.microsoft.com/en-us/windows/win32/uxguide/ctrl-command-buttons)) —
  two reasons the PRD's "Apply Selected" is the wrong word here. "Fix selected" says what happens.
- Dialog discipline is already law: negative button gets default, initial focus and Escape
  (v0.1.0 §10 via §6) — the fix button must be *activated*, never defaulted into, consistent with
  the checked-rows-are-destructive reading.
- The closing announcement rides WCAG 4.1.3 status-message phrasing already sourced in 04 Q2/05 Q2;
  the Announcement catalogue is closed, so the new message is a decided addition, not a slip-in.
  Re-diagnosis needs no new rule: Issues recompute after any Working-Copy change (spec §7).

## Q6. Where enablement lives

**Recommendation: menu-item enablement only — Edit → Fix Issues… enabled when the active Scope has
at least one Entry-level Issue AND its Session is writable (the Restore precedent: "Restore to a
non-writable Session… is a disabled control", spec §8). The PRD's toolbar button is already a
recorded deviation (ticket 02); a menu item's disabled state is NVDA-readable for free (map's
standing fact; PRD FR-menubar itself demands it). No external guidance beyond the Windows-UX
truism that a command that can do nothing should be disabled, not hidden
([UX checklist](https://learn.microsoft.com/en-us/windows/win32/uxguide/top-violations)).**

## Q7. The staleness rule

**Recommendation: two halves. (a) At open: the dialog may only be built from a diagnostic pass
computed over the Working Copy's *current* generation — if a pass is outstanding, wait for it (or
compute synchronously at open); never open over last-pass leftovers. (b) After open: the dialog is
modal and every Working-Copy mutation is UI-driven, so modality itself is the fence — nothing can
move under it; at apply, resolve checked rows to Entries by id, not by index. This is the versioned-
edit discipline from LSP, minus the hard half (concurrent editing) which modality removes.**

- The documented failure is exactly the ticket's hazard: VS Code's Problems-window code actions
  cache an edit computed against an older document version; applying the stale fix "generates
  invalid code", and the named cure is versioned document edits — refuse an edit whose version
  no longer matches ([microsoft/vscode#148723](https://github.com/microsoft/vscode/issues/148723)).
- PathMaster's async pass already has the pieces: one worker, results drained by a timer, runs
  after every Working-Copy change (spec §7 FR-diag-async) — tagging a pass with the generation it
  read and comparing at dialog-open is the whole rule. No external source needed for the modal
  half; it is an architecture fact.

## Loose ends the sources surfaced (grilling material, not recommendations)

- **What exactly does the Quoted repair remove?** Normalisation strips *one surrounding pair*;
  FR-diag-quoted flags *any* `"` anywhere. Remove-all-quotes covers both and matches the
  measurement ("the quoted spelling is dead"), but it is a decision, not a fact.
- **Row identity**: one row per Entry, or per (Entry, Issue)? Types co-occur (Missing+Duplicate;
  Quoted freely — spec §7). With a Quoted repair adopted, one Entry can carry two *different*
  proposed actions (strip quotes vs delete) — VS Preview Changes has one action per row; a
  per-Issue row model needs a rule for conflicting checks on the same Entry.
- **Does Relative appear at all?** PRD rows carry "delete / skip" as the proposed action; a
  delete-proposal for Relative is legal but aggressive, and an unfixable row in a fix dialog is
  noise by the Disk Cleanup model. Excluding Relative (and pointing at Edit) vs listing it
  unchecked is a judgement call.
