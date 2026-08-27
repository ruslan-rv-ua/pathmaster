# Locked delta-spec for v0.2.0

Type: task
Status: resolved (2026-08-27)
Blocked by: 02, 03, 04, 05, 06, 07, 08, 09, 10, 11, 12, 13, 14, 16

## Question

Assemble every resolved ticket into `spec.md` in this effort's directory — the locked
delta-specification on top of v0.1.0's spec, in its style: decisions with reasons, exact Catalogue
text marked for assembly, deviations from the PRD recorded with their tickets. This is the map's
destination.

Owed here, per the fog notes accumulated along the way:

- The **menu, accelerator and Announcement assembly**: every new command's menu home, the final
  accelerator table, the final closed Announcement set with exact wording in both languages — and
  one re-run of the menu-structure Checklist steps (31, B12, mnemonic gate) covering all menu
  growth at once.
- The **Release Checklist delta**: steps the new surfaces add, any v0.1.0 steps that change.
- **`settings.json` additions** decided by the feature tickets, folded into the settings failure
  taxonomy.
- A check on **release mechanics**: whether v0.2.0's version bump needs any decision the pipeline
  doesn't already make.
- The **requirement disposition table** for v0.2.0: each in-scope FR kept/rewritten/died, with the
  ticket that decided it — including anything that exercised its right to die.
- The list of what v0.2.0 in turn **defers**, each with its reason, so nobody re-adds them by
  accident.

Like v0.1.0's ticket 16, this is assembly, not decision: any contradiction found between resolved
tickets goes back to a ticket (or resolves by recency, recorded), never gets settled silently here.

## Resolution (2026-08-27)

**The locked delta-specification is assembled: [../spec.md](../spec.md) — the map's destination.**
Twenty-one sections in the v0.1.0 spec's style: decisions with reasons, exact Catalogue text
marked **[assembly]**, deviations recorded (§21). Everything the fog notes owed is in it:

- **Menu, accelerator and Announcement assembly** (§12–§14): the bar becomes
  File / Edit / View / Tools / Help; final accelerators fixed — Ctrl+F, **Ctrl+I** (07's proposal
  confirmed), **Ctrl+E** [assembly] for Expanded Values, **Ctrl+T** (08's proposal confirmed),
  Ctrl+C, F1 — checked against the v0.1.0 table, Windows reserved keys, NVDA (modifier-based, no
  collision) and wxMSW text-entry preprocessing (claims only Ctrl+C/X/V/A), per the standing
  research-first directive. The Filter's coarse toggle gets its own menu item ("Toggle Issues
  Filter") because every shortcut needs a menu home and neither a radio nor a check item can
  carry it honestly. **Fix Issues… carries no accelerator** [assembly] (the Settings…/Restore
  class). The Announcement catalogue closes at **fourteen**, full table with both languages in
  §13; non-Announcement Catalogue additions in §14. Steps 31/B12/mnemonic gate: voided by the
  growth, re-run once (§17).
- **Release Checklist delta** (§17): new steps for Search, Filter, Expansion Mode, Tree View,
  Fix Issues, Copy, plus 12's eight and 13's seven; changed v0.1.0 steps 2–4 (row reading gains
  the `#`), 15 (Tab cycle gains the field), 31 (Help menu two items).
- **`settings.json` additions** (§15): 06's three fields folded into §13's taxonomy — no new
  failure layer; dialog control labels fixed [assembly]; `data\help.html` added to §3's and the
  README's inventory.
- **Release mechanics** (§18): checked — nothing to decide; the pipeline already covers the
  bump, and the pin stays wxdragon 0.9.18 with the 0.9.20 delta pre-cleared if implementation
  wants it.
- **Requirement disposition table** (§1): all seven FRs plus the three new items, FR-reorder-dnd
  recorded as having exercised its right to die.
- **Deferrals** (§20): cut / deferred / declined-by-decision, each with its reason and ticket.

**One contradiction found and resolved by recency, recorded** (§2.1): v0.1.0 §10/§12 dropped the
PRD's index column; ticket 03 (later) confirmed the PRD's anchor that displayed `#` indexes are
original positions, which NVDA reads free as column text. Ticket 03 wins: the main list becomes
three columns `#` / Path / Status — the v0.1.0 rationale ("position is NVDA's setting") holds
only for an unfiltered list, and the column is permanent because the layout never reflows under
the user. Checklist steps 2–4 change accordingly. No other contradiction survived assembly; the
05↔06 membership amendment was already recorded at its source.

**The map's destination is reached**: no open question stands between the delta-spec and an
implementation effort. Implementation-order constraints handed over in §11: the borrow-discipline
retrofit is the first implementation ticket, and the Search debounce timer must be owned by a
non-Frame widget.
