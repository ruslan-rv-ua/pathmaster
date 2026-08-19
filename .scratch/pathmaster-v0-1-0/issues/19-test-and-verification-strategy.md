# Test and verification strategy

Type: grilling
Status: open
Blocked by: —

## Question

How is PathMaster v0.1.0 regression-tested by one person before each release, and what is worth
automating at all?

Graduated out of the map's **Not yet specified** on 2026-08-19, when the accessibility contract
(ticket 09) produced the artifact it was waiting on: the NVDA verification checklist (ticket 09, D8) —
a manual, user-run script with expected spoken text per step, gated on the ticket-18 sanity check.

- **The manual layer is settled in shape**: the D8 checklist, run personally on real NVDA before each
  release (charting constraint 5). This ticket decides what surrounds it.
- **What is automatable at all?** The registry I/O hazards (ticket 05 catalogued 15, all producing a
  *successful* write with wrong content), the Editing Session model (dirty-as-comparison, Checkpoint
  semantics, the Apply barrier), Normalisation, diagnostics rules — these are pure logic and look
  unit-testable. Decide the boundary: what is tested in Rust, what only the manual script covers.
- **Is any NVDA automation worth it?** [tools/nvda-drive.ps1](../tools/nvda-drive.ps1) exists from the
  ticket-02 measurements. Decide whether it becomes a repeatable harness or stays a measurement tool —
  bearing in mind the ticket-18 anomaly (NVDA can go deaf on a list and invalidate a whole pass).
- **CI's role.** Ticket 04 decided release CI gates on the artifact's import table, never the build
  config. What else gates a release: `cargo test`? The checklist as a signed-off item?
- **When the spec's numbers are verified** — cold start, exe size, the 32767 limit behaviour — once per
  release or once ever?

## Comments

**2026-08-19, from ticket 12 (elevation model):** the elevated instance is a *separate accessibility
surface*. NVDA interacts with elevated windows only when installed (uiAccess) or itself running
elevated — a portable NVDA copy goes deaf on the elevated instance entirely. The verification
checklist must therefore run its steps against the **elevated** instance explicitly (at minimum: the
ticket-18 sanity check, one list-row reading, one Announcement, the "Administrator: PathMaster"
title on Alt+Tab), and record which NVDA (installed vs portable) the pass used. The README owes the
user the installed-NVDA requirement for elevated use.
