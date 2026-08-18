# Entry editing interaction

Type: grilling
Status: open
Blocked by: 03, 06

## Question

How does the user edit an entry, given what wxdragon can actually do?

The PRD asserts inline editing (FR-edit-f2) and a Browse button in both Add and Edit (FR-browse-folder).
These two are in tension even if the binding exists: a `wxListCtrl` label editor is a bare text field with
nowhere to put a Browse button.

- If in-place label editing **is** bound: can it carry validation and a Browse affordance, and what does NVDA
  do when an edit field opens over a list row?
- If it is **not** bound, or cannot host Browse: an edit dialog — path field, Browse button, OK / Cancel —
  opened by F2 and double-click. This is very likely both more accessible and simpler, at the price of
  contradicting the PRD's "inline". Decide, and rewrite the requirement rather than pretending both hold.
- **Validation** on confirm: forbidden characters `< > | "`, empty value. Where the error appears, whether it
  blocks the commit, and how it is announced.
- **Add.** The PRD appends an empty entry and opens the editor immediately. If the user cancels, does the empty
  row stay behind — and immediately become an "Empty entry" issue? (It should not.)
- **Duplicates on entry.** Warn while typing, warn on commit, or let diagnostics flag it afterwards? Only one
  of these is consistent with diagnostics being asynchronous.
- Does editing a path that does not exist yet stay legal? (It must — creating the directory later is a normal
  workflow.)

Output: rewritten FR-edit-f2, FR-add-delete and FR-browse-folder.

## Carried in from ticket 03

In-place label editing **is** bound (`ListCtrlStyle::EditLabels`, BEGIN/END events, `get_label()`,
`is_edit_cancelled()`), so the first branch of the question is settled — but with a sting:

- **The end-of-edit event cannot be vetoed.** `ListCtrlEventData` exposes no `veto()`/`skip()` and its inner
  `Event` is private (`list_ctrl.rs:157`), so an invalid edit cannot be refused. The spec's "the field is
  highlighted and stays in edit mode" is not expressible. Choose between accept-then-revert, and driving
  editing yourself via `edit_label()` — which returns the live `TextCtrl` and so allows validation as the user
  types. `wxListCtrl::GetEditControl` is unbound, so the editor is unreachable during a *native* F2 edit.
- **Open spike (from ticket 03):** whether `edit_label()` can co-exist with the control's own F2 handling or
  the two race. Cheap to settle with a running prototype, and it decides FR-edit-f2's final wording.

## Carried in from ticket 06

- **Add is one Checkpoint; an Edit abandoned with Escape is none.** So Add-then-Escape leaves an empty Entry
  *and* one Checkpoint behind — which is exactly this ticket's "does the empty row stay?" question, now with a
  mechanism to answer it. Decide whether Add rolls itself back when its opening edit is abandoned, or whether the
  empty Entry stays and is simply undoable.
- **Invalid edits and the missing veto.** With no `veto()` available (ticket 03), accept-then-revert is the only
  route — so decide whether the rejected edit leaves a Checkpoint. It must not be possible to Ctrl+Z *into* an
  invalid state.
- **The editor must never trim or normalise what the user typed.** An Entry is the raw substring; leading and
  trailing whitespace, letter case and a trailing `\` all survive verbatim. Normalisation is comparison-time only.
- **The `%VAR%`-into-`REG_SZ` dialog fires on commit**, so it belongs to this interaction: committing an Entry
  containing a `%…%` pair into a `REG_SZ` Scope raises "[Change type to REG_EXPAND_SZ] [Keep as literal text]".
  Both outcomes are legal and each is one Checkpoint. Specify where it sits in the commit sequence relative to
  the forbidden-character validation.
- Entries carry an **opaque id** that survives Move and Edit; after any undo, focus returns to the Entry the
  Checkpoint names.
