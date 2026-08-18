# Portable data directory contract

Type: grilling
Status: resolved
Blocked by: —

## Question

Where does `data/` live, and what happens when the exe's own directory is not writable?

Settled at charting: portable-first, `data/` next to the exe, scoop declares `persist: data`, and
NFR-no-registry-writes is reworded from "nothing in AppData" to "nothing outside the app's own directory".
What remains is the behaviour at the edges.

- **Read-only install location.** Someone will drop the exe in `C:\Program Files\`. Refuse to start? Run in a
  read-only mode? Prompt for a writable location? Silently degrade is not an option — the app's whole promise is
  that it writes backups before it touches the registry.
- **Honesty of the "no traces" claim.** Under winget the exe lands inside `%LOCALAPPDATA%`, so `data/` does
  too. Decide how the README states this without the promise becoming a lie.
- **Path resolution.** `std::env::current_exe()` versus the symlink winget puts on `PATH`: resolving the link
  target and resolving the link itself place `data/` in different directories. Pick one and state why.
- **Two instances.** One elevated, one not, both writing `data/`. Single-instance lock, or last-write-wins?
  A second instance is *likely* here, because elevation is implemented by relaunching the app.
- **ACLs.** Files written by an elevated instance may be unwritable by the normal one afterwards — which breaks
  backup rotation silently. Mitigation, or accepted with a documented consequence?

Output: the rewritten NFR-no-registry-writes and TC-file-structure, plus the startup decision tree.

## Answer

### 1. The Data Directory rule

The **Data Directory** is `data\` beside the executable, where "beside the executable" is resolved
**deliberately** rather than inherited from whatever path launched the process:

```
current_exe()  ->  resolve reparse points (GetFinalPathNameByHandle / canonicalize)
               ->  strip the \\?\ prefix (and \\?\UNC\ -> \\)
               ->  parent directory  ->  append "data"
```

**This is not what `current_exe()` gives you.** Measured on rustc 1.94.0: a binary launched through a
directory junction reported `current_exe()` as the **junction path**, not the target — Windows does not
resolve the reparse point for `GetModuleFileNameW`. `fs::canonicalize` does resolve it, and additionally
returns a `\\?\`-prefixed path whose casing is the on-disk truth rather than the caller's spelling, which is
why the prefix strip is part of the rule rather than an afterthought.

The asymmetry of getting this wrong is what decides it. winget installs a portable package to
`%LOCALAPPDATA%\Microsoft\WinGet\Packages\<PackageIdentifier>_<SourceIdentifier>\` and puts a **file symlink**
in `%LOCALAPPDATA%\Microsoft\WinGet\Links\`, which is the directory it adds to the user's PATH. Under the
unresolved rule, a user launching `PathMaster` from a shell would create `Links\data\` — a directory
**shared with every other winget portable package on the machine**. Under the resolved rule the data lands in
the package directory in every case.

Whether file symlinks resolve the way junctions do was **not** measured — creating one requires
administrator rights on this machine (Developer Mode off). The decision is deliberately built so that the
answer does not matter: resolving explicitly is correct whichever way `current_exe()` behaves.

The rule is also correct for the other install shapes:

- **scoop** — resolves into the versioned directory `apps\<app>\<version>\`, where the `persist: data`
  junction already lives; identical outcome to using the `current` junction.
- **Bare exe on a USB stick, launched via a desktop shortcut symlink** — data follows the stick, not the
  desktop. This is the portability promise working as intended.

Fallbacks, in order: if resolution fails, use the unresolved `current_exe()` path (a rare failure should not
cost the user their data directory); if `current_exe()` itself fails, there is no Data Directory and the
application starts in Read-only Data.

### 2. When it cannot be written — Read-only Data

The application **starts anyway**, in a mode named **Read-only Data**: PATH is read, diagnostics run, existing
Snapshots are listed, and nothing can be written. Concretely, every Editing Session is non-writable regardless
of what its Scope would permit — which under ticket 06 disables *every* editing action, not Apply alone — the
Settings item is disabled, and Restore is disabled. The StatusBar names the mode and the reason.

**Relocating the Data Directory was rejected on principle, and this is the load-bearing argument of the whole
ticket:** remembering a location outside the application's own directory requires *writing* something outside
the application's own directory. A "pick another folder and remember it" option therefore costs exactly the
promise it is trying to preserve. There is no fallback location, no prompt, and no remembered path.

Refusing to start was rejected for a different reason: for a screen-reader user, an application that exits
explains less than one that opens and says why it is only reading. A read-only PATH viewer with working
diagnostics is genuinely useful; silence is not.

A `--data-dir` switch (which would need no persistence, so it survives the principle above) was considered
and **left out of v0.1.0** — the scope is must-only. Recorded in the map's Out of scope.

### 3. Two instances

**No single-instance lock.** Elevation is implemented by relaunching the executable, so a second instance is a
**designed-in state**, not an anomaly to be prevented. A named mutex would also be unusually awkward here: an
object created by an elevated (high-integrity) process is not writable by a medium-integrity one by default,
so cross-elevation single-instance detection would need an explicit DACL and mandatory label — real complexity
bought to suppress a situation we deliberately create.

Instead, concurrency is made harmless:

- **Every replacement write is atomic** — temp file **in the same directory**, then
  `MoveFileExW(MOVEFILE_REPLACE_EXISTING)`. Not in-place rewriting: a power cut mid-rewrite of
  `settings.json` manufactures exactly the corrupt-JSON branch the PRD already has to handle, and being the
  source of that branch is indefensible. Not `ReplaceFileW`: its value is preserving the original's ACL and
  attributes, which section 4 shows we do not need.
- **Snapshots are written through the same temp+rename**, even though they never overwrite an existing file —
  so an interrupted backup cannot leave a **half-written Snapshot that still looks restorable**.
- **Rotation tolerates files that are already gone** (another instance deleted them first).
- **Logging**: one shared file opened `FILE_APPEND_DATA` with share read/write, **one line per record** — an
  append of that size is atomic, so two instances interleave records rather than corrupting them.
  **Rotation happens only at open**: if the file exceeds 5 MB, rename to `pathmaster.log.1` (one generation,
  overwriting the previous); if the rename fails because another instance holds the file, carry on appending.
  There is no runtime rotation at all, which removes the entire "rename a file another process has open"
  problem. Logging is minimal by charting decision 9, so 5 MB within a single run is not a real state.

### 4. ACLs — the concern does not survive measurement

A fresh directory created under `%LOCALAPPDATA%` was found to inherit `FullControl` for the user, for
`BUILTIN\Administrators` and for `SYSTEM`. **Access is decided by the inherited DACL, not by ownership** — so
although a file created by an elevated instance is *owned* by Administrators, the unelevated instance retains
full access to it through inheritance, and backup rotation keeps working. No mitigation, no explicit ACL
manipulation (which would be a security-settings change bought for a non-problem).

The one genuine divergence is a directory the user cannot write at all — `C:\Program Files\`, where an
elevated instance can create and fill `data\` and an unelevated one cannot. That is not an ACL problem to
solve; it is Read-only Data working correctly, and the mode is decided per run.

### 5. NFR-no-registry-writes, rewritten

The PRD's wording ("writes nothing to the registry, AppData, `%TEMP%` or any system location") is not
achievable as a statement about the machine: Windows itself records Amcache, Prefetch, UserAssist and MuiCache
entries as a consequence of the executable being run at all. The promise is therefore rewritten as a statement
about **the process**:

> **NFR-no-registry-writes** (must) — The PathMaster process creates and modifies nothing outside its own
> Data Directory, apart from the two target PATH registry values written by Apply.
>
> **Acceptance:**
> - The only registry writes are `HKCU\Environment` (User) and
>   `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment` (System), and only during Apply.
> - Process Monitor, **filtered to the `PathMaster.exe` process**, records no file or registry write outside
>   `<exe directory>\data\` and those two values.
> - The application opens **no native file dialogs** (see below).

**Derived constraint: no native file dialogs in v0.1.0.** `GetOpenFileName`/`IFileDialog` make Windows write
MRU entries under `HKCU\Software\...\ComDlg32` **attributed to our process**, which breaks the promise above.
No v0.1.0 feature needs one — "Open Backups Folder" is a shell invocation, and Restore reads from the Backups
tab list, not a picker — so this is a fixation, not a loss. Note that it is **code discipline and cannot be
verified from the import table**: `COMDLG32` appears in the import list regardless, because wxWidgets links it
unconditionally (research/04 section 1.2).

**README honesty, stated separately from the NFR.** The promise is about the application; the package manager
is a third party and gets its own paragraph:

- **winget** places the exe under
  `%LOCALAPPDATA%\Microsoft\WinGet\Packages\<PackageIdentifier>_<SourceIdentifier>\`, adds a symlink under
  `...\WinGet\Links\` — a directory it puts **on the user's PATH**, an irony a PATH editor should document —
  and records the install in its own database. Observed on a real machine: that package directory also
  contains winget's own `.db` file and the previous `*.exe.old` after an upgrade.
- The package directory name carries **no version**, so `data\` (and every Snapshot in it) **survives
  `winget upgrade`** — and `winget uninstall` **deletes the directory, backups included**. Say so plainly.
- **scoop** writes under `~\scoop\apps\`, `~\scoop\shims\` and, with `persist: data`, `~\scoop\persist\`.

### 6. TC-file-structure, rewritten

> **TC-file-structure** (must) — The application's file structure is fixed and confined to the Data
> Directory.
>
> - `PathMaster.exe` — the executable. Its directory is resolved per the Data Directory rule.
> - `data\settings.json` — settings.
> - `data\backups\*.json` — Snapshots.
> - `data\pathmaster.log`, `data\pathmaster.log.1` — diagnostic log and its single rotated generation.
> - Transient files inside `data\`: the pid-unique write probe and the temporaries backing every atomic
>   replacement, each deleted before the operation that created it returns.
> - The application creates and modifies nothing outside `data\`.
>
> It explicitly does **not** claim the executable's own directory is exclusively the application's — under
> winget it demonstrably is not.

### 7. The startup decision tree

1. **Locate.** `current_exe()` -> resolve reparse points -> strip `\\?\` / `\\?\UNC\` -> parent -> `data`.
   Resolution failure falls back to the unresolved path; `current_exe()` failure gives **Read-only Data**
   (*own location unknown*).
2. **Create.** `create_dir_all(<app dir>\data)`. Failure — access denied, read-only medium, `data` exists as a
   *file*, path too long — gives **Read-only Data** (*data directory cannot be created*).
3. **Probe.** Create a pid-unique temp file inside `data\`, write, delete. Failure gives **Read-only Data**
   (*data directory is not writable*). Success gives **Writable Data**.
4. **Log.** Open append/share; rotate at open if > 5 MB. **A logging failure degrades logging only and never
   sets the mode** — this is why the probe is a dedicated file rather than the log itself: a third party
   holding the log would otherwise present as a sharing violation and silently demote the whole application.
5. **Settings.** `settings.json` is **read in both modes** (an elevated run may have left one). Absent gives
   in-memory defaults, and the file is created **only** in Writable Data. Corrupt gives defaults plus the
   PRD's StatusBar warning, but the corrective rewrite happens only in Writable Data.
6. **Mode is decided once, at startup, and governs the UI only. Apply never consults it** — Apply begins by
   writing a Snapshot, so it discovers the truth by doing rather than by predicting, and a stale mode cannot
   cause a silent failure. Rule: **startup predicts, Apply verifies.** No polling.

Three reason strings — *own location unknown*, *cannot be created*, *not writable* — are the complete
enumeration, and they are translatable strings (ticket 11) and inputs to the error taxonomy.

### 8. Manifest

`longPathAware` is added to the existing `app.manifest` (ticket 04's recipe) under `<windowsSettings>`, which
does not disturb that ticket's deliberate omission of `trustInfo` — the linker still contributes `asInvoker`.
It is free, but it is **not relied upon**: it only takes effect where the machine-wide `LongPathsEnabled`
policy is on, so paths stay short and a long-path failure is handled by the same branch as any other probe
failure.

### 9. Evidence

- `current_exe()` through a directory junction and `fs::canonicalize` behaviour — measured with a
  purpose-built probe binary (rustc 1.94.0) on this machine.
- winget portable layout — read from a real installed portable package.
- ACL inheritance under `%LOCALAPPDATA%` — read from a freshly created directory.
- Unmeasured, and deliberately made irrelevant: whether `current_exe()` resolves **file** symlinks.
  A real winget install test belongs to ticket 15.

### Handed on

- **Ticket 12 (elevation)** — the ACL question it inherited is answered; two instances are explicitly allowed.
- **Ticket 14 (backup/restore)** — atomic Snapshot writes, rotation tolerating missing files, Read-only Data.
- **Ticket 15 (packaging)** — the winget layout facts and the README honesty paragraph.
- **Error taxonomy (map fog)** — three named startup failure reasons.
