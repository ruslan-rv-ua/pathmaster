# Live-filtered list under NVDA

Type: prototype
Status: resolved (2026-08-26)
Blocked by: —

## Question

The Search bar's whole premise is a `SysListView32` whose rows are deleted and reinserted **while
focus sits in a text field above it**, with a debounced spoken result count. v0.1.0 proved the
Announcement mechanism, but never this shape. Build a throwaway wxdragon window — text field over a
list of ~50 fake entries, live substring filter, debounced count announcement through the v0.1.0
announcement mechanism — and let the user verify against real NVDA:

- Does NVDA stay silent (as wanted) while rows are rebuilt under an unfocused list, or does it
  chatter/go deaf (the v0.1.0 deaf-list signature involved list rebuilds — watch for it)?
- Does the debounced count announcement speak reliably while focus is in the text field, and does
  typing interrupt it acceptably?
- Tab from the field into the filtered list: does focus land on a sensible row, and does NVDA read it?
- ESC in the field: can the prototype clear-and-return-focus, and what does NVDA say?

Ends in a real verdict from the user, per the map's standing constraint. Prototype is thrown away;
findings recorded in the Answer.

## Session log (2026-08-26)

**Research first** (standing constraint 7): findings captured in
[../research/04-live-filter-best-practices.md](../research/04-live-filter-best-practices.md).
Load-bearing for this ticket: 1400 ms is the best-sourced debounce (GOV.UK accessible-autocomplete,
chosen so typing echo finishes and stale counts die); NVDA fetches SysListView32 text live (stale
*text* unlikely) but item identity is positional (stale *identity* is the hazard); NVDA filters
background list events (silence expected but unverified — hence this prototype); recommended rebuild
order is Freeze → rows → set LVIS_FOCUSED|SELECTED → Thaw; ESC-keeps-focus-in-field is the
Windows/ARIA convention while the PRD wants ESC-to-list — the prototype makes both listenable.

**Prototype built and functionally probed** (filter counts, debounce latest-wins, ESC clear+restore,
banner text, ticket-03 focus rule — all verified cross-process without NVDA):
`prototypes/04-live-filter/` — run with

    cargo run --release --manifest-path .scratch/pathmaster-v0-2-0/prototypes/04-live-filter/Cargo.toml

Text field over 50 fake entries; live substring filter; debounced count through the exact v0.1.0
announcement mechanism (LIVEREGIONCHANGED on the banner StaticText). Options menu switches, all
mid-listen (accelerators work with focus anywhere): debounce 250/500/1000/1400 ms (Ctrl+1..4,
default 1400), plain vs Freeze/Thaw rebuild (Ctrl+5/6), speak count on/off (Ctrl+7), ESC→list vs
ESC-stays-in-field (Ctrl+8). Down-arrow in the field also enters the list. Status bar mirrors mode
and count. Spoken wording is placeholder — ticket 06 owns the real sentences.

## NVDA checklist (the user's part — the verdict)

Per ticket question, with research-flagged hazards folded in:

1. **Rebuild silence.** Ctrl+7 (count off). Focus the field, type and delete letters slowly and
   quickly. Expected: NVDA speaks only your typing echo, nothing per-row. Watch for chatter AND
   for the v0.1.0 deaf-list signature. Repeat under Ctrl+6 (Freeze/Thaw rebuild) — research found
   no documentation of NVDA behaviour under WM_SETREDRAW, so this comparison is new ground.
2. **Debounced count.** Ctrl+7 back on. Type "git" → expect "4 of 50 entries" once, after the
   pause. Type fast ("pyth" quickly) → expect only the final count, no stale intermediate counts.
   Compare Ctrl+4 (1400 ms) against Ctrl+2 (500 ms): does 1400 feel safe-but-sluggish, does 500
   talk over your typing echo? Type "zz" → expect "No matching entries".
3. **Tab / Down-arrow into the filtered list.** Filter to a few rows, Tab in — does NVDA read the
   focused row, and the right one? Research hazard: re-entering the list when the focused row sits
   at the *same index* as before the rebuild may be silent (NVDA issues #5713/#8825 category).
   Try: filter "git", Tab in, Shift+Tab out, change filter to "python", Tab in again.
4. **ESC.** With text in the field: ESC → list restored, "Filter cleared, 50 entries" spoken, focus
   in the list (Ctrl+8 mode A) — does NVDA also read the landing row, and is the double-speak
   acceptable? Then Ctrl+8 (mode B, focus stays in field) — which contract sounds better?
5. **Empty-result list.** Filter "zz", Tab from the field — where does focus go, what does NVDA
   say to a 0-row list?

Record per item: silent / spoke-as-wanted / chattered / went deaf, plus which debounce and which
ESC mode should become the spec's. The verdict closes this ticket; wording and final contracts go
to ticket 06.

## Resolution (2026-08-26)

**The user's verdict against real NVDA: the mechanism works — "everything else works great."**
Every question the ticket asked came back positive:

- Rows rebuilt under an unfocused list are **silent** — no chatter, no deaf-list signature.
  **Plain rebuild** (DeleteAllItems + reinsert) is the chosen strategy; Freeze/Thaw earned
  nothing and is dropped.
- The debounced count announcement through the v0.1.0 mechanism **speaks reliably** while focus
  sits in the field, and typing interrupts it acceptably.
- Tab / Down-arrow into the filtered list lands on a sensible row and NVDA reads it.
- ESC clear-and-return-focus works, and NVDA handles the landing.

Three prototype toggles graduate into **application settings** (`settings.json`), by the user's
explicit decision:

| Setting | Default |
| --- | --- |
| Speak result count | on |
| ESC returns focus to the list | on |
| Debounce delay | **minimal — 250 ms as prototyped** |

The user prefers the snappy minimum over GOV.UK's 1400 ms — primary-user preference outranks the
research default; the setting exists precisely so it can be slowed. Names, value ranges, and what
these do to the settings failure taxonomy belong to ticket 06 and the assembly ticket, as does the
final spoken wording (prototype wording was placeholder).

Assets: prototype `prototypes/04-live-filter/` (throwaway — not product code; kept beside the
v0.1.0 prototypes for reference), research
[../research/04-live-filter-best-practices.md](../research/04-live-filter-best-practices.md).
