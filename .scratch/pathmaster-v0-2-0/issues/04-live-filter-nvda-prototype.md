# Live-filtered list under NVDA

Type: prototype
Status: open
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
