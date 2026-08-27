# 05 — Filter: the seven-state View submenu

**Spec:** [delta-spec §4, §13 (items 1, 11), §16](../../pathmaster-v0-2-0/spec.md)

**What to build:** The second narrowing axis: View → Filter, a submenu of seven exclusive radio states (`All` / `With issues` / `Missing` / `Relative` / `Quoted` / `Duplicate` / `Empty`), per Scope, composed with Search by AND through the ticket-03 engine. Ctrl+I rides its own "Toggle Issues Filter" menu item for the coarse axis. Choosing a state speaks the composed count; the StatusBar names the state while narrowed. This completes Announcement 1's two-part condition.

**Blocked by:** 03 (the Filtered View engine and its announcements).

**Status:** done — the submenu, both marks, Announcement 11 and the two-part condition verified against live NVDA, see Comments

- [x] View → Filter submenu of seven `wxITEM_RADIO` items; NVDA reads the selected state, and the checked item follows the active Scope across a tab switch; disabled on the Backups tab; no on-window control
- [x] An Entry is visible when its Issue set contains the chosen type; `With issues` = non-empty Status; Over-length is Scope-level, flags no Entry, and no state selects it
- [x] Per-Scope state, dies with the Run: every Run starts at `All` on every Scope; no `settings.json` field; outside the Undo history like the Search text
- [x] "Toggle Issues Filter" is its own plain command item with a constant label carrying **Ctrl+I**: from `All` → `With issues`; from any non-All state → `All`; the five per-type states are menu-only
- [x] Spoken: a change to a non-All state speaks item 11 — "{filter}: {n} of {m} entries", plural by {m}, zero case "{filter}: no matching entries" — one announcement, never two; a change to `All` with an empty query speaks Announcement 1, with query text present speaks item 9; Announcement 1's condition is now fully two-part (empty query AND Filter at All)
- [x] Filter-state names reuse the menu/Status strings — no new msgids for names; new menu msgids ("Filter", "All", "With issues", "Toggle Issues Filter") shipped in both languages, i18n gate green
- [x] StatusBar field 0 fragment becomes "User PATH: {filter} — {n} of {m} entries ({k} issues)" while that Scope's Filter ≠ All; the issues parenthetical keeps its meaning
- [x] Move Up/Down/Add disablement, focus rule, live silent membership recomputation and `#` stability all hold under a Filter exactly as under Search (engine reused, not extended)

## Comments

**2026-08-27 (implementation)** — The Filter's pure half joins Search's in
`pathmaster_core::filtered`, and the module's shape is the ticket's real content: `Filter` is the
seven states with `admits` (the Issue **set**, so a `Missing`, `Duplicate` Entry is both), `narrows`,
`toggled` — Ctrl+I's two rules in one place — and `catalogue_msgid`, where the five type states
**return `Issue::catalogue_msgid()` itself**, which is what "no new msgids for names" means
structurally rather than by convention. Beside it, `Criteria` is the pair a Scope actually holds, and
it is what made the two-part condition a thing that can only be stated once: `narrowing()` is
Announcement 1's whole condition, `searching()` is the Search half ESC answers to, and `admits()` is
the AND. `visible_indices` now takes each Entry as the two things the criteria ask about — the text
the list is showing, and what the last completed pass found about it.

`Command` grew a `Filter(Filter)` variant carrying its own state, so the mark a user reads and the
state a choice sets are one value; `carries_state() -> bool` became `item() -> MenuItemKind` (Plain /
Check / Radio), because appending the wrong *kind* is an item whose mark can never be set at all, and
`state()` returns `Option<bool>` so "no mark" and "no answer" are different things. The Backups tab is
the second: it answers `None` for the Filter and the radio marks are **left exactly as they are**,
still showing the Scope the user came from — writing `All` there would be a mark claiming a narrowed
Scope is not. (Expansion Mode still answers `Some` there, because it is app-wide.) `Command::ALL`
reads the seven states **positionally off `Filter::ALL`** rather than naming them again, which makes
the submenu's order a compile-time fact: the binary is bin-only and the Release Checklist is its
coverage (ADR-0007), so a rule worth keeping there has to be structural or it is not kept at all.

wxdragon's `MenuBuilder` has no submenu step, so the View menu is built whole and the submenu
`insert_submenu`'d into it — at the position `submenu_slot` counts off `Command::ALL`, so §12's order
lives in that one list and not in a second table. `AppendSubMenu` gives the title item no id, which is
why `build_menu_bar` became `Menus`: it keeps the `MenuItem` the insert returned, because that item is
the only handle by which the submenu itself can read as disabled rather than opening onto seven greyed
states. A submenu's enabled state is asked of the commands inside it, so there is no second enablement
rule to keep in step.

Choosing a state is a **discrete** gesture, and `narrow` is what a discrete gesture does — ESC's
clear takes the same path, so "criteria change" has one meaning: stop the debounce, drop any count it
owed, adopt the criteria, rebuild quietly onto §2's landing row, speak one Announcement. `set_filter`
adds the Search field's current text to the new state before handing it over. A
keystroke inside the debounce window has not reached the criteria yet, and narrowing by the Filter
alone while the field holds text would put a list on screen that neither axis describes — flushing is
also what makes "one announcement, never two" true rather than hoped for. Re-choosing the state
already in force changes no criteria and does nothing, loudly included, which is the rule a debounce
tick whose text has not moved already answered to. ESC's guard moved from `narrowed()` to
`searching()`: ESC clears the text, so a Scope narrowed only by its Filter has nothing there to
change — and because the cancelling now lives inside `narrow`, an ESC that says nothing also cancels
nothing, which is what leaves an Expansion toggle's owed count still owed.

**One path the Search axis never needed: `apply_pass`.** A Filter selects on the Issue set, and the
Issue set is exactly what a pass replaces — so under a Filter a landing pass can move membership, and
`render_status` would then write the new rows' Status cells into the old rows. It now reads the
visible set and the concerned Entry **before** the findings land (`toggle_expansion`'s order, for
`toggle_expansion`'s reason) and rebuilds only where membership actually moved, quietly and on §2's
landing row — a pass arrives on its own schedule, so taking the keyboard focus for it would be the
uninvited jump §2 forbids. The first row stands in where the rule answers nothing, as it does on the
typing path: a Run that chose a Filter before its first pass landed was looking at an empty list, so
there is no row to keep and §2's "if no rows remain" is not about the rows that just arrived.
Measured: with `Відсутній` chosen in the first frame of a Run the list holds **0 rows and no focused
item** until the pass lands, and **5 rows, focused and selected on row 0** from the tick it does.

§16's fragment needed msgids of its own. "User PATH: Missing — 4 of 50 entries" puts the name
*between* the Scope and the count, where no prefix or suffix can put it, so `FILTERED_*_NAMED` is a
third whole-string-per-Scope set beside items 10's — six msgids, plus Announcement 11's three and the
four menu strings §12 names, all filled by one `named_count`. The parenthetical is untouched and
still counts the Scope's Issues, not the view's. The seven state names are registered as plain text
rather than menu items, because the mnemonic the gate demands of a menu label is exactly what the
other two surfaces would print — so the `.po` gate gained the opposite rule instead: **no translation
of a Filter state name may carry a mnemonic**, in any shipped language.

**Verified against live NVDA on this machine** (`tools/nvda-drive.ps1`, staged copy, Ukrainian pass):
Alt+V reads «Пошук(S) Ctrl+F», Down reads «Фільтр(F) **підменю**», Right opens it on «Усі»
**«позначено»** and Down reads «З проблемами» and «Відсутній» with no mark — the radio group's
selected state read in both directions. Enter on «Відсутній» speaks **«Відсутній: 5 з 45 записів»**
(item 11) after NVDA's free reading of the landing row, and re-opening the submenu then reads «Усі»,
«З проблемами» unmarked and **«Відсутній» «позначено»**. Ctrl+I from a focused list speaks
**«З проблемами: 21 з 45 записів»**, and Ctrl+I back speaks **«PATH користувача: 45 записів»** —
Announcement 1, the two-part condition completed. §2's focus rule holds through both: the row the user
was on (`#8`, a `Missing` Entry) survives the narrowing, and a row that does not (`#1`) lands on the
same visual position.

Cross-process probes additionally confirmed: the View menu is «Пошук(&S) Ctrl+F» · «Фільтр(&F)» ▸ ·
«Перемкнути фільтр проблем(&I) Ctrl+I» · «Розгорнуті значення(&E) Ctrl+E» at ids 6011, popup, 6019,
6020, with the seven states at 6012–6018 and «Усі» checked at rest; on the Backups tab **every** View
item reads greyed **including the submenu itself**, its seven states greyed with the mark still
readable; the marks are per Scope (User on «Відсутній», System still «Усі», and back again across two
tab switches); Add/Move Up/Move Down are greyed and Edit/Delete alive under a Filter alone; the two
axes compose (`windows` alone 12 rows, `windows` ∧ `Відсутній` 1 row, `Порожній` 0 rows); and
StatusBar field 0 reads «PATH користувача: **Відсутній —** 5 з 45 записів (21 проблема) | PATH
системи: 19 записів (9 проблем)» — the state named, the parenthetical still the Scope's own 21 — with
the zero case «PATH користувача: Порожній — немає збігів (21 проблема)». Field 1 is untouched
throughout.

The README's keyboard table and the Release Checklist's Filter steps are ticket 12's by its own
checklist, and are left to it.
