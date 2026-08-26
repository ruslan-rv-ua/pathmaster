# Drag & Drop reorder — with the right to die

Type: grilling
Status: open
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
