# Fix Issues dialog contract

Type: grilling
Status: open
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
