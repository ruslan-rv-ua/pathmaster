# Single-exe build profile

Resolves [issues/04-single-exe-build-profile.md](../issues/04-single-exe-build-profile.md).

## Verdicts first

| Question | Answer | How obtained |
|---|---|---|
| 🔴 **NFR-portable — reachable?** | **YES.** With `-C target-feature=+crt-static` the exe imports **no** `VCRUNTIME140.dll`, **no** `VCRUNTIME140_1.dll`, **no** `MSVCP140.dll` — and the entire `api-ms-win-crt-*` UCRT import set disappears too. Every remaining import is an OS DLL. | **Built it**, then `dumpbin /DEPENDENTS`, then re-checked the *running* process's loaded modules |
| Does it still run? | **YES.** Window opens, `MainWindowTitle` = `PathMaster — NVDA baseline prototype`, process stays up. | **Launched it** |
| Recommended profile | `opt-level = 2`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = true`, `+crt-static` | 11-variant one-factor-at-a-time matrix |
| Its size | **7,220,224 bytes = 7.22 MB** (+ ~21 KB once the icon is added → **≈ 7.24 MB**) | **Measured**, `Get-Item .Length` |
| vs the ≤ 40 MB budget | **18.1 % of budget. 32.8 MB headroom.** Size is a non-issue. | arithmetic on the above |
| 🔴 Cold start ≤ 2 s? | **YES, by ~25×.** 79.6 ms mean, 62.2–97.2 ms over 12 runs. A genuinely cache-cold first run can add at most ~13 ms of I/O. | **Measured** (method in §3) |
| First build wall-clock (what CI pays) | **127.0 s** on this machine — 107.4 s of it is the wxWidgets 3.3.3 compile. Excludes a 45.4 MB source download. | **Measured**, stopwatch around `cargo build` |
| Icon in the exe | **Demonstrated working.** Explorer's icon extracted back out of the exe is **bit-identical** to the source (0 differing pixels / 1024). Needs no new crate: `llvm-rc` ships in the LLVM install this build already requires. | **Built and extracted it** |
| Application manifest | Already solved by ticket 02's recipe; **survives `crt-static` unchanged**. | **Extracted** with `mt.exe` from the new exe |

**One thing the ticket did not ask that must not be missed:** the icon resource makes Explorer, Alt-Tab and the taskbar-pinned shortcut correct, but the **running window still has no icon** — `WM_GETICON` and the class icon both return 0. That needs a `Frame::set_icon()` call in application code. Measured, §4.2.

---

## Measurement environment

Every number below was produced on this machine, on 2026-08-18. Nothing here is a doc claim about what a build *would* do.

| | |
|---|---|
| CPU / RAM | Intel Core i5-14500, 14 cores / 20 threads, 31.7 GB |
| Disk | Samsung SSD 990 PRO with Heatsink 1TB, **NVMe** (`Get-PhysicalDisk`) |
| OS | Windows 11 Pro 10.0.26200 |
| rustc / cargo | **1.94.0** (`4a4ef493e`, 2026-03-02), LLVM backend 21.1.8 / cargo 1.94.0 |
| Target | `x86_64-pc-windows-msvc` (passed explicitly as `--target`) |
| MSVC | toolset **14.50.35717**, `cl.exe` **19.50.35727** for x64, VS 18 BuildTools |
| Windows SDK | **10.0.22621.0** (the only one with a `bin\<ver>\x64` on this box) |
| CMake / Ninja | **4.4.2** / **1.13.2** |
| LLVM (libclang) | **22.1.8**, `C:\scoop\apps\llvm\current\bin` |
| wxdragon / wxWidgets | 0.9.18 → wxWidgets **3.3.3**, compiled from source, static, non-monolithic |

**The crate under test.** A source-only copy of ticket 02's prototype (`Cargo.toml`, `Cargo.lock`, `build.rs`, `app.manifest`, `src/main.rs` — no `target/`), built in scratch. Ticket 02's own tree was never built into and is untouched; its `/MD` exe is used read-only as the comparison baseline.

> **A build-infrastructure trap found the hard way.** The first `crt-static` attempt **failed** with *"The C++ compiler … is not able to compile a simple test program"*. The real cause was **MAX_PATH**: CMake warned `The object file directory … has 241 characters. The maximum full path to an object file is 250 characters (see CMAKE_OBJECT_PATH_MAX)` and the compiler probe then failed. Building the identical crate through a short path (a directory junction) succeeded with no other change. **This belongs in the CI notes and in developer setup docs** — the wxWidgets CMake/Ninja tree nests deeply enough that a long checkout path breaks the build with an error message that points at the compiler, not at the path.

---

## 1. CRT linkage — the 🔴 NFR-portable A/B

### 1.1 The mechanism, confirmed at source and then at the flag

`wxdragon-sys/build.rs:365-374` reads `CARGO_CFG_TARGET_FEATURE` and maps it to CMake's MSVC runtime setting:

```rust
let target_features = std::env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();
let crt_static = target_features.split(',').any(|f| f == "crt-static");
let rt_lib = if crt_static {
    if is_debug { "MultiThreadedDebug" } else { "MultiThreaded" }
} else if is_debug { "MultiThreadedDebugDLL" } else { "MultiThreadedDLL" };
```

Applied at `build.rs:382` as `.define("CMAKE_MSVC_RUNTIME_LIBRARY", rt_lib)`.

**Verified, not assumed, that the flag actually arrives.** A throwaway probe crate whose `build.rs` prints `CARGO_CFG_TARGET_FEATURE`:

| Invocation | `CARGO_CFG_TARGET_FEATURE` seen by the build script |
|---|---|
| no `RUSTFLAGS`, `--target x86_64-pc-windows-msvc` | `cmpxchg16b,fxsr,sse,sse2,sse3` |
| `RUSTFLAGS=-C target-feature=+crt-static`, same `--target` | `cmpxchg16b,`**`crt-static`**`,fxsr,sse,sse2,sse3` |

**Verified it reached CMake.** From the real build's `CMakeCache.txt` (`…/release/wxdragon_sys_cmake_build/build/CMakeCache.txt`):

```
CMAKE_BUILD_TYPE:STRING=RelWithDebInfo
CMAKE_CXX_FLAGS:STRING= /EHsc -nologo -MT -Brepro -W0
CMAKE_MSVC_RUNTIME_LIBRARY:UNINITIALIZED=MultiThreaded
wxBUILD_SHARED:BOOL=OFF
```

`-MT` is in the C++ flags. **Verified wxWidgets was genuinely recompiled** for it: 17 static libraries totalling **443,848,710 bytes** were produced (`wxmsw33u_core.lib` alone is 240.5 MB).

> **Use `--target x86_64-pc-windows-msvc` explicitly.** With `--target` present, cargo applies `RUSTFLAGS` only to target units and *not* to host build scripts / proc-macro crates (`wxdragon-macros` is a proc macro). This is the reason the build works; it is not incidental. It also happens to be the target CI must pin anyway.

### 1.2 The direct test: imports

`dumpbin /DEPENDENTS` on both exes. Same source, same crate, same wxdragon version — only the CRT flag differs.

| | `/MD` (default, ticket 02's exe) | `/MT` (`+crt-static`) |
|---|---|---|
| `VCRUNTIME140.dll` | **present** | **gone** |
| `VCRUNTIME140_1.dll` | **present** | **gone** |
| `MSVCP140.dll` | **present** | **gone** |
| `api-ms-win-crt-*.dll` | **13 of them** (`convert`, `environment`, `filesystem`, `heap`, `locale`, `math`, `runtime`, `stdio`, `string`, `time`, `utility`, …) | **all gone** |
| Remaining imports | 19 OS DLLs | **18 OS DLLs**, nothing else |

The `/MT` import list in full — every one of these ships with Windows:

```
ADVAPI32  api-ms-win-core-synch-l1-2-0  bcryptprimitives  COMCTL32  COMDLG32
GDI32  gdiplus  KERNEL32  MSIMG32  ntdll  ole32  OLEACC  OLEAUT32  RPCRT4
SHELL32  SHLWAPI  USER32  UxTheme
```

The UCRT vanishing as well is worth stating plainly: `+crt-static` on MSVC statically links `libucrt.lib` alongside `libvcruntime.lib`, so the binary does not even depend on `ucrtbase.dll` being present.

### 1.3 A stronger test than the import table

The import table cannot see `LoadLibrary` calls, and wxWidgets does use them. So the exe was **run**, and its actually-loaded modules enumerated (`Process.Modules`) while the window was up:

- 44 modules loaded.
- **Modules loaded from outside `%SystemRoot%`: the exe itself, and three NVDA DLLs** (`nvdaHelperRemote.dll`, `IAccessible2Proxy.dll`, `ISimpleDOM.dll` from `C:\Program Files (x86)\NVDA\lib64\2025.3.3\`) — because NVDA was running on the measuring machine and injected them.
- **Nothing from a VC++ redistributable.** `msvcrt.dll`, `msvcp_win.dll` and `ucrtbase.dll` *do* appear, but all three resolve inside `C:\Windows\System32` — they are OS components pulled in by other OS DLLs, not the redistributable, and the exe does not import them.

### 1.4 Verdict

> ### 🔴 **NFR-portable is REACHABLE.**
> `RUSTFLAGS="-C target-feature=+crt-static"` + `--target x86_64-pc-windows-msvc` produces a single exe with zero non-OS dependencies. The app launches, the window titled `PathMaster — NVDA baseline prototype` appears, and the process stays up. **No VC++ redistributable, no runtime install.**

Two costs, both small and both stated rather than hidden:

1. **+426,496 bytes** (+6.0 %) versus the same profile at `/MD` — the CRT now lives inside the exe. Irrelevant against a 40 MB budget.
2. **CRT security fixes no longer arrive via Windows Update.** A CRT vulnerability means PathMaster must be rebuilt and re-released. Microsoft's own guidance recommends the redistributable for exactly this reason. Map decision 3 ranks portability above size and this is the portability route, so the trade is the right one — but it should be a conscious line in the spec, and it gives releases a second reason to exist.

**MinGW/UCRT route: not investigated, and it does not need to be.** `build.rs:340-356` shows a MinGW branch exists, but since `crt-static` on MSVC already produces a dependency-free exe there is no problem left for MinGW to solve, and switching would abandon the MSVC toolchain that ticket 01 and 02 validated.

---

## 2. Size

### 2.1 The matrix

All eleven builds are `+crt-static`, same source, same machine, one factor changed at a time from BASE. Sizes are `Get-Item .Length` on the produced exe; build seconds are wall-clock and are *incremental* (the wxWidgets libraries are cached across all of these — see §6).

| Variant | `[profile.release]` | Bytes | Δ vs BASE | Build |
|---|---|---:|---:|---:|
| **BASE** | `opt-level = 2` | 7,550,976 | — | (cached) |
| `opt-level = 3` | | 7,552,000 | **+1,024** | 20.1 s |
| `opt-level = "s"` | | 7,542,272 | **−8,704** | 12.4 s |
| `opt-level = "z"` | | 7,534,080 | **−16,896** | 12.0 s |
| `lto = "thin"` | | 7,293,952 | **−257,024** | 24.7 s |
| `lto = true` (fat) | | 7,279,616 | **−271,360** | 26.1 s |
| `codegen-units = 1` | | 7,532,032 | **−18,944** | 21.0 s |
| `panic = "abort"` | | 7,517,696 | **−33,280** | 8.3 s |
| `strip = true` | | 7,550,976 | **0** | 22.9 s |
| **REC** | `opt2 + lto=true + cgu=1 + panic=abort + strip` | **7,220,224** | **−330,752** | 29.8 s |
| REC with `opt-level = "z"` | | 7,237,120 | −313,856 | 16.6 s |

For reference, ticket 02's `/MD` build at BASE settings was **7,124,480** bytes.

### 2.2 What the numbers say

- **`opt-level` is nearly irrelevant here, and `3` is very slightly *worse* than `2`.** The spread across `2`/`3`/`s`/`z` is 17,920 bytes — 0.24 % of the binary. The reason is structural: the bulk of this exe is **C++ wxWidgets**, compiled by CMake at `RelWithDebInfo` and completely unaffected by the Rust profile. Optimising `opt-level` for size here is optimising the small half.
- **LTO is the only lever that moves real bytes** — −271,360 for fat, and thin gets 95 % of that (−257,024) for slightly less build time. This is cross-crate work on the Rust half plus dead-code elimination against the C shim.
- **`strip = true` moved exactly zero bytes.** Not "almost none" — byte-identical to BASE (7,550,976 both). On MSVC the debug information was never in the exe: `split-debuginfo` is `packed` and the symbols go to a separate `.pdb`. The PDB next to this build is **52,129,792 bytes** — 7.2× the exe. So *"single exe"* is achieved on MSVC by **not shipping the PDB**, not by stripping. Keep `strip = true` anyway (it is free and harmless), but expect nothing from it, and **keep the PDB as a CI artifact** for crash triage.
- **`panic = "abort"` buys −33,280 bytes and, importantly, works.** This was the one combination worth smoke-testing, because the binary links a large C++ library built with `/EHsc` and Rust `abort` does not affect C++ exception handling. The REC build (which includes `panic = "abort"` *and* fat LTO) launches and runs correctly — verified by 12 successful window-open runs in §3.
- **`opt-level = "z"` on top of the recommended profile makes the binary *bigger*** (7,237,120 vs 7,220,224, +16,896). A good illustration of why this ticket asked for measurement: the intuitive stacking is wrong.

### 2.3 Recommended profile

```toml
[profile.release]
opt-level        = 2
lto              = true
codegen-units    = 1
panic            = "abort"
strip            = true
```

built with `--target x86_64-pc-windows-msvc` and `-C target-feature=+crt-static`.

| | |
|---|---|
| **7,220,224 bytes = 7.22 MB** | measured |
| + icon and VERSIONINFO | **+20,992 bytes** (measured, §4.2) → **≈ 7,241,216 = 7.24 MB** |
| + translation catalogs | + the byte length of each `.mo` (`include_bytes!` is verbatim). **Not measured** — no real catalogs exist yet. Realistically a few tens of KB. |
| **Against the ≤ 40 MB budget** | **18.1 % used. ~32.8 MB headroom.** |

**NFR-exe-size is not a constraint on this project.** It was relaxed to ≤ 40 MB at charting and the honest build lands at under a fifth of that. No feature should be cut, and no dependency refused, on size grounds. If future work wants the extra 271 KB back, `lto = "thin"` costs almost nothing versus fat.

---

## 3. Cold start

### 3.1 Method (stated, because the number is only as good as the method)

- **t0** = `Process.StartTime` — the value the OS itself records for process creation. Using it instead of a stopwatch around `Start-Process` removes PowerShell's own process-spawn overhead from the measurement.
- **t1** = the first instant the process reports a **non-zero `MainWindowHandle` *and* a non-empty `MainWindowTitle`**, polled in a tight loop with no sleep.
- **Poll granularity was calibrated, not assumed: 0.028 ms.** That is the error bar, and it is negligible.
- Each run confirms the title is exactly `PathMaster — NVDA baseline prototype`, so the timer stops on *our* window, not a splash or a stray handle.
- **Known bias:** `MainWindowHandle` becomes valid when the top-level window exists and is shown, which can precede the first paint by a frame. This therefore slightly **under**-states "user sees pixels" — by ~one frame (≈16 ms), not by seconds.

### 3.2 Results, 12 runs each

| Build | min | median | mean | max |
|---|---:|---:|---:|---:|
| **Recommended profile, `crt-static`** | 62.2 ms | 78.5 ms | **79.6 ms** | 97.2 ms |
| `/MD` baseline (ticket 02's exe) | 62.0 ms | 77.6 ms | 75.3 ms | 86.3 ms |

Static linking costs roughly **4 ms** of startup on average — within the run-to-run spread, i.e. not a real difference. Both are ~25× inside the 2 s requirement.

### 3.3 On "cold"

**I could not test a genuinely cache-cold, post-boot start.** Evicting the Windows standby list requires administrative tooling that is not present, and the machine was not rebooted. All 12 runs above are warm; the first run of each batch (70.9 ms and 78.8 ms) was not distinguishable from the rest, which tells us only that the *warm* first run is unremarkable.

Rather than guess at the cold penalty, I measured its upper bound: **reading the entire exe off the device with the file cache bypassed** (`CreateFile` with `FILE_FLAG_NO_BUFFERING`):

```
trial 1: 7,220,224 bytes uncached in 12.9 ms  (532 MB/s)
trial 2:                             2.2 ms  (3,126 MB/s)
trials 3-5:                          2.1-2.2 ms
```

So the whole binary comes off this NVMe SSD in **2–13 ms**. Even adding a pessimistic allowance for cold OS DLLs and a first-ever theme/COM initialisation, a genuinely cold start cannot plausibly approach 2 s on SSD-class hardware.

> **Verdict: 🔴 cold start ≤ 2 s is met with roughly 25× margin**, on SSD. On a spinning disk the number would be worse and was not tested; the requirement says SSD.

---

## 4. Embedded resources

### 4.1 Application manifest — already solved, and it survives `crt-static`

Ticket 02's dependency-free recipe (`build.rs` + `app.manifest`, no `embed_resource`/`winresource` crate) is unchanged and still correct. `mt.exe -inputresource:<exe>;#1` on the **new `crt-static` build** extracted:

- `Microsoft.Windows.Common-Controls` **6.0.0.0**, `publicKeyToken="6595b64144ccf1df"` — comctl32 v6, without which wx silently falls back to legacy controls with different accessibility behaviour.
- `<requestedExecutionLevel level="asInvoker" uiAccess="false"/>` — contributed by the **linker**, which is why `app.manifest` must keep omitting `trustInfo`.
- `PerMonitorV2` plus the legacy `true/pm` `dpiAware` fallback.

This works because `/MANIFEST:EMBED` is a special-cased linker feature: the linker generates the `RT_MANIFEST` resource itself. **There is no equivalent switch for icons** — which is the whole reason §4.2 needs a different mechanism.

### 4.2 Icon — built, embedded, and verified out of the exe

**The mechanism.** An icon is an `RT_GROUP_ICON` directory plus one `RT_ICON` per image in the PE resource directory. `include_bytes!` cannot produce that — it puts bytes in a *data* section, which the shell never reads. The only route is a compiled `.res` handed to the linker as an input file.

**The recipe, with zero new crate dependencies** — `build.rs` compiles `app.rc` to a `.res` in `OUT_DIR` and passes it through the same `rustc-link-arg-bins` channel the manifest already uses:

```rust
println!("cargo:rustc-link-arg-bins={}", res.display());          // the compiled .res
println!("cargo:rustc-link-arg-bins=/MANIFEST:EMBED");            // unchanged from ticket 02
println!("cargo:rustc-link-arg-bins=/MANIFESTINPUT:{}", manifest.display());
```

with `app.rc`:

```rc
1 ICON "app.ico"
1 VERSIONINFO ...
```

**The resource compiler costs nothing new: use `llvm-rc`.** `rc.exe` from the Windows SDK works, but it is not on `PATH` and needs registry/SDK discovery — the annoying part that `winresource` and `embed-resource` exist to solve. **`llvm-rc.exe` sits in the same LLVM installation this build already hard-requires for bindgen** (`C:\scoop\apps\llvm\current\bin\llvm-rc.exe`, 849,920 bytes, alongside the mandatory `libclang.dll`). Since `LIBCLANG_PATH` must be set anyway, `llvm-rc` is findable with one line and adds **no dependency that was not already mandatory**. The demo build.rs prefers `llvm-rc` next to `LIBCLANG_PATH`, then `llvm-rc` on `PATH`, then the newest SDK `rc.exe`.

**Measured result** — a purpose-built 4-entry `.ico` (16/32/48 px as 32-bit BMP, 256 px as PNG; 19,556 bytes) was compiled into a 20,488-byte `.res` and linked:

| Check | Result |
|---|---|
| Build time | **2.2 s** (wxWidgets fully cached) |
| Icon groups in the exe (`ExtractIconEx` with index −1) | **1** |
| Icon Explorer would show (`ExtractAssociatedIcon`) | 32×32, extracted successfully |
| **Pixel comparison, exe icon vs source `.ico` at 32×32** | **0 differing pixels out of 1024 — bit-identical** |
| VERSIONINFO read back as Explorer's Properties tab reads it | `ProductName=PathMaster`, `FileDescription=PathMaster - Windows PATH editor`, `FileVersion=0.1.0.0`, `CompanyName`, `LegalCopyright`, `InternalName`, `OriginalFilename` — all correct |
| Size cost | **+20,992 bytes** (7,571,968 with icon+VERSIONINFO vs 7,550,976 BASE, both `opt-level = 2` + `crt-static`). Caveat: the demo crate has a different package and binary name, worth a few dozen bytes of PE strings — negligible against 20,992, but it is not a pure single-variable delta. |

That is a demonstration, not a claim: the icon was put in and then pulled back out of the binary unchanged.

> #### ⚠ The running window still has **no** icon — this needs application code
>
> With the icon resource correctly embedded, the launched app was queried directly:
>
> ```
> WM_GETICON ICON_BIG   : 0
> WM_GETICON ICON_SMALL : 0
> class GCLP_HICON      : 0
> class GCLP_HICONSM    : 0
> ```
>
> **wxMSW does not adopt the executable's icon resource for the frame.** The exe icon governs Explorer, Alt-Tab thumbnails from the file, and pinned shortcuts; the *window* icon (title bar, taskbar button, Alt-Tab) is separate and is unset. The fix is a `Frame::set_icon(&Bitmap)` call — present at `wxdragon/src/widgets/frame.rs:441` → `wxd_Frame_SetIconFromBitmap` (`:446`).
>
> Getting a `Bitmap` from embedded bytes, with no image-decoding dependency: `Bitmap::from_rgba(&[u8], w, h)` (`wxdragon/src/bitmap.rs:60`) over an `include_bytes!`-ed raw RGBA blob. `Bitmap` has only two constructors — `new(w,h)` (`:40`) and `from_rgba` (`:60`); there is **no** `from_file`/`from_memory` PNG loader on `Bitmap`. The DPI-aware alternative is `BitmapBundle::from_svg_data(&[u8], Size)` (`wxdragon/src/bitmap_bundle.rs:142`) followed by `.get_bitmap(size)` (`:169`), which scales cleanly to any DPI from one embedded SVG.
>
> **Write this into the spec as a required startup step**, or v0.1.0 ships with a generic Windows icon in the taskbar while looking correct in Explorer — a defect that is easy to miss precisely because the file icon looks right.

**Icon content requirements** (Microsoft's app-icon guidance, not measured here): at minimum 16, 24, 32, 48 and 256 px in one multi-resolution `.ico`, 32-bit with alpha, transparent background; Windows looks for an exact size match and otherwise scales *down* from the next size up, which is why 256 matters. Colour depths of 8 bpp and above are treated as equal, so legacy 4-bit/8-bit variants buy nothing.

**Keep exactly one `ICON` statement.** Where several `RT_GROUP_ICON` resources exist the shell takes the first in the resource directory, and the PE resource directory is sorted ascending by ID — so "lowest ID wins". Shipping one icon makes the question moot; the demo used `1 ICON`.

### 4.3 Translation catalogs — mechanics confirmed

Ticket 03 established that the API exists. What a spec author needs is the *mechanics*, read out of the vendored source:

**The trait** (`wxdragon/src/translations.rs:429-445`), verbatim:

```rust
pub trait TranslationsLoader {
    fn load_catalog(&self, domain: &str, lang: &str) -> Option<Cow<'_, [u8]>>;
    fn available_translations(&self, domain: &str) -> Vec<String>;
}
```

Both methods are required. Installed by `Translations::set_loader<L: TranslationsLoader + 'static>` (`:188`); the `'static` bound is on the **loader type**, not on the bytes.

**Mandatory call order** — this is not stylistic, it follows from wx's control flow (`wxWidgets/src/common/translation.cpp`: `AddCatalog` → `DoAddCatalog` `:1283-1315` → `GetBestTranslation` → `DoGetBestAvailableTranslation` `:1418` → `m_loader->GetAvailableTranslations` `:1261`, *then* `LoadCatalog` `:1301`):

```
Translations::new()
  → set_loader(...)         // MUST precede add_catalog
  → set_language_str(...)   // MUST precede add_catalog, else the OS-preferred path is used
  → add_catalog("domain")   // this is what triggers available_translations + load_catalog
  → set_global()            // LAST — takes self by value (:106); translate() is dead until this runs
```

**`include_bytes!` is *not* required — and this is better than expected.** The C++ side wraps the bytes non-owningly (`wxdragon-sys/cpp/src/translations.cpp:35-59`, `wxMsgCatalog::CreateFromData` over `wxScopedCharBuffer::CreateNonOwned`), but `CreateFromData` builds a **stack-local** `wxMsgCatalogFile` and `FillHash` **copies every string out into `wxString`s** (`translation.cpp:1126-1140`, `:1063`, `:1076`, `:1079`). The written contract at `wxdragon-sys/cpp/include/core/wxd_translations.h:109-115` says the bytes need only outlive the `emit` call, and the upstream unit test proves it by returning `Cow::Owned(self.mo.clone())` from a `Vec` that dies immediately (`translations.rs:812-822`). **So a decompressed or synthesised buffer is safe**, not just a `'static` blob. `include_bytes!` remains the simplest route; it is not the only one.

**Three traps worth putting in the spec:**

1. **`add_catalog` returns `true` when nothing loaded.** `DoAddCatalog` pushes the msgid language into the available list, so with the language left at English it returns `true` having loaded no catalog (`translation.cpp:1283-1315`). wxdragon exposes only `AddCatalog`, not wx's stricter `AddAvailableCatalog`. **The reliable success check is `is_loaded(domain)`** (`translations.rs:199`).
2. **`"uk"` vs `"uk_UA"` is asymmetric.** Matching against `m_lang` is exact string equality with exactly one fallback — narrowing to the part before the first `_` (`translation.cpp:1440-1442`). So `set_language_str("uk_UA")` + loader reporting `["uk"]` **matches**; `set_language_str("uk")` + loader reporting `["uk_UA"]` **does not**. Report the narrow code, set the wide one.
3. **`load_catalog` can be called with a `lang` you never advertised.** wx tries up to three strings: `lang + "." + encoding` (e.g. `"uk.WINDOWS-1251"`), plain `lang`, then `lang.BeforeFirst('_')` (`translation.cpp:1318-1353`). A loader that panics on an unexpected `lang` will crash — it must return `None`.

Additionally, a malformed `.mo` is **indistinguishable from "no catalog"**: `CreateFromData` returns `nullptr` and the `bool` returned by the Rust trampoline is ignored (`wxdragon-sys/cpp/src/translations.cpp:56-58`). Corrupt catalogs fail silently.

**Not measured:** no real `.mo` was built or embedded here, so the size cost of the catalogs is unmeasured (it is simply their byte length) and the end-to-end path has not been exercised inside a running GUI process. The upstream unit test never calls `set_global`, never creates a `wxApp`, and never runs an event loop.

---

## 5. What CI must pin

Split by whether drift actually breaks something.

### 5.1 Load-bearing — pin these

| Thing | Value here | Why it is load-bearing |
|---|---|---|
| **Target triple** | `x86_64-pc-windows-msvc` | Must be passed **explicitly as `--target`**, not merely implied by the host: that is what keeps `RUSTFLAGS` off host build scripts and the `wxdragon-macros` proc macro. Also the only target this project supports (map: no 32-bit, no non-Windows). |
| **`RUSTFLAGS=-C target-feature=+crt-static`** | — | **This is 🔴 NFR-portable itself.** Without it the release silently needs the VC++ redistributable. |
| **`LIBCLANG_PATH`** | `C:\scoop\apps\llvm\current\bin` | `wxdragon-sys/build.rs` runs bindgen **unconditionally**; there are no pre-generated bindings and no feature to skip it. Without libclang the build dies in the build script. |
| **LLVM version** | **22.1.8** | Bindgen output is libclang-version-sensitive; and `llvm-rc` from this same install is the icon toolchain (§4.2). |
| **Ninja** | **1.13.2** | **Not optional.** `wxdragon-sys/build.rs:379` **hardcodes** `.generator("Ninja")` on the x64 MSVC path — there is no Makefiles fallback on that branch. If Ninja disappears the build fails outright. |
| **CMake** | **4.4.2** | `build.rs` drives the whole wxWidgets build through `cmake::Config`. |
| **wxdragon** | **0.9.18** (≥ 0.9.17 per ticket 01) | Earlier `AccRole` discriminants are mis-ordered. Pin exactly and commit `Cargo.lock`. |
| **Rust toolchain** | **1.94.0** here | Pin an **exact version** in `rust-toolchain.toml`, not `stable`. Neither wxdragon crate declares a `rust-version`, so there is no MSRV to read off; drift is silent until it isn't. |
| **MSVC toolset + Windows SDK** | 14.50.35717 / `cl` 19.50.35727; SDK 10.0.22621.0 | Determines the static CRT actually linked in. Pin the VS image, not "latest". |
| **Checkout path length** | — | See the MAX_PATH trap above. Keep the CI working directory short. |

```toml
# rust-toolchain.toml
[toolchain]
channel   = "1.94.0"
targets   = ["x86_64-pc-windows-msvc"]
profile   = "minimal"
components = ["clippy", "rustfmt"]
```

### 5.2 Environment hazards discovered in `build.rs` — not asked for, but they bite

Every environment variable `wxdragon-sys/build.rs` reads: `OUT_DIR`, `CARGO_CFG_TARGET_OS`, `CARGO_CFG_TARGET_ENV`, `CARGO_CFG_TARGET_ARCH`, `CARGO_CFG_TARGET_FEATURE`, `TARGET`, `PROFILE`, **`DOCS_RS`**, **`RUST_ANALYZER`**, **`WXWIDGETS_DIR`**, `CMAKE_TLS_VERIFY`, `CC`, `CXX`.

- **`DOCS_RS` or `RUST_ANALYZER=true` silently switch the build to "generate bindings only" and `return Ok(())`** (`build.rs:39-50`) — no wxWidgets, no libraries. If either leaks into a CI environment the build changes shape. Assert they are unset.
- **`WXWIDGETS_DIR`** (`build.rs:61-67`) points the build at a pre-extracted wxWidgets source tree and **skips the version check and the download** — the intended lever for a CI cache.
- **`RUSTFLAGS` silently overrides `.cargo/config.toml`.** Cargo's rustflags sources are mutually exclusive, first match wins, and the `RUSTFLAGS` env var beats `target.<triple>.rustflags` in a config file. If the project puts `crt-static` in `.cargo/config.toml` and a workflow step sets `RUSTFLAGS` for any other reason, **`crt-static` is silently dropped and the release quietly needs the VC++ redistributable.** → **CI must assert on the artifact, not on the config**: run `dumpbin /DEPENDENTS` (or equivalent) on the released exe and fail the build if `VCRUNTIME` or `MSVCP` appears. This is a cheap, decisive gate and it should be mandatory.

### 5.3 Incidental — record, don't pin

`bindgen`, `cmake-rs`, `reqwest` and the ~130 transitive build-dependency crates are pinned transitively by `Cargo.lock`; that is sufficient. The wxWidgets version itself is not a CI knob at all — `wxdragon-sys/build.rs:1-3` hard-codes the 3.3.3 release URL **and its SHA-256** (`458a1ef5…feb1`), so it is pinned by the crate and tamper-evident.

### 5.4 One caution about GitHub Actions

`windows-latest` now maps to **Windows Server 2025**. Its preinstalled versions (read from `actions/runner-images` at the time of writing, and they change every image release) **do not match this machine**: LLVM **20.1.8** there vs 22.1.8 here, Rust **1.97.1** there vs 1.94.0 here, CMake 3.31.6 vs 4.4.2. LLVM lives at `C:\Program Files\LLVM` with `bin` on the machine PATH; Ninja and CMake are preinstalled; MSVC and the SDK come with VS Enterprise 2022. **Nothing needs installing — but nothing should be inherited either.** Set `LIBCLANG_PATH=C:\Program Files\LLVM\bin` explicitly, pin the runner label (`windows-2025`, not `windows-latest`), and pin the toolchain. Note that SDK `bin` (hence `rc.exe`) is **not** on PATH there — another reason §4.2 prefers `llvm-rc`.

*(This subsection is the one place in this report built on documentation rather than measurement: no CI run was performed.)*

---

## 6. First-build wall-clock — what CI pays

Measured with a stopwatch around `cargo build --release`, into an empty `CARGO_TARGET_DIR`, with `crt-static` forcing a full C++ recompile:

| Phase | Elapsed | Duration |
|---|---:|---:|
| ~130 Rust build-dependency crates (incl. `reqwest`, `tokio`, `bindgen`) | 0 → 11.5 s | **11.5 s** |
| **`wxdragon-sys`** — bindgen + CMake configure + full wxWidgets 3.3.3 compile + C++ shim | 11.5 → 118.9 s | **107.4 s** |
| `wxdragon` + app crate + link | 118.9 → 127.0 s | **8.1 s** |
| **Total** | | **127.0 s** (cargo reported `2m 06s`) |

**This excludes the source download.** `wxdragon-sys/build.rs:72` caches the wxWidgets zip at `%TEMP%\wxWidgets.zip` and verifies its SHA-256 before reusing it; the file was already present here (**45,356,028 bytes**, from ticket 02's build). **A CI runner with a truly cold cache also pays that 45.4 MB download from GitHub Releases.** Extraction *is* included in the 127 s.

Two consequences for CI:

- **Cache `%TEMP%\wxWidgets.zip`, or set `WXWIDGETS_DIR`** to a cached extracted tree. The former saves the download; the latter saves the download *and* the extraction and the version probe.
- **The 443 MB of wxWidgets static libraries live at the profile root** (`target/<triple>/release/wxdragon_sys_cmake_build/`), not inside a fingerprint-keyed `OUT_DIR`. That is why it survives profile changes — measured directly: after the initial 127 s build, all eleven size-matrix variants rebuilt in **8–30 s each** with no wxWidgets recompile. Caching that directory is what makes CI fast; be aware it is large.

A 20-thread desktop did this in ~2 minutes. A 4-core GitHub runner will be materially slower; that was not measured.

---

## 7. Could NOT determine

1. **A genuinely cache-cold, post-boot start.** No admin tooling to evict the standby list, and no reboot. Mitigated by measuring the uncached full-file read (2–13 ms) as an upper bound on the extra I/O, but the true first-run-after-boot figure is unmeasured. Given ~25× margin this is not a risk, merely an unmeasured cell.
2. **Startup on a spinning disk or a slow/network drive.** Only the NVMe SSD was tested. The requirement specifies SSD.
3. **Behaviour on a machine with no VC++ redistributable installed.** The import table and the runtime module list are strong indirect evidence (nothing outside `%SystemRoot%` loads), but the direct test — running the exe on a clean Windows image — was not performed. **Worth doing once in ticket 15**, on a fresh VM, before the first release.
4. **CI itself.** No GitHub Actions run was performed; §5.4 is documentation-derived and the runner image versions drift.
5. **Translation catalogs end to end.** No real `.mo` was built or embedded; §4.3 is source-derived plus an upstream unit test that never starts a `wxApp`. Their size cost is unmeasured.
6. **A 4-core CI-class build time.** Only a 20-thread desktop was timed.
7. **The MinGW/UCRT route.** Not investigated — `crt-static` on MSVC already solves the problem it would solve.
8. **Whether `strip = "debuginfo"` differs from `strip = true`.** Only `true` (≡ `"symbols"`) was measured; it moved zero bytes, so the distinction is almost certainly moot on MSVC.

## 8. Discovered along the way — for later tickets

**For ticket 15 (release and manifests), which this ticket unblocks:**

- **The release CI must gate on the artifact's imports, not on the build config.** `RUSTFLAGS` silently overrides `.cargo/config.toml`, so `crt-static` can be lost without any warning and the exe still builds and runs *on a developer machine that has the redistributable*. A `dumpbin /DEPENDENTS` check that fails on `VCRUNTIME|MSVCP` is the only reliable guard for 🔴 NFR-portable. Cheap; make it mandatory.
- **`VERSIONINFO` is free once the icon `.rc` exists** and was demonstrated working — `ProductName`, `FileVersion`, `ProductVersion`, `CompanyName`, `LegalCopyright`, `OriginalFilename` all read back correctly through Explorer's Properties tab. **winget in particular cares about these**, and the version string in the `.rc` must be kept in sync with `Cargo.toml` and the git tag. That synchronisation is a ticket-15 problem.
- **A 52 MB PDB is produced next to a 7.2 MB exe even at `debug = false`.** Do not ship it (that is what makes "single exe" true on MSVC), but **do** keep it as a CI build artifact per release — it is the only way to symbolicate a crash report from an unsigned binary in the field.
- **`FileDescription` is what SmartScreen and Task Manager display.** Since v0.1.0 ships unsigned by decision (map decision 10), the VERSIONINFO strings are the *only* identity the binary carries. Worth getting right.
- **The window icon needs application code** (§4.2). A release that looks right in Explorer can still show a generic icon in the taskbar.

**For the visual-design / layout work still in "Not yet specified":**

- `PerMonitorV2` DPI awareness is already asserted by the embedded manifest, so the app opts into per-monitor scaling from the first release. Combined with ticket 03's finding that `FromDIP` is applied implicitly and invisibly at the FFI boundary and cannot be queried or bypassed, **DPI behaviour is decided by default rather than by choice** — worth a deliberate look once there is a real layout.
- One embedded SVG via `BitmapBundle::from_svg_data` (`bitmap_bundle.rs:142`) covers every DPI from a single asset, which is a better fit than a fixed-size raster for in-app iconography.

**For tickets 02 / 08 (NVDA):**

- **`OLEACC.dll` is in the exe's import table in both CRT modes**, and at runtime the process also loads `uiautomationcore.dll`. That is independent corroboration, from the linked binary rather than from a header, that the MSAA/accessibility layer is genuinely compiled in and reached.
- **`wxUSE_ACCESSIBILITY 1`** was re-confirmed in the `crt-static` build's freshly generated `setup.h` (lines 518 and 520, both branches of the `#ifdef __WXMSW__`). Switching CRT mode does not disturb it.
- While measuring, NVDA 2025.3.3 injected `nvdaHelperRemote.dll`, `IAccessible2Proxy.dll` and `ISimpleDOM.dll` into the prototype process — so NVDA is actively hooking this build, and the ticket-02 listening session has a live target.

**General:**

- **The MAX_PATH trap** (§ Measurement environment) will hit any developer who clones into a deep path, and the error message blames the C++ compiler. It belongs in CONTRIBUTING/setup docs, not just here.
- **`opt-level` barely matters and `3` is marginally worse than `2`** — because most of the binary is C++ compiled by CMake, outside the Rust profile's reach. Any future "let's optimise the build" instinct should go to `lto` and stop there.
