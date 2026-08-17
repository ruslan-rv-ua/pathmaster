# wxdragon accessibility surface

Research for ticket `01-wxdragon-accessibility-surface`. Target: **wxdragon 0.9.18** (latest published,
2026-07-28), read from the actual crate source fetched with `cargo fetch` (no build performed).

**Where the crate source lives on this machine** (fetched during this research; throwaway probe project at
`C:\Temp\claude\C--dev-PathMaster2\5cab669f-a49a-4c27-a633-ca7a38dc116d\scratchpad\wxprobe`):

- `C:\scoop\persist\rustup\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\wxdragon-0.9.18\`
- `C:\scoop\persist\rustup\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\wxdragon-sys-0.9.18\`
- `C:\scoop\persist\rustup\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\wxdragon-macros-0.9.18\`

Note `CARGO_HOME` on this machine is `C:\scoop\persist\rustup\.cargo`, not `%USERPROFILE%\.cargo`.

wxWidgets source files quoted below were fetched from the `v3.3.3` tag on GitHub and cached at
`C:\Temp\claude\C--dev-PathMaster2\5cab669f-a49a-4c27-a633-ca7a38dc116d\scratchpad\wx333\`.

---

## Headline correction to the ticket's premise

The ticket asks what "wxdragon's **prebuilt** wxWidgets binaries" enabled. **wxdragon 0.9.18 has no prebuilt
binaries.** It downloads the wxWidgets **source** archive and compiles it from scratch as a CMake subproject
of its own C++ wrapper. Every question about "what the prebuilt archive enabled" therefore becomes a question
about **what CMake options the build configures**, which is fully readable in the repo — a strictly better
position to be in.

Evidence:

- `wxdragon-sys-0.9.18\build.rs:1` — `const WX_SRC_URL: &str = "https://github.com/wxWidgets/wxWidgets/releases/download/v3.3.3/wxWidgets-3.3.3.zip";`
- `wxdragon-sys-0.9.18\build.rs:2-3` — `const WX_VERSION: &str = "3.3.3";` and a SHA256 pin
  `458a1ef598c90174ee43622e8e63bfa1eccb451ffc2258bb4f8edcb050c5feb1`, verified after download
  (`build.rs:1101-1102`).
- `wxdragon-sys-0.9.18\build.rs:69-93` — downloads the zip to `%TEMP%\wxWidgets.zip`, verifies the SHA256,
  extracts it, and skips re-download if the version already matches.
- `wxdragon-sys-0.9.18\cpp\CMakeLists.txt:88` — `add_subdirectory(${WXWIDGETS_LIB_DIR} ${WXWIDGETS_BUILD_DIR})`,
  i.e. wxWidgets is built as a nested CMake project, with wxdragon's own `set(... CACHE BOOL ...)` calls at
  lines 31-77 acting as pre-seeded cache values.
- The stale `wxdragon-sys-0.9.18\README.md` still says "Fetches the wxWidgets **3.2.4** source tarball" — it is
  out of date; build.rs is authoritative.
- Prebuilt archives did exist historically but were abandoned: the only two GitHub releases carrying assets are
  `wxwidgets-3.3.0` (2025-06-23, 28 assets) and `wxwidgets-3.3.1` (2025-07-23, 16 assets)
  (`gh api repos/AllenDang/wxDragon/releases`). All 60 other releases have zero assets. The workflow that built
  them, `.github/workflows/wxwidgets-build.yml`, is `workflow_dispatch`-only and still defaults to
  `wx_version: "3.3.0"`. CHANGELOG confirms the arc: prebuilt introduced in 0.8.0 ("Automatic download of
  pre-built libraries from GitHub releases"), and by 0.9.x the source path is what build.rs uses.

**Practical consequence:** the first `cargo build` will compile all of wxWidgets. Expect a long cold build and
a hard dependency on CMake + Ninja + MSVC being present. That belongs to the build/packaging ticket, not here.

---

## Q1 — Which wxWidgets version, and in which configuration? Where does `build.rs` decide it?

**wxWidgets 3.3.3, built from source, static, non-monolithic, Release-flavoured, with the toolchain matching the
Rust target (MSVC for `*-pc-windows-msvc`, MinGW for `*-pc-windows-gnu`).**

| Property | Value | Evidence |
|---|---|---|
| Version | 3.3.3 | `build.rs:1-2` |
| Source or prebuilt | source zip, SHA256-pinned | `build.rs:1-3, 69-93` |
| Static vs shared | **static** (`wxBUILD_SHARED OFF`) | `cpp\CMakeLists.txt:70` |
| Monolithic | no (`wxBUILD_MONOLITHIC OFF`) | `cpp\CMakeLists.txt:71` |
| MSVC build type | `RelWithDebInfo`, generator **Ninja** | `build.rs:376-384` |
| MSVC CRT | `MultiThreadedDLL`, or `MultiThreaded` when `crt-static` is in `CARGO_CFG_TARGET_FEATURE` | `build.rs:365-374` |
| MSVC debug profile | **forced to release** (`is_debug = false` at `build.rs:363`) to avoid CRT mismatch with Rust's MSVC toolchain | `build.rs:357-363` |
| MinGW (`target_env == "gnu"`) | generator `MinGW Makefiles`, `gcc`/`g++`, `CMAKE_BUILD_TYPE` = Release/Debug per cargo profile | `build.rs:339-356`, `build.rs:412-415` |
| i686 / aarch64 MSVC | switches to the Visual Studio generator with `CMAKE_GENERATOR_PLATFORM` `Win32` / `ARM64` | `build.rs:391-409` |
| Unicode | `UNICODE`, `_UNICODE`, `wxUSE_UNICODE=1` | `cpp\CMakeLists.txt:102-105` |
| Escape hatch | `WXWIDGETS_DIR` env var points the build at a custom wxWidgets source tree (skips download and version check) | `build.rs:61-68` |

CI builds and clippy-checks `i686-pc-windows-msvc`, `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`, and
`x86_64-pc-windows-gnu` (`.github/workflows/rust.yml:85-159`), so both Windows toolchains are exercised
upstream. For PathMaster, `x86_64-pc-windows-msvc` is the well-trodden path.

---

## Q2 — Is `wxUSE_ACCESSIBILITY=1`?

**Yes, on Windows — by default, and by a chain that is verifiable end to end.** wxdragon never mentions
`wxUSE_ACCESSIBILITY` anywhere, so wxWidgets' own default governs, and that default is ON for `WIN32`.

The chain:

1. wxdragon does **not** set it. Grepping `wxUSE_ACCESSIBILITY` across `wxdragon-sys-0.9.18\build.rs` and
   `wxdragon-sys-0.9.18\cpp\CMakeLists.txt` returns nothing; the only hits in the whole crate are the
   `#if wxUSE_ACCESSIBILITY` guards inside `cpp\src\core\accessible.cpp`. Because wxdragon's own options are
   set with `set(... CACHE BOOL ...)` **without** `FORCE` (`cpp\CMakeLists.txt:31-77`), any option it does not
   pre-seed simply takes wxWidgets' default.
2. wxWidgets declares it inside the `if(WIN32)` block with no explicit default:
   `build/cmake/options.cmake:498,505` — `wx_option(wxUSE_ACCESSIBILITY "enable accessibility support")`.
3. `wx_option` with exactly two arguments defaults to **ON**:
   `build/cmake/functions.cmake:1131-1145` — `if(ARGC EQUAL 2) set(default ON) ... set(${name} "${default}" CACHE ${cache_type} "${desc}")`.
4. That CMake variable is stamped into the generated `setup.h`:
   `build/cmake/setup.h.in:517-521` — `#cmakedefine01 wxUSE_ACCESSIBILITY` (both branches of the
   `#ifdef __WXMSW__`), so ON becomes `#define wxUSE_ACCESSIBILITY 1`.
5. The one condition that could silently turn it off is `wxUSE_OLE`:
   `include/wx/msw/chkconf.h:331-339` — inside `#if !wxUSE_OLE`, it does
   `#undef wxUSE_ACCESSIBILITY / #define wxUSE_ACCESSIBILITY 0`. But `wxUSE_OLE` also defaults ON on Windows
   (`build/cmake/options.cmake:182-184`, `wx_option(wxUSE_OLE "use OLE classes")` inside `if(WIN32)`), and
   wxdragon never disables it. So the guard does not fire.
6. For cross-reference, the non-CMake header default agrees: `include/wx/msw/setup.h:1357-1367` —
   *"Default is 1 on MSW, 0 elsewhere"*, `#ifdef __WXMSW__ #define wxUSE_ACCESSIBILITY 1`.

**Residual risk (small, and cheap to close):** this is a derivation across four files, not an observation of a
generated artifact. The definitive check is one line of output from a real build — read
`<target>\...\wxwidgets_cmake_build\lib\wx\include\...\wx\setup.h` (or `CMakeCache.txt`, which will contain
`wxUSE_ACCESSIBILITY:BOOL=ON`) after the first successful compile and confirm the `1`. Fold that check into the
build/toolchain ticket; do not treat it as blocking, but do treat it as **not yet observed**.

---

## Q3 — Does wxdragon bind `wxAccessible`?

**Yes. Fully, and more completely than the ticket assumed.** This is the single most consequential finding.

There is a first-class, Windows-gated `accessible` module:

- `wxdragon-0.9.18\src\lib.rs:6-7` — `#[cfg(target_os = "windows")] pub mod accessible;`
- `wxdragon-0.9.18\src\prelude.rs:2-3` — `#[cfg(target_os = "windows")] pub use crate::accessible::Accessible;`

What it gives you:

| Rust item | File + line | Maps to |
|---|---|---|
| `trait AccessibleImpl` (18 overridable methods) | `src\accessible.rs:226` | the `wxAccessible` virtual interface |
| `struct Accessible` | `src\accessible.rs:284` | `wxAccessible` |
| `Accessible::new(window, impl)` | `src\accessible.rs:291-317` | `new WxdCustomAccessible(...)` |
| `Accessible::notify_event(event_type: u32, window, object_type: AccObjectType, object_id: i32)` | `src\accessible.rs:333-337` | `wxAccessible::NotifyEvent` → `::NotifyWinEvent` |
| `enum AccStatus` | `src\accessible.rs:4-27` | `wxAccStatus` |
| `enum NavDir` | `src\accessible.rs:29-59` | `wxNavDir` |
| `enum AccObjectType` (12 variants) | `src\accessible.rs:62-87` | MSAA `OBJID_*` |
| `enum AccRole` (MSAA `ROLE_SYSTEM_*`) | `src\accessible.rs:89+` | `wxAccRole` |
| `bitflags AccState` (MSAA `STATE_SYSTEM_*`) | `src\accessible.rs:171+` | MSAA state bitmask |
| `WxWidget::set_accessible(Accessible)` | `src\window.rs:1901` | `wxWindow::SetAccessible` |
| `WxWidget::set_accessibility_label / _description / _value` | `src\window.rs:1636 / 1655 / 1674` | built-in provider (see below) |
| `WxWidget::set_accessibility_role(AccRole)` (Windows-only) | `src\window.rs:1693-1702` | built-in provider |
| `WxWidget::set_accessibility_state(AccState)` (Windows-only) | `src\window.rs:1710-1719` | built-in provider |

C layer: `wxdragon-sys-0.9.18\cpp\include\core\wxd_accessible.h` (full enum + callback-struct + function
surface) and `wxdragon-sys-0.9.18\cpp\src\core\accessible.cpp`, compiled unconditionally into the wrapper
(`cpp\CMakeLists.txt:125`).

Two design details that matter a great deal:

**(a) Every accessibility entry point is wrapped in `#if wxUSE_ACCESSIBILITY`, with silent no-op `#else`
branches.** `accessible.cpp:243-247, 253-257, 263-267, 272-279, 284-291, 296-305, 310-319, 324-333, 338-347,
352-359`. If the flag were ever 0, `wxd_Accessible_Create` returns `nullptr` and every setter and
`NotifyEvent` becomes a no-op — **the Rust API still compiles and still runs, it just does nothing.** There is
no compile error, no runtime error, no log line. That is exactly the failure mode that could sink the product
silently, and it is why the setup.h check in Q2 is worth doing once for real.

**(b) The built-in provider is a well-behaved partial override, not a hijack.** `set_accessibility_label` and
friends attach a `WxdSimpleAccessible` (`accessible.cpp:188-224`, created lazily by
`wxd_GetOrCreateSimpleAccessible`, `accessible.cpp:228-235`). It returns `wxACC_OK` **only** for properties
that were explicitly set and only for `childId == 0` (self); everything else returns `wxACC_NOT_IMPLEMENTED`
(`accessible.cpp:198-217`). wxMSW then falls back to the native control's own `IAccessible` via
`::CreateStdAccessibleObject` (`wxWidgets src/msw/ole/access.cpp:1793-1812`, and the ~20
`if (status == wxACC_NOT_IMPLEMENTED)` fallback sites throughout that file). So overriding just the name of a
`ListCtrl` does not blind NVDA to its rows and columns.

**One caveat worth carrying:** by default no wxAccessible exists at all —
`include/wx/window.h:1580-1582`, `virtual wxAccessible* CreateAccessible() { return nullptr; }`, and no
wxWidgets control overrides it (checked `include/wx/msw/listctrl.h`, `include/wx/generic/listctrl.h`,
`include/wx/msw/treectrl.h`, `include/wx/msw/notebook.h` — zero hits for `CreateAccessible`). While the
accessible is null, `WM_GETOBJECT` is not handled at all (`src/msw/window.cpp:3631-3644` — the whole `case
WM_GETOBJECT` sits inside `#if wxUSE_ACCESSIBILITY` and is skipped when `GetOrCreateAccessible()` returns
null) and `DefWindowProc` hands the screen reader the raw comctl32 object. **This confirms the map's "NVDA
reads native controls for free" claim.** But the first `set_accessibility_*` call on a widget flips that widget
onto the wx-mediated path. It should be transparent because of the NOT_IMPLEMENTED fallback — but it is a
change in the plumbing, not a pure addition, and it should be verified with NVDA rather than assumed.

**Version floor:** use **≥ 0.9.17**. `AccRole`'s discriminants were mis-ordered until PR #155 ("Reordered
`wxd_AccRole` to match `wxAccRole`'s alphabetical order", CHANGELOG line 45, in the 0.9.17 section). Before
that fix, `set_accessibility_role` would report the wrong MSAA role.

**docs.rs will not show you any of this.** docs.rs builds `wxdragon` only for `x86_64-unknown-linux-gnu`
(confirmed on <https://docs.rs/wxdragon/0.9.18/wxdragon/> — target selector shows that single target, and no
`accessible` module is listed; <https://docs.rs/wxdragon/0.9.18/wxdragon/accessible/index.html> returns 404).
Anyone evaluating wxdragon's accessibility from docs.rs alone would conclude, wrongly, that it has none.

---

## Q4 — Native window handle (`HWND`)? In what Rust type? Usable with the `windows` crate?

**Yes — on the `WxWidget` trait, so every widget has it, and it drops straight into the `windows` crate with a
zero-cost newtype wrap.**

```rust
// wxdragon-0.9.18\src\window.rs:1605
fn get_handle(&self) -> *mut std::ffi::c_void
```

- `wxdragon-0.9.18\src\window.rs:1596-1611` — safe (non-`unsafe`) trait method on `WxWidget`, returns null if
  the widget pointer is null.
- `wxdragon-sys-0.9.18\cpp\src\window.cpp:978-985` — `return reinterpret_cast<void*>(wx_window->GetHandle());`
  On wxMSW `wxWindow::GetHandle()` is the `HWND`.
- It is a **trait** method, so `Frame`, `ListCtrl`, `StatusBar`, `TextCtrl` etc. each yield their own HWND
  (`src\widgets\list_ctrl.rs:996` — `impl WxWidget for ListCtrl`; `src\widgets\statusbar.rs:159` —
  `impl WxWidget for StatusBar`).
- CHANGELOG 0.8.23: *"Added wxWindow::GetHandle() method for native window handle access … Provides access to
  HWND on Windows, NSWindow on macOS, GtkWidget on Linux"* — so it is a deliberate, documented API, not an
  accident.

Handing it to the `windows` crate is a direct wrap, no cast, no transmute:

- `windows-0.62.2\src\Windows\Win32\Foundation\mod.rs:5668-5670` —
  `#[repr(transparent)] pub struct HWND(pub *mut core::ffi::c_void);`
- `windows-0.62.2\src\Windows\Win32\UI\Accessibility\mod.rs:165` —
  `pub unsafe fn NotifyWinEvent(event: u32, hwnd: HWND, idobject: i32, idchild: i32)` (feature
  `Win32_UI_Accessibility`, links `user32.dll`).
- The MSAA event constants live one module over, in `Win32_UI_WindowsAndMessaging`, not in
  `Win32_UI_Accessibility`: `windows-0.62.2\src\Windows\Win32\UI\WindowsAndMessaging\mod.rs:3438` —
  `EVENT_OBJECT_LIVEREGIONCHANGED: u32 = 32793`; `:3447` — `EVENT_OBJECT_SHOW: u32 = 32770`; `:3455` —
  `EVENT_SYSTEM_ALERT: u32 = 2`. `OBJID_*` are there too (`:5326-5330`, e.g. `OBJID_CLIENT = -4`,
  `OBJID_ALERT = -10`). Enable both features.

So: `unsafe { NotifyWinEvent(EVENT_OBJECT_LIVEREGIONCHANGED, HWND(widget.get_handle()), OBJID_CLIENT.0, CHILDID_SELF) }`
compiles with nothing but `windows` + `wxdragon`. **The hinge the ticket was worried about is not a hinge — it
is open.**

Lifetime caveat, from wxdragon's own doc comment (`src\window.rs:1601-1603`): the pointer is only valid for the
lifetime of the widget. Fetch it at the point of use; never cache it across a widget's destruction.

---

## Q5 — `SetName`, `SetLabel`, `SetToolTip`, `SetHelpText`?

| wxWidgets member | Bound? | Rust name | Evidence | Does it feed the MSAA name/description? |
|---|---|---|---|---|
| `wxWindow::SetLabel` | **yes** | `WxWidget::set_label` | `src\window.rs:788-800` → `cpp\src\window.cpp:219-224` | Indirectly and conditionally — see below |
| `wxWindow::SetName` | **yes** | `WxWidget::set_name` | `src\window.rs:1086-1100` → `cpp\src\window.cpp:580-586` | **No** |
| `wxWindow::SetToolTip` | **yes** | `WxWidget::set_tooltip` (note: no underscore between "tool" and "tip") | `src\window.rs:497-511` → `cpp\src\window.cpp:171-178` | Not the accName; NVDA may read tooltips separately |
| `wxWindow::SetHelpText` | **no** | — | no hits for `SetHelpText` anywhere in `wxdragon-sys-0.9.18\cpp\include` except the `AccessibleImpl::GetHelpText` **callback** (`cpp\include\core\wxd_accessible.h:166`) | n/a |

Correcting the ticket's framing on the first two:

- **`SetName` does not feed MSAA.** It is wxWidgets' internal identifier used by `FindWindowByName`. wxdragon's
  own doc comment says so (`src\window.rs:1083-1085`: *"different from the label and is used for
  identification purposes, such as finding windows by name"*). Nothing in `wxWindowAccessible` reads it.
- **`SetLabel` feeds the MSAA name only through the generic `wxWindowAccessible`**, which — per Q3 — is **not
  installed by default**. `src/common/wincmn.cpp:3890-3910`, `wxAccStatus wxWindowAccessible::GetName(int
  childId, wxString* name)` returns `GetWindow()->GetLabel()` (mnemonic-stripped) or `wxACC_NOT_IMPLEMENTED`
  when empty. For native controls, the reason `SetLabel` works in practice is more direct: it sets the actual
  Win32 window text, which comctl32's own `IAccessible` uses for `accName`.
- The **supported** way to set an MSAA name in wxdragon 0.9.18 is `set_accessibility_label`
  (`src\window.rs:1636-1647` → `wxd_Window_SetAccessibleName` → `WxdSimpleAccessible::SetNameProp`). Its doc
  comment states the Windows behaviour plainly: *"stored on a built-in accessible object (active where
  `wxUSE_ACCESSIBILITY` is compiled in)"* (`src\window.rs:1629-1630`). Description and value have the same
  shape.
- `SetHelpText` being unbound is a non-issue: `set_accessibility_description` covers the MSAA description slot,
  which is what a screen reader actually reads.

---

## Q6 — Issue / PR history on accessibility, screen readers, `HWND`

Searched `AllenDang/wxDragon` via `gh search issues` / `gh search prs` / `gh api search/issues` for
`accessib`, `accessibility`, `screen reader`, `HWND`, `NVDA`. Ten items match `accessibility`; the load-bearing
ones:

| # | Kind | State | Date | Title / substance |
|---|---|---|---|---|
| [#155](https://github.com/AllenDang/wxDragon/pull/155) | PR | **merged** | 2026-06-11 | *fix: reorder `wxd_AccRole` to match `wxAccRole`'s alphabetical order* — by **LeonarddeR**. Shipped in 0.9.17. Sets the version floor. |
| [#150](https://github.com/AllenDang/wxDragon/pull/150) | PR | **merged** | 2026-06-27 | *Add macOS VoiceOver accessibility helpers to `WxWidget`* — by **trypsynth**. Origin of `set_accessibility_label` (macOS-only at first). Shipped in 0.9.17. |
| [#158](https://github.com/AllenDang/wxDragon/pull/158) | PR | **merged** | 2026-07-06 | *Add accessibility property setters to `WxWidget`* — by **LeonarddeR**. Shipped in 0.9.18. The PR body carries a per-platform behaviour table and states the design rationale explicitly, including *"wxWidgets itself only supports `wxAccessible`/MSAA under `__WXMSW__`, see `wx/chkconf.h`"*. |
| [#160](https://github.com/AllenDang/wxDragon/issues/160) | Issue | **open** | 2026-07-02 | *macOS/VoiceOver: DataViewCtrl container (group) rows are announced as empty — need per-row accessible name*. macOS/`NSOutlineView`-specific. Its core complaint — that per-row/per-cell accessible names are not reachable from the safe API — is a **real limitation to note**, though on Windows a `ListCtrl` in report mode is a native `SysListView32` whose rows MSAA exposes natively, so it should not bite PathMaster. |

- **Zero** issues or PRs mention `HWND` or `NVDA`. No open bug says wxdragon's Windows accessibility is broken —
  but equally, **no evidence exists that anyone has verified it against NVDA**.
- `gh api search/commits ... accessib` returns 0 (the commits search index does not cover this repo); CHANGELOG
  is the better history. It records `wxAccessible: Added accessibility support wrapper` under **0.9.10**
  (2026-02-13), i.e. the binding is roughly six months old.
- Repo has one documentation file, `docs/events.md` — no accessibility guide.

**The strongest soft signal in this whole report:** `LeonarddeR` is the 5th-ranked contributor by commit count
(22 commits, `gh api repos/AllenDang/wxDragon/contributors`) and authored both accessibility PRs. That handle
belongs to Leonard de Ruijter, a core NVDA developer. Someone who works on NVDA has been landing wxdragon's
Windows accessibility API. That is not proof it works, and must not be treated as such — but it makes wxdragon
a far less lonely bet than the ticket assumed.

---

## Q7 — Current version, release cadence, and forking

**Current version:** 0.9.18, published 2026-07-28 (crates.io API). 68 published versions. Repo last pushed
2026-08-17 — one day before this research.

**Cadence:** roughly monthly in the 0.9.x line, much faster earlier:

| Version | Date | | Version | Date |
|---|---|---|---|---|
| 0.9.18 | 2026-07-28 | | 0.9.13 | 2026-03-06 |
| 0.9.17 | 2026-07-01 | | 0.9.12 | 2026-02-21 |
| 0.9.16 | 2026-05-12 | | 0.9.10 | 2026-02-13 |
| 0.9.15 | 2026-04-20 | | 0.9.0 | 2025-10-15 |
| 0.9.14 | 2026-03-18 | | 0.8.0 | 2025-06-23 |

First release ~2025-05; repo created 2025-05-08. 0.8.x saw 30 releases in ~4 months (sometimes several a day),
so the project has settled down considerably. Still pre-1.0: expect breaking changes between minors.

**Project health:** 203 stars, 21 forks, 12 open issues, 14 contributors. Allen Dang has 581 of ~1000 commits —
**a single-maintainer project with a modest contributor tail** (`ssrlive` 269, `trypsynth` 39,
`aryanchoudharypro` 30, `LeonarddeR` 22). Bus factor is a real, if unquantified, risk. License is permissive:
`MIT OR Apache-2.0` (`wxdragon-0.9.18\Cargo.toml`), so forking is legally unencumbered.

**What forking would actually cost — and it is unusually cheap:**

1. The workspace is three crates under `rust/`: `wxdragon`, `wxdragon-sys`, `wxdragon-macros`.
2. FFI bindings are generated by **bindgen at build time** from `cpp/include/wxdragon.h`
   (`build.rs:95-98`, and the docs.rs/rust-analyzer short-circuit at `build.rs:39-49`). **There is no
   checked-in `bindings.rs` to regenerate or keep in sync.**
3. Adding a binding is therefore: add a `WXD_EXPORTED` function to `cpp/include/core/*.h`, implement it in
   `cpp/src/*.cpp`, add a safe wrapper in `rust/wxdragon/src/`. Bindgen picks it up automatically.
4. The C++ wrapper already links against full wxWidgets headers, so **any** wxWidgets API is reachable from the
   wrapper without touching wxWidgets itself.
5. `WXWIDGETS_DIR` (`build.rs:61-68`) lets a fork point at a patched wxWidgets source tree if that were ever
   needed, without touching build.rs.

Since `wxAccessible` is **already** bound (Q3), a fork is almost certainly unnecessary. If one were ever needed
— e.g. to expose `wxAccessible` per-cell on a control, or to add `SetHelpText` — a patch upstream is the better
first move given that PRs #150/#155/#158 were all merged within days to weeks.

---

## Open UNKNOWNs

These are not answerable from source and must not be guessed at.

1. **UNKNOWN — needs a spike: does the generated `setup.h` actually contain `#define wxUSE_ACCESSIBILITY 1`?**
   The derivation in Q2 is sound across four files but the artifact has not been observed, and the failure mode
   is silent. **Spike:** after the first successful `cargo build` for `x86_64-pc-windows-msvc`, grep
   `wxUSE_ACCESSIBILITY` in the generated `setup.h` under the wxWidgets CMake build directory, and grep
   `wxUSE_ACCESSIBILITY` in that build's `CMakeCache.txt`. Two greps, no code. Attach to the build/toolchain
   ticket.
2. **UNKNOWN — needs a spike (HITL): does NVDA announce anything sent via `NotifyWinEvent` from a wxdragon
   window, and which `EVENT_*` / `OBJID_*` / role combination works?** MSAA has several candidate idioms
   (`EVENT_SYSTEM_ALERT` on an alert-role object, `EVENT_OBJECT_SHOW`, `EVENT_OBJECT_NAMECHANGE` on a
   status bar, `EVENT_OBJECT_LIVEREGIONCHANGED` which NVDA associates primarily with UIA/IA2 rather than
   plain MSAA). Source cannot settle which NVDA acts on. **Spike:** the announcement prototype ticket, verified
   by the user with real NVDA per map constraint #5. Try, in order: (a) `set_accessibility_role(AccRole::Alert)`
   + `Accessible::notify_event(EVENT_SYSTEM_ALERT, …, AccObjectType::Alert, 0)`; (b) direct
   `NotifyWinEvent(EVENT_OBJECT_NAMECHANGE, …, OBJID_CLIENT, CHILDID_SELF)` on the StatusBar HWND after
   `set_label`; (c) `EVENT_OBJECT_LIVEREGIONCHANGED`. Record which one NVDA speaks.
3. **UNKNOWN — needs a spike (HITL): does attaching a `WxdSimpleAccessible` to a `ListCtrl` degrade NVDA's
   native reading of its rows and columns?** The `wxACC_NOT_IMPLEMENTED` → `CreateStdAccessibleObject` fallback
   says it should not, but that is a code-reading, not an observation, and `WM_GETOBJECT` interception is a
   real change of plumbing. **Spike:** in the ListCtrl prototype, read the same list under NVDA with and
   without a `set_accessibility_label` call on the ListCtrl itself, and compare.
4. **UNKNOWN — not investigated: build cost and exe size.** Building wxWidgets 3.3.3 statically from source has
   consequences for cold-build time, CI time, and final binary size (NFR-exe-size ≤ 40 MB). Belongs to the
   build/packaging ticket; flagged here only because the "no prebuilt binaries" finding changes its inputs.
5. **Not investigated (out of scope):** UIA. wxdragon binds no UIA whatsoever — grepping `UIA`, `IAccessible`,
   `UIAutomation` across both crates returns nothing beyond the MSAA names above. Any UIA work would go
   directly through the `windows` crate on a raw HWND, entirely outside wxdragon.

---

## Consequences

### (a) Putting entry status in a ListCtrl column — **safe, and the right design**

`ListCtrl` supports report mode and real columns: `src\widgets\list_ctrl.rs:39` (`Report: ffi::WXD_LC_REPORT,
"Multicolumn report view (detail view)"`), `:311` `insert_column(col, heading, format, width)`, `:352`
`insert_item`, `:390` `set_item_text_by_column(index, col, text)`. On wxMSW that is a real `SysListView32`
whose comctl32 `IAccessible` exposes column text, and — confirmed at `include/wx/window.h:1582` plus the
absence of any `CreateAccessible` override in wxWidgets' listctrl headers — **no wx accessible is interposed
unless you install one.** The map's standing fact holds. Put the diagnostic status in its own column, give the
column a clear heading, and NVDA reads it as part of the row with no accessibility API involved. Do not call
`set_accessibility_*` on the ListCtrl unless a prototype shows a specific need — every such call moves that
widget onto the wx-mediated `WM_GETOBJECT` path for no gain here.

### (b) Announcing transient messages — **two independent routes are open; which one NVDA honours is still unknown**

The mechanism is not in doubt; only NVDA's response is.

- **Route 1, in-toolkit:** `Accessible::notify_event(event_type: u32, &widget, AccObjectType, i32)`
  (`src\accessible.rs:333-337`). It reaches `wxAccessible::NotifyEvent`
  (`wxWidgets src/msw/ole/access.cpp:1836-1844`), which resolves `window->GetHWND()` and calls
  `::NotifyWinEvent(eventType, hwnd, idObject, idChild)` (`:1823-1826`) — but **deferred via
  `wxTheApp->CallAfter`**, deliberately, so the notification lands after the wx-side change completes. Note
  wxdragon exports no `EVENT_*` constants, so you supply the raw `u32` yourself (take them from the `windows`
  crate rather than hardcoding). `AccObjectType`'s discriminants were verified to match `wxAccObject` exactly
  (`cpp\include\core\wxd_accessible.h:41-54` vs `wxWidgets include/wx/access.h:120-133`, e.g. `ALERT =
  0xFFFFFFF6` on both sides) — no off-by-one hazard.
- **Route 2, direct:** `HWND(widget.get_handle())` into `windows::Win32::UI::Accessibility::NotifyWinEvent`.
  Synchronous, fully under your control, and — importantly — **independent of `wxUSE_ACCESSIBILITY`**, since it
  never enters wx code. This is the fallback if Q2's residual risk ever materialises.

Route 2 is the safer default for PathMaster: it needs nothing from wxdragon beyond `get_handle()`, it has no
hidden `CallAfter` timing, and it survives even a wxWidgets built without accessibility. Route 1 is more
idiomatic and worth trying first in the prototype. Either way, **which event/objid/role combination NVDA
actually speaks is UNKNOWN #2 and belongs to the HITL prototype ticket.**

### (c) Forking wxdragon to bind `wxAccessible` — **unnecessary; it is already bound**

The ticket's contingency plan is moot. `wxAccessible` is bound end to end as of 0.9.18: the full
`AccessibleImpl` callback interface, `Accessible`, `NotifyEvent`, `SetAccessible`, and five convenience
property setters, all Windows-gated, all backed by MSAA enums that were audited and corrected upstream in
2026-06/07 by an NVDA developer. Pin `wxdragon >= 0.9.17` (for the `AccRole` ordering fix; prefer 0.9.18).

Were a fork ever needed anyway, it is cheap: MIT/Apache-2.0, bindgen runs at build time so there is no
generated-code checked in to maintain, and adding a binding is a header line + a `.cpp` body + a safe wrapper.
The realistic maintenance burden of a fork is not the accessibility code — it is tracking a fast-moving,
pre-1.0, largely single-maintainer upstream. Given that the three accessibility PRs to date were all merged
promptly, **upstreaming beats forking.**

**The one thing to carry out of this document:** wxdragon's accessibility support is real, recent, and
better than the ticket assumed — and it is also **entirely unverified against NVDA**, and fails *silently*
if `wxUSE_ACCESSIBILITY` is ever 0. The spec must not treat any of it as proven until the HITL prototype
tickets return a verdict.
