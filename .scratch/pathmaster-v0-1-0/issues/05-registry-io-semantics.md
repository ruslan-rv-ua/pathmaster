# PATH registry I/O semantics

Type: research
Status: resolved
Blocked by: —

## Question

What are the exact read and write semantics for User and System PATH on Windows, and which Rust crate
implements them?

- **Raw read.** `PATH` is `REG_EXPAND_SZ`; the app must read it **unexpanded** so `%JAVA_HOME%\bin` survives a
  round trip. Which crate (`winreg`, `windows`) can do the `RRF_NOEXPAND` read, and how?
- **Value type.** Real machines sometimes have `Path` as `REG_SZ`. Preserve the existing type, or normalise to
  `REG_EXPAND_SZ`? What breaks in each direction?
- **Missing value.** `HKCU\Environment` with no `Path` at all is a legitimate fresh-profile state. What does a
  read return, and what must a first write create?
- **The 32767 limit.** Is it per registry value, per variable, or the process environment block? Where does the
  number come from, and what actually truncates? FR-diag-length asserts a combined User+System check —
  verify or refute that claim.
- **WOW64 / redirection** for `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment` on x64.
- **Broadcast.** `SendMessageTimeout(HWND_BROADCAST, WM_SETTINGCHANGE, 0, "Environment", SMTO_ABORTIFHUNG,
  5000, ...)` — exact signature in the `windows` crate, the string encoding it expects, and what a timeout
  means in practice (which processes actually honour it).
- **External-modification detection** for FR-apply: compare the raw string against a re-read, or use the key's
  last-write time from `RegQueryInfoKey`? Which is reliable, and what does each miss?

Findings → `../research/05-registry-io.md`.

## Answer

Full findings, including measured output from this machine: [research/05-registry-io.md](../research/05-registry-io.md).

**Raw read.** `winreg::RegKey::get_raw_value` does it, and needs no `RRF_NOEXPAND` at all — it is built on
`RegQueryValueExW`, which has no expansion behaviour to suppress. It returns `RegValue { bytes, vtype }`;
`set_raw_value` writes both back verbatim. `RRF_NOEXPAND` matters only if you reach for `RegGetValueW`.
Measured on the System `Path` here:

```
RegQueryValueExW               : type=2 cb=1382  contains '%' = True
RegGetValueW (no RRF_NOEXPAND) : type=1 cb=1382  contains '%' = False   <-- expanded AND mistyped
RegGetValueW (RRF_NOEXPAND)    : type=2          contains '%' = True
```

The middle line is the trap: it expands *and* reports `REG_SZ` for a value stored as `REG_EXPAND_SZ`, so
"preserve the type I read" is not enough on its own.

**Value type: preserve, never normalise.** `REG_SZ`→`REG_EXPAND_SZ` turns a literal `%` in a real directory
name into an expansion; staying `REG_SZ` silently denies the user new `%VAR%` entries. Preserve by default and
offer an explicit convert action for the one collision (a `%VAR%` typed into a `REG_SZ` scope). **Trap:**
`winreg`'s `set_value::<String>` writes `REG_SZ` unconditionally — the same bug .NET ships, which is precisely
*why* real machines have a `REG_SZ` `Path`. Use `set_raw_value` only.

**Missing value.** Both APIs return `ERROR_FILE_NOT_FOUND` — a distinct domain state, neither `Ok("")` nor a
hard error. A first write creates `REG_EXPAND_SZ`, UTF-16LE, exactly one trailing NUL. Key-absent is a separate
case again (`RegCreateKeyEx`).

**The 32767 is the documented maximum size of one environment variable** — not a registry-value limit (that is
~1 MB), and the 32767 *environment block* limit was lifted after Server 2003. `setx`'s 1024 crop is `setx`'s
own; 2047 is the `sysdm.cpl` dialog; 2048 is a registry performance guideline. FR-diag-length is **partially
verified**: a combined check is right (per-scope alone is meaningless, since Windows merges the two at logon),
but it must run on the **expanded, merged** string, not the raw sum — measured 2207 raw versus 2198 expanded
here. What actually breaks on overflow is UNKNOWN — warn at a threshold rather than asserting a failure mode.

**WOW64: not an issue.** `HKLM` is shared and only `HKLM\SOFTWARE` is redirected; 64KEY, 32KEY and default all
returned byte-identical data. The real hazard is *writing* from a 32-bit build, where WOW64 rewrites a leading
`%ProgramFiles%` into `%ProgramFiles(x86)%`. Ship x64/ARM64 — which NFR-compatibility already requires.

**Broadcast.** `SendMessageTimeoutW(hwnd, msg, wparam, lparam, fuflags, utimeout, lpdwresult) -> LRESULT`.
`lParam` must be **UTF-16LE, NUL-terminated, and must outlive the call**; UTF-8 into the W variant yields
garbage and UTF-16 into the A variant yields `"E"` — both "succeed". **The timeout is per top-level window and
multiplies**: with 226 windows open here, TC-wm-settingchange's 5000 ms is a theoretical 18.8-minute freeze,
while a healthy broadcast measured **37 ms**. Use 1000–2000 ms, `SMTO_ABORTIFHUNG`, off the UI thread. What it
actually delivers: Explorer refreshes its block, so processes launched from Explorer inherit the new PATH.
Already-open shells do not.

**External-modification detection.** Re-read and compare on **`(vtype, raw bytes)`** is the reliable one. The
`RegQueryInfoKey` timestamp belongs to the *key*, and `HKCU\Environment` holds 39 values here — any installer
bumps it, producing false positives that train the user to click through the very dialog meant to protect them.
Use it only as a fast negative pre-filter. For live detection, `RegNotifyChangeKeyValue` with
`REG_NOTIFY_CHANGE_LAST_SET` is the right mechanism, still followed by a re-read to confirm.

**15 hazards** are catalogued in the research file. What they share is the important part: **every one produces
a *successful* write with wrong content** — no error surfaces at the moment of corruption. Of these, H15 (backing
up a decoded string rather than raw bytes, which makes the others unrecoverable) is carried into ticket 14, and
H7–H9 into ticket 13.
