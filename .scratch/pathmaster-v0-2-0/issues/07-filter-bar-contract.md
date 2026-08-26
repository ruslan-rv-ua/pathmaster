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
