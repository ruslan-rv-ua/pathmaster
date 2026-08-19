# 05 — Logging pipeline

**Spec:** [spec §14](../../pathmaster-v0-1-0/spec.md)

**What to build:** The log a developer reads for a machine they cannot see: `logfmt` in core (line shape, levels, truncation — unit-tested) plus the platform `logwriter` and panic hook. After this ticket a healthy run leaves a 3-to-5-line skeleton and a panic leaves one `ERROR panic:` line, even under `panic=abort`.

**Blocked by:** 04 — Data Directory (the log lives there and rotation shares its file rules).

**Status:** resolved

- [x] Line format: `<RFC 3339 local+offset> <LEVEL> <area>: <message>`, exactly three levels padded to five chars (`INFO `, `WARN `, `ERROR`); English always, no `translate()` on any logging path
- [x] Healthy-run skeleton, never an empty file: startup line (version, elevation, data state, language), one audit line per Apply, clean-shutdown line (later tickets emit the Apply/shutdown lines; the API for them exists now)
- [x] Two absolute PII prohibitions enforced by the line-building API: no Entry/PATH text and no absolute filesystem paths in any record — derived facts only (counts, lengths, Value Type, Scope, `data: writable`); rejected settings values are truncated to ~100 chars with a marker
- [x] No logging failure touches the app: each record an independent attempt (no latch); failed writes silently dropped and counted; one `WARN log: N records were lost` on recovery; an unopenable log at startup = a run without a log, never Read-only Data
- [x] Rotation only at open: over 1 MB → rename to `pathmaster.log.old` (single overwritten generation); a failed rename (other instance holds it) carries on appending; the file opens with share read/write, one line per record
- [x] `std::panic::set_hook` appends one `ERROR panic:` line (message + `file:line`, no backtrace) directly past the logger, best-effort, cannot recurse; verified by a test harness or a debug-only trigger
- [x] Core `logfmt` unit tests: format, level padding, truncation marker

## Comments

Implemented 2026-08-19, TDD at the crate boundary (ADR-0007): 13 tests in
`crates/pathmaster-core/tests/logfmt.rs`, 8 in `crates/pathmaster-platform/tests/logwriter.rs`
(real temp directories, no mocks), 2 in `tests/panic_hook.rs`, 1 mapping test added to
`tests/datadir.rs`. Expected line texts are the spec's own examples, byte for byte.

- **Core `logfmt`**: `Level` (five-char padded), `Timestamp` (calendar fields + offset minutes;
  core never reads a clock), `Record` with **private fields** — the closed constructor set
  (`startup`, `apply_written`, `shutdown_clean`, `records_lost`, `settings_field_invalid`,
  `panic`) is what makes the PII prohibitions enforceable: every message is built from derived
  facts, and the one file-supplied inlet (rejected settings values) is truncated at 100 chars
  with a `… [truncated]` marker inside the constructor. `line()` emits the whole record,
  newline included, and flattens any smuggled `\r`/`\n` to spaces — one record per line is
  unconditional, not per-constructor. `DataState` names the read-only reason, structurally
  path-free.
- **Platform `logwriter::Logger`**: holds **no file handle** — each record opens (share
  read/write, deliberately no delete-sharing, per review), appends one line, closes; so no
  failure latches, and a second instance and the panic hook always get in. Rotation only in
  `Logger::open`: over 1 MB → rename to `pathmaster.log.old` (single overwritten generation),
  failed rename carries on appending (tested against a real no-delete-share holder). Unopenable
  at open (tested via a squatting directory) → `Logger::disabled()`, a run without a log.
  Failed writes drop silently and count; the first success prepends `WARN log: N records were
  lost` (tested by toggling the readonly attribute around three drops).
- **`now()`**: `GetLocalTime`/`GetSystemTime` + `SystemTimeToFileTime`; the offset is the
  measured local−UTC difference rounded to the minute — immune to `GetTimeZoneInformation`
  standard/daylight bias-selection mistakes. Tested against an independent oracle (civil-days
  arithmetic vs `SystemTime::now()`).
- **`panic_hook::install`**: formats via core, appends via the shared `append_handle`, swallows
  every error, touches no `Logger` state — nothing to recurse into. Verified by the harness
  the ticket asked for: the test binary re-runs itself filtered to a trigger test that installs
  the hook and panics; the parent asserts the one `ERROR panic: … (file:line)` line. The hook
  runs before unwind and abort alike, so the harness exercises the `panic=abort` path.
- **Wired into `main.rs`**: Data Directory state and elevation decided once, logger opened
  (Read-only Data = `disabled()`), hook installed only when a log exists, startup line emitted.
  `language: en` is a placeholder until the settings/i18n tickets decide the Interface
  Language — noted in code.

Two-axis review (Standards / Spec) run before commit; fixes applied: the duplicated
open-options triple shared as `pub(crate) append_handle`, and the share mode pinned to
read/write exactly (std's default silently adds delete-sharing, which would have made the
failed-rename branch unreachable between two real instances). Noted, not changed: a run whose
probe succeeds but whose every write then fails leaves the zero-byte file the spec calls
ambiguous (unavoidable without dropping the open probe); `Record::panic` necessarily carries
free payload text (ticket 21 accepts it) and is the one untruncated inlet. Rustfmt churn from
`cargo fmt --all` in ticket 02's core tests was reverted again to keep the commit scoped.
