# 03 — Registry adapter

**Spec:** [spec §4](../../pathmaster-v0-1-0/spec.md) · research/05

**What to build:** The `pathmaster-platform` registry adapter that reads and writes a Scope's PATH value raw — bytes and Value Type preserved — with Absent as a distinct state, verified by integration tests against a live temporary registry key. This is the only code that touches the registry; everything later (View, Apply, Refresh) calls through it.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Read and write raw (`winreg::get_raw_value` / `set_raw_value`), never `set_value::<String>`; the existing Value Type is preserved, never normalised
- [x] The registry key path is a constructor parameter (so tests point the same adapter at a test key); production paths are User `HKCU\Environment` / System `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment`, value `Path`
- [x] Absent (`ERROR_FILE_NOT_FOUND`) is a distinct state from an empty value and from a read failure
- [x] Writing zero Entries over a Present Scope writes an empty string, never deletes the value; a first write over an Absent Scope creates it as `REG_EXPAND_SZ`
- [x] External-change detection primitive: re-read returns `(vtype, bytes)` for comparison — never the key's timestamp
- [x] Integration tests (plain `#[cfg(windows)]`, no opt-in gate, no mocks) against a temporary key under `HKCU\Software\PathMasterTest`: `(vtype, bytes)` preservation, `REG_SZ`/`REG_EXPAND_SZ` round-trips, Absent as a distinct state; the test key is cleaned up

## Comments

Implemented 2026-08-19, TDD at the crate boundary (ADR-0007): 9 integration tests in
`crates/pathmaster-platform/tests/registry.rs`, each against its own live subkey of
`HKCU\Software\PathMasterTest` (unique per test + pid, Drop-deleted; cleanup verified after
the run). `winreg 0.56` target-gated to Windows; the module is `#[cfg(windows)]`.

- **`registry` module**: `ScopeKey` (`Hive` + key path + value name, all constructor
  parameters; `user()` / `system()` hold the TC-registry-keys production paths),
  `RawValue { Absent, Present { value_type, bytes } }`, `RegistryError { Io, UnsupportedType }`.
- **Read** goes through `get_raw_value` (`RegQueryValueExW` — no expansion behaviour exists
  to suppress); absent key and absent value both map to `RawValue::Absent`, distinct from
  `Present` with empty bytes and from `Err`. A non-string type (e.g. `REG_BINARY`) is a
  distinct `UnsupportedType` failure, never garbage and never Absent (research/05 §1.3).
- **Write** goes through `set_raw_value` with the caller's Value Type — `set_value::<String>`
  appears nowhere. Bytes are UTF-16LE with exactly one trailing NUL (`cbData = 2×(chars+1)`,
  H6); the empty string writes a lone NUL (`cbData = 2`), never deletes. The key is opened
  `KEY_SET_VALUE`, created on first write.
- **The Absent → `REG_EXPAND_SZ` default deliberately lives in core**, not the adapter: a
  Session over an Absent Scope loads typed `REG_EXPAND_SZ` (ticket 02), and Apply writes the
  Session's type. The adapter creates with whatever type it is handed, because Restore may
  legitimately apply a `REG_SZ` Snapshot over an Absent Scope. Enforcing it below the Session
  would corrupt that path.
- **External-change detection** is `PartialEq` on `RawValue` — re-read and compare
  `(vtype, bytes)`. Tested both ways: a byte-identical external rewrite is *not* a change
  (unlike the key timestamp, H13); a type flip with identical text *is* (the .NET bug).
- **`RawValue::decode()`** bridges to core's `ScopeValue`: UTF-16LE up to the first NUL —
  where every Windows reader of a registry string stops — lossy on unpaired surrogates. The
  raw bytes stay authoritative for comparison; an externally planted double-NUL value
  round-trips its exact bytes through read (H6 test) and decodes to the text Windows sees.

Two-axis review (Standards / Spec) run before commit. Standards: one hard glossary breach
fixed (a test local named `snapshot` where CONTEXT.md demands Baseline); the test-side
UTF-16 encoder is a deliberate independent oracle (commented so); ADR-0006's "nothing else
handles raw bytes" consequence amended to name the adapter's transport-only raw-bytes role.
Spec: the live-production-key read probe was cut as scope creep (spec §18 scopes integration
tests to the temp key, and a wrong production path reads as Absent, so the probe proved
nothing). Noted for ticket 13: `read`/`write` each open their own handle; if Apply wants
research/05 §7.2's same-handle compare-then-write TOCTOU shrink, the adapter grows a
combined primitive then.
