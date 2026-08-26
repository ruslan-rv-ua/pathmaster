# Research: filter bar contract (supports ticket 07)

Web research gathered 2026-08-26, before grilling, per the map's standing directive 7. Structured as
recommendation-per-question with sources; "no direct guidance found" is stated where true. This file
does **not** repeat [04-live-filter-best-practices.md](04-live-filter-best-practices.md) (debounce,
count wording, rebuild behaviour), [05-var-expansion-best-practices.md](05-var-expansion-best-practices.md)
Q2 (menu check-state announcement mechanics), or
[06-search-bar-best-practices.md](06-search-bar-best-practices.md) (persistent-control rationale,
counter home, Thunderbird Quick Filter's bug list); all three are cited rather than restated.

## Q1. Does a severity partition earn its keep?

**Recommendation: no. Every respected severity split is backed by a distinct consequence; PathMaster's
six Issue types all have the same consequence (informational, nothing blocks), so an Error/Warning
partition here would be a filter-button costume, not a diagnosis. Filter on "All / With issues", the
language the product already has — and note that filtering by TYPE, not severity, is well-precedented
if ever wanted.**

- ESLint's two live severities are *defined by* consequence: "warn" reports but doesn't "exit with a
  non-zero status code"; "error" does
  ([Configure Rules](https://eslint.org/docs/latest/use/configure/rules)). Clippy identical: warn
  emits a warning, "with deny the lint will emit an error", exiting with an error code "most useful
  in scripts used in CI/CD" ([Clippy book](https://doc.rust-lang.org/clippy/)). No exit code, no
  split.
- Clippy's *grouping* of its hundreds of lints is by **category**, not severity — Correctness,
  Style, Complexity, Perf, Pedantic, Restriction…
  ([Clippy's Lints](https://doc.rust-lang.org/clippy/lints.html)) — the "six types" model, at scale.
- The strongest precedent *against* inventing gradations: Go ships no warnings at all — "if it's
  worth complaining about, it's worth fixing in the code… warnings… make compilation noisy, masking
  real errors" ([Go FAQ, via search summary](https://benhoyt.com/writings/go-intro/) *(secondary
  relay of go.dev/doc/faq)*). A flat "problem / no problem" model is a deliberate design position in
  a major toolchain, not a gap.
- axe-core's critical/serious/moderate/minor "impact" is explicitly **vendor-assigned, not part of
  WCAG** — its own tracker holds the definition debate
  ([dequelabs/axe-core#2798](https://github.com/dequelabs/axe-core/issues/2798),
  [axe API docs](https://www.deque.com/axe/core-documentation/api-documentation/)). Even the a11y
  field's best-known severity scheme is an overlay someone had to invent and defend.
- The tools whose filter buttons the PRD copies *already had* severity in their data model:
  Visual Studio's Error List tabs are "Errors, Warnings, or Messages" over compiler-supplied classes
  ([Error List window](https://learn.microsoft.com/en-us/visualstudio/ide/error-list-window)); VS
  Code's Problems filter got per-severity checkboxes for the same pre-existing classes
  ([microsoft/vscode#39531](https://github.com/Microsoft/vscode/issues/39531)). No case found of a
  tool minting severity *in order to* filter.
- Filter-by-type precedent: Event Viewer filters by Event ID, Source, and Task Category — arbitrary
  type keys, not just level
  ([Create a Custom View](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2008-R2-and-2008/cc709635(v=ws.10)));
  VS Code's filter input matches text including rule codes
  ([#51103](https://github.com/microsoft/vscode/issues/51103) asks to extend exactly that). A
  future per-type filter (`Missing`, `Duplicate`, …) has precedent; severity does not need to exist
  for it.

## Q2. Control material for a small exclusive filter

**Recommendation: wxITEM_RADIO items in the View menu — constant labels, one checked at a time — as
the single home, with no on-window bar. Zero new Tab stops, matches the no-toolbar rule and "every
command has a menu home"; the checked item is the re-visitable state indicator (mechanics per 05's
Q2). If an on-window control is ever added, the NVDA-native choice is a labelled wxChoice, not a
radio row or toggle buttons.**

- NVDA's own Add-on Store — the product's best precedent — uses **no radio rows and no toggle
  buttons anywhere**: the status filter is tabs (`wx.Notebook`, cycled with `ctrl+tab`), and every
  other filter is a labelled `wx.Choice` ("Cha&nnel:", "Ena&bled/disabled:", "Sort by colu&mn:")
  plus one checkbox
  ([storeDialog.py](https://github.com/nvaccess/nvda/blob/master/source/gui/addonStoreGui/controls/storeDialog.py),
  [User Guide, Browsing add-ons](https://download.nvaccess.org/documentation/en/userGuide.html)).
  PathMaster already spends the tab metaphor on Scope, so the Choice/menu options remain.
- Toggle-button rows are the pattern with the complaint trail: Thunderbird's Quick Filter buttons
  went icons-only over objections ([bug 616933](https://bugzilla.mozilla.org/show_bug.cgi?id=616933)),
  keyboard navigation around the bar is a standing a11y grievance (06's citations:
  [bug 587478](https://bugzilla.mozilla.org/show_bug.cgi?id=587478),
  [nvaccess/nvda#17657](https://github.com/nvaccess/nvda/issues/17657)), and NVDA leaves toolbar
  radio-toggles unannounced on state change
  ([nvaccess/nvda#12678](https://github.com/nvaccess/nvda/issues/12678)).
- A radio row's Tab cost is one stop, not one per button: the first control in a group gets
  `WS_TABSTOP|WS_GROUP`, arrows move within, "the system automatically moves the style"
  ([Dialog Box Programming Considerations](https://learn.microsoft.com/en-us/windows/win32/dlgbox/dlgbox-programming-considerations);
  wxMSW applies `WS_GROUP` for `wxRB_GROUP` —
  [wxRadioButton docs](https://docs.wxwidgets.org/3.0/classwx_radio_button.html),
  [msw/radiobut.cpp](https://github.com/wxWidgets/wxWidgets/blob/master/src/msw/radiobut.cpp)). So
  the Tab-order objection to a row is small — but real, and the menu option's is zero.
- NVDA's "x of y" position report for radio groups is host-dependent and has open defects even in
  Microsoft's own apps ([#12761](https://github.com/nvaccess/nvda/issues/12761) Win11 File
  Explorer, [#19343](https://github.com/nvaccess/nvda/issues/19343) Chromium) — another reason not
  to build on a radio row without live-NVDA proof.
- Menu radio items: wxMSW radio items are native `HMENU` items whose checked state MSAA exposes as
  `STATE_SYSTEM_CHECKED`, same mechanism 05's Q2 sourced for check items; NVDA's known failures are
  WinForms `ToolStripMenuItem` ([#19281](https://github.com/nvaccess/nvda/issues/19281)) and a web
  ARIA first-item case ([#14550](https://github.com/nvaccess/nvda/issues/14550)) — nothing found
  against native menu radio items. Same verdict as 05: expected to work, needs the usual live-NVDA
  proof.

## Q3. Placement and reaching the filter without tabbing

**Recommendation: with the menu answer to Q2 there is nothing to place — the Tab order stays
tabs → search → list → buttons. Give the state change an accelerator that CYCLES (or direct
per-state items), not one that focuses a control; both accelerator styles have NVDA-store
precedent, and cycling is the one that suits a control-less filter.**

- Where on-window filters do exist, filters-before-search is the observed order: the Add-on Store
  lays out its filter Choices on line 0 and the Search field on line 1, with the status tabs above
  both ([storeDialog.py](https://github.com/nvaccess/nvda/blob/master/source/gui/addonStoreGui/controls/storeDialog.py));
  VS Code instead fuses them — one filter input whose dropdown holds the severity checkboxes
  ([#39531](https://github.com/Microsoft/vscode/issues/39531),
  [#86143](https://github.com/microsoft/vscode/issues/86143)). No source *argues* an order; the
  precedents just put the coarse narrowing before the fine one.
- Both reach-it-without-Tab styles in one dialog: NVDA's store documents `ctrl+tab` to **cycle**
  the status filter and `alt+s` to **focus** the search field
  ([User Guide](https://download.nvaccess.org/documentation/en/userGuide.html)). Windows guidance
  assigns mnemonics to interactive controls in dialogs and shortcut keys to commands
  ([Keyboard guidelines](https://learn.microsoft.com/en-us/windows/win32/uxguide/inter-keyboard));
  a menu radio group is a command surface, so it takes accelerators, and a cycle accelerator is the
  store's `ctrl+tab` translated.
- Mnemonic hygiene if a control ever lands on the window: the store's `&Search`/`Cha&nnel` shows
  per-control mnemonics, which PathMaster's search field already follows (06 Q1).

## Q4. What the filter change announces; Search+Filter both active

**Recommendation: when the FILTER changes, name the resulting state, not just the count —
"With issues: {n} of {m} entries" — because the count alone cannot distinguish which narrowing
moved. Keystroke-driven search changes keep riding catalogue item 9 unchanged (06 Q5, 04 Q1–Q2 for
debounce and wording). When both are active there is still exactly one announcement — the composed
count — and the persistent "why is this list short" answer lives in the StatusBar field ticket 06
already assigned.**

- Naming the region/state in the announcement is sourced: when a live region has an accessible
  name, "screen readers include the name of the region in the announcement" — context, not just a
  number ([Level Access, ARIA labels guide](https://www.levelaccess.com/blog/aria-labels-and-accessible-names-a-developers-guide/)
  *(secondary)*; result-state phrasing per WCAG 4.1.3's examples, cited in 04 Q2/05 Q2).
- VS Code keeps the evidence of narrowing visible and *keeps the total*: the filter badge reads
  "4 out of 10", and hiding the total when everything is filtered out was treated as a bug —
  "Still show total count of problems even when all problems are filtered out"
  ([microsoft/vscode#86134](https://github.com/microsoft/vscode/issues/86134),
  [#86143](https://github.com/microsoft/vscode/issues/86143)). "{n} of {m}" with m retained is the
  same defence.
- The stranded-user failure is documented for filters specifically, not just hidden bars: a
  Thunderbird user with the sticky filter accidentally on saw "No message found" in **every**
  folder until the button was found and cleared
  ([support thread](https://support.mozilla.org/en-US/questions/1501861); 06 cites the adjacent
  hidden-bar thread). The mitigation PathMaster already owns is the two-part rule from the ticket:
  Announcement 1 (the healthy Scope announcement) speaks **only** when query is empty AND the
  filter is unnarrowed, so a narrowed list can never introduce itself as whole.
- How the peers expose "a filter is active" to screen readers: VS Code marks it visually (funnel
  badge + count) with an accessibility-help overlay for the filter widget still being designed
  ([microsoft/vscode#292367](https://github.com/microsoft/vscode/issues/292367)); no Thunderbird
  source found that announces active-filter state at all — its absence is the complaint trail
  above. No direct guidance found beyond "name the state"; the composed announcement is judgement
  built on 4.1.3's phrasing.

## Q5. Does filter state survive the run?

**Recommendation: reset per Run, like the search text (06 Q7). The precedents that persist
view-narrowing state either demand an explicit named save or generate stranded-user reports; the
ones that reset are unremarked-on.**

- Event Viewer is the cleanest statement of the principle: "Filter Current Log" is temporary and
  dies with the console; persistence requires deliberately naming and saving a **Custom View**
  ([Create a Custom View](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2008-R2-and-2008/cc709635(v=ws.10)),
  [Custom Views](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2008-R2-and-2008/cc766522(v=ws.10))).
  Session-scoped by default; persistence is an explicit, named act.
- VS Code's Problems filter shows both failure modes at once: users must re-ask for persistence as
  a feature ([#95484](https://github.com/microsoft/vscode/issues/95484),
  [#172648](https://github.com/microsoft/vscode/issues/172648) — still open requests), while the
  part that *did* persist produced its own bug ("Clearing Problems panel filter not persisted",
  [#131769](https://github.com/microsoft/vscode/issues/131769)). Whichever way it goes, half-way
  persistence is the worst position.
- Thunderbird's sticky pin is the opt-in shape: a dedicated "Keep filters applied when switching
  folders" button ([Quick Filter Toolbar](https://support.mozilla.org/en-US/kb/quick-filter-toolbar)),
  and even its within-session memory is buggy
  ([bug 1850266](https://bugzilla.mozilla.org/show_bug.cgi?id=1850266)) and strands users when on
  by accident (the Q4 support thread). No source stating its default was found; the KB presents it
  as an action the user takes, and the accident reports only make sense if off is normal.
- NVDA's Add-on Store re-derives its opening view every time — "Available add-ons" with nothing
  installed, otherwise "Installed add-ons"
  ([User Guide](https://download.nvaccess.org/documentation/en/userGuide.html)) — not the last-used
  filter; search text and filters start fresh with the dialog.
- General "view-narrowing state should be session-scoped" guidance: **no direct guidance found —
  judgement**, resting on the pattern above plus two local facts: the Search it composes with dies
  with the Run (06 Q7), and 05's Q1 persisted the *display* toggle precisely because that one is a
  projection choice, not a narrowing — the distinction Excel also draws (view options persist;
  filters are data-state the user re-applies knowingly). No `settings.json` field, so no new
  domain to validate.
