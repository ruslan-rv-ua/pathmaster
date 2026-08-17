# PATH registry I/O semantics

Type: research
Status: claimed
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
