# NVDA verification round 2

Type: prototype
Status: resolved (2026-08-27)
Blocked by: 05, 06, 07, 08, 09, 10, 11

## Question

The grilling tickets keep producing claims about NVDA that the map's standing constraint 4 forbids
anyone from assuming: **NVDA verification is done by the user personally, and a prototype ticket
ends in a real verdict, never in an inspector-tool guess.** Ticket 04 was round one — it proved the
live-filtered list. This ticket is round two: one throwaway wxdragon window and one listening
session that discharges every obligation the feature contracts have accumulated, rather than five
separate ten-minute prototypes.

It is blocked by every remaining feature contract that could add an item to the list; each of those
tickets **appends its obligation here** when it resolves. If a session happens to be run for another
reason before then, these items ride along with it and this ticket closes early.

### The list so far

1. **A checked menu item reads as checked** — from [05](05-var-expansion-toggle.md), item 7.
   Expansion Mode's whole discoverability rests on a `wxITEM_CHECK` item in View carrying the state.
   Native menus expose `STATE_SYSTEM_CHECKED` via MSAA and NVDA's known checked-state failure is
   WinForms-ToolStrip-specific — but that is an argument, not a measurement. Probe: open View,
   arrow onto the item in both states; does NVDA say "checked"/"not checked"?
2. **Two Announcements fired in succession both land** — from [06](06-search-bar-contract.md),
   decision 15. With a Filtered View active, the Expansion Mode toggle speaks its mode message and
   then the count, separated by the tuned debounce (250 ms by default). Probe: does NVDA speak both,
   or does the second cut the first off mid-word? Try it at 250 ms and at a longer delay, and record
   the shortest separation that reliably delivers both — the answer may become the delay's floor
   rather than a new setting.
3. **A menu radio item reads its selected state** — from [07](07-filter-bar-contract.md),
   decision 3. The Filter's seven states live as `wxITEM_RADIO` items in View → Filter, and the
   checked item is the only place the state can be re-read. Same native-MSAA argument as item 1's
   check items, same caveat: argument, not measurement. Probe: open View → Filter, arrow across the
   items in both an `All` and a narrowed state — does NVDA distinguish the selected item from the
   rest? Then switch Scope tabs with different Filters and re-open the menu: does the checked item
   follow the active Scope?
4. **A compressed tree node speaks its whole joined label** — from
   [08](08-tree-browser-contract.md), decision 6. Single-child chains render as one node whose label
   is the joined text ("Program Files\Java\jdk-21"). VS Code's compact rows once made NVDA skip
   folder names, but that defect lived in their custom web tree — in a native `SysTreeView32` the
   joined text IS the item label. Argument, not measurement. Probe: arrow onto a compressed node —
   does NVDA speak the full joined label, its level, and "N of M"?
5. **A three-part leaf label speaks in full** — from [08](08-tree-browser-contract.md), decision 7.
   A leaf like `bin (%JAVA_HOME%\bin) — Missing path` carries segment + raw parenthetical + Issue
   suffix in one label, the item's only audible channel. Probe: arrow onto such a leaf — does NVDA
   deliver all three parts, including the `%VAR%` text, without truncation?
6. **Focus lands and speaks after "Go to entry"** — from [08](08-tree-browser-contract.md),
   decisions 1 and 10. Activating a leaf (Enter in the tree, and the button separately) closes the
   modal and selects the Entry's row in the main list. Probe: does NVDA speak the landed row (all
   columns, as v0.1.0 proved for focus changes) with no dead silence and no dialog-title residue —
   and after Cancel/Esc, does it speak the restored focus position?

7. **A listview checkbox row reads its state, and Space announces the toggle** — from
   [09](09-fix-issues-dialog-contract.md), decision 6. The Fix Issues dialog's checkboxes are
   native `LVS_EX_CHECKBOXES` state images enabled through the raw-`LVM_*` hatch; comctl32 exposes
   the check state via MSAA, and both NVDA failures research found are non-native controls
   (CCleaner's DirectUI list, a web date-picker) — argument, not measurement. Probe: arrow across
   checked and unchecked rows — does NVDA speak "checked"/"not checked" along with the row's
   columns? Press Space on a row — is the new state announced without leaving the row? And since
   no check events reach the app through wxdragon, confirm Space still toggles the native state
   image at all with the wx event layer silent.

<!-- Later tickets append their obligations here. Do not run the session until they are in. -->

Ends in a real verdict from the user. The prototype is thrown away; findings are recorded in the
Answer and consumed by [15](15-locked-delta-spec.md), which is blocked on this.

## Session log (2026-08-27)

**Prototype built and functionally probed** (all cross-process, without NVDA):
`prototypes/16-nvda-round-2/` — run with

    cargo run --release --manifest-path .scratch/pathmaster-v0-2-0/prototypes/16-nvda-round-2/Cargo.toml

One frame, two Scope tabs (User: 10 fake entries, System: 6), each a `#`/Path/Issue list.
Everything the seven obligations need is on the menus, all accelerators frame-wide:

- **View → "Expanded values" (Ctrl+E)** — the `wxITEM_CHECK` item (probe 1). Toggling announces
  "Showing expanded values"/"Showing raw values" through the exact v0.1.0 mechanism (banner
  StaticText + LIVEREGIONCHANGED), rebuilds the list to the other rendering, and — when the active
  Scope's Filter is narrowed — follows with the count after the tuned debounce (probe 2).
- **Options → Debounce 250/500/750/1000 ms (Ctrl+1..4, default 250)** — the mode-to-count
  separation under test in probe 2.
- **View → Filter** — seven `wxITEM_RADIO` items (All / With issues / Missing / Relative / Quoted /
  Duplicate / Empty), per-Scope state, checked item follows the active tab (probe 3). Selecting one
  rebuilds and announces the ticket-07 count pair.
- **View → "PATH tree…" (Ctrl+T)** — modal tree dialog (probes 4-6): native `SysTreeView32`,
  hardcoded to the exact probe shapes — compressed chain node `Program Files\Java\jdk-21`, whose
  child is the three-part leaf `bin (%JAVA_HOME%\bin) — Missing`, plus "Unresolved variables" and
  "Relative entries" groups. Enter on a leaf or the default "Go to entry" button closes and selects
  that Entry's row in the main list; Enter on an inner node expands/collapses; Cancel/Esc restores.
- **Edit → "Fix issues…" (Ctrl+I)** — modal checkbox dialog (probe 7): `#`/Path/Issue/Action rows
  over the fixable four Issue types, native `LVS_EX_CHECKBOXES` enabled through the raw-`LVM_*`
  hatch ticket 01 mapped, Disk-Cleanup defaults (`%VAR%`-carrying Missing rows start unchecked —
  User Scope starts False,True,True,True,False). "Fix selected" reads the native states back
  (nothing mutates) and announces "Fixed {n} entries".

Cross-process probe results (19/19 PASS): both lists populate and filter per-Scope (10→2 on
Missing, 6→1 on Duplicate, states independent across a tab round-trip); banner speaks the mode text
first and the count after the debounce; tree holds all 18 nodes, "Go to entry" no-ops on an inner
node, Cancel closes; fix dialog lists the 5 flagged User rows with the expected default checks and
reads 3 checked rows back. All spoken wording is placeholder — ticket 15 owns the real sentences.

Build notes (for whoever deletes this): the crate resolved **wxdragon 0.9.20** (caret req, no
committed lock) — same wxWidgets 3.3.3 underneath as the app's pinned 0.9.18, so the native
controls under measurement are identical; ticket 01's upgrade-delta finding stands. Two things the
main app already knew were re-learned: the exe must embed the comctl32-v6 manifest (copied from
prototype 04 — without it wx blocks startup on a warning dialog *and* the process would load the
v5 controls, measuring the wrong thing), and this wxWidgets build has asserts on — calling
`get_item_state` for rows a not-yet-populated list doesn't have raises a blocking Debug Alert.

## NVDA checklist (the user's part — the verdict)

Numbering matches the obligation list above. Start NVDA, run the prototype, stay on the User tab
unless told otherwise. Record per item: spoke-as-wanted / partial (what was missing) / silent /
wrong.

1. **Checked menu item.** Open View, arrow onto "Expanded values". Press Enter, re-open View,
   arrow onto it again. Does NVDA say "checked" after and "not checked" before (or the
   equivalents)? Toggle a few times to be sure it tracks.
2. **Two announcements in succession.** Pick a narrowed Filter first (View → Filter → Missing).
   Press Ctrl+E and just listen: expected "Showing expanded values", a beat, then
   "Missing: 2 of 10 entries". At the default 250 ms (Ctrl+1): do both land, or does the count cut
   the mode message off mid-word? Try 500/750/1000 (Ctrl+2/3/4) and note the **shortest delay where
   both reliably survive** — that number may become the delay's floor in the spec.
3. **Menu radio items.** View → Filter: arrow across the seven items — does NVDA distinguish the
   selected one from the rest? Then set User's filter to Missing, switch to the System tab
   (Ctrl+Tab or the tab control), set System's to Duplicate, and re-open View → Filter on each tab:
   does the checked item follow the active Scope?
4. **Compressed tree node.** Set Filter back to All, press Ctrl+T. Arrow onto
   "Program Files\Java\jdk-21" — does NVDA speak the whole joined label, its level, and "N of M"?
5. **Three-part leaf.** Arrow onto its child, "bin (%JAVA_HOME%\bin) — Missing" — all three parts,
   %VAR% text included, no truncation?
6. **Go to entry, and Cancel.** Press Enter on that leaf: the dialog closes and the main list's
   row 1 should be selected — does NVDA speak the landed row (all columns), with no dead silence
   and no dialog-title residue? Re-open Ctrl+T, select a leaf, click/press the "Go to entry"
   button — same landing? Re-open once more and press Esc — does NVDA speak where focus returned?
7. **Listview checkboxes.** Press Ctrl+I. Arrow across the five rows — does NVDA speak
   "checked"/"not checked" with each row's columns (rows 1 and 5 start unchecked)? Press Space on
   a row — is the new state announced without leaving the row, and does a second Space flip it
   back? Then check/uncheck to some state you can count, press "Fix selected", and confirm the
   announced "Fixed {n} entries" matches what you checked — that number is the proof the native
   state survived the wx event layer's silence.

The verdict closes this ticket; findings feed the locked delta-spec (ticket 15).

## Answer (2026-08-27)

**The user's verdict against real NVDA: all seven obligations discharged — everything the feature
contracts assumed about native-control accessibility is now a measurement.** Per probe:

1. **Checked menu item** — NVDA reads the `wxITEM_CHECK` state in both directions. Ticket 05's
   View-menu carrier for Expansion Mode stands as specified.
2. **Two announcements in succession** — at the **250 ms default both messages reliably survive**,
   the count never cuts the mode message off. No floor needs raising: `filteredCountDelayMs`
   keeps 0–5000 with default 250 exactly as tickets 04/06 set it. Longer delays were available
   (500/750/1000) and not needed.
3. **Menu radio items** — NVDA distinguishes the selected Filter state, and the checked item
   follows the active Scope across tab switches. Ticket 07's menu-only Filter surface stands.
4. **Compressed tree node** — the joined chain label ("Program Files\Java\jdk-21") speaks in full
   with level and position. Ticket 08's chain compression stands.
5. **Three-part leaf** — segment + raw parenthetical + Issue suffix all delivered, `%VAR%` text
   included, no truncation. Ticket 08's leaf label format stands.
6. **Go to entry / Cancel** — the modal closes, the landed row speaks (all columns, no dead
   silence, no title residue); Esc/Cancel speaks the restored focus. Ticket 08's exit contract
   stands.
7. **Listview checkboxes** — rows speak "checked"/"not checked" with their columns, Space toggles
   with the new state announced in place, and the native state image survives the silent wx event
   layer: the read-back count matched the user's checks. **Observed order on [Fix selected]: the
   landed list row speaks first, "Fixed {n} entries" last — confirmed as ticket 09's designed
   focus-then-Announcement order and accepted by the user.** No contract needs amendment.

Consequence for [15](15-locked-delta-spec.md): every NVDA obligation tickets 05/07/08/09 parked
here is discharged with no spec change anywhere; all spoken wording in the prototype remains
placeholder and lands in 15's assembly.

Assets: prototype `prototypes/16-nvda-round-2/` (throwaway — not product code; kept beside the
earlier prototypes for reference). Its session log above records two build facts worth keeping:
the comctl32-v6 manifest must be embedded for the measurement to be of the v6 controls (copied
from prototype 04), and the resolved wxdragon 0.9.20 sits on the same wxWidgets 3.3.3 as the
app's pinned 0.9.18, so the native layer measured here is the one the app ships.
