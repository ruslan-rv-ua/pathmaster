# Repository and crate layout for the implementation effort

Type: grilling
Status: open
Blocked by: 20, 21

## Question

What are the module seams — what is a library, what is the GUI shell, and what does the crate
structure look like when implementation starts?

Graduated out of the map's **Not yet specified** on 2026-08-19 — deliberately last, because it is
shaped by every decision above. Ticket 19 has now fixed the two seams that matter most:

- **Functional core, imperative shell** is the test boundary, so it is also the natural crate
  boundary: the pure core (splitting, Normalisation, diagnostics, the Editing Session model,
  Snapshot schema, rotation, thresholds, the msgid registry) must be testable without wx, without
  the registry, and without a window.
- The **registry adapter is parameterised by key path** (ticket 19 D2) — its seam is already
  designed; this ticket only places it.

To decide:

- One crate with modules, or a workspace (`pathmaster-core` + binary)? The single-exe build profile
  (ticket 04) and CI pins must survive whichever it is.
- Where the imperative shell's non-GUI pieces live: the announce() function, the Timer-drain
  diagnostics pump, the elevation relaunch, the Data Directory resolution.
- Where tests live (unit in-module, integration under `tests/`, the three proptest properties, the
  registry integration tests behind what cfg/feature so `cargo test` stays runnable on a
  non-Windows dev box or skips gracefully).
- What of `.scratch/` tooling (nvda-drive.ps1) is promoted into `tools/` permanently.

When this closes, ticket 16 has everything it needs.

## Comments

**2026-08-19, from ticket 21 (log format):** now unblocked — both blockers are resolved. What 21
hands this ticket: the log **format** (line shape, truncation, rotation thresholds) is pure and
belongs in the core's testable surface, while the log **writer** (append, rotate-at-open, the
silent-drop-and-count behaviour) is imperative shell; the **panic hook** writes past the logger
straight to the file, so its placement must not create a core→shell dependency. The `area` tokens
(`startup`, `apply`, `settings`, `log`, `panic`) will tend to mirror module seams — worth a glance
when naming modules, not a constraint.
