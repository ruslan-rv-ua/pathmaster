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
  layer is wrapped in `#if wxUSE_ACCESSIBILITY` with **silent no-op `#else` branches**. The flag is derived ON
  but has never been observed in a built `setup.h` (ticket 04 checks). Calling `NotifyWinEvent` directly never
  enters wx code at all, so it survives even a wxWidgets built without accessibility.
