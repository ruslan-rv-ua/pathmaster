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
- [x] Edit menu per §15 with accelerators appended in code; per-Scope buttons Add, Edit, Delete, Move Up, Move Down; every control and menu item disabled (and reading as disabled) on a non-writable Session; Apply/Cancel disabled while clean — **with one exception settled in review: Refresh stays live, because it re-reads rather than edits (see Comments)**
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
- `UndoOutcome::crossed_apply` — the ", unsaved changes" suffix. A Checkpoint records how many
  Applies its Session had seen when it was taken, and the suffix asks two questions: did this
  Checkpoint predate the last Apply, and are there unsaved changes now. (The first draft compared
  dirtiness before and after; see the review section — that fires without an Apply.)
  `Session::refresh` likewise now answers where focus lands, because the index it needs is gone by
  the time the caller could ask.

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
2. **A non-writable Session disables every Edit menu item except Refresh** — see the review
   section below, which reversed this: Refresh re-reads rather than edits. Recorded in §15.
3. **A re-read that fails at Refresh changes nothing and says nothing.** An unreadable value is not
   an Absent one (§4), and blanking a Scope over a transient failure is the one unrecoverable thing
   this screen can do. The Announcement catalogue is closed at seven, so there is nothing to speak;
   this matches the startup read's degraded road (ticket 08) and waits for the §9 taxonomy, which
   arrives with Apply along with the logger the UI does not yet hold.
4. **The negative button holds the default, the initial focus and Escape** — in the confirmations
   and in convert-or-keep alike. Found by measurement, not by reasoning: with only the default set,
   Windows still gave Enter to the *focused* button, which was [Yes]. Now Enter and Escape agree.
   In the confirmations that button changes nothing; in convert-or-keep *both* outcomes commit by
   design, so it is the one that at least leaves the Value Type alone.
5. **Browse seeds literally.** `%JAVA_HOME%\bin` names no directory to `is_dir`, so the picker
   opens at the system default. Expanding it would start a second pass over the process environment
   in a folder picker, where diagnostics owns the first (§7).

**Forty msgids** — the Edit menu (mnemonics A, E, D, M, V, U, R, C, F, gated unique), the six
buttons, the dialog and its buttons, the two rejections, convert-or-keep, both confirmations, and
Announcements 4–6 with the five operation names — all shipped in both `.po` files with translator
comments, and the completeness gate passes. `wxDirDialog` gets a title of ours ("Choose a folder"):
left unset it speaks wx's built-in English in a Ukrainian run.

Release Checklist gains B4 and B5 for the two dialogs this ticket added that had no step. The GUI
itself stays Release-Checklist territory (ADR-0007): 300 automated tests hold the rules, and none
of them links wx.

### Code review, and what it changed

Reviewed on both axes (standards and spec) before the ticket closed. **Two real bugs, both found by
reading rather than by running, and neither of which any test would have caught:**

1. **`focus_row` could never see an empty list, and left the user in silence when it was.**
   `ListCtrl::get_item_count()` answers an `i32`, so `0 - 1` is `Some(-1)`, not `None` — the guard
   was unreachable and the clamp handed comctl32 `-1`, its index for *every* row. Harmless only
   because the count was zero. The real cost was the branch underneath it: deleting the last Entry,
   or refreshing a Scope to none, left keyboard focus on the button that did it, and this
   application says nothing except by putting focus on a row. **The list now takes the focus whether
   or not a row survives to land on** — which is where FR-refresh's "else the list" ends. Verified
   live: after deleting every Entry, `GetGUIThreadInfo` reports `SysListView32` focused.

2. **The ", unsaved changes" suffix fired without an Apply.** It compared dirtiness before and after
   — but dirtiness is a comparison of *content*, so an Add and its Delete leave a clean Session with
   two Checkpoints behind it, and undoing one of those re-dirties a Session no Apply has ever
   touched. A Checkpoint now records how many Applies its Session had seen when it was taken, and
   the suffix asks two questions: did this Checkpoint predate the last Apply, and are there unsaved
   changes now. That is a count of Applies, not a dirty flag — the thing ADR-0001 rules out is a
   *flag standing in for the comparison*, and this stands in for nothing.

**One decision reversed, and it was the reviewer's to win.** F5 was disabled on a non-writable
Session, on the ticket's "every control and menu item disabled". §5 disables "every editing action"
and Refresh is not one; `CONTEXT.md` promises that Read-only Data "still reads, diagnoses and
lists"; and an unelevated System tab would otherwise never see an external change without a restart.
**Refresh now stays live on every Scope**, and §15's amendment says so. Verified live: on the
unelevated System tab every Edit item is greyed except «Оновити(F)».

Three shape fixes in the same pass, all from the standards axis: `scope: usize` threaded through
fourteen methods into two parallel arrays became one `ScopeTab { scope, session, page }` — which
also retired a `scope_key()` whose `else` branch meant "System" without checking; the flag arguments
(`move_entry(scope, true)`) now carry the `Command` that already distinguished them; and the
four-line "borrow, ask for the focused Entry, drop the borrow" preamble became one method, because
that preamble *is* the rule that keeps the modal dialogs from panicking and it should live in one
place. Two naming corrections: `MENU_BAR`/`MENU_EDIT` are mnemonic group keys, never translated, so
they are `MENU_GROUP_*` now and say so; and «Відновлення з резервної копії» became «Відновлення
знімка», because the glossary reserves «резервна копія» for the act and the directory, never the
file.
