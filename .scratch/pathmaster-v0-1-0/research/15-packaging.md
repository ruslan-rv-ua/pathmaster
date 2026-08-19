# Ticket 15 — Release pipeline and package manifests: findings

Established 2026-08-19 against primary sources: the `microsoft/winget-pkgs` and `microsoft/winget-cli`
repos (docs, JSON schemas, and the portable-install **source code**), the Scoop wiki and Scoop/Shim
**source**, and `actions/runner-images`. No CI run and no real package install was performed — this is
documentation- and source-derived, unlike the measured tickets; what still needs a live run is listed in §7.

Draft artifacts, ready to lift into the implementation effort (placeholders marked `TODO`):

- winget multi-file manifest: [`15-packaging/winget/`](15-packaging/winget/)
- scoop manifest: [`15-packaging/scoop/pathmaster.json`](15-packaging/scoop/pathmaster.json)
- release workflow: [`15-packaging/github/release.yml`](15-packaging/github/release.yml)

## 1. Name availability and PackageIdentifier

- **winget: free.** GitHub code search over `microsoft/winget-pkgs` for `PathMaster` returns
  **0 hits** (checked 2026-08-19 via the GitHub search API, `q=PathMaster repo:microsoft/winget-pkgs`).
  No publisher currently ships a package named PathMaster.
- **scoop: free in both curated buckets.** `bucket/pathmaster.json`, `PathMaster.json` and
  `path-master.json` all return 404 in `ScoopInstaller/Main` and `ScoopInstaller/Extras`
  (raw.githubusercontent.com, 2026-08-19). Irrelevant for v0.1.0 (own bucket by decision), but it means
  the name is not squatted and an Extras submission stays open for later.
- **Publisher.** The author's GitHub account is `ruslan-rv-ua`, display name **"Ruslan Iskov"**
  (GitHub users API). winget's guidance is that `Publisher` in the manifest should match what the
  binary's Add/Remove-Programs entry shows, and PackageIdentifier is `Publisher.Package` with the
  publisher folder under `manifests/r/…` ([manifest docs](https://learn.microsoft.com/en-us/windows/package-manager/package/manifest)).
  Two workable options, **not settled here** (the user picks):
  - **`RuslanIskov.PathMaster`** (recommended): derived from the display name, matches
    `CompanyName`/`Publisher: Ruslan Iskov` in VERSIONINFO and the locale manifest. Drafts use this.
  - `ruslan-rv-ua.PathMaster`: hyphens are legal in an identifier segment, but the publisher folder
    then diverges from the human-readable Publisher string; no precedent advantage.

  Whichever is chosen, **`CompanyName` in the `.rc` VERSIONINFO must match** — for a portable
  install winget writes its own ARP entry from the manifest (see §2), and consistency between
  VERSIONINFO, the locale manifest and the ARP entry is the only identity story an unsigned binary has.

## 2. winget portable mechanics

Sources: the portable-apps spec
[winget-cli `doc/specs/#182`](https://github.com/microsoft/winget-cli/blob/master/doc/specs/%23182%20-%20Support%20for%20installation%20of%20portable%20standalone%20apps.md),
and the implementation:
[`PortableFlow.cpp`](https://github.com/microsoft/winget-cli/blob/master/src/AppInstallerCLICore/Workflows/PortableFlow.cpp),
[`PortableInstaller.cpp`](https://github.com/microsoft/winget-cli/blob/master/src/AppInstallerCLICore/PortableInstaller.cpp),
[`PortableARPEntry.cpp/.h`](https://github.com/microsoft/winget-cli/blob/master/src/AppInstallerCommonCore/PortableARPEntry.cpp).

- **Install layout (user scope, the default):** exe goes to
  `%LOCALAPPDATA%\Microsoft\WinGet\Packages\<PackageIdentifier>_<SourceIdentifier>\`, and a **file
  symlink** to it goes to `%LOCALAPPDATA%\Microsoft\WinGet\Links\`, which winget appends **to the
  user's PATH**. Matches ticket 07's on-machine observation exactly.
- **The alias renames the exe itself.** `PortableFlow.cpp` (`GetDesiredStateForPortableInstall`):
  symlink name = `--rename` arg, else **`Commands[0]`**, else the installer filename — and `.exe` is
  appended (`AppendExtension(commandAlias, ".exe")`). Crucially the **target file in the Packages
  directory is renamed to the same alias**: with `Commands: ["pathmaster"]` the installed binary is
  `pathmaster.exe`, stable across versions, not `PathMaster-v0.1.0-x64.exe`. `PortableCommandAlias`
  as a manifest field exists **only inside `NestedInstallerFiles`** (zip case); for a bare exe the
  alias *is* `Commands[0]` (verified in the 1.12.0 installer JSON schema and in `PortableFlow.cpp`).
- **What winget writes to the registry** (`PortableARPEntry`): a key
  **`HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\<ProductCode>`** (user scope always uses
  the x64 view; ProductCode defaults to `<PackageIdentifier>__<source>` per spec), with values
  `DisplayName`, `DisplayVersion`, `Publisher`, `InstallDate`, `URLInfoAbout`, `HelpLink`,
  `UninstallString` (`winget uninstall --product-code …`), `WinGetInstallerType`,
  `WinGetPackageIdentifier`, `WinGetSourceIdentifier`, `InstallLocation`, `PortableSymlinkFullPath`,
  `PortableTargetFullPath`, `SHA256`, `InstallDirectoryCreated`, `InstallDirectoryAddedToPath`
  (exact enum in `PortableARPEntry.h`). **The README must say this**: installing PathMaster *via
  winget* writes an HKCU uninstall entry and edits the user PATH — that is winget, not the app, and
  NFR-no-registry-writes (a claim about the process, per ticket 07) is untouched.
- **Upgrade / uninstall:** upgrade overwrites the exe in place in the same un-versioned directory
  (spec #182; ticket 07 observed the `*.exe.old` leftover) — so **`data\` survives `winget upgrade`**.
  Uninstall removes exe + symlink + ARP entry; **extra files are preserved unless `--purge`** per
  spec #182, but ticket 07 *observed* the directory and its Snapshots deleted on a real uninstall —
  treat the observed behavior as the one to document (winget's `purgePortablePackage` setting and
  behavior defaults have shifted across versions). README: back up `data\` before uninstalling.
- **Schema version:** current is **1.12.0** (learn.microsoft.com manifest page, updated 2026-03;
  JSON schemas live at `winget-cli/schemas/JSON/manifests/v1.12.0/`). Multi-file manifest = three
  files minimum: `version`, `installer`, `defaultLocale`, all `ManifestVersion: 1.12.0`, under
  `manifests/r/RuslanIskov/PathMaster/0.1.0/`.

## 3. scoop mechanics

Sources: [App-Manifests wiki](https://github.com/ScoopInstaller/Scoop/wiki/App-Manifests),
[Autoupdate wiki](https://github.com/ScoopInstaller/Scoop/wiki/App-Manifest-Autoupdate),
[Persistent-data wiki](https://github.com/ScoopInstaller/Scoop/wiki/Persistent-data),
[`lib/core.ps1`](https://github.com/ScoopInstaller/Scoop/blob/master/lib/core.ps1) and
[`ScoopInstaller/Shim` `cs/shim.cs`](https://github.com/ScoopInstaller/Shim/blob/master/cs/shim.cs) (source),
[BucketTemplate](https://github.com/ScoopInstaller/BucketTemplate).

- **Bare exe is fine.** `url` may point straight at the release exe; the `#/Name.exe` URL fragment
  renames the downloaded file (documented wiki mechanism), so
  `…/PathMaster-v0.1.0-x64.exe#/PathMaster.exe` gives a stable `PathMaster.exe` in the versioned app
  dir and stable `bin`/`shortcuts` entries across versions.
- **GUI shim: no console flash, nothing special needed.** `lib/core.ps1` (~line 904–913): when
  creating an exe shim, scoop reads the **target's** PE subsystem and if it is GUI (2) **patches the
  copied `shim.exe`'s own subsystem field to GUI** ("Making $shim.exe a GUI binary."). The default
  shim (`scoopcs`, from ScoopInstaller/Shim) additionally calls `FreeConsole()` when it detects it is
  running as a GUI-subsystem binary with no args. So `bin: "PathMaster.exe"` yields a flash-free
  launcher; `shortcuts` adds the Start-Menu entry a GUI app actually wants.
- **`persist: "data"`** links `<app>\data` → `~\scoop\persist\pathmaster\data` (directory junction;
  wiki: "linked from the installed application directory to the data directory"). Survives
  `scoop update`; survives `scoop uninstall` unless `-p/--purge`. **Interaction with ticket 07's
  resolution rule checks out:** under scoop the exe's resolved path is the *versioned* dir
  (`apps\pathmaster\0.1.0\`), where scoop has planted the `data` junction — the app's writes flow
  through the junction into `persist\`. One caveat to verify live: persist links are created **at
  install time**, so `data\` exists (as a junction) before first run — which also satisfies the app's
  "can I write here" probe.
- **`checkver: "github"`** applies the built-in regex `\/releases\/tag\/(?:v|V)?([\d.]+)` to the
  homepage's releases page, ignoring pre-releases. `autoupdate` substitutes `$version` into the URL
  template; the hash comes from our published sidecar via `"hash": { "url": "$url.sha256" }` —
  scoop's `extract_hash` handles the standard `<hex64> *<filename>` format (the `$sha256` /
  `$basename` machinery in the Autoupdate wiki; `$url`/`$basename` exclude the `#/` fragment).
- **Own bucket:** generate from [ScoopInstaller/BucketTemplate](https://github.com/ScoopInstaller/BucketTemplate),
  which ships `excavator.yml` — a scheduled workflow that runs checkver/autoupdate and commits the
  bumped manifest. That is the "how scoop gets bumped on release" answer: automatic, given the
  `.sha256` sidecar exists on the release.

## 4. Release workflow (GitHub Actions)

Draft: [`15-packaging/github/release.yml`](15-packaging/github/release.yml). Shape: tag `v*` →
one `windows-2025` job → build → gates → `gh release create` (gh CLI is preinstalled on hosted
runners; no third-party release action to pin).

- **Runner drift since ticket 04, worth knowing:** `windows-2025` / `windows-latest` now resolve to
  the **Windows Server 2025 + Visual Studio 2026 Enterprise** image (`windows-2025-vs2026`,
  [runner-images README](https://github.com/actions/runner-images/blob/main/README.md)). Its
  [current readme](https://github.com/actions/runner-images/blob/main/images/windows/Windows2025-VS2026-Readme.md):
  LLVM **20.1.8**, CMake **4.4.2**, Ninja **1.13.2**, Rust 1.97.1 (irrelevant — we pin via
  `rust-toolchain.toml`), VS 2026 18.8. Two consequences: (a) CMake and Ninja now *match* the dev
  machine's pins, and the VS2026 toolset family (`cl` 19.5x) matches the dev machine's 14.50 toolset —
  closer than when ticket 04 looked; (b) the image label pins the OS, **not the tool versions** —
  they drift with every image release, which is exactly why the workflow asserts on the artifact,
  not the toolchain.
- **LLVM/libclang:** `LIBCLANG_PATH=C:\Program Files\LLVM\bin` set explicitly (ticket 04 pin).
  Image LLVM is 20.1.8 vs 22.1.8 on the dev machine. Bindgen output is generated *and compiled* in
  the same CI run, so it is self-consistent; the workflow carries a commented opt-in step to install
  an exact LLVM if byte-identical builds are ever wanted. `llvm-rc` for the icon/VERSIONINFO comes
  from the same install (SDK `rc.exe` is still not on PATH on the image).
- **MAX_PATH:** the deep-path trap lives under the target dir (`…\wxdragon_sys_cmake_build\…`), so
  the workflow sets **`CARGO_TARGET_DIR=C:\t`** rather than fighting the checkout path.
- **Env hazards:** the workflow asserts `DOCS_RS`/`RUST_ANALYZER` are unset (they silently turn the
  wxdragon-sys build into "bindings only", research/04 §5.2), and sets `RUSTFLAGS=-C
  target-feature=+crt-static` at job level with `--target x86_64-pc-windows-msvc` explicit (keeps
  RUSTFLAGS off host proc-macros; and any later step adding to RUSTFLAGS is caught by the gate below).
- **The dumpbin gate (mandatory):** locate `dumpbin.exe` via `vswhere -find '**\Hostx64\x64\dumpbin.exe'`
  (vswhere ships on the image), run `/DEPENDENTS` on the built exe, **fail on
  `VCRUNTIME|MSVCP|api-ms-win-crt`**, and also fail if the output doesn't contain `KERNEL32.dll`
  (proves the parse saw a real import table). This is the artifact-level guard for 🔴 NFR-portable.
- **Version sync (three-way):** one step compares the tag (`v0.1.0` → `0.1.0`), `Cargo.toml`
  `version`, and the `.rc` `FileVersion`/`ProductVersion` strings; any mismatch fails the release.
  The `.rc` path is a TODO until the implementation effort fixes the repo layout.
- **Artifacts:** exe renamed to `PathMaster-v<version>-x64.exe`; sidecar
  `PathMaster-v<version>-x64.exe.sha256` in `<hex64> *<filename>` format (feeds both the release page
  and scoop autoupdate). **PDB uploaded via `actions/upload-artifact`, never attached to the release**
  — the only way to symbolicate crash reports from an unsigned binary in the field (ticket 04).
  Current action majors at draft time: `actions/checkout@v7`, `actions/upload-artifact@v7`
  (v7.0.1, GitHub releases API 2026-08-19); pin to full SHAs at implementation.
- **No build cache in the release job** — hermetic over fast; a cold wxWidgets build cost ~2 min on a
  20-thread desktop (research/04 §6), so expect roughly 10–20 min on a 4-core runner, paid once per release.
- **Manifest bumps on release:** scoop — automatic via the bucket's excavator (above). winget — a new
  PR to `microsoft/winget-pkgs` per version; `wingetcreate update RuslanIskov.PathMaster --urls … --version … --submit`
  can run from CI with a PAT, or be done manually the first few releases. First submission is always
  a manual PR (new package = new publisher folder + moderation).

## 5. Archive or bare exe?

**Bare exe (+ `.sha256` sidecar). No zip.**

- winget portable takes the exe directly (`InstallerType: portable`); the zip route
  (`zip` + `NestedInstallerType: portable` + `NestedInstallerFiles`) exists but adds fields and a
  nested path for zero gain when there is exactly one file.
- scoop takes the exe directly with the `#/PathMaster.exe` rename fragment.
- Manual users get MOTW/SmartScreen either way (zips propagate MOTW on extract with Explorer), so a
  zip does not dodge the unsigned-binary warning; it only adds an extraction step and a second
  artifact to hash. "Single exe" is also the product's story — the release page should look like it.

## 6. What the README must say (feeds the docs ticket)

1. winget install writes an `HKCU\…\Uninstall\<ProductCode>` entry and puts the WinGet `Links`
   directory on the **user PATH** — winget's doing, not PathMaster's; the app itself never writes
   outside its own directory. (Yes: installing a PATH editor edits your PATH.)
2. `winget upgrade` keeps `data\`; `winget uninstall` was **observed** to delete the package
   directory including `data\` — export/back up Snapshots first.
3. scoop: `persist` keeps `data\` across updates and uninstall (unless `--purge`).
4. Unsigned binary: SmartScreen warning expected; the exe's VERSIONINFO (Publisher "Ruslan Iskov")
   is its identity; verify the SHA256 against the `.sha256` sidecar on the release.

## 7. Could NOT determine (and new questions)

1. **No live run of anything.** The workflow, both manifests, and the winget PR validation pipeline
   are drafted from docs/schemas/source, not executed. First tag on the real repo is the test.
2. **The clean-VM run** (no VC++ redistributable) owed since ticket 04 remains owed — it needs a VM,
   not a manifest. Slot it before the first public release.
3. **Repository name/URL.** Drafts assume `https://github.com/ruslan-rv-ua/PathMaster` — the public
   repo does not exist yet (working dir is `PathMaster2`); every URL is marked TODO.
4. **License.** Not decided anywhere in the map; winget `License` and scoop `license` are required
   fields. Marked TODO — needs a user decision before first submission.
5. **Publisher choice** (§1) is presented as a recommendation, not a settled fact.
6. **Whether `current_exe()` resolves winget's Links *file symlink*** — ticket 07's open observation
   — still needs a real winget install of PathMaster; ticket 07's rule is built so either answer works.
7. **winget uninstall vs `data\`:** spec says non-package files are preserved without `--purge`,
   ticket 07 observed deletion. Version-dependent defaults (`purgePortablePackage`) are the likely
   cause; the README should assume deletion (the safe claim). Verifying on a current winget with a
   real install belongs to the same live-run session as (6).
8. **MinimumOSVersion.** Drafted as `10.0.17763.0` (winget itself requires Win10 1809+; PerMonitorV2
   needs 1703+) — plausible, not established; no ticket has pinned the actual OS floor.
