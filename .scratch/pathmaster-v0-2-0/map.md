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

**In scope**: FR-reorder-dnd (carried the right to die and used it — see its ticket and Out of
scope), FR-var-expansion-toggle, FR-search, FR-filter-bar, FR-tree-browser, FR-fix-issues,
FR-copy-entry; the Help → Documentation item giving `F1` a menu home; a `--data-dir` switch; making
the UI's borrow discipline structural.

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
- [Command surface: does v0.2.0 grow a toolbar?](issues/02-command-surface-no-toolbar-question.md) — No toolbar (spec §12 stands; the PRD's three toolbar placements are recorded deviations); the menu bar becomes File / Edit / **View** / Tools / Help, with view-state commands in View and Working-Copy commands in Edit; exact items, accelerators and mnemonics stay fogged until the feature contracts and assembly (15).
- [Live-filtered list under NVDA](issues/04-live-filter-nvda-prototype.md) — proven against real
  NVDA: rebuilding rows under an unfocused list is silent (plain DeleteAllItems+reinsert; Freeze/Thaw
  dropped), the debounced count speaks reliably through the v0.1.0 mechanism, Tab/Down lands and
  reads, ESC clear-and-return works; speak-count (default on), ESC-to-list (default on) and debounce
  delay (default minimal, 250 ms) become `settings.json` settings — names, wording and taxonomy to
  tickets 06/15.
- [%VAR% expansion display toggle](issues/05-var-expansion-toggle.md) — **Expansion Mode** (now in
  `CONTEXT.md`): app-wide, per-Run (every Run starts raw; no settings field), view state outside
  dirty/Checkpoint/Undo; undefined `%VAR%` stays literal via Normalisation's own reading (no new
  Issue type — the PRD's "Warning" marker is a recorded deviation); Edit/Add always carry raw, list
  unchanged mid-dialog; expansion unconditional regardless of Value Type; the Announcement catalogue
  grows to eight ("Showing expanded values"/"Showing raw values" — «Показано розгорнуті значення»/
  «Показано збережені значення»); state carried by a checked View menu item, with the
  NVDA-reads-checked **verification obligation attached to the next NVDA prototype session**;
  search-over-what and copy-what handed to 06/11 as input.
- [Filtered view semantics](issues/03-filtered-view-semantics.md) — a **Filtered View** (now in `CONTEXT.md`) is derived, per-Scope view state, like Issues: outside the Undo history, changed only by the user's own typing; Edit/Delete/Copy act on the focused visible Entry while all reorder (D&D included, if it lives) and Add are disabled; membership recomputes live after every Working-Copy change; focus falls concerned-Entry → same-position → last-visible → empty list, with no new Announcements; `#` keeps original positions and Search+Filter compose with AND, both confirmed.
- [Search bar contract](issues/06-search-bar-contract.md) — a **permanent** native `TextCtrl` per
  Scope tab (never `SearchCtrl`), label + field above the list, Tab order tabs → field → list →
  buttons; matches the **currently displayed** rendering, case- and slash-folded (Unicode, not
  ASCII) and nothing else — so the Expansion toggle now *does* change membership and ticket 05's
  item 6 is amended; Ctrl+F focuses-and-selects (View menu, disabled on Backups), Enter is swallowed,
  Down/Tab enter the list, ESC always returns to the list (clearing first if there is text). The
  catalogue reaches ten items / six msgids: a short count for typing pauses and a Scope-named one
  for tab activation and Refresh, plural selected by **{m}**; with no Filtered View, Announcement 1
  speaks. StatusBar field 0 is the on-demand home of "N of M" (its issues count keeps its old
  meaning); nothing persists; three flat `settings.json` fields with dialog controls
  (`speakFilteredCount`, `filteredCountDelayMs` 0–5000, `searchEscapeReturnsFocus`) and no new
  failure layer.
- [Filter bar contract](issues/07-filter-bar-contract.md) — no severity (the PRD's Errors/Warnings
  buttons are a recorded deviation): a **Filter** (now in `CONTEXT.md`) is an exclusive per-Scope
  choice of seven states — All / With issues / the five Entry-level Issue types (Over-length is
  Scope-level and takes no part) — living as a View → Filter submenu of `wxITEM_RADIO` items with
  no on-window control (NVDA menu-radio proof → 16), toggled on the coarse axis by proposed Ctrl+I
  (final key → 15), starting at All every Run with no `settings.json` field; the Catalogue grows
  one item ("{filter}: {n} of {m} entries" pair), Announcement 1's condition becomes two-part
  (empty query AND Filter at All), and StatusBar field 0 names the state when narrowed.
- [Tree View browser contract](issues/08-tree-browser-contract.md) — a **Tree View** (now in
  `CONTEXT.md`) is a modal, per-Scope comprehension surface: the Scope's Filtered View snapshotted
  at open, merged by the expanded reading into a prefix tree (chains compressed, siblings
  alphabetical, misfits under top-level "Unresolved variables"/"Relative entries" groups, one leaf
  per Entry); Enter on a leaf selects that Entry's row and closes, Enter on an inner node
  expands/collapses — the PRD's fill-the-Search-bar coupling and Alt+T are recorded deviations
  (proposed Ctrl+T on a View → "PATH tree…" item, disabled on Backups); leaf labels carry raw form
  and Issue suffix only-when-present; "Go to entry" (default, leaf-only) + Cancel; no live
  diagnostics, no new Announcements, no settings, fallback branch closed (native SysTreeView32
  confirmed); three NVDA obligations → 16.
- [Fix Issues dialog contract](issues/09-fix-issues-dialog-contract.md) — **Fix Issues** (now in
  `CONTEXT.md`) is a modal, per-Scope repair surface over the active Scope's fixable Entry-level
  Issues: one row per flagged Entry (# / raw Path / Issue / Action, native `LVS_EX_CHECKBOXES` via
  01's hatch, state read at apply time), delete for Missing/Duplicate/Empty, one safe repair —
  remove every `"` — for Quoted; Relative-only and Over-length excluded by name. Disk-Cleanup
  defaults (safe rows on; Missing off when the raw text carries `%VAR%` or the root is non-Fixed;
  the PRD's network row reconciles to nothing — never probed). One Checkpoint in the active
  Session ("Fixing issues"); [Fix selected]/[Cancel] — "Apply" banned from the label; Announcement
  12 "Fixed {n} entries" (zero checked = Cancel, focus then Announcement, Delete's focus law);
  Edit → "Fix issues…" enabled iff ≥ 1 fixable row ∧ writable Session; staleness by
  generation-stamped passes (build only from a current-generation pass, apply by Entry id under
  modality). NVDA checkbox proof → 16; strings/keys → 15; no settings field.
- [Drag & Drop reorder — with the right to die](issues/10-dnd-reorder-right-to-die.md) — **it
  dies**: mouse-only and NVDA-invisible by construction (no UIA Drag pattern short of a custom
  provider), redundant beside the shipped Move Up/Down, all-bespoke in wxdragon, never promised in
  the README, and absent from Windows' own PATH editor; the delta-spec records the cut as a
  decision, not an omission.
- [Ctrl+C copy entry contract](issues/11-copy-entry-contract.md) — **copy-what-is-shown**: the
  focused visible Entry's currently displayed rendering (raw in raw mode, expanded in expanded —
  the PRD's "raw" amended the same way 06 amended Search), exact fidelity, no added quotes, always
  exactly one Entry (single-select reaffirmed); Edit → Copy with `\tCtrl+C`, disabled on Backups
  via `session: None`, frame-wide — wxMSW text entries claim Ctrl+C before accelerators (pinned
  3.3.3 source), so the Search field needs no app-side scoping; Announcement 13 "Copied to
  clipboard" / «Скопійовано до буфера обміну», Announcement 14 "Could not copy to clipboard" /
  «Не вдалося скопіювати до буфера обміну» spoken immediately on a failed `set_text` (no retry),
  silent no-op with no selection (Edit/Delete precedent), `flush()` best-effort after success so
  the copy outlives the Run; no settings field, no new NVDA obligation.
- [F1 and Help → Documentation](issues/12-f1-help-documentation.md) — a **User Guide** (now in
  `CONTEXT.md`): one page per Interface Language, written as two purpose-written user-facing
  Markdown documents (`docs/help/en.md`, `docs/help/uk.md` — **not** the README, whose badges would
  make a local help page phone `img.shields.io`), converted at build time by `pulldown-cmark` the
  way `polib` already makes `.mo`, embedded, and **overwritten unconditionally** into
  `data\help.html` on every open before the shell hands it to the browser — so staleness is
  structurally impossible, which "write only if missing" is not (scoop persists `data\`). Write
  failure → the version-pinned GitHub URL plus a `WARN` line; no network → the browser's own
  offline page; **no Announcement on any rung**, because every rung opens the browser. Menu home
  Help → "&User Guide" / «Посібник користувача(&U)» carrying `\tF1`, first with About last, no
  `…`, no separator, enabled in every state; the Help menu reaches two items (mnemonics U, A).
  **F1 in dialogs does nothing**, decided rather than defaulted. The page sets **no colours** —
  `color-scheme: light dark` and layout only — plus `lang` and a version-carrying `<title>`.
  Drift gated twice: a Checklist step against the product, a heading-parity `#[test]` between the
  languages; generating the keyboard table from the menus' own source is recorded as not bought.
  CHM, eWriter/MSHC, `wxHtmlWindow`, `%TEMP%`, shell-opening `.md` and F1 → About are ruled out by
  name. No settings field, no ADR.
- [--data-dir switch](issues/13-data-dir-switch.md) — the switch substitutes **only the locate step**
  of the v0.1.0 startup tree: both `--data-dir <path>` and `=` spellings (space form canonical;
  trailing quote/separator artifacts stripped first), relative paths resolved against the **CWD**
  once at startup into the absolute path every surface then uses, a missing directory created like
  the default one, and any unusable target → **Read-only Data via a fourth reason naming the
  switch** — never a fallback to the default `data\`. Elevation (and any future self-relaunch)
  re-serializes `--tab <active> --data-dir <resolved>` through one ArgvQuote writer/reader type in
  `pathmaster-platform`, unknown arguments dying at the boundary. Whole-app argument posture decided
  incidentally: unknown switch → dialog-and-continue with a shared usage line + one `WARN`;
  valueless `--data-dir` → the fourth reason; `--tab` leniency untouched; `--help`/`-?` → the same
  usage line in a dialog, then exit. Documented in the README and a User Guide "Command line"
  subsection; the startup log line gains `dataDir:` on override Runs; one sentence amends
  CONTEXT.md's Data Directory (no new term); no ADR, no settings field, no new Announcement.
- [Making the UI's borrow discipline structural](issues/14-structural-borrow-discipline.md) —
  **scoped access + one modal door**, decided in ADR-0011 (written): every cell reached by more
  than one kind of call (command / tick / synchronous callback) goes behind a `with`/`with_mut`
  wrapper whose guard cannot escape; one Drop-guarded modal-depth door that every `show_modal`
  passes through (source-scan `#[test]` enforces it), with the Timer's tick handler inert while
  depth > 0 — the Timer itself keeps firing, preserving `Pump`'s self-healing. Full retrofit of
  all ~47 existing sites lands as the **first implementation ticket**, before any new surface;
  the `App` doc comment is deleted. Full Elm dispatch, copy-out and GhostCell rejected by name.
  Hands 15/impl one hard constraint: the Search debounce timer must own a non-Frame widget
  (wxdragon binds `on_tick` on the owner, no id filter). No menu item, no Announcement, no
  settings field, no CONTEXT.md term.

In scope, but not yet sharp enough to ticket. Graduates as the frontier advances.

- **Menu, accelerator and Announcement assembly** — where every new command lives in the menus, the
  final accelerator table (Ctrl+F, Alt+T, Ctrl+C, F1 and whatever the tickets add), and the final
  closed Announcement set with exact wording in both languages. Sharpens only once each feature's
  command surface is decided; lands in the locked-spec assembly ticket unless it grows into its own.
  Fixed so far: 05's pair (item 8) and 06's six msgids (items 9 and 10), Search's home in View;
  07 adds the View → Filter submenu (seven radio items, disabled on Backups), a proposed Ctrl+I
  coarse toggle, one more count pair ("{filter}: …"), and makes Announcement 1's condition
  two-part (empty query AND Filter at All); 08 adds a View → "PATH tree…" item (disabled on
  Backups), a proposed Ctrl+T, and the dialog's title, two buttons and two group names as
  Catalogue strings — the Announcement set itself is unchanged by 08; 09 adds an Edit →
  "Fix issues…" item (disabled on Backups; accelerator open), the dialog's Scope-naming title, two
  buttons, four column headers and two action strings as Catalogue entries, and Announcement 12
  ("Fixed {n} entries" pair, plural by {n}); 11 adds an Edit → Copy item carrying Ctrl+C (disabled
  on Backups; no Ctrl+Insert twin) and Announcements 13/14 ("Copied to clipboard" / "Could not
  copy to clipboard" pairs, no placeholders) — the set reaches fourteen; 12 adds a Help →
  "&User Guide" / «Посібник користувача(&U)» item carrying F1, first in its menu with About last
  (mnemonics U and A, no separator, no `…`, enabled in every state), and **no** Announcement — the
  set stays at fourteen; 13 adds no menu item and no Announcement either (the set holds at
  fourteen), but grows the Catalogue by the fourth Read-only reason pair, the unknown-argument
  dialog title, the shared usage line and the `--help` dialog title; 14 adds nothing to menus,
  Announcements or the Catalogue — its contribution to 15 is a sequencing constraint (the
  mechanism retrofit is the first implementation ticket) and the debounce-timer-owner rule. Steps
  31, B12 and the mnemonic gate are voided by the menu growth and are 15's to re-run once, for
  all of it together.
- **Release Checklist delta** — which steps the new surfaces add (search, filter, tree, Fix Issues
  dialog, Ctrl+C copy with its two Announcements; D&D died in ticket 10), and whether any v0.1.0
  steps change. Sharpens with the feature contracts. Ticket 12 has named its eight: the two-item
  Help menu, F1 opening the browser onto a title NVDA speaks, `data\help.html` present and in the
  Interface Language, rewritten after a language change with no orphan, restored after deletion,
  the unwritable-`data\` run falling to the online URL with one `WARN` line, F1 silent in a dialog,
  and the item available on Backups and in Read-only Data. Ticket 13 names its seven: a fresh-path
  `--data-dir` launch creates the directory there with `data\` beside the exe untouched, a relative
  path resolves against the shell's CWD, an unusable target speaks the fourth Read-only reason and
  writes nothing anywhere, "Restart as Administrator" during an override Run with a spaced path
  lands the elevated instance in the same directory, an unknown argument shows the dialog and the
  app continues with one `WARN` line, `--help` shows the usage dialog and exits, and the startup
  log line carries `dataDir:` on override Runs.
- **`settings.json` additions** — mostly landed. Ticket 05: Expansion Mode does **not** persist, no
  field. Ticket 06 named ticket 04's three members, fixed their domains and defaults
  (`speakFilteredCount`, `filteredCountDelayMs` 0–5000, `searchEscapeReturnsFocus`), gave each a
  dialog control, established that they are ordinary field-layer members needing **no new failure
  layer**, and settled that the Search text dies with the Run. Ticket 07 answered its own
  question — the Filter does not persist, so no field. Ticket 08 adds none — the Tree View remembers
  nothing. Ticket 09 adds none — nothing about the Fix Issues dialog persists. Ticket 11 adds
  none — nothing about Copy persists or configures. Ticket 12 adds none either — the User Guide is
  neither configurable nor remembered, though it does add a **fourth file to `data\`**
  (`help.html`), so §3's and the README's by-name inventory of that directory grow by one. Ticket 13
  adds none by construction — the switch is per-launch and remembers nothing (ADR-0002). What is
  left is folding all of it into §13 at assembly.
- **Whether v0.2.0's release mechanics need any decision at all** — scoop autoupdate and the release
  pipeline exist; possibly nothing to decide beyond version numbers. Check at assembly time.

## Out of scope

Ruled beyond this destination. Does not graduate.

- **Building v0.2.0.** This map produces the delta-spec; implementation is a separate effort.
- **FR-reorder-dnd (drag & drop reorder)** — killed by
  [its ticket](issues/10-dnd-reorder-right-to-die.md) (2026-08-26): a mouse-only gesture NVDA
  cannot hear (native list, no UIA Drag pattern), duplicating the shipped Move Up/Down at real
  bespoke cost. The delta-spec records the cut as a decision, not an omission; returns only as a
  fresh effort if the destination is ever redrawn.
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
