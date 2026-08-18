# Window layout, sizing and iconography

Type: grilling
Status: open
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
