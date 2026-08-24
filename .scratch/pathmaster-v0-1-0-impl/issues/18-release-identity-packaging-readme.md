# 18 — Release: icon, identity, pipeline, manifests, README

**Spec:** [spec §12 (icon), §16, §19 (support ladder), §15 (keyboard table)](../../pathmaster-v0-1-0/spec.md); manifest drafts ready to lift in `.scratch/pathmaster-v0-1-0/research/15-packaging/`

**What to build:** A releasable PathMaster: the exe carries its icon and VERSIONINFO, a tag push produces a gated release with the `.sha256` sidecar, winget and scoop manifests are ready to submit, and the README tells a screen-reader user the truth about trust, keyboard use, and the app's known behaviours. Help → About lands here too.

**Blocked by:** 11, 12, 13, 14, 15, 16, 17 — gates and README document the finished behaviour.

**Status:** resolved

- [x] Icon: one source design, two assets — embedded SVG via `BitmapBundle::from_svg_data` → `Frame::set_icon()`, and the `.ico` exe resource via `llvm-rc` (16/24/32/48/256, 256 PNG-compressed); no other in-app iconography
- [x] `VERSIONINFO` via `llvm-rc`: `CompanyName` "Ruslan Iskov", matching `PackageIdentifier` `RuslanIskov.PathMaster`; License MIT; unsigned by decision
- [x] Help → About (name, version, license)
- [x] Release workflow on tag `v*`: `windows-2025`, LLVM/libclang + Ninja pinned, `LIBCLANG_PATH` explicit; three-way version gate (tag / `Cargo.toml` / `.rc`); shallow build dir against MAX_PATH; dumpbin gate failing on `VCRUNTIME|MSVCP|api-ms-win-crt`; exe-size gate ≤ 40 MB; release via `gh`; PDB kept as CI artifact only, never shipped; gates run on the artifact, never the build config
- [x] Release shape: bare `PathMaster.exe` + `.sha256` sidecar (`<hex64> *<name>`), no zip
- [x] winget manifest (schema 1.12.0, three files, `InstallerType: portable`, `Commands: ["pathmaster"]`) and scoop bucket manifest (bare-exe URL with rename, `bin` + `shortcuts` + `persist: "data"`, `checkver: "github"`, autoupdate from the sidecar) finalized from the research drafts with the real repo URL
- [x] README as trust documentation: SmartScreen + `Get-FileHash` verification against the sidecar; the keyboard map table mirroring spec §15; the ComDlg32 MRU exception; the NVDA deaf-list support ladder (Alt+Tab away and back → restart the app → restart NVDA) and Sanity Check; network-rooted Entries never probed; winget `upgrade` keeps `data\`, `uninstall` deletes it (Snapshots included)
- [x] Release-time actions recorded as a pre-release checklist in the README or release notes: one clean-VM run with no VC++ redistributable; one live winget install observing symlink-resolve and uninstall

### What the boxes do not say

**The icon is generated, and that is what makes "one source design" true.** `icon.svg` is the
design; `tools/make-icon.ps1` rasterises it into the five layers of `app.ico` and assembles the
file by hand, because ImageMagick refuses a 256 px ICO layer outright and will write 255 — which
is not a size Windows looks for. The `.ico` is committed because the build reads it, but nothing
edits it. Verified out of the linked exe rather than claimed: `PrivateExtractIcons` answers at 16,
32, 48 **and 256**, and the 256 layer is **0 differing pixels** against the source.

**The two icon surfaces are two jobs and fail apart.** The exe resource governs Explorer and
pinned shortcuts; `Frame::set_icon` governs the title bar, the taskbar and Alt+Tab. Measured
before: `WM_GETICON` answered 0 for both sizes. Measured after: a live handle. A build that got
only the resource right would look correct in Explorer while showing the generic Windows icon
where the user actually works, which is why the Release Checklist now looks at both (L8, L9).

**Two of the three version legs are now checked on every `cargo test`.** The three-way gate the
spec asks for lives in the release workflow, but a tag is a bad place to first learn that a
version bump forgot the `.rc`. `crates/pathmaster/src/version.rs` compares `Cargo.toml` with the
resource script on every test run — sensitivity confirmed by breaking it — and the workflow keeps
the tag leg, which is the one only a release can check.

**A fourth CI gate the ticket does not name: the identity, read back out of the artifact.** A
`.res` that failed to link leaves a perfectly working binary carrying no `VERSIONINFO` at all. For
a binary unsigned by decision that is the whole of its identity, so the workflow reads
`CompanyName`, `ProductName` and both versions off the staged file. In the same spirit the
`.sha256` sidecar is written **after** every gate has passed: a published hash is an invitation to
install what it names, and a failed run must not leave one.

**The repo URL the drafts left open is `ruslan-rv-ua/pathmaster2`** — not a choice but a
consequence, since that is where `gh release create` publishes and therefore what both manifests
must download from. The scoop bucket is its own repository
(`ruslan-rv-ua/scoop-bucket`); the winget manifests validate against the real 1.12.0 schema
(`winget validate`). The `License` field both drafts left as TODO is **MIT**, which is also the
`LegalCopyright` line in the exe and the `LICENSE` file the winget manifest links to.

**The README is two complete documents, not one and a summary.** `README.uk.md` mirrors every
section of `README.md`; commands and code blocks stay untranslated, and E1 of the Release
Checklist is the drift guard. The screenshot's place is reserved rather than filled: the only
screenshot available here is of this machine's real PATH.

### Not verified here

**Nothing past `git push --tags`.** The release workflow has never run — it cannot, without a tag
and a release to make. Its YAML parses, its action refs are pinned to the commit SHAs behind
`v7.0.1`, and every gate it runs was rehearsed by hand against a locally built artifact, but the
run itself is F2/F3 of the checklist. So are both live installs: the symlink-resolve rule and the
`upgrade`-keeps / `uninstall`-deletes behaviour are documented in the README from the packaging
research, and only a real `winget install` can confirm them (F6, F7).

**Steps 31–33, L8 and L9 are the NVDA and eyes-on half**, and are the user's to run.
