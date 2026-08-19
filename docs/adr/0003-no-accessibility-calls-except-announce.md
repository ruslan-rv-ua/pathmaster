# The application makes no accessibility API calls except `announce()`

PathMaster is built for a screen-reader user first, and its accessibility strategy is to make **zero**
accessibility API calls — no `set_accessibility_*`, no per-widget labels, no events — except a single
`announce(text)` function. That sounds backwards, so the reasoning is recorded.

**The free path is measured and wide.** wxMSW wraps native comctl32 controls, and NVDA reads them
natively: list rows with both columns and the column header name, tabs, buttons with roles and access
keys, menus with accelerators and disabled state, dialog titles and buttons (ticket 02's baseline).
None of that comes from code in this application — it comes from *not interfering*.

**The first label call is a change of plumbing, not a pure addition.** By default no wx control
overrides `CreateAccessible()`, so `WM_GETOBJECT` goes unhandled and comctl32's own `IAccessible`
serves the control — that is where the free reading comes from. The first `set_accessibility_*` call
moves the widget onto the wx-mediated path, and whether that degrades the native row reading is
unknown (a code reading suggests a fallback exists; nothing tested it). Given a measured-good state
and an unmeasured alternative, the contract keeps the measured one.

**Everything the PRD wanted from labels is had another way.** The `AccessibleName`-on-everything
requirement is rewritten as: every interactive element has a *visible* text label, read by the native
path. What visible labels cannot carry — transient messages not tied to a focus change — is carried by
`announce()`: set the label of one dedicated message `StaticText`, fire
`NotifyWinEvent(EVENT_OBJECT_LIVEREGIONCHANGED)` on it. That is the one mechanism measured to speak
verbatim, every time, regardless of focus (ticket 08 killed every alternative, including UIA
notifications and the wx event route).

## Consequences

- **Any future `set_accessibility_*` call is a re-measure, not an improvement.** The widget it touches
  must be re-tested against the ticket-02 baseline before the change is believed.
- **Announcements are a closed catalogue.** Because `announce()` is the only voice, adding a message is
  a contract change, not a convenience — the catalogue lives in the accessibility contract (ticket 09)
  and over-announcing is treated as a defect equal to silence.
- **No message is audio-only.** Every announcement also sets the visible Banner text, so sighted and
  screen-reader users receive the same information through one code path.
- **Dialog bodies stay unheard, by discipline.** NVDA speaks a `MessageDialog`'s title and buttons but
  never its body (measured), and the application does not compensate mechanically — instead all
  critical dialog information must be carried by the title and buttons.
- **The status bar carries only what may be missed.** It is command-only under NVDA (`NVDA+End`), and
  no measured event makes it speak — so nothing must-hear is routed there.
