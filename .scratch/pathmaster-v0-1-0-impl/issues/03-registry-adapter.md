# 03 — Registry adapter

**Spec:** [spec §4](../../pathmaster-v0-1-0/spec.md) · research/05

**What to build:** The `pathmaster-platform` registry adapter that reads and writes a Scope's PATH value raw — bytes and Value Type preserved — with Absent as a distinct state, verified by integration tests against a live temporary registry key. This is the only code that touches the registry; everything later (View, Apply, Refresh) calls through it.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] Read and write raw (`winreg::get_raw_value` / `set_raw_value`), never `set_value::<String>`; the existing Value Type is preserved, never normalised
- [ ] The registry key path is a constructor parameter (so tests point the same adapter at a test key); production paths are User `HKCU\Environment` / System `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment`, value `Path`
- [ ] Absent (`ERROR_FILE_NOT_FOUND`) is a distinct state from an empty value and from a read failure
- [ ] Writing zero Entries over a Present Scope writes an empty string, never deletes the value; a first write over an Absent Scope creates it as `REG_EXPAND_SZ`
- [ ] External-change detection primitive: re-read returns `(vtype, bytes)` for comparison — never the key's timestamp
- [ ] Integration tests (plain `#[cfg(windows)]`, no opt-in gate, no mocks) against a temporary key under `HKCU\Software\PathMasterTest`: `(vtype, bytes)` preservation, `REG_SZ`/`REG_EXPAND_SZ` round-trips, Absent as a distinct state; the test key is cleaned up
