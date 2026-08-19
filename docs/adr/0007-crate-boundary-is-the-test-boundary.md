# The crate boundary is the test boundary

PathMaster is a three-crate Cargo workspace — `pathmaster-core`, `pathmaster-platform`, and the
`pathmaster` binary — for an application that will likely stay near five thousand lines. Prevailing
guidance (matklad's "Large Rust Workspaces", the Rust Project Primer) puts the workspace threshold an
order of magnitude higher and warns against premature splitting, so the reasoning is recorded.

**The seams were not invented for this decision.** The test strategy (ticket 19) fixed three tiers
before any layout existed: pure rules unit-tested without wx, without the registry, without a window;
a registry adapter integration-tested against the live `HKCU` under a temporary key; a GUI shell
covered by the Release Checklist only. Each crate is one tier — the split records boundaries that were
already load-bearing, which is precisely the case the "don't split prematurely" advice exempts.

**No test ever links wxWidgets.** wxdragon compiles wxWidgets 3.3.3 from source, statically. In a
single crate every `cargo test` binary links that C++ archive, taxing exactly the fast feedback loop
the pure core exists to provide. With the split, `cargo test -p pathmaster-core` and
`-p pathmaster-platform` never touch it; the one wx-linking test (the msgid smoke test in the binary)
runs in CI, where wx is built anyway.

**Structural beats disciplinary — again.** "The core does not depend on wx" as a module convention is
enforced by code review; as a crate boundary it is enforced by Cargo, which cannot express the illegal
dependency without editing a manifest. The same trade was made in ticket 11 (one Catalogue made
structural) and it is this project's standing preference.

## Consequences

- **Dependency direction is fixed**: `pathmaster` → `pathmaster-platform` → `pathmaster-core`, never
  the reverse. The panic hook illustrates the rule: core provides the log line *format* (pure),
  platform owns the hook that writes it past the logger — no core→shell edge exists.
- **`pathmaster-core` builds and tests on any OS** as a free side effect. This is a bonus, not a
  promise — nothing outside core claims portability.
- **The binary stays bin-only** (no lib target). Integration tests cannot import a binary crate, and
  that is correct here: the GUI's coverage is the Release Checklist, and anything worth an automated
  test must first move down a tier.
- **The single-exe build profile survives**: `[profile.release]` (ticket 04) lives once in the
  workspace root's virtual manifest, and release CI still gates on the linked artifact's imports,
  never on build config.
- **Merging back is the escape hatch.** If the workspace ever costs more than it returns, collapsing
  three crates into one is mechanical; re-cutting seams out of a fused crate later is not. The split
  errs on the side that is cheap to undo.
