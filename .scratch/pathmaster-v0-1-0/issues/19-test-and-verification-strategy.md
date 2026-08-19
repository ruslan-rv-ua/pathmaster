# Test and verification strategy

Type: grilling
Status: resolved
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

**2026-08-19, from ticket 17 (window layout):** the checklist gains one layout step — **drag the
window between monitors with different DPI scale factors and confirm the layout survives**. wx
documents this transition as layout-destructive in some versions; ticket 17 accepted it as a
documented risk rather than verifying against hardware, so the release checklist is where it gets
covered (skippable with a note when only one monitor is available).

**2026-08-19, from ticket 12 (elevation model):** the elevated instance is a *separate accessibility
surface*. NVDA interacts with elevated windows only when installed (uiAccess) or itself running
elevated — a portable NVDA copy goes deaf on the elevated instance entirely. The verification
checklist must therefore run its steps against the **elevated** instance explicitly (at minimum: the
ticket-18 sanity check, one list-row reading, one Announcement, the "Administrator: PathMaster"
title on Alt+Tab), and record which NVDA (installed vs portable) the pass used. The README owes the
user the installed-NVDA requirement for elevated use.

**2026-08-19, from ticket 24 (deaf-state detection):** D3's reopened condition is now answered, and
D3 itself holds. The `WM_GETOBJECT` signature (research/18 §5) enters the **harness only** — the
watcher backs the manual `NVDA+Tab` Sanity Check as diagnostics, never replaces it as the gate,
because the signature misses post-creation rejections. Nothing ships in-app; `nvda-drive.ps1` stays
a measurement tool, never a CI gate, exactly as this ticket decided. Threshold: "about one second,
tune in the harness".

**2026-08-19, from ticket 18 (resolved):** the condition on D3 fired — the deaf state **does** have a
detectable signature (focus change with no `WM_GETOBJECT (OBJID_CLIENT)` within ~1 s, observable via
`SetWindowSubclass`, no accessibility code). Per this ticket's own terms the automation/detection
question reopens; it is re-posed as
[ticket 24](24-deaf-state-detection-decision.md), which now blocks the spec. D3's other half —
`nvda-drive.ps1` is never a CI gate — is not touched by that ticket.

## Answer

Resolved 2026-08-19 by a grilling session, preceded (at the user's direction) by an internet survey of
current practice: NVDA's own Robot-Framework system tests, Guidepup's CI-driven NVDA automation, the
state of Win32 UI automation (WinAppDriver dead since 2020; FlaUI alive but .NET), functional-core /
imperative-shell testability, and solo-developer release-checklist practice. All six recommendations
were accepted as recommended, plus two follow-ups.

**D1 — The test boundary is functional core, imperative shell.** Everything pure is unit-tested in
Rust: splitting, Normalisation, the diagnostics rules and their severity order, the Editing Session
model (dirty-as-comparison, Checkpoint semantics, the Apply barrier), Snapshot serialize/parse
including the two-layer Corrupted validation, per-Scope rotation, the 8,191/32,767 threshold logic,
and the ticket-11 i18n registry gate. The GUI shell is deliberately untested by code — it is covered
by the Release Checklist.

**D2 — The registry adapter gets real-registry integration tests through a key-path seam.** The
adapter takes its registry key path as a constructor parameter; tests point it at a temporary key
under `HKCU\Software\PathMasterTest` on the live registry and hold ticket 05's hazard catalogue
closed forever: `(vtype, bytes)` preservation, `REG_SZ` vs `REG_EXPAND_SZ` round-trips, Absent as a
distinct state. Mocks were rejected: the hazards are precisely about real API behaviour, and every
one of them produces a *successful* write with wrong content — a mock would encode the same wrong
assumption twice. These run under plain `cargo test`, locally and in CI (a CI runner has its own
`HKCU`, so this is safe and also proves independence from the developer machine's registry state).

**D3 — `nvda-drive.ps1` stays a measurement tool and an assistant to the manual pass, never a CI
gate.** A semi-automated smoke run before the checklist is permitted but not required. Rationale:
ticket 18 — until the deaf-NVDA state has a detectable signature, an automated pass cannot
distinguish a regression from a broken environment, and a flaky red gate is a gate a solo developer
learns to ignore. Charting constraint 5 stands: the NVDA verdict is the user's, personally.

**D4 — No UI automation in v0.1.0** (FlaUI, WinAppDriver or otherwise). WinAppDriver is effectively
dead; FlaUI would drag a .NET toolchain into a Rust repo; every critical flow is already walked by
the Release Checklist, which must be run anyway for the accessibility verdict. Recorded on the map
as out of scope so it is not re-added "for completeness".

**D5 — Release CI gates, all on the artifact, never the build config** (ticket 04's rule): the
three-way version gate (t15), `cargo test` (D1 + D2 + the i18n gate), the dumpbin import-table gate
(t15), and a new **exe-size gate: ≤ 40 MB**. The Release Checklist is a release artifact, not a CI
step.

**D6 — The Release Checklist is recorded, not ritual.** The canonical checklist lives at
`docs/release-checklist.md` (created during ticket 16's assembly): the 17 D8 steps, the
elevated-instance section (t12 comment), the cross-DPI window-drag step (t17 comment), every NVDA
step gated on the ticket-18 sanity check. Each release produces a filled copy — results, NVDA
version, installed-vs-portable — attached to the GitHub Release. A failed step blocks the release.

**D7 — Cadence of the expensive numbers.** Exe size: automated, every release (D5). The
8,191/32,767 boundaries: unit-tested every release, confirmed against the real registry once
(largely already done in research 05). Cold start: re-measured only when the startup path changes —
the 25× margin (79.6 ms vs 2 s) makes a per-release measurement noise. Clean-VM run: once for
v0.1.0, then only when packaging changes.

**D8 — A property-based layer of exactly three properties** (`proptest`, dev-dependency, ~50 lines,
no growth without a new decision): split→join reproduces the raw string byte-for-byte (the naive
split makes this provable), Snapshot serialize→parse round-trips `(valueType, entries | absent)`,
and Normalisation is idempotent. These catch the class of exotic-bytes bugs that example-based
tests systematically miss — the same class as ticket 05's hazards.

**D9 — Push CI in addition to release CI**: `cargo test` + clippy on every push and PR to
`develop`, no artifact build. Minutes of free runner as insurance against "it passed on my
machine".

**Ticket 16 is deliberately *not* blocked on ticket 18.** The anomaly does not reproduce and its
investigation must not stall the spec; instead the spec records it as a documented open risk with
its interim mitigation (the sanity-check precondition on every NVDA pass) and the user-facing
workaround (restart NVDA). Noted on ticket 16.

New term **Release Checklist**: [CONTEXT.md](../../CONTEXT.md). No ADR: the no-NVDA-automation
choice is cheap to reverse once ticket 18 produces a detectable signature, so it fails the
hard-to-reverse test.

**2026-08-19, from ticket 22 (README and user docs):** the Release Checklist gains one non-NVDA
step — "`README.uk.md` is in sync with `README.md`, or the release did not change the README" —
the drift guard for the full Ukrainian README translation ticket 22 decided on. To be included
when `docs/release-checklist.md` is authored at implementation time.
