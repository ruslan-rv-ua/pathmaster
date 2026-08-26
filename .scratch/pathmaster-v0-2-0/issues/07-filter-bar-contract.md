# Filter bar contract — and the severity question

Type: grilling
Status: resolved (2026-08-26)
Blocked by: 03

## Question

FR-filter-bar: show only entries with a given status. The PRD's buttons are "All / Issues only /
Errors / Warnings" — but v0.1.0 deliberately has **no severity classes**: six Issue types, an empty
Status column as the only healthy state, no severity prefix anywhere (spec §7). Two decisions:

- **The severity question first.** Does v0.2.0 introduce a severity partition of the six types just
  to power two filter buttons — or does the filter speak the language the product already has:
  "All / With issues", or per-type filtering (`Missing`, `Duplicate`, …)? Introducing severity
  touches the Status column contract, Fix Issues wording, and the Catalogue; not introducing it
  rewrites the PRD's buttons. This is the ticket that decides it, once, for everything downstream.
- **Then the bar itself**: control material (radio row per the PRD, or a menu-based filter given
  the command-surface model from 02), where it sits, its tab-order position, what each state change
  announces (count wording, both languages), the "Filtered view — N of M" reminder's home, and
  whether filter state is per-Scope and whether it persists (`settings.json`) or resets per Run.
- Composition with Search is AND (03 confirmed or amended it); what this ticket owes is only the
  user-visible story: what is announced when both are active.

## Input from ticket 06 (2026-08-26)

The Search contract is settled and it pre-answers three of this ticket's bullets:

- **The "Filtered view — N of M" reminder has a home**: StatusBar field 0, where a Scope with an
  active Filtered View reads "User PATH: {n} of {m} entries ({k} issues)". The parenthetical keeps
  its old meaning — that Scope's Issues, not the view's — which is worth re-examining here, since
  a status filter is exactly the case where "4 of 50 entries (12 issues)" reads oddest.
- **The count msgids already exist** — catalogue items 9 and 10, six msgids: a short form for
  criteria changes ("{n} of {m} entries" / "No matching entries") and a Scope-named form for tab
  activation and Refresh. A filter state change should ride item 9 rather than mint new strings.
  Rule to inherit: **whenever no Filtered View is active, Announcement 1 speaks** — 06 made that a
  one-part condition (empty query); this ticket makes it two-part (empty query AND unnarrowed
  filter).
- **Two of the three new settings are shared**: `speakFilteredCount` and `filteredCountDelayMs` are
  named for the Filtered View precisely because the filter changes the same count.
  `searchEscapeReturnsFocus` belongs to the field alone. Whether the filter's own state persists is
  still this ticket's call; the Search text does not (it dies with the Run).

## Resolution (2026-08-26)

Researched first: [research/07-filter-bar-best-practices.md](../research/07-filter-bar-best-practices.md),
per the map's standing directive 7. Decisions:

1. **No severity — a recorded PRD deviation.** Every respected Error/Warning split is backed by a
   distinct consequence (ESLint/clippy exit codes; VS/VS Code filter compiler-supplied classes); no
   precedent exists of a tool minting severity *in order to* filter. PathMaster's six Issue types
   share one consequence, so the PRD's "Errors / Warnings" buttons join the toolbar and 05's
   "Warning" marker as recorded deviations. The Status column, Fix Issues and the Catalogue's Issue
   words are untouched.
2. **The Filter** (new `CONTEXT.md` term) is an **exclusive, per-Scope choice among seven states**:
   `All` / `With issues` / `Missing` / `Relative` / `Quoted` / `Duplicate` / `Empty`. An Entry is
   visible when its Issue set contains the chosen type; `With issues` means a non-empty Status.
   **Over-length is Scope-level** — it flags no Entry and no state selects it. Since types co-occur
   (Quoted freely; Missing+Duplicate possible), one Entry can satisfy several per-type states.
   Multi-select was rejected: an exclusive state is a strict subset of it and can grow later without
   breaking the model.
3. **Home: a View → Filter submenu of seven `wxITEM_RADIO` items; no on-window control.** Tab order
   stays tabs → search field → list → buttons. The items are disabled on the Backups tab, like
   Ctrl+F (06). Radio/toggle rows were rejected on the research's complaint trail (NVDA leaves
   toolbar radio-toggles unannounced, radio-group "x of y" has open defects); the live-NVDA proof
   that a native menu radio item reads its selected state is **appended to ticket 16**.
4. **One coarse-axis toggle accelerator**: from `All` → `With issues`; from any non-All state →
   `All`; the five per-type states are menu-only. Proposed key **Ctrl+I**; the final key and the
   submenu's mnemonics belong to assembly (15).
5. **Per-Scope state**, like the Search text and the Filtered View itself; the submenu's checked
   item mirrors the active Scope on tab switch (menu state is read when the menu opens, so this is
   mechanics, not an NVDA hazard).
6. **The Filter dies with the Run**: every Run starts at `All` on every Scope. No `settings.json`
   field, hence no new validation domain. (Event Viewer's temporary-unless-saved-as-Custom-View is
   the model; VS Code's half-persistence and Thunderbird's sticky pin are the counter-examples.)
7. **The Catalogue grows exactly one item — a pair of msgids, plural by {m}**:
   `"{filter}: {n} of {m} entries"` / `"{filter}: no matching entries"`
   (uk: «{filter}: {n} з {m} записів» / «{filter}: збігів немає»). Filter-state names reuse the
   menu/Status strings (`With issues` = «З проблемами»; the five type words are spec §7's — no new
   msgids for names). Spoken on every change to a non-All state, with the already-composed
   Search∧Filter count — one announcement, never two.
8. **Change to `All`**: with an empty query, **Announcement 1 speaks** — the ticket's two-part rule
   is now in force (Announcement 1 whenever query is empty AND the Filter is at All); with query
   text present, **item 9** (short count) speaks, since the remaining narrowing is search-only.
   Item 10 (tab activation / Refresh) and search keystrokes (item 9) are unchanged from 06.
9. **StatusBar field 0 names the state when the Filter ≠ All**:
   "User PATH: Missing — 4 of 50 entries (12 issues)" (uk: «PATH користувача: Відсутній — 4 з 50
   записів (12 проблем)»). "{n} of {m}" describes the view, the name says what narrows it, and {k}
   keeps its 06 meaning — the Scope's Issues. Search contributes no name: its "why" is visible in
   the field itself, one Ctrl+F away. This answers 06's hand-off about the parenthetical reading
   oddly.
10. **Ticket 03 amended mechanically**: the Filtered View's criteria change by the user's own
    narrowing actions — Search typing or Filter commands (03 said "typing" when typing was the only
    changer). Focus fall-back rules and the disabled set (Add, all reorder) are inherited from 03
    unchanged.

Downstream: NVDA menu-radio obligation → ticket 16; final accelerator, mnemonics and Catalogue
numbering → assembly (15); `CONTEXT.md` gains **Filter** and Filtered View's wording is aligned.
