# Map: PathMaster v0.2.0

Wayfinder map. Tickets are the files in `issues/`; the frontier is every ticket that is `open`, unclaimed,
and whose `Blocked by` list is fully `resolved`.

## Destination

A **locked delta-specification for PathMaster v0.2.0** — `spec.md` in this directory — describing only
what v0.2.0 adds or changes on top of the locked v0.1.0 spec
([../pathmaster-v0-1-0/spec.md](../pathmaster-v0-1-0/spec.md)), which is not reopened. Every decision
needed to start building v0.2.0 is settled, and every mechanism its accessibility depends on is proven
against real NVDA rather than assumed.

Reaching it means: no open question stands between the delta-spec and an implementation effort.
Building v0.2.0 is **not** part of this map; prototypes here exist to kill or confirm a decision and are
thrown away.

## Notes

**Domain.** Same product as v0.1.0: a portable Windows desktop app that reads, edits and diagnoses the
`PATH` environment variable, built for a screen-reader user first. Glossary: `CONTEXT.md` at the repo
root; decisions: `docs/adr/`. Where the source PRD
([../pathmaster-v0-1-0/spec-input.md](../pathmaster-v0-1-0/spec-input.md)) and a resolved ticket — from
either effort — disagree, the ticket wins.

**Driver (settled at charting, 2026-08-26).** v0.2.0 is **delivering what was promised**: the seven 🟡
should-features the v0.1.0 charting cut, plus the small items parked beside them. Not field feedback
(v0.1.0 shipped 2026-08-25; there is none yet), not architecture for its own sake.

**In scope**: FR-reorder-dnd (with the right to die — see its ticket), FR-var-expansion-toggle,
FR-search, FR-filter-bar, FR-tree-browser, FR-fix-issues, FR-copy-entry; the Help → Documentation item
giving `F1` a menu home; a `--data-dir` switch; making the UI's borrow discipline structural.

**Settled at charting** (standing constraints for every session):

1. Destination is a **delta-spec**, not a full re-issue and not a build. The v0.1.0 spec is the
   fixed foundation; this map's tickets decide only what sits on top of it.
2. Stack is unchanged and remains a hard constraint: Rust + wxdragon. The accessibility question is
   *how*, never *whether to switch*.
3. One release: all in-scope items target v0.2.0 together; no interim v0.1.1.
4. NVDA verification is done **by the user personally** — prototype tickets are HITL and end in a real
   verdict, never in an inspector-tool guess. Unchanged from v0.1.0, reaffirmed at charting.
5. Out of this effort: in-app deaf-state detection (once-observed, unreported; revisit only on field
   recurrence), the network-path deadline prober, the winget submission (deferred indefinitely,
   recently and deliberately), and collapsing `ScopeDiagnosis` into `Findings` (buys depth, not
   behaviour; nothing in v0.2.0 presses on it).
6. All artifacts (map, tickets, research, spec) are written in **English**; conversation with the user
   is Ukrainian.
7. Before asking the user questions in any HITL session, **research best practices first** — bring the
   user informed options with evidence, not open-ended questions (standing directive from v0.1.0).

**Skills every session should consult.** `/grilling` and `/domain-modeling` for every grilling ticket;
`/research` for research tickets; `/prototype` for prototype tickets; `/codebase-design` for the borrow
discipline ticket. Domain terms resolved by a ticket go into `CONTEXT.md`; a decision that is hard to
reverse, surprising, and a genuine trade-off earns an ADR under `docs/adr/`.

**Facts worth carrying into every session.**

- wxMSW wraps native comctl32 controls; NVDA reads them for free, including column text. What native
  controls will *not* do for free is announce transient, non-focus messages — v0.1.0's Announcement
  mechanism (proven in its ticket 08) exists for that, and its message set is deliberately **closed**:
  any new spoken message is a change to that set and must be decided, not slipped in.
- v0.1.0 has **no toolbar and no in-app iconography** (spec §12), and its standing rule is that every
  shortcut has a menu home, because a menu item's label is the only place wxdragon can carry one. The
  PRD puts several v0.2.0 features "in the toolbar" — that conflict is a ticket, not an oversight.
- v0.1.0's diagnostics have **six Issue types and deliberately no severity classes** (spec §7: no
  severity prefix, an empty Status column is the only healthy state). The PRD's filter bar names
  "Errors" and "Warnings" — that conflict is a ticket too.
- Several PRD features interlock: Search and Filter bar compose (AND), Tree View's Enter **fills the
  Search bar**, and the expansion toggle changes what text the list shows — so search-over-what is a
  real question. The blocking edges below encode this.

## Decisions so far

<!-- one line per resolved ticket: gist + link. Charting-time constraints live in Notes, not here. -->

- [wxdragon widget surface for v0.2.0](issues/01-wxdragon-widget-surface-v0-2-0.md) — TreeCtrl (native SysTreeView32), SearchCtrl (generic composite on MSW), Clipboard, RadioButton/ToggleButton, Freeze/Thaw, DropSource/Text-FileDropTarget and LIST_BEGIN_DRAG are all bound in the pinned 0.9.18; ListCtrl checkboxes are NOT (raw `LVM_*` via `get_handle()` is the hatch, check *events* unreceivable through wxdragon), no row-hiding exists (rebuild or bound Virtual mode), no reorder-drag helper exists anywhere; 0.9.20 fixes a `get_item_text` UTF-8 truncation bug present in 0.9.18.

## Not yet specified

In scope, but not yet sharp enough to ticket. Graduates as the frontier advances.

- **Menu, accelerator and Announcement assembly** — where every new command lives in the menus, the
  final accelerator table (Ctrl+F, Alt+T, Ctrl+C, F1 and whatever the tickets add), and the final
  closed Announcement set with exact wording in both languages. Sharpens only once each feature's
  command surface is decided; lands in the locked-spec assembly ticket unless it grows into its own.
- **Release Checklist delta** — which steps the new surfaces add (search, filter, tree, Fix Issues
  dialog, D&D if it lives), and whether any v0.1.0 steps change. Sharpens with the feature contracts.
- **`settings.json` additions** — whether filter state, expansion mode or search text persist across
  runs, and what that does to the settings failure taxonomy. Sharpens per feature.
- **Whether v0.2.0's release mechanics need any decision at all** — scoop autoupdate and the release
  pipeline exist; possibly nothing to decide beyond version numbers. Check at assembly time.

## Out of scope

Ruled beyond this destination. Does not graduate.

- **Building v0.2.0.** This map produces the delta-spec; implementation is a separate effort.
- **In-app deaf-state detection** — parked by v0.1.0's ticket 24; revisit only if the state recurs in
  the field. Nothing has changed since.
- **The network-path deadline prober** — same ground as v0.1.0: a dead UNC blocks uncancellably, and
  the cure costs more infrastructure than the disease.
- **The winget submission** — deferred indefinitely by recent, deliberate decision; scoop and direct
  download carry distribution.
- **Collapsing `ScopeDiagnosis` into `Findings`** — the 2026-08-20 review's deferral stands: correct
  and tested as is; buys depth, not behaviour.
- **Everything v0.1.0 already ruled out**: similar-path/typo diagnostics, the `theme` setting, code
  signing until there are real users, UI automation, PRD §10 (other variables, sync, plugins, web/CLI,
  auto-update), non-Windows platforms, 32-bit, screen readers other than NVDA.
