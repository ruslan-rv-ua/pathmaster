# 11 — Entry editing interaction

**Spec:** [spec §6, §5 (FR-refresh, FR-cancel), §10.1 items 4–6, §15 (Edit menu)](../../pathmaster-v0-1-0/spec.md)

**What to build:** The user edits their PATH end-to-end without a mouse: Add/Edit through the modal dialog (with Browse and validation), Delete, Move Up/Down, Undo/Redo, Cancel, and Refresh — each one Checkpoint, each with its Announcement, with per-Scope buttons and the Edit menu whose enabled states follow the active Session. Nothing writes the registry yet (that is Apply, ticket 13), but every edit is visible, undoable, and heard.

**Blocked by:** 06 (all strings), 08 (Sessions in the UI, announce).

**Status:** resolved

- [x] F2, Enter, and double-click all open the same modal dialog (title "Edit entry" / "Add entry"): one labelled path field, Browse, OK, Cancel; `EditLabels` not used
- [x] Validation on OK blocks `< > | "`, the separator `;`, and length-zero only; whitespace-only commits verbatim; a failed validation shows a `MessageDialog` whose title is the message ("The entry contains a forbidden character: <"), single OK; on dismiss focus returns to the field with text intact; a rejected edit leaves no Checkpoint
- [x] OK sequence fixed: validate → `%VAR%`-into-`REG_SZ` dialog (title per spec, buttons [Change type to REG_EXPAND_SZ] / [Keep as literal text], each outcome one Checkpoint) → commit as one Checkpoint
- [x] Browse opens `wxDirDialog` (`.destroy()` called — its `Drop` leaks); seeds from the field text when it names an existing directory; the chosen folder replaces the field text; focus returns to the field
- [x] Add is dialog-first: OK appends at the end as one Checkpoint, focus on the new row; Cancel/Escape leaves nothing; Delete has no confirm — focus stays at the same index, clamped to the new last row
- [x] Move Up / Move Down (Alt+Up / Alt+Down), one Checkpoint each; duplicates and not-yet-existing paths commit legally, never blocked or warned at commit
- [x] Undo/Redo (Ctrl+Z / Ctrl+Y) restore the Checkpoint, move focus to the hinted Entry, and fire Announcement 4 ("Undone: {operation}" / "Redone: {operation}") with the fixed operation names, distinct from button labels; the ", unsaved changes" suffix rides an undo across the Apply barrier
- [x] Cancel: confirmation ("Discard changes?" [Yes] [No]) only while dirty, disabled while clean, is itself a Checkpoint, announces "Changes discarded"
- [x] Refresh (F5): active Scope only, confirmation while dirty, clears that Session's stacks, announces the entry count; focus same id / nearest neighbour / list
- [x] Edit menu per §15 with accelerators appended in code; per-Scope buttons Add, Edit, Delete, Move Up, Move Down; every control and menu item disabled (and reading as disabled) on a non-writable Session; Apply/Cancel disabled while clean
- [x] All new strings in the Catalogue with Ukrainian translations; the completeness gate passes

## Comments

Implemented 2026-08-20 on `feature/entry-editing-interaction`. The application now edits: F2,
Enter or double-click opens the dialog, every operation is one Checkpoint, and Ctrl+Z speaks.
Verified live against the real machine — 42 User entries — through the whole loop: Add commits at
the end with focus on the new row, `<` is rejected by title with the text left intact, Ctrl+Z
speaks "Скасовано: Додавання запису", F5 confirms and re-reads, and the Edit menu greys itself
correctly. Nothing writes the registry; Apply is ticket 13.

**Four rules moved down into the pure core, where a test can reach them** (ADR-0007):

- `path::rejection` — the character rules, beside the `split` they are the other end of: an Entry
  cannot contain the separator it is defined by. Whitespace-only passes, which is the point.
- `normalize::has_variable_reference` — the one question the convert-or-keep dialog asks. It walks
  the text exactly as `expand` does, and a test runs both over the same fixtures so they cannot
  drift: a text expansion changes is a text the dialog must have asked about.
- `session::Operation` — a Checkpoint now carries the name of what it stands for, which is the one
  thing focus landing on a row cannot say. `catalogue_msgid()` names each one's string, and a test
  holds the seven to seven distinct registered msgids.
- `UndoOutcome::crossed_apply` — the ", unsaved changes" suffix, computed the way dirtiness itself
  is: a comparison of before and after, never a record that an Apply happened. `Session::refresh`
  likewise now answers where focus lands, because the index it needs is gone by the time the
  caller could ask.

`Session::batch` changed shape while it was here: it takes the operation name, and its closure
*returns* the focus hint rather than the caller predicting it — an Add's new Entry does not exist
until the work has run, so the converted-Add path would otherwise have undone to nowhere while the
plain Add undid to its neighbour.

**The UI is four modules, and one enum is the map.** `Command` names the nine editing commands and
answers, in one `match` each, what the menu item says, what the button says, which keystroke, and
when it is available — which matters more than tidiness here, because `wxAcceleratorTable` is
absent from wxdragon at every level: **a shortcut can only exist as a menu item's label**, so the
menu's enabled state is also the shortcut's. `question::ask` is the one two-button modal (both
confirmations *and* convert-or-keep), `entry_dialog` the one editing surface, `scope_page` the list
and its buttons.

**Focus is the feedback, so every mutating operation ends in the list.** Delete has no
Announcement and no confirmation; a committed Add has none either — what the user hears is NVDA
reading the row focus landed on. A row focused inside a control that is not focused is silent,
which for this application is the same as not having happened. The cost is that clicking Move Up
twice needs a Tab back; the charter says keyboard first.

**Five roads the ticket left open, and the ones taken:**

1. **"Cancel" cannot be three strings, and the spec asked for it twice.** §10.1 item 4 lists the
   operation name as "Cancel" *and*, in the same sentence, requires operation names to differ from
   the buttons that perform them (D14). Three meanings need three English strings, and the third —
   the "Cancel" every modal dialog carries, «Скасувати» — is the one that cannot be renamed. So the
   command keeps §15's name one word longer (**"Cancel Changes"**/«Відхилити зміни», ADR-0004's own
   worked example) and the operation becomes D14's verbal noun (**"Discard changes"**/«відхилення
   змін»). Spec §10.1 and §15 amended to say so. "Add entry" and "Edit entry" *do* double as the §6
   dialog titles — identical English for one meaning is what ADR-0004 asks for, and the capital
   they keep for the title costs nothing audibly.
2. **A non-writable Session disables the whole Edit menu, Refresh included.** §5 says "disables
   every editing action" and Refresh is not one; the ticket says "every control and menu item".
   Taken literally, so an unelevated System tab reads as uneditable throughout rather than offering
   one live item. Recorded in §15.
3. **A re-read that fails at Refresh changes nothing and says nothing.** An unreadable value is not
   an Absent one (§4), and blanking a Scope over a transient failure is the one unrecoverable thing
   this screen can do. The Announcement catalogue is closed at seven, so there is nothing to speak;
   this matches the startup read's degraded road (ticket 08) and waits for the §9 taxonomy, which
   arrives with Apply along with the logger the UI does not yet hold.
4. **The negative button holds the default, the initial focus and Escape** — in the confirmations
   and in convert-or-keep alike. Found by measurement, not by reasoning: with only the default set,
   Windows still gave Enter to the *focused* button, which was [Yes]. Now Enter and Escape agree,
   and both land on the outcome that changes least.
5. **Browse seeds literally.** `%JAVA_HOME%\bin` names no directory to `is_dir`, so the picker
   opens at the system default. Expanding it would start a second pass over the process environment
   in a folder picker, where diagnostics owns the first (§7).

**Forty msgids** — the Edit menu (mnemonics A, E, D, M, V, U, R, C, F, gated unique), the six
buttons, the dialog and its buttons, the two rejections, convert-or-keep, both confirmations, and
Announcements 4–6 with the five operation names — all shipped in both `.po` files with translator
comments, and the completeness gate passes. `wxDirDialog` gets a title of ours ("Choose a folder"):
left unset it speaks wx's built-in English in a Ukrainian run.

Release Checklist gains B4 and B5 for the two dialogs this ticket added that had no step. The GUI
itself stays Release-Checklist territory (ADR-0007): 299 automated tests hold the rules, and none
of them links wx.
