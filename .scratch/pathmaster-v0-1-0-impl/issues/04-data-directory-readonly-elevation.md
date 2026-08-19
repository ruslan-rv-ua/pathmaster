# 04 — Data Directory, Read-only Data decision, elevation detection

**Spec:** [spec §3, §9 (detection only)](../../pathmaster-v0-1-0/spec.md) · ADR-0002

**What to build:** The platform startup facts every later ticket consumes: where the Data Directory is, whether this run is Writable or Read-only Data (and which of the three reasons), and whether the process is elevated. Pure decision logic is unit-tested; the probe runs against a real temp directory.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] Locate rule: `current_exe()` → resolve reparse points (`fs::canonicalize`) → strip `\\?\` (and `\\?\UNC\` → `\\`) → parent → append `data` (so a winget junction resolves to the real install dir, not `Links\`); resolution failure falls back to the unresolved path; `current_exe()` failure → Read-only Data
- [ ] Startup sequence: locate → `create_dir_all` → pid-unique probe file → mode decided once; the mode governs the UI only (startup predicts, Apply verifies)
- [ ] Exactly three Read-only reasons exist as distinct values: own location unknown / data directory cannot be created / data directory is not writable
- [ ] Read-only Data never relocates the directory and never prompts
- [ ] No single-instance lock (two instances are a designed state); an atomic-replace write helper exists for later consumers (temp file in the same directory + `MoveFileExW(MOVEFILE_REPLACE_EXISTING)`)
- [ ] Elevation detected once at startup via `GetTokenInformation(TokenElevation)` — never `TokenElevationType`
- [ ] Unit tests cover the path-mangling steps (`\\?\` strip, UNC form) and the reason selection; the probe is exercised against a writable and a read-only temp directory
