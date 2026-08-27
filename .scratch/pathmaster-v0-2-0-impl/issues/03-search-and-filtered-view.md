# 03 — Search and the Filtered View engine

**Spec:** [delta-spec §2, §3, §13 (items 1, 9, 10), §15 (fields), §16](../../pathmaster-v0-2-0/spec.md)

**What to build:** The first narrowing slice, end to end: each Scope tab gains a permanent Search field above the list, typing narrows the visible Entries live, and NVDA hears the debounced count. Underneath lands the whole Filtered View engine — per-Scope derived view state, live membership recomputation, the focus rule, command enablement under a filter — which tickets 04, 05, 06 and 07 all build on. The Filter axis does not exist yet: the engine composes Search ∧ Filter with the Filter fixed at `All`.

**Blocked by:** 01 (retrofit), 02 (`#` column — rebuilt rows carry it).

**Status:** done — key contract and both counts verified against live NVDA, see Comments

- [x] A permanent native `TextCtrl` (never `SearchCtrl`) with a "Search:" label sits above the list on each Scope tab; the Backups tab has neither; label constant, no mnemonic, never carries the count; both Catalogue strings shipped
- [x] Matching: case-insensitive substring with Unicode case folding (`str::to_lowercase`, both sides — never ASCII), slash-folded (`/`→`\`), and nothing else; the query is never trimmed; a search for `"` finds the `Quoted` Entries; matches the currently displayed rendering (raw-only until ticket 04)
- [x] Filtered View state is per-Editing-Session: each Scope keeps its own Search text, switching tabs keeps it, nothing persists — text dies with the Run
- [x] Keyboard contract: Ctrl+F (new View menu, first item "Search", disabled on Backups) focuses the field and selects its contents; tab order becomes tabs → search field → list → buttons; Enter is consumed by the field and does nothing; Down-arrow and Tab enter the list; ESC clears and returns focus to the list (honouring `searchEscapeReturnsFocus`; ESC on an already-empty field still returns focus and says nothing)
- [x] Rebuild strategy: plain `DeleteAllItems` + reinsert under the unfocused list — no Freeze/Thaw; silent under NVDA, no chatter, verified live
- [x] Live membership: the visible set recomputes silently after every Working-Copy change (Edit commit, Delete, Undo, Redo, Refresh, Restore); an Entry edited out of the match set vanishes at dialog OK; `#` cells keep original positions under any narrowing
- [x] Focus rule on membership change: concerned Entry if visible → same visual position → last visible row → empty list; focus never jumps to the Search field uninvited; nothing new is spoken
- [x] Enablement: Move Up, Move Down and Add are disabled (menu items and buttons) while a Filtered View is active; Edit, Delete work on the focused visible Entry; an empty result set shows zero rows, no placeholder, and disables Edit/Delete
- [x] Search text is outside the Undo history: Checkpoints never capture it, Ctrl+Z never mutates the field, and no command — Refresh, Restore, Apply — changes the criteria; only the user's typing does
- [x] Announcements: item 9 ("{n} of {m} entry/entries", zero case "No matching entries") on typing pauses debounced through `filteredCountDelayMs`; item 10 (Scope-named forms) on tab activation and Refresh while that Scope has a Filtered View; Announcement 1 whenever the query is empty (the Filter half of the two-part condition completes in ticket 05); plural selected by **{m}**, all forms in both languages, i18n gate green
- [x] The debounce timer is owned by a **non-Frame widget** (wxdragon 0.9.18 binds `on_tick` on the owner with no id filter — two timers on one owner fire each other's handlers)
- [x] Three new flat `settings.json` fields with defaults — `speakFilteredCount` (bool, true), `filteredCountDelayMs` (int 0–5000, 250), `searchEscapeReturnsFocus` (bool, true) — ordinary field-layer failure handling: out-of-domain → default in memory, file keeps raw text, one `WARN`, no dialog, no clamping; hand-editing the file demonstrably changes behaviour (dialog controls are ticket 06)
- [x] StatusBar field 0 per-Scope fragment becomes "User PATH: {n} of {m} entries ({k} issues)" while that Scope is narrowed; the parenthetical keeps counting the Scope's Issues, not the view's; field 1 untouched
- [x] Read-only Data searches normally

## Comments

**2026-08-27 (implementation)** — The engine's pure half is `pathmaster_core::filtered`: `matches`
(one fold — `str::to_lowercase` + `/`→`\` — applied to both sides, empty query matches all, nothing
trimmed) and `visible_indices` (the Working-Copy indices the view shows, in order). Everything else is
the window's: each `ScopeTab` holds the **applied** query behind `Scoped` — what the list on screen was
last rebuilt under, which is not always what the field holds, since typing sits in the field until the
debounce tick applies it. Membership, `view_row_of` and `focused_entry` all recompute from the live
Working Copy through `visible()`, so no cached view can describe a list that has moved on.

The rebuild follows the prototype exactly: narrowing and the count apply together at the debounce tick
— one code path, the one ticket 04 measured — so `filteredCountDelayMs` paces both. A tick that fires
while a dialog is up re-arms itself instead of rebuilding under a modal loop (the same door the Pump's
tick answers to); a tick whose text equals the applied criteria does nothing, loudly included (typing
`a` then Backspace inside one window is not a criteria change). `apply_search` speaks only when its
Scope is the active tab: the debounce survives a tab switch, and item 9 describing a hidden list would
be noise — the criteria still apply silently, and arrival speech (item 10) covers the return.

§2's focus rule is `landing_row`: concerned Entry's view row, else the visual position `focused_row`
still reads off the not-yet-rebuilt list, with `render`'s clamp supplying "else the last visible row".
Command paths land it with keyboard focus (`render`); the typing and ESC paths mark it without
(`render_quiet` — state set, no `set_focus`, the uninvited-jump rule), falling back to row 0 so a Run
whose first gesture is typing still has a row for Down/Tab to land on. `after_edit` now takes the
concerned `EntryId` rather than a row, so every Working-Copy change goes through the one rule; Restore
keeps its first-row landing via `after_edit_at`, past the rule on purpose.

The debounce `Timer` is owned by the search `TextCtrl` — each tab's field owns its own — never the
Frame the Pump's timer sits on. ESC clears via `change_value` (never `set_value`), so the programmatic
clear cannot fire the typing path on top of the discrete one. `#` cells carry original positions by
construction: `Row::compose_visible` numbers from the Working-Copy index, not the view row.

Enablement rides two new `Availability` facts (`narrowed`, `visible_rows`) read per-tab, so a Scope
narrowed in the background keeps its buttons honest. `settings` moved behind `Scoped` — the debounce
tick now reads it, which is ADR-0011's classification rule doing its work — and ADR-0009 gained a dated
amendment for the Announcement set growing 7 → 14 with the tickets that speak the new items.

**Verified against live NVDA on this machine** (`tools/nvda-drive.ps1`, staged copy, Ukrainian pass):
Ctrl+F reads «Пошук:» + «порожньо»; each typing pause speaks the debounced count («15 з 45 записів» /
«14 з 45 записів») with **no list chatter between** — the unfocused rebuild is silent; Down enters the
list onto a read row whose `#` cell holds the original position under narrowing («20; Шлях:
C:\Windows\system32; Стан: Дублікат»); Ctrl+F re-selects the query («виділено win»); ESC speaks
Announcement 1 («PATH користувача: 45 записів») and lands focus back on the concerned Entry, now among
the full 45. Cross-process probes additionally confirmed: 45→12 rows on "windows", zero-case «Немає
збігів» at zero matches, Add/Move Up/Move Down greyed and Edit/Delete alive while narrowed, and a
hand-edited `filteredCountDelayMs: 3000` demonstrably slowing the apply (unchanged at 1.2 s, narrowed
at 3.7 s). Item 10 and the narrowed StatusBar fragment are covered by the composition tests; their live
NVDA pass belongs to the Release Checklist round like every other arrival announcement.
