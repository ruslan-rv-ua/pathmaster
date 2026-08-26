# Research: drag & drop list reorder — accessibility and implementation facts

Supporting ticket [10-dnd-reorder-right-to-die](../issues/10-dnd-reorder-right-to-die.md).
Researched 2026-08-26, per the map's standing directive 7 (research before grilling).

## 1. The standards direction of obligation (WCAG 2.5.7)

[WCAG 2.2 SC 2.5.7 "Dragging Movements"](https://www.w3.org/WAI/WCAG22/Understanding/dragging-movements.html)
(Level AA) requires that anything operable by dragging also be operable by a single pointer
*without* dragging — e.g. Move Up / Move Down buttons. The obligation runs one way only:
**drag requires a non-drag alternative; a non-drag UI never requires a drag.** PathMaster already
ships the alternative (Move Up / Move Down buttons + `Alt+Up`/`Alt+Down`, spec v0.1.0 §15). Adding
D&D adds no compliance; omitting it costs none.

## 2. What accessible D&D costs the teams that tried (GitHub, web)

GitHub's engineering write-up on their accessible sortable list
([Exploring the challenges…](https://github.blog/engineering/user-experience/exploring-the-challenges-in-creating-an-accessible-sortable-list-drag-and-drop/))
— and that is on the *web*, with the full ARIA toolbox available — found:

- NVDA turns Enter/Space into simulated *mouse* events, so the app cannot tell an NVDA keyboard
  user from a mouse user; they needed `role="application"` applied "narrowly and conditionally".
- Movement announcements lagged and went stale; they added a 100 ms debounce over
  `aria-live="assertive"`.
- Screen-reader users mostly had *no prior experience* of working drag-and-drop, because almost no
  implementation is accessible — the pattern itself is unfamiliar.
- Their conclusion: ship a **separate move dialog** as the accessible path, validate with daily
  screen-reader users. The dialog turned out to be preferred by some sighted users too.

I.e. the industry's best effort at accessible D&D converges on "provide the non-drag mechanism and
treat the drag as a sighted-mouse extra" — which is the position PathMaster is already in.

## 3. What NVDA can and cannot hear during a native drag

NVDA learns about drags through the **UIA Drag / DropTarget control patterns** (`IsGrabbed`,
drop-target states) — see
[UIA drag-and-drop support](https://learn.microsoft.com/en-us/windows/win32/winauto/ui-automation-support-for-drag-and-drop)
and [nvaccess/nvda#14081](https://github.com/nvaccess/nvda/issues/14081) (milestone 2022.4; the
Windows 11 virtual-desktop / Start-menu cases). Those patterns exist where the *app* implements a
UIA provider that exposes them (UWP/WinUI surfaces do).

A hand-rolled reorder drag in a native `SysListView32` — mouse capture + motion tracking inside
wx — exposes **no such pattern**. MSAA/UIA sees selection change at the end, nothing during.
**The drag would be entirely silent to NVDA**: no grab, no target, no drop announcement, and no
hatch short of writing a custom UIA provider (far beyond the raw-`LVM_*` hatch of ticket 01).
For NVDA users wanting to *perform* drags elsewhere there is a community
[DragAndDrop add-on](https://nvda-addons.org/addon.php?id=8) — user-side emulation, not something
an app can rely on.

Consequence: D&D here can only ever be a **redundant, mouse-only** gesture. That is acceptable
*only because* Move Up/Down already carries the function — but it also means the feature buys
nothing for the app's first-class user.

## 4. Implementation shape in wx / wxdragon

Ticket 01 (resolved) established: `LIST_BEGIN_DRAG` is bound in wxdragon 0.9.18; **no reorder-drag
helper exists anywhere** in wx or wxdragon. wxWidgets forum threads on ListCtrl row reorder
(2005–2012, still the state of the art —
[e.g.](https://forums.wxwidgets.org/viewtopic.php?t=1046)) all hand-roll it:

- `EVT_LIST_BEGIN_DRAG` → capture mouse → track motion → `HitTest()` for the target row →
  draw a drop indicator → on mouse-up, delete + reinsert rows.
- Known pitfalls each needing code: auto-scroll at list edges, multi-select drags, flicker,
  drop above-vs-below disambiguation, cancel on Esc/capture-loss.
- The one native nicety, the ListView **insert mark** (`LVM_SETINSERTMARK`), is not bound —
  raw-handle hatch again.

Estimate: a few hundred lines of bespoke mouse choreography in the UI layer, plus Release-Checklist
steps (mouse-only steps — the only ones NVDA verification cannot cover), for a gesture invisible
to the screen reader.

## 5. What comparable products do

- **Windows' own "Edit environment variable" dialog** (the direct competitor surface): Move Up /
  Move Down buttons, **no drag-and-drop**
  ([BetaNews on its introduction](https://betanews.com/article/windows-10-finally-adds-a-new-path-editor/)).
- **PowerToys Environment Variables editor**: supports reordering PATH entries; its docs present
  add/remove/reorder without leading on drag
  ([docs](https://learn.microsoft.com/en-us/previous-versions/windows/powertoys/environment-variables)).

Neither flagship comparable treats D&D as table stakes for a PATH editor.

## 6. Facts already fixed by this map (constraints, not open questions)

- Ticket 03: in a **Filtered View, all reorder — D&D included, if it lives — is disabled**.
  So D&D would work only on the full, unfiltered list.
- v0.1.0 undo law: one user-visible operation → one Checkpoint. A drag from position *i* to *j*
  would be one Checkpoint (Move Up/Down record one each per step today).
- v0.1.0 §20 / README: FR-reorder-dnd was deferred "live in the tracker, **not promised in the
  README**" — the public promise surface never carried it.
