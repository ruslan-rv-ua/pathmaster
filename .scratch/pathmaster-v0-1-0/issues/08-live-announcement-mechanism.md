# Live announcement mechanism

Type: prototype
Status: open
Blocked by: —

> **Scope widened 2026-08-18 by ticket 02's measurement.** This ticket was written for messages *not*
> tied to a focus change. It now also owns a prior and larger case: the focus-tied announcement that
> was assumed free and is not. Part A below must be settled first — until the base case speaks, no
> strategy for the rest is testable.

## Question A — why does a focused list row say nothing, and what makes it speak?

Ticket 02 measured a stock wxdragon `wxListCtrl` in report mode with no accessibility code:

- Arrowing moved the focused row 0 → 3 in an 11-row list. NVDA spoke **nothing at all**.
- `NVDA+Tab` reports `['список', 'у фокусі', 'з 11 рядків і 2 стовпців']` — the list, never the row.
- MSAA on that list returns 11 `ROLE_SYSTEM_LISTITEM` children with correct names, and `accFocus`
  names the exact row the arrows moved to, with `selected + focused` state set.
- No NVDA error is involved, and NVDA is demonstrably injected into the process (ticket 04 saw
  `nvdaHelperRemote.dll`, `IAccessible2Proxy.dll`, `ISimpleDOM.dll`; ticket 02 saw NVDA's own
  `sysListView32` in-process helper running against this very control).

So the accessible content is present and correct, and the screen reader is present and hooked. **What
is missing is the event that says the focused row changed.**

**First experiment, before choosing any rung.** Hook `SetWinEventHook` out-of-process, filtered to the
prototype's PID, for `EVENT_OBJECT_FOCUS` (0x8005) and `EVENT_OBJECT_SELECTION` (0x8006) — and log
`hwnd`, `idObject`, `idChild` for each. Arrow through the list. Three outcomes, three different fixes:

1. **No events fire.** The control genuinely never announces row changes. Fix: fire them ourselves on
   row change (rung 2 or 4), with the list's `HWND`, `OBJID_CLIENT`, and the 1-based row as `idChild`.
2. **Events fire and NVDA is still silent.** The problem is on the NVDA side — wrong `idObject`/
   `idChild`, or the events arriving from a thread NVDA discounts. Record the exact parameters; the
   fix is to correct them, not to add more events.
3. **Events fire only for some movements** (e.g. mouse but not keyboard). Narrowest fix of the three,
   and it tells us precisely where to hook.

Do not skip this step in favour of "just call `NotifyWinEvent` and see". Which event is missing is the
whole question, and firing a second event on top of a working one produces double-speaking, which is
its own accessibility defect.

**Two questions ticket 02 could not reach, both downstream of this one. They are this ticket's
acceptance criteria, not follow-ups:**

- **Is the Status column read together with the Path?** NVDA reads columns beyond the first itself via
  `LVM_GETITEMTEXT` in its in-process helper, not from `accName` (which carries the Path only). So this
  can only be answered once a row is announced at all. If the answer is no, the Issue text has to reach
  the user some other way and ticket 13 changes shape.
- **Are column headers announced?** Same reason, same timing.

## Question B — how do transient messages reach NVDA?

The original question, unchanged: "PATH refreshed", "Copied to clipboard", "Settings file was corrupted
and has been reset to defaults", the PATH-length warning banner.

None of these is tied to a focus change, so native controls will not speak them on their own.

## Candidate rungs

In the order the charting session agreed to prefer them. Rung 1 applies to B only — a focused row
cannot be designed away.

1. **Design it away.** Deliver the message somewhere the user is already going: the label of the control
   that now has focus, a dialog they opened, or a status the app moves focus to deliberately. Costs
   nothing and cannot regress.
2. **`windows` crate on the widget's `HWND`**: `NotifyWinEvent` with `EVENT_OBJECT_FOCUS` /
   `EVENT_OBJECT_SELECTION` for question A, or `EVENT_OBJECT_NAMECHANGE` /
   `EVENT_OBJECT_LIVEREGIONCHANGED` / a UIA notification event for question B. wxWidgets is untouched.
3. **Patch or fork wxdragon** to bind `wxAccessible` — **off the table**, see "Carried in from ticket 01".
4. **`Accessible::notify_event` in wxdragon's safe API** — see "Carried in from ticket 03" and the
   ticket 04 note that removes the reason to distrust it.

Build the cheapest artifact that can test the live rungs, and confirm with real NVDA which one actually
speaks — in both focus and browse mode, and while focus sits in the list. Record the spoken text
verbatim, and record what stays silent.

**The prototype to test against already exists and must not be replaced:**
`../prototypes/02-nvda-baseline/`. It carries the app's real shape and is the thing the baseline was
measured on, so a fix demonstrated there is a fix measured against a known-silent starting point. Any
accessibility call added to it invalidates it as a baseline — copy it, do not edit it in place.

The answer must end with a single rule the whole app can follow, not a menu of options.

Findings → `../research/08-announcements.md`. Baseline to compare against: `../research/02-nvda-baseline.md`.

## Carried in from ticket 02

Beyond question A, two more silences were measured, and both need a home in this ticket's single rule:

- **The status bar is unreachable and unreadable.** Not in the Tab order, and `NVDA+End` answers
  `['Рядок стану невиявлено']` although the frame really does own an `msctls_statusbar32` child window.
  `F6` is silent too. So the status bar is not currently an information channel at all — decide here
  whether it becomes one, and ticket 09 records the consequence.
- **An empty list announces only `['список']`** — no count, no "порожньо", while NVDA does say
  `'порожньо'` for an empty edit field elsewhere in the same log.

And one thing that must survive: **menu accelerators are spoken only because `\t` puts them inside the
label** (`Apply\tCtrl+S` is one string and NVDA reads the string). Any announcement work that rewrites
labels must not drop the `\t` convention.

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
