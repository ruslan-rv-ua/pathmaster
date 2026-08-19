# 01 — Workspace skeleton and window shell

**Spec:** [spec §1, §3 (app.manifest), §12, §16, §17, §18](../../pathmaster-v0-1-0/spec.md) · ADR-0007

**What to build:** A running, launchable PathMaster window on a fresh three-crate Cargo workspace. The user can start the exe, see and Tab through the whole map — Banner line, three notebook tabs ("User PATH", "System PATH", "Backups", User active at start), a two-column list (Path, Status) on each Scope tab, a native status bar — and close it cleanly. NVDA reads tabs and the empty lists on the free native path. No data is loaded yet; the shell proves the stack builds and speaks.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Three-crate workspace per spec §17: `pathmaster-core` (pure, any-OS), `pathmaster-platform` (no wx), `pathmaster` (bin-only, no lib target, `[[bin]] name = "PathMaster"`); dependency direction bin → platform → core enforced by the manifests; no test links wxWidgets
- [x] wxdragon pinned ≥ 0.9.17 (not 0.9.18), wxWidgets compiled from pinned source, statically, `crt-static` propagating into the C++ build; release profile `lto=true, codegen-units=1, panic=abort` in the virtual manifest root
- [x] `app.manifest` embedded: comctl32 v6, `PerMonitorV2`, `longPathAware`, no `trustInfo`
- [x] Window: one vertical sizer — Banner (fixed height, empty `StaticText`) above the notebook (`proportion=1, wxEXPAND`), status bar attached to the frame outside the sizer; first run 900×650 DIP, minimum 800×600, maximize supported
- [x] Scope tabs each hold a `wxListCtrl` with exactly two columns, Path and Status (no index column, no icons); the Status column width is the app's single explicit `FromDIP()` call; Path takes the remaining width
- [x] Tab / Shift+Tab traverses everything with no traps; Ctrl+Tab / Ctrl+Shift+Tab switches notebook tabs
- [x] Nothing anywhere sets a colour; zero `set_accessibility_*` calls (ADR-0003)
- [x] Push CI: `cargo test` + clippy on every push/PR to `develop`, green

## Comments

Implemented 2026-08-19.

- **Workspace**: virtual manifest at the root with the §16 release profile (`lto=true`,
  `codegen-units=1`, `panic="abort"`, opt-level left at default per the locked spec);
  `crt-static` in `.cargo/config.toml`; toolchain pinned exactly (1.94.0) in
  `rust-toolchain.toml`; `Cargo.lock` committed, resolving wxdragon to 0.9.18 (manifest
  floor `"0.9.17"` per the ticket).
- **Verified on the built artifact, not the config**: the debug exe's import table is 19 OS
  DLLs with no `VCRUNTIME*`/`MSVCP*`/`api-ms-win-crt-*` (crt-static propagated into the C++
  build), and the embedded manifest carries Common-Controls 6.0.0.0, `PerMonitorV2` and
  `longPathAware` with no `trustInfo` of its own.
- **Shell verified live**: the exe starts, the native child tree is exactly the map —
  Banner `Static`, native tab control, two `SysListView32` + `SysHeader32` (Scope lists),
  empty Backups panel, `msctls_statusbar32` (two fields) — and `CloseMainWindow` exits
  cleanly with code 0.
- **The single explicit FromDIP**: wxdragon's shim applies FromDIP implicitly to sizes
  crossing the FFI, but ListCtrl **column widths cross it raw** (`list_ctrl.cpp:97-112`
  passes width straight through) — so `ui::from_dip()` scales the one Status-column
  constant via `ClientDC::get_ppi()`. This is the app's only explicit conversion.
- **CI**: `.github/workflows/ci.yml` runs `cargo test --workspace --locked` + clippy
  `-D warnings` on push/PR to `develop` (windows-2025, Ninja installed, `LIBCLANG_PATH`
  set, `CARGO_TARGET_DIR=C:\t` for MAX_PATH, build tree cached). The repo has **no remote
  yet**, so the workflow has not run on GitHub; the identical commands are green locally.
- Ctrl+Tab / Ctrl+Shift+Tab page switching and full Tab traversal ride the native
  wxNotebook/comctl32 behaviour measured in wayfinder ticket 02; the NVDA pass itself is
  Release-Checklist work, not CI.
- **Column fit probed at runtime** (`LVM_GETCOLUMNWIDTH` on both `SysListView32`):
  Status = 220 px (the DIP constant at 100 % scale), Path = 636 px — all remaining width,
  set by the size handler from the initial layout (Path is inserted at width 0).
- **On the spec's wxdragon sentence**: §1's "decided against 0.9.18 over wxWidgets 3.3.3"
  is garbled; the research record is unambiguous (research/01: "prefer 0.9.18";
  research/04 pin list: "wxdragon 0.9.18 — pin exactly and commit Cargo.lock"; every
  measured number the spec quotes was taken on 0.9.18). Read as "decided on 0.9.18";
  manifest floor `"0.9.17"` per this ticket's wording, lock resolves 0.9.18.
- Reviewed on both axes (standards + spec) before commit; fixes applied: toolkit-init
  failure now exits nonzero, the stray Path-width constant and clamp removed, the
  Banner-height no-double-DIP fact recorded at the call site.
