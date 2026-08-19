# 18 — Release: icon, identity, pipeline, manifests, README

**Spec:** [spec §12 (icon), §16, §19 (support ladder), §15 (keyboard table)](../../pathmaster-v0-1-0/spec.md); manifest drafts ready to lift in `.scratch/pathmaster-v0-1-0/research/15-packaging/`

**What to build:** A releasable PathMaster: the exe carries its icon and VERSIONINFO, a tag push produces a gated release with the `.sha256` sidecar, winget and scoop manifests are ready to submit, and the README tells a screen-reader user the truth about trust, keyboard use, and the app's known behaviours. Help → About lands here too.

**Blocked by:** 11, 12, 13, 14, 15, 16, 17 — gates and README document the finished behaviour.

**Status:** ready-for-agent

- [ ] Icon: one source design, two assets — embedded SVG via `BitmapBundle::from_svg_data` → `Frame::set_icon()`, and the `.ico` exe resource via `llvm-rc` (16/24/32/48/256, 256 PNG-compressed); no other in-app iconography
- [ ] `VERSIONINFO` via `llvm-rc`: `CompanyName` "Ruslan Iskov", matching `PackageIdentifier` `RuslanIskov.PathMaster`; License MIT; unsigned by decision
- [ ] Help → About (name, version, license)
- [ ] Release workflow on tag `v*`: `windows-2025`, LLVM/libclang + Ninja pinned, `LIBCLANG_PATH` explicit; three-way version gate (tag / `Cargo.toml` / `.rc`); shallow build dir against MAX_PATH; dumpbin gate failing on `VCRUNTIME|MSVCP|api-ms-win-crt`; exe-size gate ≤ 40 MB; release via `gh`; PDB kept as CI artifact only, never shipped; gates run on the artifact, never the build config
- [ ] Release shape: bare `PathMaster.exe` + `.sha256` sidecar (`<hex64> *<name>`), no zip
- [ ] winget manifest (schema 1.12.0, three files, `InstallerType: portable`, `Commands: ["pathmaster"]`) and scoop bucket manifest (bare-exe URL with rename, `bin` + `shortcuts` + `persist: "data"`, `checkver: "github"`, autoupdate from the sidecar) finalized from the research drafts with the real repo URL
- [ ] README as trust documentation: SmartScreen + `Get-FileHash` verification against the sidecar; the keyboard map table mirroring spec §15; the ComDlg32 MRU exception; the NVDA deaf-list support ladder (Alt+Tab away and back → restart the app → restart NVDA) and Sanity Check; network-rooted Entries never probed; winget `upgrade` keeps `data\`, `uninstall` deletes it (Snapshots included)
- [ ] Release-time actions recorded as a pre-release checklist in the README or release notes: one clean-VM run with no VC++ redistributable; one live winget install observing symlink-resolve and uninstall
