# 04 — Expansion Mode

**Spec:** [delta-spec §5, §13 (item 8)](../../pathmaster-v0-2-0/spec.md)

**What to build:** View → Expanded Values (Ctrl+E, `wxITEM_CHECK`) flips the application between raw and expanded path rendering — app-wide, both Scope tabs alike — and NVDA hears "Showing expanded values" / "Showing raw values". Because ticket 03 landed first, this ticket also owns the coupling: Search matches the displayed rendering, so the toggle changes membership under an active Filtered View and speaks twice — mode message, then the count through the same debounced path.

**Blocked by:** 03 (Search matches the displayed rendering — the coupling lands here).

**Status:** done — the toggle, both messages and the doubled speech under a Filtered View verified against live NVDA, see Comments

- [x] One app-wide flag; per-Run, default raw; no `settings.json` field, nothing persists
- [x] Display expansion is Normalisation's own reading (`ExpandEnvironmentStringsW`, process environment); an undefined `%VAR%` stays literal in place — no new Issue type, no inline marker; expansion is unconditional regardless of Value Type
- [x] Not an edit: no Checkpoint, invisible to Undo/Redo both ways; Ctrl+Z under expanded mode shows the rolled-back Working Copy, still expanded
- [x] Edit/Add dialogs always carry the raw text whatever the list shows; the list does not change while a dialog is open; on OK the row re-renders in the current mode — no mixed per-row display
- [x] State carrier: `wxITEM_CHECK` item "Expanded Values" in View with a constant label and Ctrl+E; disabled on the Backups tab with the check mark still readable; menu msgid in both languages
- [x] Toggling speaks Announcement 8; focus stays on the list and an arrow key re-reads the row; both msgid pairs shipped («Показано розгорнуті значення» / «Показано збережені значення»)
- [x] Search now matches the displayed rendering in both modes: toggling with a Filtered View active changes membership (`%JAVA_HOME%\bin` vs `C:\jdk21\bin` are different haystacks) and speaks twice — item 8, then item 9's count through the debounced path, separated by `filteredCountDelayMs`, no combined msgid
- [x] Copy behaviour is untouched by this ticket (arrives with ticket 08); the StatusBar composition is untouched

## Comments

**2026-08-27 (implementation)** — The pure half is `pathmaster_core::expansion`: a `Mode`
(`Raw` by `Default`, so "every Run starts raw" has one home), `toggled()`, `expanded()` — the
one reading the menu's check mark is written from — and `render()`, which is `Cow::Borrowed` in
raw mode (every rebuild renders every visible row, and the default mode has nothing to do) and
`normalize::expand`'s text in expanded mode. Only that step of Normalisation: quote stripping,
slash folding, the trailing separator and the case fold answer "are these the same path?", and a
rendering is not a comparison key — so `"%JAVA_HOME%\bin"` keeps its quotes and `%SystemRoot%/`
keeps its slash. `filtered::visible_indices` became generic over `AsRef<str>` to take either
side.

The window holds the flag in `ui::rendering::Rendering` — the mode and the `ProcessEnvironment`
it expands against, which never travel apart — shared by `Rc` with both `ScopeTab`s. Sharing
rather than threading it through is what makes "one flag, both Scopes alike" structural: every
question a tab answers about its view (`visible`, `rows`, `counts`) reads the mode now in force
and no caller can hand in a different one. `Cell`, not `Scoped`: ADR-0011 is about borrows
escaping into a dispatch and a `Copy` mode has none, the same shape `merged_length` already has.

`Command::ExpandedValues` carries Ctrl+E and lives in View, enabled wherever Search is (a
read-only Run may still change how it reads paths) and closed by the Backups tab's `session:
None`. Two new methods answer the item's *kind*: `carries_state()` — asked at build time, to
append a `wxITEM_CHECK` item rather than a plain one — and `state()`, written on every sync from
`Availability`'s new `expansion` field. wx toggles a check item's mark itself before the command
reaches the window, so the mark is only ever as true as the flag it is written back from.

**How each list is redrawn is decided by whether its membership moved**, and that was measured
rather than assumed. A rebuild has to re-mark the landing row, and NVDA re-reads a row marked in
a list holding the keyboard focus — the first live pass spoke the row *before* «Показано
розгорнуті значення», where §5 says the toggle speaks its message and an arrow key re-reads the
row. So where the mode changed how the Entries read but not which of them the view shows, only
the Path cells are written (`render_paths`, `render_status`'s mechanism and its reason: no item
state is touched, which is silent); where membership moved — the Filtered View case — the list is
rebuilt and lands on §2's row like any other membership change. The visible set and the concerned
Entry are both read **before** the flip: a list row is a position in the visible set, and after
the flip that set can be a different one.

The doubled speech rides the Search debounce rather than a second timer: the toggle arms the
active Scope's `count_due` and restarts its one-shot `Timer`, and `apply_search` became
`apply_criteria` — one tick speaks one count however many reasons it had, so a toggle landing
inside a typing window is answered by the tick the typing already asked for. Only the *speaking*
is debounced; the rows re-render the moment the command is given. ESC clears the owed count with
the tick it was on, since it speaks its own.

**Verified against live NVDA on this machine** (`tools/nvda-drive.ps1`, staged copy, Ukrainian
pass, System PATH's five `%SystemRoot%` Entries as the subject): Ctrl+E on a focused list speaks
«Показано розгорнуті значення» **and nothing else** — no row chatter — and the next arrow key
re-reads the row in the new rendering (`18; Шлях: %SYSTEMROOT%\System32\OpenSSH\` before,
`18; Шлях: C:\WINDOWS\System32\OpenSSH\` after, Стан «Дублікат» unchanged either way); Ctrl+E
back speaks «Показано збережені значення». Under a Filtered View the toggle speaks **twice**,
one `filteredCountDelayMs` apart and never combined: «Показано розгорнуті значення» then «Немає
збігів» (the query `systemroot` matches 5 raw Entries and none of their expanded readings), and
back again «Показано збережені значення» then «5 з 19 записів».

Cross-process probes additionally confirmed: the View menu carries «Розгорнуті значення(&E)
Ctrl+E» at 6012, unchecked at rest; the check mark follows the flag in both directions; on the
Backups tab the item reads **greyed and still checked**; the focused row survives the toggle
(row 13 of 19, before and after); the Edit dialog opened over an expanded row holds the **raw**
`%TEMP%\zzprobe` and the list does not change while it is up; and that same Entry renders
expanded (`C:\Temp\zzprobe`, Status `Missing`) in a **`REG_SZ`** Scope — expansion is
unconditional regardless of Value Type, with no new Issue type and no inline marker. The Add that
staged it was never applied, so nothing reached the registry.

The README's keyboard table and the Release Checklist's Expansion Mode steps are ticket 12's by
its own checklist, and are left to it.
