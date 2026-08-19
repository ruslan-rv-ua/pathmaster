# 12 — Async diagnostics pass, Status column, StatusBar fields

**Spec:** [spec §7 (FR-diag-async, FR-diag-status), §12 (StatusBar)](../../pathmaster-v0-1-0/spec.md)

**What to build:** Diagnostics come alive in the UI: a worker thread runs the ticket-09 rulebook over the Working Copies after load and after every change, the Status column fills with translated Issue-type words, and both StatusBar fields stay current — including the always-visible merged-length field. NVDA reads "{path}; Status: {types}" for free on every arrow key.

**Blocked by:** 08 (UI + Sessions), 09 (the rules).

**Status:** ready-for-agent

- [ ] One worker thread runs a pass over the Working Copies (never the process environment, never the registry); results reach the UI via an `mpsc` channel drained by a wx Timer (~100 ms, running only while a pass is outstanding); widgets never called off the UI thread
- [ ] A pass runs at load and after every Working Copy change (edit, undo/redo, Refresh, Restore); a System edit recomputes User's Issues too (cross-scope duplicates); Issues never enter Checkpoints
- [ ] Status column carries the flagged types' words, comma-joined most-severe-first (Missing > Relative > Quoted > Duplicate > Empty; uk: Відсутній, Відносний, У лапках, Дублікат, Порожній); an empty column is the only healthy state — never "OK", no severity prefix, no icons
- [ ] StatusBar field 0: "User PATH: {n} entries ({m} issues) | System PATH: {n} entries ({m} issues)", updated after every pass and Apply
- [ ] StatusBar field 1 always shows "Merged PATH: {n} chars", appending " — exceeds 8,191 (cmd.exe limit)" past that threshold; over-length never appears in the Status column and is never an Announcement
- [ ] Budget: full pass < 1 s for ≤ 200 entries
- [ ] Issue-type words and StatusBar texts are in the Catalogue with Ukrainian translations
