# 10 — Snapshot schema and rotation (core)

**Spec:** [spec §8](../../pathmaster-v0-1-0/spec.md) · ADR-0006

**What to build:** The Snapshot file format and the rotation policy in `pathmaster-core`, verified by `cargo test`: encode/decode of the decoded-not-raw schema, two-layer Corrupted validation, filename rules, and the independent per-Scope budget. (Writing Snapshots at Apply is ticket 13; the Backups tab is ticket 14.)

**Blocked by:** 02 — shares the Entry and Value Type types.

**Status:** ready-for-agent

- [ ] Schema (human-readable JSON, exactly one of `entries`/`absent`), from the resolved spec:

  ```json
  { "timestamp": "2026-08-19T14-32-07", "scope": "System", "valueType": "REG_EXPAND_SZ",
    "entries": ["C:\\Windows", "%JAVA_HOME%\\bin"] }
  ```
  ```json
  { "timestamp": "2026-08-19T14-32-07", "scope": "System", "absent": true }
  ```
- [ ] Two-layer validation: parse; then shape (`timestamp` string, `scope` `System|User`, `valueType` `REG_SZ|REG_EXPAND_SZ`, exactly one of `entries` (string array) / `absent: true`); any failure = Corrupted
- [ ] Filename rule: `YYYY-MM-DDTHH-MM-SS-<Scope>.json`, local time, Scope in the name, numeric suffix on same-second collision (`…-System-1.json`); Scope and ordering identifiable from the filename alone; foreign names and `.tmp` are not Snapshots
- [ ] Rotation: `maxBackups` (≥ 1) is an independent per-Scope budget; the oldest of that Scope is deleted on overflow; Corrupted files count toward their Scope's budget and rotate like valid ones; rotation tolerates files already deleted by another instance
- [ ] Property test: round-trip of `(valueType, entries|absent)` through encode/decode
