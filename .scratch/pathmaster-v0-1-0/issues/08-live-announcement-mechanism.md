# Live announcement mechanism

Type: prototype
Status: open
Blocked by: —

> **Scope note, 2026-08-18.** This ticket was briefly widened to also own "make list-row focus reach
> the screen reader at all", on the strength of a ticket 02 measurement that turned out to be an
> artefact. Rows are announced fine. The widening is **withdrawn** and the ticket is back to its
> original question.

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

Findings → `../research/08-announcements.md`. Baseline to compare against: `../research/02-nvda-baseline.md`.

**Tooling.** `../tools/nvda-drive.ps1` drives a prototype with synthetic keystrokes and returns the slice of
NVDA's log the run produced, so these passes do not need a human at the keyboard. Its `-Probe` mode reads the
control's own state back, which is what separates "the screen reader said nothing" from "nothing happened".

**The prototype to test against exists:** `../prototypes/02-nvda-baseline/`. It carries the app's real shape.
**Do not add accessibility calls to it** — it is the baseline later measurements are compared against. Copy it.

## Carried in from ticket 02

The baseline is friendlier than expected, which narrows this ticket rather than widening it. Rows, buttons,
tabs, menus, disabled and checked states all speak for free, and the status bar answers `NVDA+End` with both
its fields. Three things landed here:

- **The status bar is command-only.** It is not in the Tab order and `F6` is silent, so a user reaches it
  only by asking for it with `NVDA+End`. If this ticket's answer routes any transient message to the status
  bar, that message is heard **only** by someone who thinks to check. Rung 1 ("design it away") should not
  quietly mean "put it in the status bar".
- **An empty list announces only `['список']`** — no count, no "порожньо". If a message needs to say "this
  list is now empty", nothing says it on its own.
- **Menu accelerators are spoken only because `\t` puts them inside the label** (`Apply\tCtrl+S` is one
  string and NVDA reads the string). Any announcement work that rewrites labels must not drop the `\t`.

Also carried: **verify NVDA is in a sane state before trusting a pass.** Ticket 02 lost a measurement to a
~7-minute window in which NVDA treated the list as a leaf and announced nothing (now ticket 18). The cheap
check is `NVDA+Tab` on a list row — it must answer `'елемент списку'`, not `'список'`.

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
