# Entry editing interaction

Type: grilling
Status: resolved
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

## Answer

Resolved 2026-08-19 by grilling. The editing surface is a **modal Edit dialog, not inline editing** — every
other decision flows from that.

**D1 — Surface.** F2, Enter, and double-click on a row all open the same modal dialog: title ("Edit entry" /
"Add entry"), one labelled path field, a Browse button, OK, Cancel. `ListCtrlStyle::EditLabels` is not used,
so the un-vetoable end-of-edit event and the `edit_label()`-vs-native-F2 spike carried in from ticket 03 are
both **moot**. No blind NVDA measurement is needed: the dialog pattern (title + labelled field + buttons)
rides the measured native comctl32 path; it joins the D8 checklist instead (D8 below). FR-edit-f2 is
rewritten — "inline" is dropped deliberately, not forgotten.

**D2 — Browse survives, as a named exception.** The user overrode ticket 07's derived "no native file
dialogs" constraint for exactly one dialog: Browse opens **`wxDirDialog`**, and the ComDlg32 MRU registry
writes it causes (HKCU, under our process) are **accepted and documented**. NFR-no-registry-writes'
"nothing outside the app's own directory" gains this one named exception; the README honesty paragraph must
mention it, and the Process-Monitor release check must expect it (→ comment on ticket 15). Mechanics: the
picker seeds from the field text when that text is an existing directory, system default otherwise; the
chosen folder **replaces** the field text; focus returns to the field. Implementation note from ticket 03:
call `.destroy()` on the `DirDialog` — its `Drop` leaks.

**D3 — Add is dialog-first.** Add opens the same dialog with an empty field. OK appends the new Entry **at
the end** of the list (end = lowest search precedence, the safe default for PATH) as **one Checkpoint**,
with focus landing on the new row. Cancel/Escape leaves nothing behind: no empty Entry, no Checkpoint, no
"Empty entry" Issue. This closes the Add-then-Escape question ticket 06 carried in.

**D4 — Delete loses its confirm dialog.** Nothing is irreversible before Apply: Delete is one Checkpoint and
Ctrl+Z (an Announcement — catalogue item) restores it. Focus stays at the same row index, clamped to the new
last row, and NVDA speaks the newly focused row for free. FR-add-delete is rewritten accordingly.

**D5 — Validation on OK, and the fixed commit sequence.** Forbidden: `< > | "` **plus `;`** — an Entry
cannot contain the separator it is defined by (typing a second path means a second Entry, not a character).
The **length-zero** value is also blocked. **Whitespace-only commits verbatim** — blocking `"   "` would
smuggle a trim into validation, and the editor never trims or normalises (ticket 06); whether it reads as
"Empty entry" is diagnostics' call (→ comment on ticket 13). A failed validation shows an error
`MessageDialog` whose **title is the message** (ticket 09 dialog discipline — NVDA never speaks a body),
single OK button; on dismiss, focus returns to the field with the user's text intact. The OK sequence is
fixed: **(1) validate → (2) `%…%` typed into a `REG_SZ` Scope raises "[Change type to REG_EXPAND_SZ] [Keep
as literal text]" (each outcome legal, one Checkpoint) → (3) commit as one Checkpoint.** A rejected edit
never reaches the Working Copy, so it leaves no Checkpoint — **Ctrl+Z into an invalid state is impossible by
construction.**

**D6 — What validation does not police.** A duplicate commits legally and is flagged asynchronously by
diagnostics in the Status column — never warned during typing, never blocked at commit; that is the only
behaviour consistent with FR-auto-diagnose being asynchronous. A path that does not exist yet is a legal
edit — creating the directory later is a normal workflow; `Non-existent` is an Issue, not an input error.

**D7 — Menu and focus rules.** The Edit menu gains **"Edit Entry…\tF2"** — the PRD's menu lacked it, and the
`\t` accelerator in the label is what makes NVDA speak the shortcut (ticket 02). Focus after the dialog:
OK → the edited/new row; Cancel/Escape → the row focused before opening. Nothing else moves.

**D8 — Verification.** The ticket-09 checklist gains steps: the Edit dialog (title and labelled field
spoken), the validation error dialog (title spoken), the folder picker (standard Windows dialog). **No new
Announcements** — the closed seven-item catalogue is untouched; Delete and commit are heard through focus
landing on a row.

**Rewritten requirements:** FR-edit-f2 (dialog, validation set, commit sequence), FR-add-delete
(dialog-first Add, confirm-less Delete), FR-browse-folder (kept, hosted in the dialog, exception
documented). **Consequences posted as comments:** ticket 15 (README must list ComDlg32 MRU writes; winget
Publisher fixed as `RuslanIskov.PathMaster`), ticket 13 (whitespace-only handed over; quoted-entry question
surfaced for Normalisation).
