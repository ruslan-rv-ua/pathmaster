# Repository and crate layout for the implementation effort

Type: grilling
Status: resolved
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

## Answer

Resolved 2026-08-19 with the user, per the standing directive: best practices researched first
(matklad's "Large Rust Workspaces", Rust Project Primer, the Cargo book, corrode.dev), then decided.

**Topology: a three-crate Cargo workspace — the crate boundary is the test boundary
([ADR-0007](../../../docs/adr/0007-crate-boundary-is-the-test-boundary.md)).** Prevailing guidance
puts the workspace threshold at ~10k lines, but two local facts override it: ticket 19 already fixed
three test tiers (the seams are not invented, only recorded), and wxdragon statically compiles
wxWidgets from source, so in a single crate every `cargo test` binary would link that C++ archive.
With the split, no test ever links wxWidgets.

**Layout (flat, matklad-style): virtual manifest root, all crates under `crates/`, folder named
exactly as the crate.** `[profile.release]` from ticket 04 lives once in the root manifest; release
CI keeps gating on the artifact's imports, never the build config.

- **`crates/pathmaster-core`** — pure, no I/O, builds and tests on any OS (a free bonus, not a
  promise — the user confirmed non-Windows dev is theoretical). Modules (indicative, renameable;
  the *inter-crate* seams are what the spec fixes hard): `path` (split/join), `normalize`,
  `diagnostics`, `session`, `snapshot`, `rotation`, `thresholds`, `settings` (parse + per-field
  fallback rules), `logfmt` (line shape, truncation, levels), `msgids` (msgid-constant registry +
  `.po` integrity gate via polib parsing — no wx).
- **`crates/pathmaster-platform`** — imperative shell without wx; depends on core. `registry`
  (adapter, key path as constructor parameter), `datadir`, `elevation`, `logwriter`
  (append, rotate-at-open, silent-drop-and-count), `panic_hook` (writes past the logger straight to
  the file; core supplies only the line format, so no core→shell edge), `broadcast`
  (`WM_SETTINGCHANGE`).
- **`crates/pathmaster`** — the GUI shell, **bin-only, no lib target** (integration tests cannot
  import a binary, and the GUI's coverage is the Release Checklist by ticket 19). `ui/*`, `announce`,
  `pump` (Timer-drain), `catalog` (TranslationsLoader, embedded `.mo`), `main.rs` (startup order:
  panic hook → settings → language → window), `build.rs` (polib → `.mo`, llvm-rc → icon/VERSIONINFO),
  `i18n/` with the `.po` files. `[[bin]] name = "PathMaster"` so cargo emits `PathMaster.exe` and the
  release workflow has no rename step.

**Tests.** Unit tests in-module throughout. The three proptest properties each live in the test
file of the module they constrain — they deliberately exercise the core's *public* surface, and
`proptest` stays a dev-dependency of core alone. Registry integration tests are plain
`#[cfg(windows)]` modules against a temporary key on the live `HKCU` — no opt-in feature gate (a
gate nobody enables is a test nobody runs); on non-Windows they do not exist, which *is* the ticket's
"skips gracefully". **Ticket 11's msgid gate is split**: `.po` integrity (placeholders, mnemonic
uniqueness, fuzzy-exclusion, self-sensitivity) is checked wx-free in core via polib; one smoke test
(`get_string(…).is_some()` over a few keys through real wxTranslations) lives in the binary and runs
in CI, where wx is built anyway.

> **Amended 2026-08-20 by impl tickets 02 and 09.** The property sentence originally read "The
> three proptest properties live in `crates/pathmaster-core/tests/properties.rs`". A shared file
> was the one thing about the layout that separated a rule from the tests that constrain it: the
> split→join property reads with the split examples it generalises, and Normalisation idempotence
> reads with the five pipeline steps it holds — both need the same fixtures (ticket 02 landed the
> first this way; ticket 09 the second). Everything else this paragraph fixes is unchanged, the
> cap of exactly three included, and spec §18 now names the three files.

**Tooling promotion.** `nvda-drive.ps1` moves from `.scratch/` to a permanent repo-root `tools/`,
and the ticket-24 `WM_GETOBJECT` watcher joins it there when built — the Release Checklist is a
permanent document and the harness backing its Sanity Check cannot live in an effort-scoped
scratch directory. The rest of `.scratch/` remains this wayfinding effort's archive.

No new CONTEXT.md terms — crate layout is implementation, and the glossary stays free of it.
