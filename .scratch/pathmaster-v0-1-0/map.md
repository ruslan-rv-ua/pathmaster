# Map: PathMaster v0.1.0

Wayfinder map. Tickets are the files in `issues/`; the frontier is every ticket that is `open`, unclaimed,
and whose `Blocked by` list is fully `resolved`.

## Destination

A **locked, technically de-risked specification for PathMaster v0.1.0** — `spec.md` in this directory —
in which every decision needed to start building is settled, and every mechanism the product's
accessibility depends on has been proven against real NVDA rather than assumed.

Reaching it means: no open question stands between the spec and an implementation effort.
Building v0.1.0 is **not** part of this map; prototypes here exist to kill or confirm a decision and are thrown away.

## Notes

**Domain.** A portable Windows desktop app that reads, edits and diagnoses the `PATH` environment variable.
Source PRD (Ukrainian, verbatim, input only): [spec-input.md](spec-input.md). Where the PRD and a resolved
ticket disagree, the ticket wins.

**Stack (fixed, not up for reconsideration).** Rust + [wxdragon](https://github.com/AllenDang/wxdragon).
The user ruled this a hard constraint at charting. The accessibility question is therefore *how* to get
NVDA working inside wxdragon, never *whether* to switch toolkits.

**Settled at charting** (standing constraints for every session; not ticket decisions, so not listed below):

1. Destination is a spec, not a build. Wayfinder's plan-don't-do default holds.
2. Stack is a hard constraint (above).
3. NFR priority when they collide: **accessibility > portability (single exe, no runtime install) > exe size**.
   NFR-exe-size stays 🟡 and its budget is relaxed from 20 MB to **≤ 40 MB**.
4. v0.1.0 scope = **🔴 must only**, plus StatusBar, `settings.json`, and minimal logging.
   All other 🟡 features are deferred to v0.2.0 (see Out of scope).
5. NVDA verification is done **by the user personally** — prototype tickets are HITL and end in a real verdict,
   never in an inspector-tool guess.
6. i18n: language change takes effect **after restart**; `maxBackups` applies immediately.
7. Similar-path / typo detection is **cut** from v0.1.0; five diagnostic types remain.
8. The `theme` setting is **removed** — system colours always, High Contrast is a Windows mode, not an app choice.
9. Logging stays in v0.1.0, minimal.
10. Distribution: GitHub Releases + GitHub Actions, **unsigned** in v0.1.0 (SmartScreen accepted, documented in
    the README). Scoop via an **own bucket** with `persist: data`; winget submitted to `microsoft/winget-pkgs`
    as `InstallerType: portable`.
11. Portability: `data/` sits next to the exe; NFR-no-registry-writes is reworded from "nothing in AppData"
    to "**nothing outside the app's own directory**".
12. All artifacts (map, tickets, research, spec) are written in **English**; conversation with the user is Ukrainian.

**Skills every session should consult.** `/grilling` and `/domain-modeling` for every grilling ticket;
`/research` for research tickets; `/prototype` for prototype tickets. Domain terms resolved by a ticket go
into `CONTEXT.md` at the repo root; a decision that is hard to reverse, surprising, and a genuine trade-off
earns an ADR under `docs/adr/`.

**A fact worth carrying into every session.** wxMSW does not draw its own controls — it wraps native Win32
comctl32 ones. A `wxListCtrl` *is* a `SysListView32`, a notebook *is* a `SysTabControl32`. NVDA therefore reads
them for free, including column text. That makes "announce the issue type" a **design** problem (put status in a
real column) rather than an accessibility-API problem. What native controls will *not* do for free is announce
transient, non-focus messages — which is why that has its own ticket.

## Decisions so far

<!-- one line per resolved ticket: gist + link. Charting-time constraints live in Notes, not here. -->

_(nothing resolved yet)_

## Not yet specified

In scope, but not yet sharp enough to ticket. Graduates as the frontier advances.

- **Error and failure taxonomy** — what the user is shown, what is logged, and what is announced when a registry
  write, a backup write, or a settings load fails. Waits on the registry, elevation and backup tickets.
- **Test and verification strategy** — how the accessibility contract is regression-tested by one person before
  each release, and what is worth automating at all. Waits on the accessibility contract.
- **Visual design and layout** — window layout, minimum size behaviour, DPI scaling, icons, and what "status
  icon" means once colour cannot carry meaning. Waits on the widget inventory.
- **Log format and rotation details** — waits on the failure taxonomy.
- **README and user-facing docs** — including the honest description of what winget/scoop themselves write to
  the machine. Waits on the packaging ticket.
- **Repository and crate layout for the implementation effort** — module seams, what is a library vs the GUI
  shell. Deliberately last: it is shaped by every decision above.

## Out of scope

Ruled beyond this destination. Does not graduate.

- **Building v0.1.0.** This map produces the spec; implementation is a separate effort.
- **All 🟡 should features**, deferred to v0.2.0: Drag & Drop reorder, `%VAR%` expansion toggle, Search bar,
  Filter bar, Tree View browser, Fix Issues dialog, Ctrl+C copy entry.
- **Similar-path / typo diagnostics** — cut at charting; a false-positive generator (`C:\Python312` vs
  `C:\Python313` are both legitimate) and trust in the diagnostics matters more than their breadth.
- **The `theme` setting** — cut at charting; system colours always.
- **Code signing** — deferred until there are real users; v0.1.0 ships unsigned by decision, not by oversight.
- **Everything in PRD §10**: other environment variables, cross-machine sync, plugins, web/CLI front ends,
  auto-update.
- **Non-Windows platforms, 32-bit Windows, and screen readers other than NVDA** (JAWS/Narrator must not be
  deliberately broken, but are not targeted or tested).
