# Filter bar contract — and the severity question

Type: grilling
Status: open
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
