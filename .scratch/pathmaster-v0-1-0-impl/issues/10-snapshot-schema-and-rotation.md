# 10 — Snapshot schema and rotation (core)

**Spec:** [spec §8](../../pathmaster-v0-1-0/spec.md) · ADR-0006

**What to build:** The Snapshot file format and the rotation policy in `pathmaster-core`, verified by `cargo test`: encode/decode of the decoded-not-raw schema, two-layer Corrupted validation, filename rules, and the independent per-Scope budget. (Writing Snapshots at Apply is ticket 13; the Backups tab is ticket 14.)

**Blocked by:** 02 — shares the Entry and Value Type types.

**Status:** resolved

- [x] Schema (human-readable JSON, exactly one of `entries`/`absent`), from the resolved spec:

  ```json
  { "timestamp": "2026-08-19T14-32-07", "scope": "System", "valueType": "REG_EXPAND_SZ",
    "entries": ["C:\\Windows", "%JAVA_HOME%\\bin"] }
  ```
  ```json
  { "timestamp": "2026-08-19T14-32-07", "scope": "System", "absent": true }
  ```
- [x] Two-layer validation: parse; then shape (`timestamp` string, `scope` `System|User`, `valueType` `REG_SZ|REG_EXPAND_SZ`, exactly one of `entries` (string array) / `absent: true`); any failure = Corrupted
- [x] Filename rule: `YYYY-MM-DDTHH-MM-SS-<Scope>.json`, local time, Scope in the name, numeric suffix on same-second collision (`…-System-1.json`); Scope and ordering identifiable from the filename alone; foreign names and `.tmp` are not Snapshots
- [x] Rotation: `maxBackups` (≥ 1) is an independent per-Scope budget; the oldest of that Scope is deleted on overflow; Corrupted files count toward their Scope's budget and rotate like valid ones; rotation tolerates files already deleted by another instance
- [x] Property test: round-trip of `(valueType, entries|absent)` through encode/decode

## Comments

Implemented 2026-08-20. Two modules land in the pure core — `snapshot` (the file: its shape, its
name, and what makes it Corrupted) and `rotation` (the budget) — held by 42 tests at the crate
boundary. Neither module does any I/O: a directory arrives as file names, a clock as the
`logfmt::Timestamp` the platform already produces for the log, and rotation *names* files to
delete rather than deleting one.

- **The schema is the spec's, character for character.** `Snapshot::encode` renders the two
  examples exactly as §8 prints them, pretty-printed and newline-terminated, because the reason
  this file is JSON at all is that a person can open it (ADR-0006).
- **Two layers, one word.** `Snapshot::decode` returns `Decoded::{Valid, Corrupted}` — the same
  all-or-nothing shape `settings.json` has, and the word `CONTEXT.md` already gives the user.
  There is no partial recovery and no guessed field. A UTF-8 BOM is stripped for the same reason
  `settings.json` strips one: a backup lost to an invisible character would be unexplainable.
- **The name is the only part of a Snapshot that still speaks when the content does not**, so
  `SnapshotName` keeps the name *as the directory spells it* and never re-renders it — what a
  caller deletes must be the file it was handed, letter for letter. (Re-rendering would have
  turned a hand-written `…-System-01.json` into a delete aimed at `…-System-1.json`.) The
  collision suffix is compared as the number it is, so `-10` follows `-2` rather than preceding
  it, and it only ever climbs: a suffix rotation has freed is never reissued.
- **A Snapshot is built from the name it will be written under** (`Snapshot::under`), not from a
  second reading of the clock, so the instant and Scope a file states and the ones its name states
  are the same values rather than two arguments a caller could get out of step.
- **Rotation cannot see content**, which is how "a Corrupted Snapshot counts toward its Scope's
  budget" is a structural fact rather than a rule to remember. Tolerating another instance falls
  out of the same shape: the selection is computed from the listing it was given, so a file that
  has since gone is simply not named, and the caller's delete treats not-found as success.

**One bug, caught in review and worth recording, because the first rule was plausible.** The
suffix originally took the *lowest free* number, so a gap rotation had left behind was filled.
Within one second the suffix is the age, and rotation frees the *oldest* name — so the fourth
Apply in a second was handed a name that sorted before the survivors, and the rotation that runs
straight after a write named the Snapshot that write had just taken. Neither module's own tests
could see it: `snapshot` was asked only "which name is free?" and `rotation` only "which name is
oldest?", and each answered correctly. `tests/rotation.rs` now runs the loop the two share —
three Applies, a rotation, a fourth Apply — and fails against the old rule.

Three decisions the ticket left open, and the roads taken:

1. **`valueType` belongs to the `entries` shape, not to every file.** FR-backup-ui listed it among
   the required fields, but §8's own Absent example prints no `valueType` — requiring one would
   make the spec's own file Corrupted. An Absent Scope has no value and therefore no Value Type.
   Spec §8 amended in this pass to say so.
2. **A field this version does not know is ignored, never Corrupted.** Forward compatibility in
   the direction that matters: a v0.2 field must not make today's Snapshots unrestorable in the
   version that wrote them. Corrupted stays what the ticket defined — a *missing or mistyped*
   required field.
3. **Letter case in the file name does not hide a Snapshot.** Windows names one file either way,
   so `-SYSTEM.JSON` parses as System; the `scope` field *inside* the file stays exact, because
   that is JSON content where `system` is simply a different string.

All three, plus the BOM the decoder strips, are recorded in spec §8 as an amendment rather than
left in this file: they are rules about what a Snapshot written by any version is *accepted* as,
which is a product promise, not an implementation note. ADR-0006 and `CONTEXT.md` both described
`valueType` as a peer of the Entries rather than part of their shape, and both now match.

Two smaller notes: the stamp pattern is a shape, not a calendar (`dddd-dd-ddTdd-dd-dd`) — nothing
here needs the 31st of February to be a different kind of wrong; and `rotation::overflow` floors
the budget at 1 rather than trusting every caller, because zero cannot reach it from
`settings.json` (spec §13) and obeying one would delete the Snapshot the Apply in progress has
just taken.

The third and last property test of the three spec §18 allows now exists, in the file §18 names
it in: `(valueType, entries|absent)` round-trips through encode/decode.
