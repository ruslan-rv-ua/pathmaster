# Single-exe build profile

Type: research
Status: resolved
Blocked by: —

## Question

Can a wxdragon build satisfy "one exe, no runtime to install, ≤ 40 MB, cold start ≤ 2 s" — and with exactly
which build settings?

Prefer **measurement over documentation**: build the minimal app and report real numbers.

- **CRT linkage.** Do wxdragon's prebuilt libraries force `/MD` (needs the VC++ redistributable) or allow
  `/MT` (static)? Can it be forced from the consuming crate? Does the MinGW/UCRT route avoid the problem?
  NFR-portable — "runs with no dependency install" — is 🔴 must and hangs entirely on this answer.
- **Size.** Actual release `.exe` size for a minimal wxdragon window, and what `opt-level`, `lto`,
  `codegen-units`, `panic=abort` and `strip` each buy. Compare against the ≤ 40 MB budget set at charting.
- **Cold start.** Measured time from process start to a visible window on an SSD.
- **Embedded resources.** How the icon and the translation catalogs get inside the exe (`include_bytes!`,
  a Windows `.rc` resource, wx XRC), and how the application manifest is embedded — `requestedExecutionLevel:
  asInvoker`, per-monitor DPI awareness, and **comctl32 v6**, without which wx silently falls back to
  legacy-looking controls with different accessibility behaviour.
- **The config CI must pin**: toolchain, target (`x86_64-pc-windows-msvc`), and anything that must not drift.

Findings → `../research/04-build-profile.md`. If any 🔴 must is unreachable, say so plainly — that is a finding,
not a failure.

## Carried in from tickets 01 and 03

Two of this ticket's questions are already answered at source; a third became the ticket's most important job.

- **There are no prebuilt libraries.** The README is stale. wxdragon 0.9.18 downloads pinned wxWidgets **3.3.3
  source** and compiles it as a CMake subproject — static libs (`wxBUILD_SHARED OFF`), non-monolithic, MSVC via
  Ninja at `RelWithDebInfo`, release CRT forced even for debug Rust profiles. So the first build is a full
  wxWidgets compile, and the measured cost belongs in this ticket's answer.
- **The CRT is a build-flag decision, not a requirement rewrite.** `wxdragon-sys/build.rs:365-374` reads
  `CARGO_CFG_TARGET_FEATURE`: with `crt-static` present it builds wxWidgets as `MultiThreaded` (`/MT`),
  otherwise `MultiThreadedDLL` (`/MD`, needing the VC++ redistributable). Confirm by building **both** ways and
  checking the resulting exe's imports for `VCRUNTIME140.dll` — that is the direct test of 🔴 NFR-portable.
- **Grep the generated `setup.h` for `wxUSE_ACCESSIBILITY` once the build finishes.** This is ticket 01's open
  UNKNOWN and this ticket is the first to produce a build. It matters because wxdragon's C layer wraps every
  accessibility entry point in `#if wxUSE_ACCESSIBILITY` with **silent no-op `#else` branches**: a 0 there
  compiles, runs, and does nothing — the quietest way this product could fail.
- Pin **wxdragon ≥ 0.9.17** in whatever the build config recommends (earlier `AccRole` discriminants are
  mis-ordered).

## Carried in from ticket 02's prototype (2026-08-18)

Ticket 02's throwaway app (`../prototypes/02-nvda-baseline/`) is the first thing in this effort that
actually got built, so four of this ticket's questions already have measured answers. None of these
depend on ticket 02's NVDA measurement, which has not been run.

- **`libclang` (LLVM) is a hard build requirement, and this is a new one.** `wxdragon-sys/build.rs`
  runs bindgen over the C++ shim headers **unconditionally** — there are no pre-generated bindings
  and no feature to skip it, and `src/lib.rs:4` does `include!(concat!(env!("OUT_DIR"),
  "/bindings.rs"))`. Without it the build dies in the build script with *"Unable to find libclang"*.
  LLVM **22.1.8** was installed here via `scoop install llvm`; the build then needs
  `LIBCLANG_PATH=C:\scoop\apps\llvm\current\bin`, because scoop shims the executables but
  `libclang.dll` itself lives in `bin\` and is not on `PATH`. **This belongs in the CI pin list
  alongside the toolchain and target** — it was not in this ticket's original question.

- **The default build is `/MD`, and the exe proves it.** With no `crt-static`, the resulting binary
  imports `VCRUNTIME140.dll`, `VCRUNTIME140_1.dll` and `MSVCP140.dll` (plus the usual
  `api-ms-win-crt-*` UCRT set) — so it needs the VC++ redistributable and **fails 🔴 NFR-portable as
  built**. This confirms the `build.rs:365-374` code reading exactly. The `crt-static` half of the
  A/B is still unrun and is still this ticket's job.

- **Size is not going to be the problem.** 7.12 MB for the full shell — menubar, notebook, three
  ListCtrls, buttons, status bar — against a 40 MB budget. And that is a *loose* build:
  `opt-level = 2` only, no LTO, no `strip`, no `panic = "abort"`, no icon and no translation
  catalogs. The measurements of what each of those flags buys are still owed, but they are now
  optimising a number that already has ~33 MB of headroom.

- **`wxdragon-sys` embeds no application manifest at all.** It links `comctl32`
  (`build.rs:892`) but ships no `.rc`, no `.manifest`, and no `embed_resource`/`winresource` step —
  so **comctl32 v6 does not happen by default**, and a stock build silently gets the legacy v5
  controls. The prototype proves a dependency-free recipe: an `app.manifest` next to `Cargo.toml`
  plus two lines in `build.rs` —
  `println!("cargo:rustc-link-arg-bins=/MANIFEST:EMBED")` and
  `println!("cargo:rustc-link-arg-bins=/MANIFESTINPUT:{}", ...)`. Verified embedded in the built exe,
  carrying the Common-Controls 6.0.0.0 dependency, `PerMonitorV2` DPI awareness, and the linker's own
  `asInvoker` trustInfo. Deliberately **omit** `trustInfo` from `app.manifest` — the linker adds its
  own and two blocks collide.

- **First build compiles wxWidgets 3.3.3 from source; incremental rebuilds of the app crate are
  ~1.5 s.** The first-build wall-clock was not timed cleanly here and is still worth measuring, since
  it is what CI pays on a cold cache.

## Answer

**Every 🔴 must in this ticket is reachable, and none of them is close to its limit.** Full evidence,
with every number marked by how it was obtained: [research/04](../research/04-build-profile.md).

**🔴 NFR-portable is reachable, via `RUSTFLAGS=-C target-feature=+crt-static`.** `wxdragon-sys`
reads `CARGO_CFG_TARGET_FEATURE` and switches the vendored wxWidgets build to `/MT`. Verified twice,
independently, on the linked binary rather than from the build config: the import table drops from
32 DLLs to 19, losing `VCRUNTIME140.dll`, `VCRUNTIME140_1.dll`, `MSVCP140.dll` **and all eleven
`api-ms-win-crt-*` UCRT imports**. Everything remaining is an OS DLL. The app still runs. The full
wxWidgets recompile this forces cost 107 s, not the hour that was budgeted.

**Size is a non-issue: 7.22 MB against a 40 MB budget — 18 %.** Recommended profile is
`opt-level = 2`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = true`, plus
`crt-static`. Three surprises came out of the 11-variant matrix, and all three contradict intuition:
`strip = true` moved **exactly zero bytes** (MSVC never put debug info in the exe — it is in a 52 MB
PDB beside it, so "single exe" means *not shipping the PDB*); `opt-level = 3` is marginally **worse**
than `2`, and stacking `opt-level = "z"` on top of the recommendation made the binary **bigger**;
LTO is the only lever that moves real bytes (−271 KB). Most of the binary is C++ compiled by CMake,
outside the Rust profile's reach — so future "optimise the build" instincts should go to `lto` and stop.

**Cold start is ~25× inside budget: 79.6 ms mean, 62–97 ms over 12 runs**, against a 2 s requirement.

**Embedded resources are all solved and demonstrated, not asserted.** The icon needs no new crate —
`llvm-rc` ships in the LLVM install the build already hard-requires; the icon extracted back out of
the built exe is bit-identical to the source (0 differing pixels / 1024), and `VERSIONINFO` comes free
with the same `.rc`. The manifest recipe from ticket 02 survives the CRT switch unchanged. Translation
catalogs go through ticket 03's `TranslationsLoader` over `include_bytes!` — mechanics confirmed at
source, but **not built end to end, and their size cost is unmeasured**.

**Two build-infrastructure traps were found the hard way and both belong in setup docs.** A deep
checkout path breaks the build via **MAX_PATH** with an error that blames the C++ compiler
("not able to compile a simple test program") — CMake's object paths hit 241 of 250 characters.
And **`RUSTFLAGS` silently overrides `.cargo/config.toml`**, so `crt-static` can be dropped with no
warning while the exe still builds and still runs on any dev machine that has the redistributable.
The consequence for release CI is carried into ticket 15: **gate on the artifact's imports, never on
the build config**.

**CI pin list**, split into load-bearing vs incidental, is §5 of the research file. The item that was
not in this ticket's original question: **LLVM/libclang must be pinned** — bindgen runs
unconditionally, there are no pre-generated bindings, and `llvm-rc` from that same install is the icon
toolchain. Also newly load-bearing: **Ninja is not optional** (`build.rs:379` hardcodes
`.generator("Ninja")` with no fallback on the x64 MSVC path), and `DOCS_RS` / `RUST_ANALYZER` leaking
into the environment silently reduce the build to bindings-only.

**First build costs 127 s** on a 20-thread desktop (107 s of it wxWidgets), plus a 45.4 MB source
download on a cold cache. The 443 MB of wxWidgets static libs sit at the profile root, not in a
fingerprint-keyed `OUT_DIR` — which is why the eleven profile variants each rebuilt in 8–30 s, and
which is the directory CI should cache.

**One defect risk the ticket did not ask about.** With the icon correctly embedded, the **running
window still has no icon** — `WM_GETICON` and the class icon both return 0, because wxMSW does not
adopt the exe's resource for the frame. Explorer looks right while the taskbar shows a generic icon.
It needs an explicit `Frame::set_icon()` at startup (`frame.rs:441`), and since `Bitmap` has no PNG
loader that means `Bitmap::from_rgba` over embedded RGBA or `BitmapBundle::from_svg_data`.

**Corroboration for the accessibility spine, from the linked binary rather than a header.**
`OLEACC.dll` is imported in **both** CRT modes, the running process also loads `uiautomationcore.dll`,
and `wxUSE_ACCESSIBILITY 1` was re-confirmed in the `crt-static` build's freshly generated `setup.h`.
Switching CRT mode does not disturb any of it. (NVDA 2025.3.3 was also observed injecting
`nvdaHelperRemote.dll`, `IAccessible2Proxy.dll` and `ISimpleDOM.dll` into the prototype process.)

**Eight things could not be determined** — §7 of the research file. The one that matters for release:
**the exe was never run on a machine without the VC++ redistributable**. The import table and the
runtime module list are strong indirect evidence, but the direct test is owed, and it is carried into
ticket 15 as one VM run before the first release.
