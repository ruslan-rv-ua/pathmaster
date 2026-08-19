# 05 — Logging pipeline

**Spec:** [spec §14](../../pathmaster-v0-1-0/spec.md)

**What to build:** The log a developer reads for a machine they cannot see: `logfmt` in core (line shape, levels, truncation — unit-tested) plus the platform `logwriter` and panic hook. After this ticket a healthy run leaves a 3-to-5-line skeleton and a panic leaves one `ERROR panic:` line, even under `panic=abort`.

**Blocked by:** 04 — Data Directory (the log lives there and rotation shares its file rules).

**Status:** ready-for-agent

- [ ] Line format: `<RFC 3339 local+offset> <LEVEL> <area>: <message>`, exactly three levels padded to five chars (`INFO `, `WARN `, `ERROR`); English always, no `translate()` on any logging path
- [ ] Healthy-run skeleton, never an empty file: startup line (version, elevation, data state, language), one audit line per Apply, clean-shutdown line (later tickets emit the Apply/shutdown lines; the API for them exists now)
- [ ] Two absolute PII prohibitions enforced by the line-building API: no Entry/PATH text and no absolute filesystem paths in any record — derived facts only (counts, lengths, Value Type, Scope, `data: writable`); rejected settings values are truncated to ~100 chars with a marker
- [ ] No logging failure touches the app: each record an independent attempt (no latch); failed writes silently dropped and counted; one `WARN log: N records were lost` on recovery; an unopenable log at startup = a run without a log, never Read-only Data
- [ ] Rotation only at open: over 1 MB → rename to `pathmaster.log.old` (single overwritten generation); a failed rename (other instance holds it) carries on appending; the file opens with share read/write, one line per record
- [ ] `std::panic::set_hook` appends one `ERROR panic:` line (message + `file:line`, no backtrace) directly past the logger, best-effort, cannot recurse; verified by a test harness or a debug-only trigger
- [ ] Core `logfmt` unit tests: format, level padding, truncation marker
