# Drag & Drop reorder — with the right to die

Type: grilling
Status: resolved (2026-08-26)
Blocked by: 01

## Question

FR-reorder-dnd is the one promised feature that conflicts with the product's identity: a mouse-only
gesture in an app built screen-reader-first, whose keyboard equivalent (Move Up/Down, one Checkpoint
each) already ships. Settled at charting: it stays in scope **with the right to die** — this ticket
decides, and either outcome is a resolution.

- The widget facts (01) come first: is an in-list reorder drag even reachable through wxdragon's
  bindings without raw-handle surgery?
- If reachable: does the drag harm the list for NVDA (drag feedback, drop-target states, focus after
  drop)? What does a drop record — one Checkpoint, named how? Does it respect the filtered-view rule
  (03) — dragging in a filtered list reorders what, or is it disabled there (recorded here, decided
  against 03's rule)?
- The honest weighing: what does D&D buy a sighted mouse user of *this* app, against its
  implementation and Checklist cost? "It was promised" is the argument for; the v0.1.0 sieve — every
  promise re-earns its place — is the argument that lets it die.
- If it dies: it moves to the map's Out of scope with the reason, and the delta-spec records the
  cut as a decision, not an omission — nobody re-adds it by accident.

## Resolution (2026-08-26)

Researched first: [research/10-dnd-reorder-best-practices.md](../research/10-dnd-reorder-best-practices.md),
per the map's standing directive 7. **FR-reorder-dnd dies.** The user's verdict, given the evidence,
was one word: kill it.

The weighing that killed it — every fact pulled the same way:

1. **Standards owe it nothing.** WCAG 2.5.7's obligation runs one way: a drag requires a non-drag
   alternative, never the reverse. Move Up / Move Down (+ `Alt+Up`/`Alt+Down`, v0.1.0 §15) already
   ship; D&D would add zero compliance and its absence costs none.
2. **It is invisible to NVDA by construction.** NVDA hears drags only through UIA Drag/DropTarget
   patterns, which the *app* must implement. A hand-rolled mouse-tracked drag in a native
   `SysListView32` through wxdragon exposes no pattern; making it speak would take a custom UIA
   provider — far beyond ticket 01's raw-`LVM_*` hatch. So the gesture could only ever be a
   redundant, mouse-only extra in a screen-reader-first product.
3. **The implementation is all bespoke.** Ticket 01: no reorder-drag helper exists in wxdragon or
   wxWidgets — begin-drag + mouse capture + motion tracking + `hit_test` + indicator
   (`LVM_SETINSERTMARK`, unbound) + delete/reinsert, with auto-scroll, flicker, cancel and
   above/below as known hand-rolled pitfalls. Plus mouse-only Release-Checklist steps — the one
   category the user's personal NVDA verification cannot cover.
4. **The promise was thinner than remembered.** v0.1.0 §20 deferred it "live in the tracker, not
   promised in the README" — the public promise surface never carried it.
5. **Comparables live without it.** Windows' own "Edit environment variable" dialog reorders by
   Move Up / Move Down only; PowerToys' editor does not lead with drag.
6. Already-settled map facts made it smaller still: under a Filtered View all reorder was disabled
   anyway (03), so it would have worked only on the full list.

Consequences recorded on the map: FR-reorder-dnd moves to **Out of scope** with the reason; the
delta-spec (15) must record the cut as a **decision, not an omission**. The conditional sub-questions
(drop feedback, Checkpoint naming, filtered-view interaction) die with it. Tickets 16 and 15 lose
this blocker with nothing else to change — neither carried a D&D item beyond the blocking edge.
