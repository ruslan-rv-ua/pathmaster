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
