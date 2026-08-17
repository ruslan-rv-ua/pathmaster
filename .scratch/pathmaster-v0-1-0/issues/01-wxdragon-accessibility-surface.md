# wxdragon accessibility surface

Type: research
Status: resolved
Blocked by: —

## Question

What accessibility surface does wxdragon actually expose to Rust, and what did its prebuilt wxWidgets
binaries enable? The stack is fixed, so this is a fact-finding job, not a comparison.

Answer each with evidence (file + line, or doc page), and say **"unknown — needs a spike"** where no source
settles it rather than inferring:

- Which wxWidgets version do wxdragon's prebuilt libraries bind, and in which configuration (MSVC vs MinGW,
  release, static vs shared)? Where does `build.rs` decide it?
- Is `wxUSE_ACCESSIBILITY=1` in the prebuilt libraries' `setup.h`? `wxAccessible` is compiled out entirely
  when it is not, and it is **not** the default in every build.
- Does wxdragon bind `wxAccessible` at all? Search the crate API and source for `accessible`, `MSAA`, `UIA`,
  `IAccessible`.
- Does it expose a native window handle (`wxWindow::GetHandle` → `HWND`) for any widget, and in what Rust type?
  Can that value be handed to the `windows` crate? This is the hinge for the announcement ticket.
- Does it bind `SetName`, `SetLabel`, `SetToolTip`, `SetHelpText` — the members that feed the MSAA name and
  description on wxMSW?
- Any issue or PR history in the wxdragon repo about accessibility, screen readers, or `HWND` access?
- Current published version, release cadence, and how a fork would be maintained if one were ever needed.

Findings → `../research/01-wxdragon-accessibility-surface.md`.

## Answer

Full findings: [research/01-wxdragon-accessibility-surface.md](../research/01-wxdragon-accessibility-surface.md).
**The ticket's premise was wrong in two ways, both in our favour.**

**1. There are no prebuilt binaries.** wxdragon 0.9.18 downloads the **wxWidgets 3.3.3 source** (SHA256-pinned)
and compiles it as a CMake subproject — static libraries (`wxBUILD_SHARED OFF`), non-monolithic, MSVC via Ninja
at `RelWithDebInfo` with the release CRT forced even for debug Rust profiles. The prebuilt path existed in
0.8.x and is dead. The crate README still claims wxWidgets 3.2.4 and prebuilt libs — stale; ignore it.

**2. `wxAccessible` is already fully bound.** `#[cfg(target_os = "windows")] pub mod accessible` gives the whole
`AccessibleImpl` callback interface, `Accessible`, `notify_event`, `SetAccessible`, and five
`set_accessibility_*` setters. **A fork is unnecessary** — rung 3 of the ladder is off the table, which removes
the effort's largest risk. It is invisible on docs.rs only because docs.rs builds for Linux.

**`wxUSE_ACCESSIBILITY` is derived ON**, through four files: wxdragon never sets it, so `wx_option` defaults it
ON inside `if(WIN32)`, `#cmakedefine01` writes it into `setup.h`, and the only thing that could kill it
(`!wxUSE_OLE`) does not fire because OLE also defaults ON. **Derived, not observed** — see UNKNOWN 1.

**The `HWND` route is open.** `WxWidget::get_handle() -> *mut c_void` (`window.rs:1605`) is a *trait* method, so
every widget has one, and the `windows` crate's `HWND` is `#[repr(transparent)]` over the same pointer —
`HWND(w.get_handle())` is a zero-cost wrap.

**Pin wxdragon ≥ 0.9.17.** PRs #155 and #158 — the MSAA role-ordering fix and the property setters — were
authored by a **core NVDA developer** (LeonarddeR) and merged mid-2026. Before #155, `AccRole`'s discriminants
were mis-ordered, so an older version reports wrong roles. The crate is MIT OR Apache-2.0, ~monthly releases,
but effectively single-maintainer.

**`SetName`/`SetLabel` are a red herring**: `SetName` feeds `FindWindowByName`, not MSAA, and `SetLabel` only
reaches MSAA through `wxWindowAccessible`, which is not installed by default. The real API is
`set_accessibility_label` / `_description` / `_value`. `SetHelpText` is unbound.

### Correction to the research file's CRT claim

The research file reports the MSVC runtime as `MultiThreadedDLL` (`/MD`, needing the VC++ redistributable),
which would put 🔴 NFR-portable at risk. That is only the **default branch**. `wxdragon-sys/build.rs:365-374`
reads `CARGO_CFG_TARGET_FEATURE` and, when `crt-static` is present, builds wxWidgets with `MultiThreaded`
(`/MT`) instead. **The static-CRT choice propagates into the C++ build**, so a runtime-free exe is a build-flag
decision, not a requirement rewrite. Verified directly in the vendored source. → ticket 04 confirms by building.

### Consequences

- **(a) Status in a ListCtrl column is the right design**, confirmed at source: `wxWindow::CreateAccessible()`
  returns `nullptr` by default and no wx control overrides it, so `WM_GETOBJECT` goes unhandled and comctl32's
  own IAccessible serves the rows. **But** the first `set_accessibility_*` call on a widget flips it onto the
  wx-mediated path — a change of plumbing, not a pure addition. → tickets 02, 09.
- **(b) Announcements via `NotifyWinEvent`/UIA on a raw `HWND` are available** and are arguably preferable to
  the wx route, because they never enter wx code at all. → ticket 08.
- **(c) Forking is unnecessary.** If it ever were, it is cheap — bindgen runs at build time, so there is no
  generated code checked in.

### UNKNOWN — needs a spike

1. **`setup.h` was never observed** — the `wxUSE_ACCESSIBILITY=1` chain is derived from CMake logic, not seen in
   a built artifact. **This is the one that matters**, because every accessibility entry point in the C layer is
   wrapped in `#if wxUSE_ACCESSIBILITY` with **silent no-op `#else` branches**: if the flag were ever 0, the
   Rust API would still compile, still run, and do nothing — no error, no log. Two greps after the first real
   build settle it. → ticket 04 (it builds first).
2. Whether NVDA announces anything via `NotifyWinEvent`, and with which event/objid/role. → ticket 08.
3. Whether attaching an accessible to a ListCtrl degrades its native row reading. The
   `NOT_IMPLEMENTED` → `CreateStdAccessibleObject` fallback suggests not, but that is a code reading. → ticket 09.
