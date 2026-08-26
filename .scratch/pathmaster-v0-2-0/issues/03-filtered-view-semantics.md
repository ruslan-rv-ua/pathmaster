# Filtered view semantics

Type: grilling
Status: resolved (2026-08-26)
Blocked by: —

## Question

Search and the Filter bar both make the ListView show a **subset** of the Working Copy — a state
v0.1.0 never had. Pin down what a filtered view *is* in the domain model before either feature is
specified, because every editing command has to answer to it:

- What is the term, and where does it live? (A view over the Working Copy — never a change to it;
  presumably per-Scope, so switching tabs raises: does each Scope keep its own filter/search state?)
- The PRD fixes two anchors: displayed `#` indexes are the **original positions** (no renumbering),
  and Search + Filter compose with AND logic. Confirm or amend.
- **Editing under a filter** — the hard part. What do Move Up / Move Down mean when the adjacent
  entry is hidden? Is Delete allowed? Add — where does the new entry land and is it visible if it
  doesn't match the filter? Does an edit that makes an entry stop matching make it vanish mid-keystroke?
  The cheap, honest option to weigh first: editing commands are disabled while the view is filtered
  (a filtered view is for *finding*, not editing) — measured against what that costs a long-PATH user.
- Undo/redo and Checkpoints: does a Checkpoint restore filter state, or is filter state outside the
  undo history entirely (like the diagnostic results are)?
- Refresh, Restore, Apply while filtered: what does each do to the view?
- Focus and NVDA: when the visible set changes under the user, where does focus land, and what is
  spoken? (The count announcements are each feature's ticket; the *focus rule* is this one.)

Resolved terms go into `CONTEXT.md`.

## Resolution (2026-08-26)

**A Filtered View is derived view state — like Issues: it reads the Working Copy and is never part
of it.** Term recorded in `CONTEXT.md`; everything below follows from that framing.

1. **Term and home.** *Filtered View*: an Editing Session's view of its Working Copy, narrowed to
   the Entries matching that Scope's Search text and Filter, composed with **AND** (PRD anchor
   confirmed). It is per-Editing-Session — **each Scope keeps its own Search/Filter state**, the
   direct extension of Session independence; a global filter would be action at a distance (text
   typed for one list silently shrinking the other). Signpost for tickets 06/07, not decided here:
   non-persistence across runs follows naturally, since a Session never survives the process.
2. **Editing under a filter — the partial model.** Commands whose effect is fully visible act on
   the focused visible Entry: **Edit, Delete, Copy work; Move Up, Move Down, and Add are disabled**
   while the Filtered View is active (menu items grey, buttons disabled). Reorder's effect concerns
   positions in the full list the user cannot see — and that verdict covers *any* reorder, so if
   ticket 10's drag-and-drop lives, it is disabled under a filter too. No Excel-style trap is
   possible: the list is single-selection and every allowed command touches exactly one visible
   Entry, never a hidden one.
3. **Live membership.** The visible set is recomputed after **every** Working-Copy change — Edit
   commit, Delete, Undo, Redo, Refresh, Restore. The view never lies: what is shown (and counted)
   is exactly what matches. An Entry edited out of the match set vanishes at dialog OK — a discrete
   moment; dialog-first editing means no mid-keystroke vanish exists.
4. **Focus rule** (this ticket's deliverable; count announcements stay with tickets 06/07):
   1. If the Entry the operation concerned is visible → focus it (v0.1.0 Checkpoint focus hint,
      unchanged).
   2. If it is hidden or gone → focus the row at the same visual position (next visible), else the
      last visible row — the standard win32 delete behaviour users already know.
   3. If no rows remain visible → focus stays on the empty list; it never jumps to the Search
      field uninvited.
   Nothing new is spoken: NVDA reads the newly focused row free (ADR-0003), and the closed
   Announcement set **does not grow** from the focus rule.
5. **Undo history excludes the view.** Checkpoint is unchanged — it does not capture Search/Filter
   state. Ctrl+Z never mutates the Search or Filter controls; after Undo/Redo, membership follows
   rule 3 and focus follows rule 4.
6. **Refresh, Restore, Apply — one rule, zero special cases**: no command changes the filter; only
   the user's own input in the Search/Filter controls does. Apply changes nothing visible (registry
   write + Baseline move only). Refresh and Restore replace the Working Copy → rules 3 and 4 apply;
   "0 of N" after a Restore is honest and one Esc away from a full view.
7. **PRD anchors confirmed**: displayed `#` indexes are the Entries' **original positions** — no
   renumbering (the one honest option: it keeps an Entry's place in the full list readable exactly
   when reorder is disabled, and NVDA reads column text free); Search + Filter compose with AND.

**Evidence** (researched before grilling, per the standing directive):

- Reorder under a temporary view is canonically disabled: Microsoft Lists/SharePoint disables
  drag-to-reorder whenever a sort is applied
  ([Drag and drop to reorder list items](https://support.microsoft.com/en-US/SharePoint/lists/drag-and-drop-to-reorder-list-items));
  enterprise-table guidance says manual reordering under sort/filter must be disabled or explained,
  because a move in a filtered view cannot show where the item lands in the full list
  ([How to Reorder Lists Correctly in Tables](https://www.gadgetsfarms.com/how-to-reorder-lists-correctly-in-tables/)).
- Excel's delete-under-filter is the famous trap — range deletes silently take hidden rows with
  them ([How to Delete Filtered Rows in Excel](https://spreadsheetplanet.com/delete-filtered-rows-excel/));
  it is a *range*-operation hazard, which the single-focused-Entry model cannot reproduce.
- Undo-history consensus: the document belongs to undo, the view state (selection, scroll, filter)
  does not ([A Guide to Undo](https://www.linkedin.com/pulse/guide-undo-part-1-3-andre-milota);
  the [Figma selection-undo debate](https://forum.figma.com/t/user-preference-to-include-or-exclude-object-selection-in-undo/5333?page=2)
  reaches the same place).
- Focus management: focus never falls into a void and never moves except as a predictable
  consequence of the user's action; set changes are announced via a live-region mechanism, not by
  teleporting focus ([Cloudscape focus-management principles](https://cloudscape.design/foundation/core-principles/accessibility/focus-management-principles/),
  W3C APG practices).

No new tickets; no fog graduates (the settings.json question stays with tickets 06/07, sharpened
by the signpost in 1). No ADR: the model follows industry consensus at every branch, is cheap to
reverse before the spec locks, and the evidence left no genuine trade-off. Consumed by tickets 04,
06, 07 (and 10: reorder-disabled-under-filter binds D&D if it lives).
