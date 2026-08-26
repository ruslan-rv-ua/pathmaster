# Search bar contract

Type: grilling
Status: open
Blocked by: 03, 04, 05

## Question

FR-search: Ctrl+F, live substring filter over the active Scope's list, counter, ESC to clear and
return. The filtered-view semantics (03), the NVDA mechanism verdict (04) and the expansion toggle
(05) are in; specify the feature:

- **Search over which text** — raw, expanded, or "whatever the list currently shows" (the expansion
  toggle changes that)? And case folding: plain case-insensitive contains, or the full Normalisation
  reading (quote stripping, slash reconciliation)?
- Placement and construction: the field sits above the ListView (per PRD) — always visible, or
  appearing on Ctrl+F and collapsing on clear? What the prototype (04) said about focus/announcement
  shapes this.
- The counter ("N of M entries") — where it lives given no toolbar-decision assumptions (Banner?
  StatusBar field? beside the field?), and the debounced spoken count's exact wording, both
  languages (a new member of the closed Announcement set).
- ESC semantics: clear text + return focus to the list (PRD) — to which row (the focus rule from 03
  applies)? And what does ESC do when the field is already empty?
- Empty result set: what the list shows, what is spoken, and whether the editing-command rule
  from 03 has anything special to say.
- Per-Scope or shared: does switching tabs keep, share, or clear the search text (03 sets the frame;
  this ticket sets the value).
- Does search state survive Refresh, Restore, Apply?

## Input from ticket 05 (2026-08-26)

Expansion Mode is decided: app-wide, per-Run, display expands via Normalisation's own reading.
On search-over-which-text, [research/05](../research/05-var-expansion-best-practices.md) Q5
recommends matching the **currently displayed** rendering, so the spoken count equals the rows the
user will hear (raw-matching in expanded mode produces audibly inexplicable hits); Excel's
dual-mode Find is an explicit user choice defaulting to raw, and its Replace binds to raw only.
Consequence to weigh here: if search matches displayed text, toggling the mode recomputes Filtered
View membership.
