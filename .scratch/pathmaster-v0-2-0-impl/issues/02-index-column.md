# 02 — The `#` column returns to the main list

**Spec:** [delta-spec §2.1, §14 (header msgid)](../../pathmaster-v0-2-0/spec.md)

**What to build:** The main list on both Scope tabs becomes three columns — `#` / Path / Status — and NVDA's per-row reading gains the leading position number: "{#}; {path}; Status: {types}". The `#` cell carries the Entry's 1-based position in the Working Copy and never renumbers; the column is permanent (the window never reflows under the user), so it is there before any narrowing exists to need it.

**Blocked by:** 01 (retrofit lands first).

**Status:** ready-for-agent

- [ ] Both Scope lists show `#` / Path / Status; `#` is the Entry's 1-based Working-Copy position, recomputed with the list contents on every Working-Copy change (Move, Delete, Add, Undo, Redo, Restore renumber the *data*; the column always shows current original positions)
- [ ] The `#` column is a deliberate pixel constant — one more explicit `FromDIP()` call beside the Status column's; Path still takes all remaining width
- [ ] Column header `#` is a Catalogue msgid, shipped in both languages, gated
- [ ] NVDA reads a row as "{#}; {path}; Status: {types}" on the free native path — verified live on this machine
- [ ] The count compensation does **not** return: entry counts still come from Announcements, and NVDA's row-position setting stays uncompensated
- [ ] Existing tests green; no other layout change rides along
