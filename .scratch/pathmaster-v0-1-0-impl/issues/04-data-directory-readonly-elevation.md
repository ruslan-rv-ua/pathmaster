# 04 — Data Directory, Read-only Data decision, elevation detection

**Spec:** [spec §3, §9 (detection only)](../../pathmaster-v0-1-0/spec.md) · ADR-0002

**What to build:** The platform startup facts every later ticket consumes: where the Data Directory is, whether this run is Writable or Read-only Data (and which of the three reasons), and whether the process is elevated. Pure decision logic is unit-tested; the probe runs against a real temp directory.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Locate rule: `current_exe()` → resolve reparse points (`fs::canonicalize`) → strip `\\?\` (and `\\?\UNC\` → `\\`) → parent → append `data` (so a winget junction resolves to the real install dir, not `Links\`); resolution failure falls back to the unresolved path; `current_exe()` failure → Read-only Data
- [x] Startup sequence: locate → `create_dir_all` → pid-unique probe file → mode decided once; the mode governs the UI only (startup predicts, Apply verifies)
- [x] Exactly three Read-only reasons exist as distinct values: own location unknown / data directory cannot be created / data directory is not writable
- [x] Read-only Data never relocates the directory and never prompts
- [x] No single-instance lock (two instances are a designed state); an atomic-replace write helper exists for later consumers (temp file in the same directory + `MoveFileExW(MOVEFILE_REPLACE_EXISTING)`)
- [x] Elevation detected once at startup via `GetTokenInformation(TokenElevation)` — never `TokenElevationType`
- [x] Unit tests cover the path-mangling steps (`\\?\` strip, UNC form) and the reason selection; the probe is exercised against a writable and a read-only temp directory

## Comments

Implemented 2026-08-19, TDD at the crate boundary (ADR-0007): 15 tests in
`crates/pathmaster-platform/tests/datadir.rs` + 1 in `tests/elevation.rs`, all against real
filesystem behaviour (no mocks — the hazards being guarded are measured API behaviour).

- **`datadir` module**: `locate` (the exe-path half of the rule, exe path as a parameter),
  `startup` (= `current_exe()` → `decide`), `decide` (the reason selection — `startup`
  minus the one call a test cannot make fail, public as that test seam), `establish`
  (create + probe, directory as a parameter — the `ScopeKey::at` seam pattern),
  `strip_verbatim_prefix`, `write_replace`.
- **The winget hazard is tested live**: a real `mklink /J` junction in a temp dir; `locate`
  through the junction lands `data\` beside the real binary, never in `Links\`. Verbatim
  mangling covers `\\?\C:\` and `\\?\UNC\` (via `std::path::Prefix`, Unicode-safe), built
  on `Component`s rather than string surgery.
- **Reasons carry their evidence**: `ReadOnlyReason::{OwnLocationUnknown, CannotCreate(PathBuf),
  NotWritable(PathBuf)}` — the two reasons that found a directory hold it (settings may
  still be readable there, spec §3); own-location-unknown structurally has none. Illegal
  combinations (unknown location + a dir) are unrepresentable, per review.
- **The probe** is `probe-<pid>.tmp`, written and deleted (TC-file-structure's transient
  probe); `NotWritable` is exercised against a genuinely deny-ACL'd temp directory
  (`icacls /deny *S-1-1-0:(WD)`, restored on drop). `CannotCreate` via a file squatting on
  the `data` path. The Writable case asserts the established directory is left empty.
- **`write_replace`**: pid-unique `<name>.<pid>.tmp` in the target's own directory (rename
  is only atomic within a volume; two instances are a designed state), then
  `MoveFileExW(MOVEFILE_REPLACE_EXISTING)` via `windows-sys`. A failed replace cleans its
  temp file and leaves the target untouched — tested.
- **`elevation::is_elevated`**: `OpenProcessToken` + `GetTokenInformation(TokenElevation)`;
  `TokenElevationType` appears nowhere; any query failure reads as not elevated. Tested
  against an independent oracle — the token's mandatory integrity level SIDs from
  `whoami /groups` (locale-proof) — so the test is honest both on an elevated CI runner
  and an unelevated developer shell.
- **Not wired into `main.rs` yet, deliberately**: the first consumer is ticket 05's startup
  log line (version, elevation, data state); wiring a fact nobody reads would be dead code.
  "Decided once" is the documented caller contract, enforced when ticket 05 builds the
  startup sequence.

Two-axis review (Standards / Spec) run before commit; fixes applied: the reason-selection
seam extracted (`decide`) so the own-location-unknown branch is tested, the directory folded
into the reason variants (illegal states unrepresentable), one test name renamed off the
glossary's avoided "install directory", the leftover-files assertion deduplicated, `tempfile`
pinned to the minor. Rustfmt churn from `cargo fmt --all` in ticket 02's core tests was
reverted to keep the commit scoped; the equivalent 12 formatting-only lines in this crate's
`registry.rs` ride along (same crate, rustfmt-canonical, no semantic change).
