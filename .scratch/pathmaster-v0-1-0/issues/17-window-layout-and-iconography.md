# Window layout, sizing and iconography

Type: grilling
Status: resolved
Blocked by: —

## Question

What does the window actually look like, how does it behave when resized and rescaled, and where do
its icons come from?

Graduated out of the map's **Not yet specified** on 2026-08-18. It waited on the widget inventory
(ticket 03, resolved), and ticket 04 has now settled the icon and DPI mechanics — so the remaining
questions are decisions rather than unknowns.

- **Layout.** Where the notebook, the lists, the buttons, the hand-built banner (ticket 08's rung 1)
  and the status bar sit relative to each other; which sizer structure expresses it. Ticket 02's
  throwaway shell is a first sketch of this, not a decision — react to it rather than inheriting it.
- **Sizing.** NFR-window-sizing wants 800×600 minimum. What is the *default* size on first run, what
  happens between 800×600 and a maximised 4K window, and which parts stretch. Does window geometry
  persist between runs, and if so where — `settings.json` is the only candidate, and portability
  (map decision 11) constrains it.
- **DPI.** `PerMonitorV2` is already asserted by the embedded manifest, so the app opts into
  per-monitor scaling from the first release. Combined with ticket 03's finding that `FromDIP` is
  applied **implicitly and invisibly** at the FFI boundary and can be neither queried nor bypassed,
  **DPI behaviour is currently decided by default rather than by choice**. Decide whether that
  default is accepted, and what "correct at 100 %, 150 % and 200 %" means for the column widths
  which are the one place this app hardcodes pixels.
- **The application icon.** Ticket 04 demonstrated the exe-resource route end to end (`llvm-rc`, no
  new crate, bit-identical extraction). Two things remain: what the icon *is*, and the fact that the
  **running window still has no icon** — `WM_GETICON` and the class icon both return 0, because wxMSW
  does not adopt the exe's resource for the frame. That needs an explicit `Frame::set_icon()` at
  startup, and since `Bitmap` has no PNG loader it means `Bitmap::from_rgba` over embedded RGBA or
  `BitmapBundle::from_svg_data`. Decide which, and note that one embedded SVG through
  `BitmapBundle::from_svg_data` covers every DPI from a single asset.
- **In-app iconography.** Whether there is any beyond the window icon — the PRD's toolbar is 🟡 and
  out of scope, and `ArtProvider` covers the banner's warning icon. Keep this small or rule it out.
- **Colour.** Nothing to decide, but state it: ticket 03 established there is no way to read a system
  colour, so the app must **never set one** and let native controls inherit the system theme. This
  ticket must not quietly reintroduce a hardcoded colour in a layout decision — the hand-built banner
  is the tempting place.

## Not this ticket

**What is announced, and what status text says, belong to ticket 09.** Ticket 03 already established
that a ListCtrl sub-item cannot carry an icon, so Status is text-only; that decision and the
verbosity policy around it are the accessibility contract's, not this ticket's. Keep the boundary:
this ticket decides what is *seen*, ticket 09 decides what is *said*.

## Carried in from ticket 13

The StatusBar gains a **passive merged-PATH-length field** (ticket 13, D6/FR-diag-overlength): the
current length of `expand(System) + ";" + expand(User)` in characters, always visible, queried via
`NVDA+End`, never announced. This ticket owns where that field sits among the StatusBar's fields
and how it reads. Diagnostics claims no Banner use and no colours — the Status column is text-only
and the over-length warning is an Apply-time dialog, not a layout element.

## Carried in from ticket 08

The banner's announcement mechanism is settled and imposes exactly one structural requirement: the
banner contains a **dedicated message `StaticText`** that the app-wide `announce(text)` function
owns (label set + `EVENT_OBJECT_LIVEREGIONCHANGED` fired on its HWND — see
[research/08](../research/08-announcements.md)). Nothing else about the banner's design is
constrained by accessibility: it needs no focusability, no role, no wx accessibility calls — and the
standing rule stands, **no background colour**.

## Answer

Resolved 2026-08-19 by a grilling session, grounded in a prior web-research pass (wx high-DPI docs,
Windows icon guidance, window-placement persistence practice). Ten decisions:

**D1 — Layout.** One vertical `wxBoxSizer` on the frame: the **Banner above the notebook**, the
notebook at `proportion=1, wxEXPAND` taking all remaining space, and the native status bar attached
to the frame outside the sizer (frame client size already excludes it). The Banner is **always
visible with a fixed height**, its `StaticText` empty at rest — a stable layout that never reflows
the list under the user on Refresh/Apply, and consistent with `announce()` reusing one persistent
`StaticText` (ticket 08's structural requirement).

**D2 — Default size and stretch.** First-run size **900×650 DIP**; minimum 800×600 per
NFR-window-sizing. On resize the list fills its tab in both directions; the **Status column has a
fixed width** (the one deliberate pixel constant, converted via `FromDIP`), and the **Path column
takes all remaining width**. Rationale: Status text is of predictable length (comma-joined one-word
Issue types, ticket 13), while paths are unbounded.

**D3 — Geometry persistence.** Window position, size and maximised state persist in
`settings.json`, written on clean shutdown only. On restore the saved rectangle is **clamped against
the currently connected monitors' work area**; if it lies fully outside the visible area (monitor
unplugged since last run), fall back to the default size centred on the primary monitor — never
"nearest visible point". This is the standard mitigation for the most common persistence failure
(topology change between runs reopening the window off-screen).

**D4 — DPI default accepted.** The implicit, unqueryable `FromDIP` at the wxdragon FFI boundary is
accepted as a structural decision — there is no alternative within the fixed stack. The app adds
exactly **one explicit `FromDIP()` call: the Status column width**. "Correct at 100/150/200 %" means:
native controls scale themselves (comctl32 + PerMonitorV2), and the only hardcoded pixel value goes
through the conversion.

**D5 — Cross-monitor DPI-change risk accepted, not verified here.** wx documents that dragging a
window between monitors with different scale factors has been layout-destructive in some versions.
Accepted as a documented risk in the spec; **one checklist line goes to ticket 19** ("drag the window
between monitors with different scaling — layout must survive") rather than blocking this ticket on
hardware. Recorded as a comment on ticket 19.

**D6 — Icon design: a stylised path** (user's call) — a path/route motif, not a letterform.

**D7 — Frame icon wiring: one embedded SVG via `BitmapBundle::from_svg_data`** passed to
`Frame::set_icon()` at startup — a single asset covers every DPI. This is a **separate asset and a
separate job** from the exe's `.ico` resource (ticket 04's `llvm-rc` route; 16/24/32/48/256 px with
the 256 layer PNG-compressed, per Microsoft's guidance), which serves Explorer/taskbar surfaces.
Both derive from the same stylised-path source design.

**D8 — No in-app iconography beyond the window icon.** The Banner is **purely textual** — no
`ArtProvider` warning icon. Ticket 09's catalogue is closed at seven text messages with no
colour/graphic dependency; a banner icon would be a second visual language maintained apart from
what NVDA speaks, with no clear benefit.

**D9 — Colour, restated as settled.** The app never sets a colour anywhere — the Banner included.
Native controls inherit the system theme; High Contrast works because nothing punches through it.

**D10 — StatusBar field placement** (carried in from ticket 13). Two fields: **field 0 (left) is
the general status field** ticket 02 measured, **field 1 (right) is the passive merged-PATH-length
field** — metrics right, messages left, the standard Windows convention. The over-limit state rides
the field's text (ticket 09: the status bar cannot highlight a field), with exact wording owned by
the spec ticket alongside ticket 13's thresholds.
