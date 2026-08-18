# Live announcement mechanism

Type: prototype
Status: open
Blocked by: 01, 02

## Question

How do transient messages reach NVDA — "PATH refreshed", "Copied to clipboard", "Settings file was corrupted
and has been reset to defaults", the PATH-length warning banner?

None of these is tied to a focus change, so native controls will not speak them on their own. This is the one
accessibility gap the free ride does not cover.

Candidate rungs, in the order the charting session agreed to prefer them:

1. **Design it away.** Deliver the message somewhere the user is already going: the label of the control that
   now has focus, a dialog they opened, or a status the app moves focus to deliberately. Costs nothing and
   cannot regress.
2. **`windows` crate on the widget's `HWND`** (available only if ticket 01 found a handle):
   `NotifyWinEvent` with `EVENT_OBJECT_NAMECHANGE` or `EVENT_OBJECT_LIVEREGIONCHANGED`, or a UIA notification
   event. wxWidgets is untouched.
3. **Patch or fork wxdragon** to bind `wxAccessible` — accepted at charting only as a last resort, and only if
   the prebuilt libraries have `wxUSE_ACCESSIBILITY=1`.

Build the cheapest artifact that can test rungs 1 and 2, and have the user confirm with real NVDA which one
actually speaks — in both focus and browse mode, and while focus sits in the list. Record the spoken text
verbatim, and record what stays silent.

The answer must end with a single rule the whole app can follow, not a menu of options.

Findings → `../research/08-announcements.md`.

## Carried in from ticket 03

- A **fourth rung appeared, ahead of rung 3**: wxdragon binds `Accessible::notify_event(...)` and a full
  `AccessibleImpl` provider in the safe API (`wxdragon/src/accessible.rs:226`, `:333`), ungated. If the
  vendored wxWidgets was built with `wxUSE_ACCESSIBILITY=1` (ticket 01), forking is likely unnecessary —
  test this rung before the raw-`HWND` route.
- **The banner is now a known problem case.** `wxInfoBar` does not exist in the binding, so the PRD's
  InlineAlert/InlineBanner become a hand-built `Panel` that is shown and hidden. A shown Panel is not a live
  region and announces nothing by itself, yet FR-diag-length and US-admin-elevation both require it to be
  announced. Make the banner one of the cases this ticket tests against real NVDA.

## Carried in from ticket 01

- **Rung 2 is open**: `WxWidget::get_handle() -> *mut c_void` is a *trait* method, so every widget has an
  `HWND`, and the `windows` crate's `HWND` is `#[repr(transparent)]` over the same pointer — the wrap is
  zero-cost.
- **Rung 3 is off the table**: wxdragon already binds `wxAccessible` in full, so no fork is needed.
- **But prefer rung 2 to the wx route on reliability grounds.** Every accessibility entry point in wxdragon's C
  layer is wrapped in `#if wxUSE_ACCESSIBILITY` with **silent no-op `#else` branches**. ~~The flag is derived ON
  but has never been observed in a built `setup.h`.~~ **Superseded by ticket 04 — it is 1; see below.** Calling `NotifyWinEvent` directly never
  enters wx code at all, so it survives even a wxWidgets built without accessibility.

## Carried in from ticket 04

**The reliability argument for preferring the raw-`HWND` route over the wx route has lost its
foundation.** Ticket 01 flagged that every accessibility entry point in wxdragon's C layer sits behind
`#if wxUSE_ACCESSIBILITY` with silent no-op `#else` branches, and that the flag had never been
observed in a real build — so a 0 there would fail invisibly. **It is 1.** Confirmed in the generated
`setup.h` the build actually compiled against
(`target/release/wxdragon_sys_cmake_build/lib/vc_x64_lib/mswu/wx/setup.h:518-520`, `1` in **both**
branches of the `#ifdef __WXMSW__`), and re-confirmed in a separately produced `crt-static` build.
Those `#else` branches are dead code. Rung 4 (`Accessible::notify_event`) is therefore live, and
should be tested on its merits rather than distrusted on this ground.

**Independent corroboration from the linked binary, not from a header.** `OLEACC.dll` is in the exe's
import table in **both** CRT modes, and the running process also loads `uiautomationcore.dll` — so the
MSAA/UIA layer is genuinely compiled in and reached, not merely configured. While ticket 04 was
measuring, NVDA 2025.3.3 was also seen injecting `nvdaHelperRemote.dll`, `IAccessible2Proxy.dll` and
`ISimpleDOM.dll` into the prototype process: NVDA actively hooks this build.

**The banner must not set a background colour** — already noted from ticket 03, repeated here because
ticket 17 (window layout and iconography) now owns the banner's visual design and is the place a
hardcoded colour would slip in.
