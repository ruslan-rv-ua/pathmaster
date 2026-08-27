# 05 — Filter: the seven-state View submenu

**Spec:** [delta-spec §4, §13 (items 1, 11), §16](../../pathmaster-v0-2-0/spec.md)

**What to build:** The second narrowing axis: View → Filter, a submenu of seven exclusive radio states (`All` / `With issues` / `Missing` / `Relative` / `Quoted` / `Duplicate` / `Empty`), per Scope, composed with Search by AND through the ticket-03 engine. Ctrl+I rides its own "Toggle Issues Filter" menu item for the coarse axis. Choosing a state speaks the composed count; the StatusBar names the state while narrowed. This completes Announcement 1's two-part condition.

**Blocked by:** 03 (the Filtered View engine and its announcements).

**Status:** ready-for-agent

- [ ] View → Filter submenu of seven `wxITEM_RADIO` items; NVDA reads the selected state, and the checked item follows the active Scope across a tab switch; disabled on the Backups tab; no on-window control
- [ ] An Entry is visible when its Issue set contains the chosen type; `With issues` = non-empty Status; Over-length is Scope-level, flags no Entry, and no state selects it
- [ ] Per-Scope state, dies with the Run: every Run starts at `All` on every Scope; no `settings.json` field; outside the Undo history like the Search text
- [ ] "Toggle Issues Filter" is its own plain command item with a constant label carrying **Ctrl+I**: from `All` → `With issues`; from any non-All state → `All`; the five per-type states are menu-only
- [ ] Spoken: a change to a non-All state speaks item 11 — "{filter}: {n} of {m} entries", plural by {m}, zero case "{filter}: no matching entries" — one announcement, never two; a change to `All` with an empty query speaks Announcement 1, with query text present speaks item 9; Announcement 1's condition is now fully two-part (empty query AND Filter at All)
- [ ] Filter-state names reuse the menu/Status strings — no new msgids for names; new menu msgids ("Filter", "All", "With issues", "Toggle Issues Filter") shipped in both languages, i18n gate green
- [ ] StatusBar field 0 fragment becomes "User PATH: {filter} — {n} of {m} entries ({k} issues)" while that Scope's Filter ≠ All; the issues parenthetical keeps its meaning
- [ ] Move Up/Down/Add disablement, focus rule, live silent membership recomputation and `#` stability all hold under a Filter exactly as under Search (engine reused, not extended)
