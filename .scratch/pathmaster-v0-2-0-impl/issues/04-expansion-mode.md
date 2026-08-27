# 04 — Expansion Mode

**Spec:** [delta-spec §5, §13 (item 8)](../../pathmaster-v0-2-0/spec.md)

**What to build:** View → Expanded Values (Ctrl+E, `wxITEM_CHECK`) flips the application between raw and expanded path rendering — app-wide, both Scope tabs alike — and NVDA hears "Showing expanded values" / "Showing raw values". Because ticket 03 landed first, this ticket also owns the coupling: Search matches the displayed rendering, so the toggle changes membership under an active Filtered View and speaks twice — mode message, then the count through the same debounced path.

**Blocked by:** 03 (Search matches the displayed rendering — the coupling lands here).

**Status:** ready-for-agent

- [ ] One app-wide flag; per-Run, default raw; no `settings.json` field, nothing persists
- [ ] Display expansion is Normalisation's own reading (`ExpandEnvironmentStringsW`, process environment); an undefined `%VAR%` stays literal in place — no new Issue type, no inline marker; expansion is unconditional regardless of Value Type
- [ ] Not an edit: no Checkpoint, invisible to Undo/Redo both ways; Ctrl+Z under expanded mode shows the rolled-back Working Copy, still expanded
- [ ] Edit/Add dialogs always carry the raw text whatever the list shows; the list does not change while a dialog is open; on OK the row re-renders in the current mode — no mixed per-row display
- [ ] State carrier: `wxITEM_CHECK` item "Expanded Values" in View with a constant label and Ctrl+E; disabled on the Backups tab with the check mark still readable; menu msgid in both languages
- [ ] Toggling speaks Announcement 8; focus stays on the list and an arrow key re-reads the row; both msgid pairs shipped («Показано розгорнуті значення» / «Показано збережені значення»)
- [ ] Search now matches the displayed rendering in both modes: toggling with a Filtered View active changes membership (`%JAVA_HOME%\bin` vs `C:\jdk21\bin` are different haystacks) and speaks twice — item 8, then item 9's count through the debounced path, separated by `filteredCountDelayMs`, no combined msgid
- [ ] Copy behaviour is untouched by this ticket (arrives with ticket 08); the StatusBar composition is untouched
