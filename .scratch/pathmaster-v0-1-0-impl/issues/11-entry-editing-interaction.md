# 11 — Entry editing interaction

**Spec:** [spec §6, §5 (FR-refresh, FR-cancel), §10.1 items 4–6, §15 (Edit menu)](../../pathmaster-v0-1-0/spec.md)

**What to build:** The user edits their PATH end-to-end without a mouse: Add/Edit through the modal dialog (with Browse and validation), Delete, Move Up/Down, Undo/Redo, Cancel, and Refresh — each one Checkpoint, each with its Announcement, with per-Scope buttons and the Edit menu whose enabled states follow the active Session. Nothing writes the registry yet (that is Apply, ticket 13), but every edit is visible, undoable, and heard.

**Blocked by:** 06 (all strings), 08 (Sessions in the UI, announce).

**Status:** ready-for-agent

- [ ] F2, Enter, and double-click all open the same modal dialog (title "Edit entry" / "Add entry"): one labelled path field, Browse, OK, Cancel; `EditLabels` not used
- [ ] Validation on OK blocks `< > | "`, the separator `;`, and length-zero only; whitespace-only commits verbatim; a failed validation shows a `MessageDialog` whose title is the message ("The entry contains a forbidden character: <"), single OK; on dismiss focus returns to the field with text intact; a rejected edit leaves no Checkpoint
- [ ] OK sequence fixed: validate → `%VAR%`-into-`REG_SZ` dialog (title per spec, buttons [Change type to REG_EXPAND_SZ] / [Keep as literal text], each outcome one Checkpoint) → commit as one Checkpoint
- [ ] Browse opens `wxDirDialog` (`.destroy()` called — its `Drop` leaks); seeds from the field text when it names an existing directory; the chosen folder replaces the field text; focus returns to the field
- [ ] Add is dialog-first: OK appends at the end as one Checkpoint, focus on the new row; Cancel/Escape leaves nothing; Delete has no confirm — focus stays at the same index, clamped to the new last row
- [ ] Move Up / Move Down (Alt+Up / Alt+Down), one Checkpoint each; duplicates and not-yet-existing paths commit legally, never blocked or warned at commit
- [ ] Undo/Redo (Ctrl+Z / Ctrl+Y) restore the Checkpoint, move focus to the hinted Entry, and fire Announcement 4 ("Undone: {operation}" / "Redone: {operation}") with the fixed operation names, distinct from button labels; the ", unsaved changes" suffix rides an undo across the Apply barrier
- [ ] Cancel: confirmation ("Discard changes?" [Yes] [No]) only while dirty, disabled while clean, is itself a Checkpoint, announces "Changes discarded"
- [ ] Refresh (F5): active Scope only, confirmation while dirty, clears that Session's stacks, announces the entry count; focus same id / nearest neighbour / list
- [ ] Edit menu per §15 with accelerators appended in code; per-Scope buttons Add, Edit, Delete, Move Up, Move Down; every control and menu item disabled (and reading as disabled) on a non-writable Session; Apply/Cancel disabled while clean
- [ ] All new strings in the Catalogue with Ukrainian translations; the completeness gate passes
