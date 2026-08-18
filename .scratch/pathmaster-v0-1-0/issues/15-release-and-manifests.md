# Release and package manifests

Type: research
Status: open
Blocked by: 04, 07

## Question

What does the release pipeline look like, and what exactly goes in the scoop and winget manifests?

Settled at charting: GitHub Releases built by GitHub Actions on `windows-latest`; **unsigned** in v0.1.0 with
the SmartScreen warning documented in the README; scoop via an **own bucket** with `persist: data`; winget
submitted to `microsoft/winget-pkgs` with `InstallerType: portable`.

Facts to establish and drafts to produce:

- **Name availability.** Is `PathMaster` free in winget and in the main scoop buckets? Which
  `PackageIdentifier` (`<Publisher>.PathMaster`) is available, and what Publisher name should be used?
- **winget portable specifics.** `PortableCommandAlias`, the Links directory it adds **to the user's PATH**
  (an irony worth documenting in a PATH editor), what winget itself writes to the registry — it does, and the
  README must not claim otherwise — and whether `data/` survives `winget upgrade`.
- **scoop specifics.** Manifest fields, `persist: data`, `checkver` / `autoupdate`, and how the shim behaves
  for a GUI executable.
- **Workflow.** Build, artifact naming (`PathMaster-v0.1.0-x64.exe`), SHA256 publication, tag → release, and
  how each manifest gets bumped on release.
- **Archive or bare exe?** Whether the release should also ship a zip, and which installer type each package
  manager is happiest with.

Output: draft manifests and workflow file in `../research/15-packaging/`, ready to lift into the
implementation effort unchanged.

## Carried in from ticket 04

Five things came out of the build-profile measurements that land squarely here.

- **Release CI must gate on the artifact's imports, never on the build config.** `RUSTFLAGS` silently
  overrides `.cargo/config.toml` — Cargo's rustflags sources are mutually exclusive and the env var
  wins — so if `crt-static` lives in a config file and any workflow step sets `RUSTFLAGS` for an
  unrelated reason, **`crt-static` is dropped with no warning**, the build succeeds, and the exe still
  runs on any developer machine that has the VC++ redistributable. A `dumpbin /DEPENDENTS` check that
  fails the build on `VCRUNTIME|MSVCP` is the only reliable guard for 🔴 NFR-portable. Cheap; make it
  mandatory.
- **Run the release exe once on a clean Windows VM with no VC++ redistributable.** Ticket 04 verified
  the import table and the runtime module list — strong indirect evidence, nothing outside
  `%SystemRoot%` loads — but never performed the direct test. It is owed before the first release.
- **`VERSIONINFO` is free once the icon `.rc` exists**, and was demonstrated working end to end
  (`ProductName`, `FileVersion`, `ProductVersion`, `CompanyName`, `LegalCopyright`,
  `OriginalFilename` all read back through Explorer's Properties). **winget cares about these**, and
  the version string in the `.rc` must stay in sync with `Cargo.toml` and the git tag — that
  synchronisation is this ticket's problem. Note `FileDescription` is what SmartScreen and Task
  Manager display, and since v0.1.0 ships **unsigned** by decision (map decision 10), VERSIONINFO is
  the *only* identity the binary carries. Worth getting right.
- **A 52 MB PDB is produced next to the 7.2 MB exe even at `debug = false`.** Not shipping it is what
  makes "single exe" true on MSVC — but **do** keep it as a per-release CI artifact: it is the only
  way to symbolicate a crash report from an unsigned binary in the field.
- **Pin the runner, not `windows-latest`.** That label now maps to Windows Server 2025, whose
  preinstalled versions do not match the development machine (LLVM 20.1.8 vs 22.1.8, Rust 1.97.1 vs
  1.94.0, CMake 3.31.6 vs 4.4.2). Nothing needs installing there, but nothing should be inherited
  either: pin `windows-2025`, pin the toolchain, and set `LIBCLANG_PATH=C:\Program Files\LLVM\bin`
  explicitly. The Windows SDK `bin` (hence `rc.exe`) is **not** on PATH on that image — one more
  reason the icon toolchain is `llvm-rc`. Full CI pin list, split load-bearing vs incidental:
  [research/04 §5](../research/04-build-profile.md).
- **Keep the CI working directory short.** A deep checkout path breaks the wxWidgets CMake build via
  MAX_PATH, with an error that blames the C++ compiler rather than the path.

## Carried in from ticket 07

Several of this ticket's open facts were measured on a real machine while resolving the data directory contract.

- **winget portable layout, observed.** A real installed portable package lives at
  `%LOCALAPPDATA%\Microsoft\WinGet\Packages\<PackageIdentifier>_<SourceIdentifier>\` — the directory name
  carries **no version**, and it also holds winget's own `.db` file plus the previous `*.exe.old` after an
  upgrade. So **`data\` survives `winget upgrade`** (strong evidence: the upgrade happened in place, in the
  same directory), and `winget uninstall` **deletes the directory and every Snapshot in it**. Both belong in
  the README. Confirming the upgrade path against a real PathMaster install is still owed here.
- **The Links directory is on the user's PATH**, which is the irony this ticket already flagged — and it is
  also why the Data Directory is located by **resolving** the executable path rather than trusting the launch
  path. Whether `current_exe()` resolves a **file symlink** was *not* measurable (creating one needs admin
  rights); ticket 07's rule is built so the answer does not matter, but a real winget install is the place to
  observe it, and this ticket is that place.
- **`longPathAware` joins `app.manifest`** under `<windowsSettings>`, alongside ticket 04's comctl32 v6 and
  `PerMonitorV2`. It does not disturb that ticket's deliberate omission of `trustInfo`.
- **The release check for NFR-no-registry-writes is a Process Monitor run filtered to the `PathMaster.exe`
  process**, not a machine-wide one — the promise is now about the process, because Windows writes Amcache,
  Prefetch and MuiCache entries merely because the binary ran.
- **A derived constraint the packaging story should not quietly break:** v0.1.0 opens **no native file
  dialogs**, since ComDlg32 MRU writes would land under our process. `COMDLG32` is in the import list anyway
  (wxWidgets links it unconditionally), so no artifact check can verify this — it is code discipline.
