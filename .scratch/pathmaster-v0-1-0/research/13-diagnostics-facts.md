# Facts for the diagnostics contract (ticket 13)

Web research run 2026-08-19, before the grilling round, at the user's request. Four parallel
investigations against primary sources (MS Learn, Raymond Chen, tool source code read via GitHub,
UCRT sources shipped in the Windows SDK, plus an empirical test matrix run on this Windows 11
machine). Each claim below is marked documented / source-code / observed / anecdotal.

## 1. Quoted PATH entries — a consumer lottery (measured)

Empirical matrix: PATH entries written literally as `"C:\...\space dir"` and `"C:\...\semi;colon"`,
probed through each consumer with launch controls.

| Consumer | Strips quotes | Honors `;` inside quotes | Evidence |
|---|---|---|---|
| Win32 `SearchPathW` | **No** — quoted entry is dead | **No** | observed; docs silent |
| `CreateProcessW` exe search | **No** | **No** | observed |
| cmd.exe lookup | **Yes** | **Yes** | observed + [Raymond Chen](https://devblogs.microsoft.com/oldnewthing/20060929-06/?p=29533) |
| `where.exe` | **No** | **No** | observed |
| PowerShell 5.1 & 7 | **No** | **No** | observed |
| MSVC CRT `_spawnvp`/`_searchenv` | **Yes** | **Yes** | UCRT source `ucrt\env\getpath.cpp` |
| Rust `std::process::Command` / `env::split_paths` | **Yes** | **Yes** | [rust source, sys/paths/windows.rs](https://github.com/rust-lang/rust/blob/master/library/std/src/sys/paths/windows.rs) |
| Python `shutil.which` | **No** | **No** | source + observed |
| Node.js (libuv `search_path`) | **Yes** (also `'`) | **Yes** | [libuv src/win/process.c](https://github.com/libuv/libuv/blob/v1.x/src/win/process.c) + observed |

- No normative MS documentation of quote support exists; the "quotes protect semicolons" rule is
  Microsoft-authored only in Raymond Chen's post and a UCRT source comment.
- Rust's `env::join_paths` **refuses** to emit `"` inside an entry.
- Consequence A: `"C:\Program Files\foo"` and `C:\Program Files\foo` are the same directory for half
  the ecosystem and a broken entry for the other half (including the OS's own `CreateProcess` search).
- Consequence B: **splitting is itself ambiguous** — cmd/CRT/Rust/Node split quote-aware (a `;`
  inside quotes does not separate entries); the OS, PowerShell and Python split naively.

## 2. Duplicate detection — what the field does (tool source read directly)

| Tool | Case | Trailing `\` | `/`→`\` | `%VAR%` expand | 8.3 | Quotes | Flagged copy |
|---|---|---|---|---|---|---|---|
| PowerToys Env Variables | value: sensitive | — | — | display only | no | no | **does not dedupe entries at all** (by design: user reviews) |
| Chocolatey `Paths.cs` | insensitive | trims both sides | no | no (raw) | no | **quote-aware split + strip** | first match wins |
| pathman (Go) | insensitive | via `filepath.Abs` only | via Abs | `%USERPROFILE%` only | no | no | first |
| HermitOS PathManager | insensitive | no | no | no | no | no | groups all copies; reports **cross-scope** with side labels |
| WindowsPathEditor | insensitive | trims | **yes** | **yes** (compare expanded, keep raw — the two-string model) | no | no | hash over normalized |
| PS community scripts | insensitive (by PS default) | no | no | no | no | no | keep first |

- **Nobody touches the filesystem to compare** — no 8.3 resolution, no symlinks, no canonical casing.
- Search order documented: [CreateProcessW](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw)
  (app dir → … → PATH), and PATH itself left-to-right, first match wins
  ([`path` command docs](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/path)).
- Merge order System-then-User: Microsoft's own PowerToys source (`MainViewModel.cs`: "add USER value
  to the end of the SYSTEM value") + [MS Q&A corroboration](https://learn.microsoft.com/en-us/answers/a/1444838);
  no formal MS Learn page states it. So a cross-scope duplicate is a real runtime duplicate and the
  **System copy is the one that wins**.

## 3. Existence checks — blocking behavior and mitigation (documented + observed)

- **No async stat exists on Win32**; `GetFileAttributesW` on a dead UNC path blocks **20–45 s**
  (TCP 445 retransmit + name-resolution fallbacks; consistently reported), and a live-then-dead
  connection is governed by SMB `SessTimeout` default **60 s**
  ([SMB timeouts, MS archived](https://learn.microsoft.com/en-us/archive/blogs/openspecification/smb-2-x-and-smb-3-0-timeouts-in-windows)).
- A hung check **cannot be cancelled from within**; `CancelSynchronousIo` from another thread is
  best-effort only ([MS](https://learn.microsoft.com/en-us/windows/win32/fileio/cancelsynchronousio-func)).
  The only correct pattern is a prober thread whose *waiter* times out and abandons it.
- `GetDriveTypeW` classifies a root **without a network round trip** (documented return values,
  empirical speed): `DRIVE_REMOTE` / `DRIVE_NO_ROOT_DIR` / UNC prefix → slow lane; everything local
  checks in microseconds — 200 warm local checks are well under 10 ms total.
- Dedupe probes **by server** — one dead server must cost one hang, not one per entry.
- Rust: prefer `GetFileAttributesW` via `windows-sys` over `std::fs::metadata` — std opens a handle
  (`CreateFileW` + `FindFirstFileExW` fallback) which costs an extra round trip and **hides
  ERROR_ACCESS_DENIED**, the exact signal a diagnostic wants
  ([rust source, sys/fs/windows.rs](https://github.com/rust-lang/rust/blob/master/library/std/src/sys/fs/windows.rs)).
- Result states one `GetFileAttributesW` call can distinguish: directory exists / exists but is a
  **file** (inert in PATH — search appends `\name.exe` to the entry) / not found
  (`ERROR_FILE_NOT_FOUND`, `ERROR_PATH_NOT_FOUND`, `ERROR_BAD_NETPATH`…) / **access denied**
  (exists but unreadable — calling this "missing" is the long-standing .NET `File.Exists` complaint) /
  prober timeout (**unknown**, a distinct fifth state).

## 4. Over-length — what actually breaks, at which numbers

| Threshold | Applies to | What breaks | Currency |
|---|---|---|---|
| **1,024** | `setx` | truncates *and saves*, destroying everything past 1024 — the most common real PATH-destruction event ([setx docs](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/setx)) | documented, current |
| **2,047** | Vista/7 Shell32 bug; legacy edit dialog | hotfixed a decade ago; **2010-era folklore** on Win10/11, at most a legacy-dialog annoyance | historical |
| **8,191** | cmd.exe | cmd **ignores inherited env vars longer than 8191** — the PATH is simply absent inside cmd; `set PATH=%PATH%;…` fails ([KB 830473](https://learn.microsoft.com/en-us/troubleshoot/windows-client/shell-experience/command-line-string-limitation), updated 2026) | documented, current |
| **32,767** | one env variable | hard cap; cannot be materialized into any process environment ([SetEnvironmentVariableW](https://learn.microsoft.com/en-us/windows/win32/api/processenv/nf-processenv-setenvironmentvariablew)); practical max ~32,760 (Raymond Chen) | documented, current |
| env **block** | whole environment | limit lifted at Vista; not a modern constraint | historical |

- The unit is UTF-16 code units, and the number to measure is the **expanded merged** string
  (System + `;` + User) — consistent with ticket 05.
- Neither RapidEE nor PowerToys documents a length threshold — there is no industry number to copy;
  the honest primary warning is **8,191**, the honest hard error is **≥ 32,767**.

## Full agent reports

The four complete reports (verdict tables, all sources, empirical setup) are preserved in the
session transcript of 2026-08-19; the tables above carry every load-bearing fact and source.
