# %VAR% expansion display toggle

Type: grilling
Status: resolved (2026-08-26)
Blocked by: —

## Question

FR-var-expansion-toggle: a command flips the list between raw entries (`%JAVA_HOME%\bin`) and
expanded ones (`C:\jdk21\bin`). v0.1.0 already expands at comparison time (Normalisation), never for
display. Decide the display-mode contract:

- Is the mode per-Scope or app-wide? Does it persist in `settings.json` or reset per Run?
- The PRD says the mode change is not an edit (no dirty state) — confirm, and decide whether it's a
  Checkpoint no-op too.
- What exactly is displayed in expanded mode for an entry whose `%VAR%` is undefined? The PRD invents
  a "Warning: Unknown variable" marker — but v0.1.0 has no severity classes, and an undefined var
  already flags `Missing` naturally (spec §7). Does the toggle need any new Issue type at all, or
  does the existing Status column already say everything?
- Editing while expanded: the Edit dialog edits the **raw** text (the stored truth). Confirm, and
  decide what the list shows mid-edit.
- What is announced on toggle (new member of the closed Announcement set — exact wording, both
  languages), and how does the current mode remain discoverable to NVDA afterwards?
- Interaction with Search: recorded here but decided in the Search ticket — expansion mode changes
  the visible text, so "search over which text" must not be decided twice.

## Resolution (2026-08-26)

**Expansion Mode is derived view state of the same class as a Filtered View — app-wide instead of
per-Scope.** Term recorded in `CONTEXT.md`.

1. **App-wide.** One flag for the application; both Scope tabs render alike. Search/Filter are
   per-Scope because they are queries against data (ticket 03); Expansion Mode is how the user is
   reading paths right now — different nature, different scope. It also keeps the one menu check
   mark truthful regardless of the active tab, and Ctrl+Tab never lands in a silently different
   mode.
2. **Per-Run, default raw.** Every Run starts in raw mode; nothing persists — no new
   `settings.json` field. A deliberate deviation from the Excel/VS Code persistence precedent:
   Show Formulas persists *with the document* because it is a document property; this is a
   property of the user's glance, and a run that silently opened expanded a week later would hand
   an NVDA user `C:\jdk21\bin` with the only clue buried in an unvisited menu.
3. **Not an edit, not a Checkpoint, invisible to Undo/Redo both ways.** The mode never touches the
   Working Copy, so Dirty cannot move; toggling creates no Checkpoint, and Undo/Redo never changes
   the mode — Ctrl+Z under expanded mode shows the rolled-back Working Copy, still expanded.
4. **Undefined `%VAR%`: literal in place.** Display expansion uses the same reading Normalisation
   already uses (`ExpandEnvironmentStringsW`; unknown names stay literal), so what is shown can
   never disagree with what is diagnosed. **No new Issue type, no inline marker** — the PRD's
   "Warning: Unknown variable" is a recorded deviation; the Status column's natural `Missing`
   already answers "why", and a literal `%VAR%` amid expanded paths is audibly anomalous by
   itself.
5. **Editing always works on raw.** Edit/Add dialogs carry the raw text (the stored truth)
   whatever the list shows — the Excel model (grid computed, editor formula). The list does not
   change while a dialog is open; on OK the row re-renders in the current mode. Mixed per-row
   display is ruled out: the toggle is strictly all-or-nothing.
6. **Announcement — the catalogue grows to eight.** One new pair of msgids, spoken on toggle,
   result-state phrasing: **"Showing expanded values"** / **"Showing raw values"**; uk:
   **«Показано розгорнуті значення»** / **«Показано збережені значення»**. Focus stays on the
   list; NVDA does not re-read a focused row on a background text change, so the Announcement is
   the immediate feedback and an arrow key re-reads the row. The count is not included — the
   toggle never changes it.
7. **Discoverability: a check menu item.** The command is a `wxITEM_CHECK` item in View with a
   constant label — the check mark is the canonical, always-inspectable state carrier (exact
   label and accelerator are assembly, per ticket 02). **Verification obligation**: "NVDA reads
   checked/not-checked on a native wx check menu item" attaches to the first upcoming NVDA
   prototype session (native menus expose `STATE_SYSTEM_CHECKED` via MSAA and NVDA's known
   checked-state failure is WinForms-ToolStrip-specific — but the map's rule is measured, never
   assumed); if no such session happens before assembly, it degrades to a ten-minute
   micro-prototype. The Release Checklist delta gains a step covering both the toggle Announcement
   and the check-mark read.
8. **Value Type does not condition the display.** Expansion is unconditional — a literal
   (`REG_SZ`) Scope expands in the display exactly as it already does in diagnostics
   (FR-diag-normalise expands regardless of Value Type). A literal Scope's runtime divergence is a
   property of the data, visible in the Value Type control, not a reason to fork the mode.

**Recorded here, decided elsewhere** (so nothing is decided twice):

- **Search** (ticket 06): research Q5's recommendation — match the **currently displayed**
  rendering, so the spoken count equals the rows the user will hear — plus its consequence that
  toggling the mode then recomputes Filtered View membership. Handed to ticket 06 as input.
- **Copy** (ticket 11): copy-raw vs copy-as-shown stays that ticket's question; the research's
  cross-cutting finding (every strong analogue binds extraction and mutation to raw — Excel's
  Replace operates on formulas only) is handed over as input.

**Amended by ticket [06](06-search-bar-contract.md) (2026-08-26).** Item 6's "the count is not
included — the toggle never changes it" holds only while **no Filtered View is active**. Ticket 06
decided that Search matches the currently displayed rendering, so with a Filtered View active the
toggle *does* change membership (`%JAVA_HOME%\bin` and `C:\jdk21\bin` are different haystacks). In
that state the toggle speaks its mode message and then the filtered count, separated by the tuned
debounce — two Announcements, not a new combined msgid. Item 7's verification obligation now has a
home: ticket [16](16-nvda-verification-round-2.md), which also carries the measurement this
amendment creates.

**Evidence**: [research/05-var-expansion-best-practices.md](../research/05-var-expansion-best-practices.md)
(gathered 2026-08-26, before grilling, per the standing directive). Notable: no
environment-variable editor (PowerToys, RapidEE) has such a toggle at all — the precedent base is
general-Windows, Excel Show Formulas the closest analogue; Windows itself owns the leave-literal
convention for undefined variables.

No new tickets (the verification obligation rides an already-planned session); no ADR — every
branch follows either the domain model already in place or uncontested precedent, and all of it is
reversible until the spec locks; per-Run non-persistence is the one deliberate precedent deviation
and is recorded here. Consumed by tickets 06 (search-over-what), 11 (copy-what) and 15 (menu item,
accelerator, Announcement assembly, Release Checklist delta).
