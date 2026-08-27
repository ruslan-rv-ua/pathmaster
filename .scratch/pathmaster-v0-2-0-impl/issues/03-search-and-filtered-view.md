# 03 — Search and the Filtered View engine

**Spec:** [delta-spec §2, §3, §13 (items 1, 9, 10), §15 (fields), §16](../../pathmaster-v0-2-0/spec.md)

**What to build:** The first narrowing slice, end to end: each Scope tab gains a permanent Search field above the list, typing narrows the visible Entries live, and NVDA hears the debounced count. Underneath lands the whole Filtered View engine — per-Scope derived view state, live membership recomputation, the focus rule, command enablement under a filter — which tickets 04, 05, 06 and 07 all build on. The Filter axis does not exist yet: the engine composes Search ∧ Filter with the Filter fixed at `All`.

**Blocked by:** 01 (retrofit), 02 (`#` column — rebuilt rows carry it).

**Status:** ready-for-agent

- [ ] A permanent native `TextCtrl` (never `SearchCtrl`) with a "Search:" label sits above the list on each Scope tab; the Backups tab has neither; label constant, no mnemonic, never carries the count; both Catalogue strings shipped
- [ ] Matching: case-insensitive substring with Unicode case folding (`str::to_lowercase`, both sides — never ASCII), slash-folded (`/`→`\`), and nothing else; the query is never trimmed; a search for `"` finds the `Quoted` Entries; matches the currently displayed rendering (raw-only until ticket 04)
- [ ] Filtered View state is per-Editing-Session: each Scope keeps its own Search text, switching tabs keeps it, nothing persists — text dies with the Run
- [ ] Keyboard contract: Ctrl+F (new View menu, first item "Search", disabled on Backups) focuses the field and selects its contents; tab order becomes tabs → search field → list → buttons; Enter is consumed by the field and does nothing; Down-arrow and Tab enter the list; ESC clears and returns focus to the list (honouring `searchEscapeReturnsFocus`; ESC on an already-empty field still returns focus and says nothing)
- [ ] Rebuild strategy: plain `DeleteAllItems` + reinsert under the unfocused list — no Freeze/Thaw; silent under NVDA, no chatter, verified live
- [ ] Live membership: the visible set recomputes silently after every Working-Copy change (Edit commit, Delete, Undo, Redo, Refresh, Restore); an Entry edited out of the match set vanishes at dialog OK; `#` cells keep original positions under any narrowing
- [ ] Focus rule on membership change: concerned Entry if visible → same visual position → last visible row → empty list; focus never jumps to the Search field uninvited; nothing new is spoken
- [ ] Enablement: Move Up, Move Down and Add are disabled (menu items and buttons) while a Filtered View is active; Edit, Delete work on the focused visible Entry; an empty result set shows zero rows, no placeholder, and disables Edit/Delete
- [ ] Search text is outside the Undo history: Checkpoints never capture it, Ctrl+Z never mutates the field, and no command — Refresh, Restore, Apply — changes the criteria; only the user's typing does
- [ ] Announcements: item 9 ("{n} of {m} entry/entries", zero case "No matching entries") on typing pauses debounced through `filteredCountDelayMs`; item 10 (Scope-named forms) on tab activation and Refresh while that Scope has a Filtered View; Announcement 1 whenever the query is empty (the Filter half of the two-part condition completes in ticket 05); plural selected by **{m}**, all forms in both languages, i18n gate green
- [ ] The debounce timer is owned by a **non-Frame widget** (wxdragon 0.9.18 binds `on_tick` on the owner with no id filter — two timers on one owner fire each other's handlers)
- [ ] Three new flat `settings.json` fields with defaults — `speakFilteredCount` (bool, true), `filteredCountDelayMs` (int 0–5000, 250), `searchEscapeReturnsFocus` (bool, true) — ordinary field-layer failure handling: out-of-domain → default in memory, file keeps raw text, one `WARN`, no dialog, no clamping; hand-editing the file demonstrably changes behaviour (dialog controls are ticket 06)
- [ ] StatusBar field 0 per-Scope fragment becomes "User PATH: {n} of {m} entries ({k} issues)" while that Scope is narrowed; the parenthetical keeps counting the Scope's Issues, not the view's; field 1 untouched
- [ ] Read-only Data searches normally
