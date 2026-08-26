# NVDA verification round 2

Type: prototype
Status: open
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

<!-- Later tickets append their obligations here. Do not run the session until they are in. -->

Ends in a real verdict from the user. The prototype is thrown away; findings are recorded in the
Answer and consumed by [15](15-locked-delta-spec.md), which is blocked on this.
