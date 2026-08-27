# PathMaster v0.2.0 — Locked Delta-Specification

**Status: locked.** Assembled 2026-08-27 by ticket [15](issues/15-locked-delta-spec.md) from the
resolved wayfinder map ([map.md](map.md)). This document describes **only what v0.2.0 adds or
changes** on top of the locked v0.1.0 specification
([../pathmaster-v0-1-0/spec.md](../pathmaster-v0-1-0/spec.md)), which is the fixed foundation and
is not reopened; a section reference like "§7" with no other qualifier means that document's
section. The source PRD is the same
([../pathmaster-v0-1-0/spec-input.md](../pathmaster-v0-1-0/spec-input.md)); where the PRD and this
document disagree, **this document wins** — every deviation is listed in
[§21 PRD deviation notes](#21-prd-deviation-notes). Each requirement names the ticket that settled
it; ticket answers and the research files behind them are the authority on detail this document
only gists.

Decisions marked **[assembly]** were fixed by ticket 15 itself, under the delegations the resolved
tickets left it: the final accelerator table and menu order (tickets 05 D7, 07 D4, 08 D11, 09 D7),
exact Catalogue strings the tickets left open, the StatusBar field-0 composition (06 D17, 07 D9),
and the Settings-dialog control labels (06 D21). Everything else traces to a resolved ticket. One
contradiction between the v0.1.0 spec and a resolved ticket was found at assembly and resolved by
recency, recorded in [§2.1](#21-the--column-a-recency-resolution) — never silently.

Every mechanism this document's accessibility depends on is **measured against real NVDA**, not
assumed: round one ([ticket 04](issues/04-live-filter-nvda-prototype.md)) proved the live-filtered
list; round two ([ticket 16](issues/16-nvda-verification-round-2.md), 2026-08-27) discharged all
seven parked obligations with no contract amended. §19 records what was measured.

## 0. Scope

**v0.2.0 = delivering what was promised**: the 🟡 should-features v0.1.0's charting cut, plus the
small items parked beside them — not field feedback, not architecture for its own sake
(map, settled at charting 2026-08-26).

In scope and specified here: **FR-var-expansion-toggle** (§5), **FR-search** (§3),
**FR-filter-bar** (§4), **FR-tree-browser** (§6), **FR-fix-issues** (§7), **FR-copy-entry** (§8),
the **Help → User Guide item giving F1 a menu home** (§9), a **`--data-dir` switch** (§10), and
**making the UI's borrow discipline structural** (§11). **FR-reorder-dnd carried the right to die
and used it** — the cut is a decision, recorded in §1 and §20, not an omission.

Standing constraints, unchanged from charting: the stack stays Rust + wxdragon (pinned 0.9.18);
one release — everything here targets v0.2.0 together, no interim v0.1.1; the v0.1.0 spec is not
reopened. The menu bar model for all of it: **no toolbar** — every new command lives in a menu
with an optional accelerator, exactly as every v0.1.0 command does
([ticket 02](issues/02-command-surface-no-toolbar-question.md); §12 of this document).

## 1. Requirement disposition

Every v0.2.0-scoped requirement, explicitly **kept**, **rewritten**, **cut**, or **new** (no PRD
id). "Rewritten" means the intent survives with changed acceptance criteria.

| Requirement | Disposition | Settled by | Where |
|---|---|---|---|
| FR-reorder-dnd | **cut — exercised its right to die**: mouse-only and NVDA-invisible by construction (no UIA Drag pattern short of a custom provider), redundant beside the shipped Move Up/Down, all-bespoke in wxdragon, never promised in the README | [10](issues/10-dnd-reorder-right-to-die.md) | §20 |
| FR-var-expansion-toggle | rewritten — app-wide per-Run **Expansion Mode**, check menu item, no persistence, no new Issue type | [05](issues/05-var-expansion-toggle.md) | §5 |
| FR-search | rewritten — permanent per-Scope field (not summoned), matches the displayed rendering, count is an Announcement | [06](issues/06-search-bar-contract.md) | §3 |
| FR-filter-bar | rewritten — **no severity, no bar**: a View → Filter submenu of seven radio states | [07](issues/07-filter-bar-contract.md) | §4 |
| FR-tree-browser | rewritten — modal comprehension surface; the Search-bar coupling and Alt+T are dropped | [08](issues/08-tree-browser-contract.md) | §6 |
| FR-fix-issues | rewritten — one row per Entry with one computed action; three deletions + the Quoted repair | [09](issues/09-fix-issues-dialog-contract.md) | §7 |
| FR-copy-entry | rewritten — copy-what-is-shown; two new Announcements; platform-scoped Ctrl+C | [11](issues/11-copy-entry-contract.md) | §8 |
| Help → Documentation / F1 (v0.1.0 §20 parking) | new — the **User Guide**: embedded per-language page, rewritten into `data\` on every open, browser as viewer | [12](issues/12-f1-help-documentation.md) | §9 |
| `--data-dir` switch (v0.1.0 §20 parking) | new — substitutes only the locate step; plus the app's whole argument posture | [13](issues/13-data-dir-switch.md) | §10 |
| Structural borrow discipline (2026-08-20 review deferral) | new — scoped access + one modal door, ADR-0011; total retrofit lands first | [14](issues/14-structural-borrow-discipline.md) | §11 |

## 2. The Filtered View — the model everything narrows through

Settled by [ticket 03](issues/03-filtered-view-semantics.md); term in `CONTEXT.md`. A **Filtered
View** is an Editing Session's view of its Working Copy, narrowed to the Entries matching that
Scope's Search text (§3) and Filter (§4), composed with **AND**. It is derived view state of the
same class as Issues: it reads the Working Copy and is never part of it.

- **Per-Editing-Session.** Each Scope keeps its own Search text and Filter; switching tabs keeps
  both. The Backups tab is not a Scope and has neither.
- **Editing under a filter — the partial model.** Commands whose effect is fully visible act on
  the focused visible Entry: **Edit, Delete, Copy work; Move Up, Move Down and Add are disabled**
  while a Filtered View is active (menu items and buttons read as disabled). A reorder's effect
  concerns positions the user cannot see — the verdict that covered D&D too, while it lived. The
  list is single-selection and every allowed command touches exactly one visible Entry, never a
  hidden one — the Excel delete-hidden-rows trap is unrepresentable.
- **Live membership.** The visible set recomputes after **every** Working-Copy change — Edit
  commit, Delete, Undo, Redo, Refresh, Restore — and recomputation is **silent** (what is spoken
  is §3's business, and it speaks only when the *criteria* change). An Entry edited out of the
  match set vanishes at dialog OK — a discrete moment; dialog-first editing means no
  mid-keystroke vanish exists.
- **Focus rule** on any membership change: (1) the Entry the operation concerned, if visible;
  (2) else the row at the same visual position, else the last visible row; (3) if no rows remain,
  focus stays on the empty list — it never jumps to the Search field uninvited. Nothing new is
  spoken: NVDA reads the newly focused row free (ADR-0003).
- **Outside the Undo history.** Checkpoints do not capture Search/Filter state; Ctrl+Z never
  mutates the Search or Filter controls. **No command changes the criteria** — not Refresh, not
  Restore, not Apply; only the user's own narrowing actions do (typing in the Search field,
  choosing a Filter state — [07](issues/07-filter-bar-contract.md) D10's amendment).
- **Nothing persists.** Search text and Filter die with the Run (§3 D18, §4 D6).

### 2.1 The `#` column — a recency resolution

The v0.1.0 spec dropped the PRD's index column (§10, §12 D8: "position is NVDA's setting").
Ticket 03 D7, seven days later, confirmed the PRD's anchor: **displayed `#` indexes are the
Entries' original positions — no renumbering** — "the one honest option: it keeps an Entry's place
in the full list readable exactly when reorder is disabled, and NVDA reads column text free."
These contradict, and per this ticket's charter the contradiction resolves by **recency**:
ticket 03 wins, recorded here.

- **The main list becomes three columns: `#` / Path / Status.** `#` carries the Entry's position
  in the Working Copy (1-based), unchanged by any narrowing. The v0.1.0 rationale for the cut
  holds only for an unfiltered list: under a Filtered View, NVDA's own "3 of 12" names the
  *visible* position, so original position needs a carrier, and column text is the free channel.
  The column is **permanent** — §12's layout rule (the window never reflows under the user)
  forbids a column that appears when filtering starts.
- The `#` column joins the Status column as a deliberate pixel constant (one more explicit
  `FromDIP()` call); Path still takes all remaining width. NVDA's per-row reading becomes
  **"{#}; Path: {path}; Status: {types}"** — Release Checklist steps 2–4 change accordingly
  (§17). *(Amended 2026-08-27 from the measurement in §19's round three; assembly had written
  "{#}; {path}; …" here. NVDA reads the leftmost report column bare and prefixes every other
  with its header, so promoting `#` into column 0 is what puts "Path:" in front of the path —
  a consequence of the column, not a choice. Nothing is lost, and nothing can be changed from
  the application's side that does not cost the visible header or the free comctl32 path.)*
- The Fix Issues dialog (§7) and the ticket-03 convention use the same `#` meaning; the column
  header `#` is a Catalogue string.
- What does **not** return: the count compensation. Entry counts still come from Announcements
  (v0.1.0 §10), and NVDA's row-position setting stays uncompensated.

## 3. Search

Settled by [ticket 06](issues/06-search-bar-contract.md), on the mechanism
[ticket 04](issues/04-live-filter-nvda-prototype.md) proved against real NVDA.

> **FR-search** (rewritten) — a **permanent** search field, one per Scope tab, label + field above
> the list; label **"Search:"** **[assembly]** (uk «Пошук:»), constant, no mnemonic, never
> carrying the count (a changing label is a `NAMECHANGE`, measured dead in v0.1.0). Built from a
> native **`TextCtrl`**, never `SearchCtrl` (the generic composite on MSW — ticket 01 — unmeasured
> with NVDA, while `TextCtrl` is the exact control ticket 04 proved). The Backups tab has no
> field.
>
> - **Matches the currently displayed rendering** (raw text in raw mode, the expanded reading in
>   expanded mode — §5): what the spoken count counts is exactly what the arrow keys will read.
>   Consequence, paid deliberately: with a Filtered View active, toggling Expansion Mode changes
>   membership (`%JAVA_HOME%\bin` and `C:\jdk21\bin` are different haystacks).
> - **Case-insensitive substring, slash-folded (`/`→`\`), and nothing else.** Case and slash
>   direction are foldings the domain already applies everywhere; quote stripping, trailing-`\`
>   trimming and `%VAR%` expansion change *what text exists* and stay out — a search for `"`
>   **must** find the `Quoted` Entries. **Unicode case folding, never ASCII**
>   (`str::to_lowercase`, both sides): an ASCII fold would be silently case-sensitive for every
>   Cyrillic path. **The query is never trimmed** — whitespace is Entry content.
> - **Keyboard**: Ctrl+F (menu home View → Search, §12) focuses the field and selects its whole
>   contents; disabled on the Backups tab. Tab order becomes **tabs → search field → list →
>   buttons** — one extra Tab stop on every run, named as the cost it is. **Enter is consumed by
>   the field and does nothing** (an unhandled Enter reaches the default button; an Enter that
>   moved focus would arm the list's Edit on the next press). **Down-arrow and Tab** enter the
>   list (proven, ticket 04). **ESC** clears the text and returns focus to the list (default;
>   reversible by `searchEscapeReturnsFocus`), landing by §2's focus rule; ESC on an
>   already-empty field still returns focus and says nothing — one gesture, one meaning.
> - **Rebuild strategy**: plain `DeleteAllItems` + reinsert under the unfocused list — measured
>   silent under NVDA, no chatter, no deaf-list signature; Freeze/Thaw earned nothing and is
>   dropped (ticket 04's verdict).
> - **Read-only Data searches normally** — that Run still reads, diagnoses and lists.

**What is spoken** (the exact msgids are in §13): the count Announcement fires when the **view
criteria** change — search text (debounced through `filteredCountDelayMs`), Filter state, or
Expansion Mode — never on Working-Copy changes. A short count (item 9) answers typing pauses; a
Scope-named count (item 10) answers tab activation and Refresh while that Scope has a Filtered
View; **whenever no Filtered View is active — empty query AND Filter at All — Announcement 1
speaks**, the two-part condition [07](issues/07-filter-bar-contract.md) D8 completed. The plural
form of the count msgids is selected by **{m}**, the total — written down or lost, because the
i18n gate checks plural presence, not which number chose them. An empty result set shows zero
rows and no placeholder, speaks the zero-case msgid, and disables Edit/Delete/Copy (no focused
visible Entry — §2 applied, not extended).

## 4. Filter

Settled by [ticket 07](issues/07-filter-bar-contract.md); term **Filter** in `CONTEXT.md`.

> **FR-filter-bar** (rewritten — **no severity, and no bar**) — the PRD's "Errors / Warnings"
> buttons are a recorded deviation: v0.1.0's six Issue types share one consequence and no severity
> partition is minted to power two buttons. A **Filter** is an exclusive, per-Scope choice among
> **seven states**: `All` / `With issues` / `Missing` / `Relative` / `Quoted` / `Duplicate` /
> `Empty`. An Entry is visible when its Issue set contains the chosen type; `With issues` means a
> non-empty Status. **Over-length is Scope-level, flags no Entry, and no state selects it** — its
> whole surface remains the StatusBar length field and the two Apply gates.
>
> - **Home: a View → Filter submenu of seven `wxITEM_RADIO` items; no on-window control.**
>   NVDA reads a native menu radio item's selected state, and the checked item follows the active
>   Scope — both measured (ticket 16, probes 3). Disabled on the Backups tab.
> - **One coarse-axis toggle**: from `All` → `With issues`; from any non-All state → `All`. The
>   five per-type states are menu-only. Final key **Ctrl+I** **[assembly]**, riding its own menu
>   item (§12) because every shortcut has a menu home.
> - **Per-Scope state, dies with the Run**: every Run starts at `All` on every Scope; no
>   `settings.json` field.
> - **Spoken**: a change to a non-All state speaks the already-composed Search∧Filter count as
>   item 11 — "{filter}: {n} of {m} entries" — one announcement, never two. A change to `All`
>   with an empty query speaks Announcement 1; with query text present, item 9 (the remaining
>   narrowing is search-only). Filter-state names reuse the menu/Status strings — no new msgids
>   for names.
> - **StatusBar field 0 names the state when the Filter ≠ All** (§16).

## 5. Expansion Mode

Settled by [ticket 05](issues/05-var-expansion-toggle.md), amended by 06; term in `CONTEXT.md`.

> **FR-var-expansion-toggle** (rewritten) — **Expansion Mode** is app-wide derived view state:
> one flag for the application, both Scope tabs render alike (Search/Filter are per-Scope because
> they are queries against data; the mode is how the user is reading paths right now).
>
> - **Per-Run, default raw; nothing persists** — no `settings.json` field. A run that silently
>   opened expanded a week later would hand an NVDA user `C:\jdk21\bin` with the only clue buried
>   in an unvisited menu (deliberate deviation from the Excel Show-Formulas persistence
>   precedent, recorded in the ticket).
> - **Not an edit, not a Checkpoint, invisible to Undo/Redo both ways.** The mode never touches
>   the Working Copy; Ctrl+Z under expanded mode shows the rolled-back Working Copy, still
>   expanded.
> - **Display expansion is Normalisation's own reading** (`ExpandEnvironmentStringsW`, process
>   environment; **undefined `%VAR%` stays literal in place**) — what is shown can never disagree
>   with what is diagnosed. **No new Issue type, no inline marker**: the PRD's "Warning: Unknown
>   variable" is a recorded deviation; the Status column's natural `Missing` already answers
>   "why". Expansion is unconditional regardless of Value Type — a `REG_SZ` Scope expands in the
>   display exactly as it already does in diagnostics.
> - **Editing always works on raw.** Edit/Add dialogs carry the raw text whatever the list shows;
>   the list does not change while a dialog is open; on OK the row re-renders in the current
>   mode. Mixed per-row display is ruled out.
> - **State carrier: a `wxITEM_CHECK` item in View** with a constant label — NVDA reads the
>   checked state in both directions (measured, ticket 16 probe 1). Toggling speaks Announcement
>   8 ("Showing expanded values" / "Showing raw values"); focus stays on the list and an arrow
>   key re-reads the row. **With a Filtered View active on the visible Scope, the toggle speaks
>   twice**: its mode message, then item 9's count through the same debounced path — both
>   reliably land at the 250 ms default (measured, ticket 16 probe 2; no floor change).
> - **[assembly]** The menu item is **disabled on the Backups tab**, like every other View item:
>   the command changes what Scope lists show, and on Backups nothing it changes is visible. The
>   check mark stays readable on the disabled item; the mode itself is app-wide and unaffected.

## 6. Tree View

Settled by [ticket 08](issues/08-tree-browser-contract.md); term in `CONTEXT.md`.

> **FR-tree-browser** (rewritten) — a **Tree View** is a modal, per-Scope **comprehension**
> surface, not a navigation-by-search one: the PRD's fill-the-Search-bar coupling is dropped as a
> recorded deviation (no surveyed tool couples tree activation to a search field, and expanded
> tree text filling a raw-mode Search field would make an empty list the feature's normal
> result — the 06/05 trap).
>
> - **Content**: the active Scope's **Filtered View snapshotted at open** — in the unnarrowed
>   state, the whole Working Copy. The dialog never touches the narrowing criteria; wanting the
>   whole PATH's shape = clear the narrowing first. **Snapshot, no live diagnostics, no refresh
>   affordance** — reopening is the refresh; the snapshot keeps every timer out of the modal's
>   event loop.
> - **Shape**: Entries merged by the **expanded reading** (Normalisation's own, undefined `%VAR%`
>   literal, independent of Expansion Mode) into a prefix tree. Single-child chains compress into
>   one node with the joined label ("Program Files\Java\jdk-21"); siblings sort alphabetically,
>   case-insensitive; Entries with no filesystem position get top-level group nodes —
>   **"Unresolved variables"** and **"Relative entries"** — sorted after the drive roots, hidden
>   when empty; no artificial super-root. **One leaf per Entry** — duplicates are sibling leaves.
> - **Leaf label** — the whole audible payload: segment/joined chain + **raw form in parentheses
>   only when it differs** from the expansion + **Issue suffix in the exact Status-column words
>   only when an Issue exists** — `bin (%JAVA_HOME%\bin) — Missing`. Inner nodes and groups carry
>   no suffixes. Compressed labels and three-part leaves speak in full (measured, ticket 16
>   probes 4–5).
> - **Interaction**: Enter on a leaf **selects that Entry's row in the main list — by Entry
>   identity, never by text — and closes**; Enter on an inner node expands/collapses (the native
>   default action; wxMSW's tree eats unmodified Enter and raises `ITEM_ACTIVATED`, so the
>   activation handler is the single home of the commit logic). Buttons **"Go to entry"**
>   (default; disabled while an inner node or group is selected) + **Cancel**; Esc closes; no OK,
>   no Close. Tab order: tree → Go to entry → Cancel; initial focus on the first top-level node.
>   The landed row speaks in full and Cancel speaks the restored focus (measured, ticket 16
>   probe 6).
> - **Widget**: wxdragon `TreeCtrl` — the native `SysTreeView32` (ticket 01); the fallback branch
>   closed unused.
> - **Command**: View → "PATH Tree…", final key **Ctrl+T** **[assembly]** (the PRD's Alt+T is a
>   recorded deviation — the UX guide forbids Alt+letter shortcuts); disabled on the Backups tab.
>   The title names the Scope (§14). **No new Announcements, no `settings.json` fields** — the
>   dialog remembers nothing, expansion state not preserved.

## 7. Fix Issues

Settled by [ticket 09](issues/09-fix-issues-dialog-contract.md); term in `CONTEXT.md`.

> **FR-fix-issues** (rewritten) — **Fix Issues** is a modal, per-Scope repair surface over the
> **active Scope only**, changing only the Working Copy — nothing reaches the registry.
>
> - **Fixable = three deletions + one repair.** Missing, Duplicate and Empty propose **Delete
>   entry**; Quoted's repair finally exists: **Remove quotes — every `"` in the Entry** (`"` is
>   illegal in Windows file names, so no quote can be path content; the repair is
>   guaranteed-behaviour-preserving, the one respected auto-fix criterion). **Relative gets no
>   repair** (qualification needs a base directory only the user knows) and **Relative-only
>   Entries are excluded** — a row that can fix nothing is noise; Edit and the Filter's
>   `Relative` state are where those Entries are found and cured. **Over-length is excluded
>   entirely** — no row, no reminder text (§4's gesture: Scope-level, takes no part).
> - **One row per Entry, one computed action** (the PRD's row-per-problem amended): the Issue
>   column carries the comma-joined Status string; the action is **Delete entry** when any of
>   Missing/Duplicate/Empty is flagged (deletion cures Quoted too), else **Remove quotes**.
> - **Columns # / Path / Issue / Action**; `#` the original position (§2.1); **Path is always
>   the raw text**, whatever the Expansion Mode — the dialog shows what will be deleted or
>   repaired, and the `%VAR%` default rule must be visible in the row it judges. Checkboxes are
>   native `LVS_EX_CHECKBOXES` through ticket 01's raw-`LVM_*` hatch; check state is read once,
>   by `LVM_GETITEMSTATE` **at apply time** — check events (unreceivable through wxdragon) are
>   never needed. Rows read their state, Space toggles with the change announced in place, and
>   the native state survives the silent wx event layer (measured, ticket 16 probe 7).
> - **Defaults — the Disk Cleanup principle.** ON: Remove quotes; Delete via Duplicate or Empty;
>   Delete via Missing on a `DriveType=Fixed` local root with no `%VAR%` in the raw text. OFF:
>   Delete via Missing when the raw text contains `%VAR%` or the root is a non-Fixed drive. The
>   PRD's network row reconciles to **nothing** — network roots are never probed and never flag
>   (§7 FR-diag-missing).
> - **Buttons [Fix selected] [Cancel]** — "Apply" is banned from the label (this product's
>   reserved word for the registry write). Title names the Scope (§14). Initial focus on the
>   first row; [Cancel] keeps default and Escape. No Select-all/Clear-all — Space on a row is the
>   whole mechanism. **Zero rows checked at activation = Cancel**: no Checkpoint, no
>   Announcement; the button is never dynamically disabled.
> - **Applying = one Checkpoint** in the active Session, operation name **"Fixing issues"**
>   (uk «Виправлення проблем»); focus first (Delete's law: same index clamped to the new last
>   row), then **Announcement 12 "Fixed {n} entries"** — last heard is the summary (order
>   confirmed as designed, ticket 16 probe 7). Re-diagnosis is the existing §7 law — recompute
>   after every Working-Copy change.
> - **Enablement**: Edit → "Fix Issues…", disabled on the Backups tab; enabled iff the active
>   Scope has **≥ 1 fixable row** (not merely "Issues exist" — all-Relative or Over-length-only
>   would open empty) **and** its Session is writable (System unelevated and Read-only Data
>   disable it). Menu enablement is the only indicator — the Status column and the StatusBar's
>   "({k} issues)" already say there is work. **[assembly]** The item carries **no accelerator**:
>   an occasional bulk-review dialog, in the Settings…/Restore class, not the F2 class; every
>   shortcut needs a menu home, not every item a shortcut.
> - **The staleness rule, two halves**: *(a) at open*, every diagnostic pass is stamped with the
>   Working-Copy generation it read, and the dialog builds only from a pass whose stamp equals
>   the current generation — if none exists yet, the command waits for the outstanding pass
>   (< 1 s budget, §7; no spinner). *(b) After open*, modality is the fence; apply resolves
>   checked rows to Entries **by id**, never by index, and asserts the generation unchanged — an
>   invariant named so no implementation unmakes it silently (e.g. by going modeless).
> - **No `settings.json` field** — nothing about the dialog persists.

## 8. Copy entry

Settled by [ticket 11](issues/11-copy-entry-contract.md).

> **FR-copy-entry** (rewritten) — **copy-what-is-shown**: Ctrl+C puts the focused visible
> Entry's **currently displayed rendering** on the clipboard — raw in raw mode, expanded in
> expanded mode (the PRD's "raw" amended the same way Search was; every Run starts raw, so the
> default behaviour still matches the PRD, and the Expansion toggle becomes the one way to
> extract an expanded value from the application at all). Exact text fidelity — no quotes added,
> an Entry's own quotes are content. Always exactly one Entry — **single-select is reaffirmed**.
>
> - **Scoping is the platform's own**: wxMSW text entries claim Ctrl+C/X/V/A before accelerator
>   translation (pinned 3.3.3 source), so the menu-label accelerator never steals the Search
>   field's or a dialog field's copy — no focus-checking handler, no dynamic tables. The command
>   is otherwise **frame-wide** like every v0.1.0 Entry command.
> - **Menu home: Edit → Copy**, `\tCtrl+C`; `session: None` disables it on the Backups tab
>   exactly as Edit/Delete. No Ctrl+Insert twin (it would need a hidden duplicate menu item; no
>   recorded need).
> - **Spoken**: success — **Announcement 13 "Copied to clipboard"**, fixed text, no echo of the
>   payload (the row was just read by focus; Entries run long). Failure — **Announcement 14
>   "Could not copy to clipboard"**, spoken immediately on a failed `set_text`, no retry: for a
>   blind user the Announcement is the only channel, and a copy that silently did nothing is
>   indistinguishable from a missed keystroke. NVDA itself announces nothing for app-side copies
>   (nvda#75). **No selection = silent no-op** (the `edit`/`delete` precedent) — silence only
>   ever means "nothing was selected".
> - **The copy outlives the Run**: after a successful `set_text`, `flush()` — best-effort, its
>   own result never announced (a failed flush merely restores stock wx behaviour).
> - **No settings field, no new NVDA obligation** — both Announcements ride the mechanism
>   v0.1.0's ticket 08 proved.

## 9. The User Guide and F1

Settled by [ticket 12](issues/12-f1-help-documentation.md); term **User Guide** in `CONTEXT.md`.

> **FR-user-guide** (new) — a **User Guide** the executable carries: one page per Interface
> Language, embedded in the exe, written into the Data Directory when opened, handed to the
> default browser — which *is* the help viewer (NVDA's own model, and the only route that buys
> browse mode: heading navigation, a headings list, find). Ruled out **by name** so they are not
> re-proposed: CHM, eWriter/MSHC, `wxHtmlWindow`, `%TEMP%`, shell-opening a `.md`, and F1 →
> About.
>
> - **Source: two purpose-written Markdown documents**, `docs/help/en.md` and `docs/help/uk.md` —
>   **not** the README (front door for someone *choosing* the application; its badges would make
>   a local help page phone `img.shields.io` on every open). Content contract (prose written at
>   implementation): what PATH is; the window; editing; **what each of the six Status words
>   means**; Backups and restore; what v0.2.0 adds; **the full keyboard table**; Settings;
>   the System PATH and administrator rights; what is written where; troubleshooting; and a
>   **"Command line" subsection** (§10). Deliberately absent: installation, release
>   verification, contributing, the licence. No screenshots — pure text, zero external requests.
> - **`data\help.html` — one file, no language suffix, overwritten unconditionally on every
>   open** through the existing atomic `datadir::write_replace` — staleness is structurally
>   impossible ("write only if missing" is poisoned: scoop persists `data\` as a junction, and a
>   v0.2.0 binary would show v0.1.0's guide forever). **§3's and the README's by-name inventory
>   of `data\` grows by this fourth file.**
> - **The failure ladder — and no Announcement on any rung.** Write succeeds → `ShellExecuteW`
>   on `data\help.html`. Write fails (Read-only Data, full disk) → the **version-pinned** GitHub
>   URL `…/blob/v{version}/docs/help/<code>.md` plus one `WARN` line — `{version}` from the same
>   `CARGO_PKG_VERSION` the §16 gates keep honest (in a development build the URL 404s until the
>   tag exists — named here, not a bug; the Release Checklist runs on a tagged build). No
>   network → the browser's own offline page, visible. Nothing is announced because nothing is
>   silent: every rung opens the browser and NVDA names the window. A shell that opens nothing
>   at all takes the `open_backups_folder` precedent: silence plus a log line.
> - **The page sets no colours**: `:root { color-scheme: light dark; }` — the HTML equivalent of
>   §12's rule (a bare page with *no* stylesheet is painted black-on-white regardless of theme,
>   which would not satisfy it); layout only (`max-width`, `font-family: system-ui`,
>   `line-height`); forced: `<meta charset="utf-8">`, `lang="en"`/`"uk"` (without it NVDA may
>   read the Ukrainian guide in an English voice), and a `<title>` "PathMaster {version} — User
>   Guide" — the first thing NVDA speaks when the page loads.
> - **Build: the `.mo` mechanism, mirrored** — `pulldown-cmark` (one new build-dependency beside
>   `polib`) converts `docs/help/<code>.md` → `OUT_DIR/help-<code>.html`, embedded via the same
>   `include_bytes!` pattern.
> - **Menu home: Help → "&User Guide"** (uk «Посібник користувача(&U)») carrying `\tF1`, **first
>   in the menu, About last**; mnemonics **U** and **A**; no `…`, no separator; **enabled in
>   every state** — how to use the application is true in every state it can be in. **F1 in
>   dialogs does nothing, as a decision** (the price of the opposite is `EVT_CHAR_HOOK` in every
>   dialog as a standing obligation no gate would catch a future dialog breaking; the
>   against-silence rule governs commands that *failed*, and F1 in a dialog is an unbound key).
> - **Drift is gated twice**: a Release Checklist step against the product, and a
>   **heading-parity `#[test]`** — both documents exist, are non-empty, and carry the same set
>   of headings — in `pathmaster-core/tests/` reading `../../docs/help/*.md` (the
>   `versioninfo.rs` precedent: pure text stays out of the crate that links wxWidgets).
>   **Not bought**, recorded so it is not rediscovered as an oversight: generating the keyboard
>   table from the menus' own source (it would introduce a (command, msgid, accelerator) table
>   the product does not have).
> - **No settings field, no new Announcement, no ADR.**

## 10. The `--data-dir` switch and the argument posture

Settled by [ticket 13](issues/13-data-dir-switch.md); `CONTEXT.md`'s Data Directory amended by
one sentence (no new term).

> **FR-data-dir** (new) — `--data-dir` substitutes **only the locate step** of the §3 startup
> tree; everything downstream runs unchanged: a missing directory is `create_dir_all`-created
> like the default one, and a target that cannot be created or written lands the Run in
> **Read-only Data through a fourth reason naming the switch** — **never a fallback to the
> default `data\`**: the application never writes where it was not pointed.
>
> - **Both spellings**, `--data-dir <path>` and `--data-dir=<path>` (the README documents the
>   space form). Before resolution the value is stripped of trailing `"` and trailing path
>   separators — recognizable artifacts of the backslash-before-quote parsing rule — and both
>   run **before** elevation forwarding.
> - **Relative paths resolve against the CWD**, once at startup, make-absolute (never
>   `fs::canonicalize` — a `\\?\` result must not ride a command line); the resolved absolute
>   path is the single truth every downstream surface uses. **The startup log line grows
>   `dataDir: <resolved path>` on every override Run** — the log is the only diagnostic
>   artifact, and a Run that wrote elsewhere is otherwise unreconstructable. (The no-PII rule's
>   path prohibition yields to the audit need here by the ticket's decision — this one derived
>   fact is the record of *where the application wrote*, not of PATH content.)
> - **Elevation forwards by re-serialization, never the verbatim command-line tail**: the
>   relaunch line is built from parsed state (`--tab <active> --data-dir <resolved>`) through
>   **one ArgvQuote writer/reader type** in `pathmaster-platform` (Colascione's 2n/2n+1 rules;
>   `std::process::Command` quoting does not apply to `ShellExecuteExW`'s hand-built
>   `lpParameters`). The rule is general: **any future self-relaunch carries the override**, or
>   it silently writes elsewhere. Unknown arguments die at the boundary — no second dialog in
>   the elevated instance.
> - **Whole-app argument posture** (decided here, incidentally): unknown switch →
>   **dialog-and-continue** — message in the title ("Unknown argument {arg} was ignored"
>   **[assembly]**), one shared usage line in the body, [OK], one `WARN` line, then a normal
>   start. A valueless or malformed `--data-dir` is not an unknown argument but a broken
>   override → Read-only Data, fourth reason. `--tab`'s v0.1.0 leniency stays. **`--help` and
>   `-?` are recognized**: a dialog carrying the same usage line, then **exit** — a query, not a
>   launch.
> - **Documented** in the README's portability section and the User Guide's "Command line"
>   subsection (§9). **No settings field, no new Announcement** — the fourth reason rides
>   Announcement 7; **no ADR** (ADR-0002 and ADR-0005 already record the surprising parts).

## 11. Structural borrow discipline

Settled by [ticket 14](issues/14-structural-borrow-discipline.md);
**[ADR-0011](../../docs/adr/0011-borrow-discipline-is-structural.md)** (written) is the full
statement — this section gists it and fixes the sequencing.

- **Mechanism: scoped access + one modal door.** Every cell reached by more than one kind of
  call (command / Timer tick / synchronous toolkit callback) goes behind a `with`/`with_mut`
  wrapper whose guard cannot escape — today both Sessions, both `findings`, the Backups page's
  file cell; a future cell classifies itself by the same rule. One module owns a Drop-guarded
  modal-depth `Cell` and the single function every `show_modal`/message box passes through; the
  Timer's tick handler is inert while depth > 0 — the Timer itself keeps firing, preserving
  `Pump`'s self-healing (a pass landing mid-dialog is collected ≤ 100 ms after close). A
  **source-scan `#[test]`** fails the build if `show_modal` appears outside the door module.
  Full Elm dispatch, copy-out and GhostCell rejected by name (ADR-0011).
- **Sequencing — a hard constraint on the implementation effort**: the retrofit of all ~47
  existing borrow sites and every dialog call lands as the **first implementation ticket**,
  before any new surface is coded; the `App` doc comment is then deleted. No two-regime
  transition period.
- **One more hard constraint handed to implementation**: the Search debounce timer must be
  owned by a **non-Frame widget** — wxdragon 0.9.18 binds `on_tick` on the timer's owner with
  no id filter, so two timers on one owner fire each other's handlers.
- No menu item, no Announcement, no settings field, no `CONTEXT.md` term.

## 12. Menus and keyboard **[assembly]**

Assembled per the tickets' delegations. **No toolbar and no in-app iconography** — §12 and D8
stand ([ticket 02](issues/02-command-surface-no-toolbar-question.md)); the PRD's three toolbar
placements are recorded deviations. **The menu bar becomes File / Edit / View / Tools / Help** —
View returns because v0.2.0 ships exactly the features it was cut with. The model: commands that
change *what the list shows* live in **View**; commands that change the *Working Copy* live in
**Edit**; everything else follows v0.1.0.

| Menu | Items (additions in **bold**) |
|---|---|
| **File** | unchanged: Apply `Ctrl+S` · Exit `Alt+F4` |
| **Edit** | Add Entry… · Edit Entry… `F2` · Delete Entry `Del` · **Copy `Ctrl+C`** · Move Up `Alt+Up` · Move Down `Alt+Down` · **Fix Issues…** · Undo `Ctrl+Z` · Redo `Ctrl+Y` · Cancel Changes · Refresh `F5` |
| **View** (new) | **Search `Ctrl+F`** · **Filter ▸** (All / With issues / Missing / Relative / Quoted / Duplicate / Empty — `wxITEM_RADIO`) · **Toggle Issues Filter `Ctrl+I`** · **Expanded Values `Ctrl+E`** (`wxITEM_CHECK`) · **PATH Tree… `Ctrl+T`** |
| **Tools** | unchanged: Settings… · Open Backups Folder · Restart as Administrator |
| **Help** | **User Guide `F1`** · About |

**[assembly] decisions in this table**, each with its reason:

- **Edit order**: Copy joins the per-Entry group after Delete Entry (the one read-only member
  closes the group); Fix Issues… sits after the Move pair — a bulk Working-Copy command, before
  the history block (Undo/Redo) it feeds one Checkpoint into.
- **View order**: the two narrowing criteria first in Tab-order sympathy (Search, then Filter
  with its coarse toggle beside it), then the rendering (Expanded Values), then the dialog
  (PATH Tree…).
- **The coarse toggle is its own menu item** — "Toggle Issues Filter" (uk «Перемкнути фільтр
  проблем») — because every shortcut has a menu home and a label is the only place wxdragon can
  carry one; a radio item carrying Ctrl+I would fire that radio selection, not the toggle, and a
  check item would carry a mark that lies whenever a per-type state is active. A plain command
  item with a constant label carries no state; the Filter submenu's radio marks are the state.
- **Final accelerators**: **Ctrl+I** (Filter coarse toggle, 07's proposal confirmed), **Ctrl+T**
  (PATH Tree, 08's proposal confirmed — in the UX guide's conflict-free set), **Ctrl+E**
  (Expanded Values — the key the ticket-16 prototype carried through the user's own NVDA
  verification). Checked against: the v0.1.0 table (no collision), Windows system-wide reserved
  keys (only Win-key combos and the Ctrl+C/Z-class conventions — none taken), NVDA (its
  commands ride the NVDA modifier; plain Ctrl+letter passes through), and wxMSW text-entry
  preprocessing (claims only Ctrl+C/X/V/A — so all four new accelerators fire frame-wide even
  with focus in the Search field, which is intended). Ctrl+I's "italic" convention belongs to
  rich-text editors, which this application is not.
- **Fix Issues… carries no accelerator** (§7): the Settings…/Restore class, not the F2 class.
- **Enablement on the Backups tab**: every View item is disabled there (Search, Filter and its
  toggle, Expanded Values, PATH Tree…), as are Edit → Copy and Edit → Fix Issues… via the
  existing `session: None` model. Help → User Guide is enabled in every state (§9).

**Menu item msgids** **[assembly]** (English is the msgid; ADR-0004): "Search" («Пошук»),
"Filter" («Фільтр»), "All" («Усі»), "With issues" («З проблемами»; the five type states reuse
§7's Issue words), "Toggle Issues Filter" («Перемкнути фільтр проблем»), "Expanded Values"
(«Розгорнуті значення»), "PATH Tree…" («Дерево PATH…»), "Copy" («Копіювати»), "Fix Issues…"
(«Виправити проблеми…»), "&User Guide" («Посібник користувача(&U)» — fixed by ticket 12).

**Mnemonics** stay the code's, gated as before — the i18n completeness gate's per-menu
uniqueness check **re-runs over all of v0.2.0's menu growth at once, in both languages**
(the growth voids the v0.1.0 assignments' proof, not the rule). Proposed English set for the new
menu, for the gate to confirm: View **S**, **F**, **I**, **E**, **T** — unique on paper; the
Ukrainian set is fixed at implementation under the same gate. Release Checklist steps 31 and B12
are likewise voided by the growth and re-run once (§17).

Keyboard map additions (the README table and the User Guide's keyboard table mirror it):
Ctrl+F focus-and-select the Search field; Down/Tab from the field into the list; ESC clear and
return; Ctrl+I coarse filter toggle; Ctrl+E expansion toggle; Ctrl+T PATH tree; Ctrl+C copy
entry; F1 User Guide; Space toggles a checkbox row in Fix Issues. No scenario requires a mouse.

## 13. The Announcement catalogue — closed at fourteen **[assembly]**

The closed set grows 7 → **14**. Items 1–7 are v0.1.0's, two amended below. Canonical English
(msgids); Ukrainian ships in the Catalogue — where a ticket fixed the Ukrainian, it is quoted.

| # | When | English msgid(s) | Ukrainian (where fixed) | Ticket |
|---|---|---|---|---|
| 1 | Scope tab activation and Refresh, **with no Filtered View** — the condition is now two-part: empty query **and** Filter at `All` | unchanged | — | 06 D14, 07 D8 |
| 2–6 | unchanged | — | — | — |
| 7 | Read-only Data at startup | "Read-only: {reason}" — **a fourth reason**: "the --data-dir location cannot be used" **[assembly]** | «розташування, вказане в --data-dir, неможливо використати» **[assembly]** | 13 |
| 8 | Expansion Mode toggled | "Showing expanded values" / "Showing raw values" | «Показано розгорнуті значення» / «Показано збережені значення» | 05 |
| 9 | Filtered count on a criteria change: typing pause, ESC into a still-filtered view, Expansion toggle (second of two), Filter → All with query text | "{n} of {m} entry" / "{n} of {m} entries"; zero case "No matching entries". Plural by **{m}** | «{n} з {m} запису» / «{n} з {m} записів» / «{n} з {m} записів»; «Немає збігів» | 06 |
| 10 | Scope tab activation and Refresh **while that Scope has a Filtered View** | "User PATH: {n} of {m} entry/entries", "System PATH: …"; zero cases "User PATH: no matching entries", "System PATH: no matching entries" — two Scope-named strings, never one frame | «PATH користувача: {n} з {m} записів» (3 forms); «PATH системи: …»; «PATH користувача: немає збігів»; «PATH системи: немає збігів» | 06 |
| 11 | Filter changed to a non-All state (the composed Search∧Filter count — one announcement, never two) | "{filter}: {n} of {m} entries" (plural by {m}); zero case "{filter}: no matching entries" | «{filter}: {n} з {m} записів»; «{filter}: збігів немає» | 07 |
| 12 | Fix Issues applied (after focus lands — last heard is the summary) | "Fixed {n} entries", plural by {n} | «Виправлено {n} запис / записи / записів» | 09 |
| 13 | Copy succeeded | "Copied to clipboard" (no placeholder) | «Скопійовано до буфера обміну» | 11 |
| 14 | Copy failed | "Could not copy to clipboard" (no placeholder) | «Не вдалося скопіювати до буфера обміну» | 11 |

Rules carried with the set: the Expansion toggle with a Filtered View active speaks item 8 then
item 9 **through the same debounced path** — separated by exactly `filteredCountDelayMs`, no
combined msgid (measured reliable at the 250 ms default, ticket 16). Working-Copy changes
recompute membership **silently**. Tickets 08, 09 (beyond item 12), 12, 13 and 14 add **no**
Announcement — in particular, every rung of the User Guide's failure ladder opens the browser and
needs none, and the Tree View's open/close/landing all speak through native focus.

## 14. Catalogue additions beyond the Announcements **[assembly]**

New non-Announcement strings (English is the msgid; uk fixed where a ticket fixed it):

- **Search**: field label "Search:" («Пошук:»).
- **Main list**: the `#` column header "#" (§2.1).
- **Tree View**: titles "PATH Tree — User PATH" / "PATH Tree — System PATH" (two Scope-named
  strings, §11's rule; «Дерево PATH — PATH користувача» / «… — PATH системи» **[assembly]**);
  buttons "Go to entry" («Перейти до запису»), Cancel (existing msgid); group names
  "Unresolved variables" («Нерозв'язані змінні» **[assembly]**), "Relative entries"
  («Відносні записи» **[assembly]**).
- **Fix Issues**: titles "Fix issues — User PATH" / "Fix issues — System PATH" («Виправлення
  проблем — PATH користувача» / «… — PATH системи» **[assembly]**); buttons "Fix selected"
  («Виправити позначені»), Cancel (existing); column headers "#", Path (existing), "Issue"
  («Проблема»), "Action" («Дія») **[assembly]**; action cells "Delete entry" (**reuses**
  Announcement 4's operation msgid — same meaning, same English, per ADR-0004) and
  "Remove quotes" («Прибрати лапки» **[assembly]**); Checkpoint operation "Fixing issues"
  («Виправлення проблем»).
- **Command line** (ticket 13): dialog title "Unknown argument {arg} was ignored" («Невідомий
  аргумент {arg} проігноровано» **[assembly]**); the shared usage line **[assembly]**:
  "Usage: PathMaster.exe [--tab user|system|backups] [--data-dir <path>] [--help]"; the
  `--help` dialog title "PathMaster command line" («Командний рядок PathMaster» **[assembly]**).
- **Menu items**: §12's list.

The i18n completeness gate (registry, plural presence, placeholder integrity, per-menu mnemonic
uniqueness) extends over all of the above by construction — the msgid registry grows, the gate's
rules do not change.

## 15. `settings.json` delta

Settled by 06 (fields), 04 (their existence and defaults), 07/08/09/11/12/13 (each explicitly:
**no field**), folded into §13's taxonomy here.

> **FR-settings-file** (amended) — three new flat `camelCase` fields, deliberately **not** a
> record (§13's "geometry falls back as a unit" is a rule for members with no individual
> defaults; these three have them, and a record would let one typo silently reset all three):
>
> | Field | Type and domain | Default | Meaning |
> |---|---|---|---|
> | `speakFilteredCount` | bool | `true` | whether items 9/10/11 speak |
> | `filteredCountDelayMs` | int, `0`–`5000` | `250` | the debounce before a count speaks |
> | `searchEscapeReturnsFocus` | bool | `true` | ESC in the Search field returns focus to the list |
>
> `0` is a legal delay; the upper bound exists so a typo cannot silently mute a feature the user
> believes is on. **The failure taxonomy gains no new layer**: ordinary field-layer members —
> out-of-domain value → that field's default in memory, the file keeps the raw text, one `WARN`
> line, no dialog, no clamping (§13). The defaults are the user's own verdict from the ticket-04
> NVDA session (snappy 250 ms over the research's 1400 ms — primary-user preference outranks the
> research default; the setting exists precisely so it can be slowed).

**All three get Settings-dialog controls**, labels **[assembly]** (amendable at implementation
like v0.1.0's dialog strings were): "Speak filtered entry counts" («Озвучувати кількість
відфільтрованих записів»), "Delay before speaking the count (ms)" («Затримка перед озвученням
кількості (мс)»), "Escape returns focus to the list" («Escape повертає фокус до списку»). The
dialog's existing rules (§13 as amended by impl 16) extend unchanged: only changed settings are
written, domains are one rule read twice, Read-only Data disables the controls and OK.

Nothing else in v0.2.0 persists: the Search text, the Filter, Expansion Mode, the Tree View and
Fix Issues dialogs, Copy and the User Guide all remember nothing; `--data-dir` is per-launch by
construction. **§3's Data Directory inventory grows by one file**: `data\help.html` (§9) —
`TC-file-structure`'s list becomes `settings.json` (+`.bad`), `backups\`, the two log files, the
probe file, **and `help.html`**; the README's inventory grows the same line.

## 16. StatusBar delta **[assembly]**

Field 0 keeps its two-Scope composition ("{User fragment} | {System fragment}") and its
per-Scope fragment gains the view's narrowing, composed as:

- No Filtered View (query empty, Filter at All): "User PATH: {n} entries ({m} issues)" —
  unchanged.
- Filtered View, Filter at All: "User PATH: {n} of {m} entries ({k} issues)" (06 D17).
- Filter ≠ All: "User PATH: {filter} — {n} of {m} entries ({k} issues)" (07 D9), e.g.
  "User PATH: Missing — 4 of 50 entries (12 issues)" («PATH користувача: Відсутній — 4 з 50
  записів (12 проблем)»).

**The parenthetical never changes meaning** — it counts that Scope's Issues, not the view's: a
filter is a view, the diagnosis is a fact about the data. Search contributes no name — its "why"
is visible in the field itself, one Ctrl+F away. Field 1 (merged length) is untouched: the
merged PATH is a property of the Working Copies, not of what is being shown. The StatusBar
remains command-only (`NVDA+End`), which is what makes the count re-readable after the Banner
has moved on.

## 17. Release Checklist delta

The new surfaces add the steps below (folded into
[the Checklist](../../docs/release-checklist.md) at implementation, numbered there); **steps 31
and B12 and the i18n mnemonic gate are voided by the menu growth** and re-run once, for all of
it together — 31 rewritten (the Help menu now holds two items), B12 re-run as written (Tools is
unchanged, but the bar it sits on gained View).

**Changed v0.1.0 steps**: 2–4 — the row reading gains the leading position number **and the Path
column's header** ("{#}; Path: {path}; Status: …", §2.1 as amended — step 2's healthy Entry now
reads "{#}; Path: {path}" rather than path text alone); 15 — the full Tab cycle now includes the
Search field (tabs → field → list → buttons); 31 — replaced by the Help-menu step below.

**New steps** (expected speech per §13; every NVDA step gated on the Sanity Check as ever):

*Search (06/04):* type in the field → only typing echo while rows rebuild (no chatter, no deaf
signature), then the debounced count once; a query with no matches speaks "No matching entries"
over a zero-row list; Tab and Down-arrow from the field land on a row NVDA reads; Enter in the
field does nothing; ESC clears and returns focus to the list (and with
`searchEscapeReturnsFocus` off, stays); Ctrl+F from anywhere focuses and selects; Ctrl+Tab onto
a Scope with a Filtered View speaks item 10; the item is disabled on the Backups tab; `NVDA+End`
reads the narrowed field-0 text.

*Filter (07):* View → Filter — NVDA distinguishes the selected radio item, and the checked item
follows the active Scope across a tab switch; choosing a type speaks "{filter}: {n} of {m}
entries"; Ctrl+I toggles All ↔ With issues and from a type state back to All; clearing both
narrowings (ESC + All) speaks Announcement 1; the StatusBar names the state while narrowed.

*Expansion Mode (05):* View → Expanded Values reads its checked state both ways; toggling
speaks the mode message; with a Filtered View active it speaks the mode message **and** the
count, both surviving at the configured delay; the Edit dialog carries raw text while the list
shows expanded.

*Tree View (08):* Ctrl+T opens a dialog whose title names the Scope and whose first node
speaks; a compressed chain node speaks its whole joined label with level and position; a
three-part leaf speaks segment, raw parenthetical and Issue suffix without truncation; the
group nodes speak their names; Enter on a leaf (and the "Go to entry" button) closes and the
landed row speaks in full; "Go to entry" reads as unavailable on an inner node; Esc/Cancel
speaks the restored focus; the item is disabled on the Backups tab.

*Fix Issues (09):* Edit → Fix Issues… reads as unavailable when no fixable row exists, on the
Backups tab, on the unelevated System tab, and in Read-only Data; opened, its title names the
Scope and rows read "checked"/"not checked" with their columns; the `%VAR%`-carrying Missing
row starts unchecked; Space toggles with the new state announced in place; [Fix selected] lands
focus first and speaks "Fixed {n} entries" last, with {n} matching the checked count; one
Ctrl+Z restores every fixed Entry ("Undone: Fixing issues"); zero checked closes silently with
nothing to undo.

*Copy (11):* Ctrl+C on a row speaks "Copied to clipboard" and the clipboard holds the displayed
rendering — repeat in expanded mode and the clipboard holds the expansion; with focus in the
Search field Ctrl+C copies the query, not the Entry; on the Backups tab the item reads as
unavailable and Ctrl+C does nothing; close the application — the clipboard still pastes.

*User Guide (12, the eight steps as fixed by the ticket):* Alt+H speaks two items, the first
carrying F1; F1 opens the browser, NVDA speaks "PathMaster {version} — User Guide", `H` walks
the headings; `data\help.html` exists afterwards, in the Interface Language; change language,
restart, F1 — the file is rewritten, no orphan; delete the file, F1 — it returns; the
unwritable-`data\` run sends the browser to the online URL and leaves one `WARN` line; F1 in
the Edit dialog says nothing, the dialog stays open, focus does not move; the item is available
on the Backups tab and in Read-only Data.

*Command line (13, the seven steps as fixed by the ticket):* a fresh-path `--data-dir` launch
creates the directory there with `data\` beside the exe untouched; a relative path resolves
against the shell's CWD; an unusable target speaks the fourth Read-only reason and writes
nothing anywhere; "Restart as Administrator" during an override Run with a spaced path lands
the elevated instance in the same directory; an unknown argument shows the dialog and the app
continues with one `WARN` line; `--help` shows the usage dialog and exits; the startup log line
carries `dataDir:` on override Runs.

## 18. Release mechanics — checked, nothing to decide

The assembly-time check the map's fog note ordered, with its answer: **v0.2.0's release needs no
decision the pipeline does not already make.** The version bump is F2's two files plus the tag,
three-way-gated; the release workflow re-runs the full test gate itself; scoop's Excavator
(F10) picks the release up with no manifest edit; winget stays deferred with its block intact.
The one new release-coupled fact is already recorded where it belongs: the User Guide's fallback
URL is version-pinned and 404s until the tag exists — sound, because the Checklist runs on a
tagged build (§9). The `pulldown-cmark` build-dependency changes nothing in the workflow (the
same `cargo build` compiles it). E2's Process-Monitor expectation is unchanged — `help.html` is
written inside `data\`.

One implementation note, no decision required: the pin stays **wxdragon 0.9.18**. The 0.9.20
delta (a `get_item_text` UTF-8 truncation fix, `CHAR_HOOK`) touches nothing this spec requires —
the application renders from the Working Copy and never reads text back out of a list — and
ticket 16 measured the round-2 prototype on 0.9.20 over the **same wxWidgets 3.3.3**, so the
native layer measured is the one the app ships either way. If implementation finds itself
wanting either API, the upgrade is pre-cleared by ticket 01's delta finding.

## 19. NVDA verification record

Everything this delta's accessibility rides was measured against real NVDA, per the map's standing
constraint — never assumed from an inspector tool. Rounds one and two were the user personally, as
that constraint asks; round three's provenance is stated in its own entry:

- **Round one** ([ticket 04](issues/04-live-filter-nvda-prototype.md), 2026-08-26): rows rebuilt
  under an unfocused list are silent (plain rebuild; Freeze/Thaw dropped); the debounced count
  speaks reliably through the v0.1.0 mechanism; Tab/Down land and read; ESC clear-and-return
  works. Three prototype toggles graduated into §15's settings by the user's decision.
- **Round two** ([ticket 16](issues/16-nvda-verification-round-2.md), 2026-08-27): all seven
  parked obligations discharged, **no contract amended** — check menu items speak their state;
  two successive Announcements survive at the 250 ms default; menu radio items speak selection
  and follow the active Scope; compressed tree nodes and three-part leaves speak in full;
  "Go to entry"/Cancel land and speak; native listview checkboxes read, toggle on Space with
  the change announced, and survive the silent wx event layer; the focus-then-Announcement
  order on [Fix selected] confirmed as designed.
- **Round three** ([impl ticket 02](../pathmaster-v0-2-0-impl/issues/02-index-column.md),
  2026-08-27): the `#` column measured on the built application rather than a prototype, both
  languages. **Provenance differs from rounds one and two**: this was an unattended
  `tools/nvda-drive.ps1` run against a staged copy, read back out of NVDA's own log — the
  harness, not the user at the keyboard. It is a real NVDA measurement and not an inspector
  tool, but it is not a HITL session, and the row reading is worth one keyboard confirmation
  when the Checklist is next walked. **One contract amended** — §2.1's predicted row reading. A
  row reads "{#}; Path: {path}; Status: {types}": NVDA reads the leftmost report column bare, as
  the item's name, and prefixes every other column with that column's header, skipping an empty
  cell. That is the same rule v0.1.0's baseline measured (its ticket 02, "both columns and the
  second column's header name"), applied to a list whose column 0 is no longer Path. Renumbering
  confirmed live on the same run: Alt+Down moved entry 1 and it read back as `2`, Up read `1` as
  the *other* entry, Del renumbered the rest up, Ctrl+Z restored both. No count compensation
  appeared and none was added.

The deaf-list risk posture (§19 of the v0.1.0 spec) is unchanged: v0.2.0 ships zero deaf-state
code, the Sanity Check gates every measurement, and in-app detection stays deferred (§20).

## 20. Cut, deferred, and declined

Nobody re-adds these by accident; each carries its reason.

**Cut in v0.2.0, by decision — not an omission**:

- **FR-reorder-dnd (drag & drop reorder)** — [ticket 10](issues/10-dnd-reorder-right-to-die.md),
  2026-08-26, the user's verdict on the evidence: mouse-only and NVDA-invisible by construction
  (NVDA hears drags only through UIA Drag/DropTarget patterns the app would have to implement —
  a custom UIA provider, far beyond the raw-`LVM_*` hatch); redundant beside the shipped
  Move Up/Down (WCAG 2.5.7's obligation runs the other way); all-bespoke in wxdragon (no
  reorder helper exists at any layer); never promised in the README; absent from Windows' own
  PATH editor. Returns only as a fresh effort if the destination is ever redrawn.

**Deferred beyond v0.2.0**, each with its reason:

- **In-app deaf-state detection** — once-observed, unreproduced, unreported upstream; revisit
  only on field recurrence (v0.1.0 ticket 24's parking, unchanged).
- **The network-path deadline prober** — a dead UNC blocks uncancellably; the cure costs more
  infrastructure than the disease.
- **The winget submission** — deferred indefinitely 2026-08-25, a channel decision; manifests
  stay finished and identity-guarded.
- **Collapsing `ScopeDiagnosis` into `Findings`** — buys depth, not behaviour; nothing in
  v0.2.0 pressed on it.
- **Multi-select Filter states** — 07 chose an exclusive state as a strict subset that can grow
  later without breaking the model.
- **"Show me everything under X" (a prefix filter from the tree)** — named by 08 as a future
  Filter feature, not the Tree View's job.
- **A Ctrl+Insert copy twin** — needs a hidden duplicate menu item; no recorded need (11).
- **Generating the User Guide's keyboard table from the menus' source** — the only thing that
  cannot drift, at the price of a (command, msgid, accelerator) table the product does not
  have; deliberately not bought (12).

**Declined within v0.2.0, decided rather than defaulted** (recorded here because each looks
like an oversight until its ticket is read): the Search-bar coupling from the tree (08); any
severity partition (07); a new Issue type or marker for undefined `%VAR%` (05); a repair for
Relative (09); F1 in dialogs (12); any Announcement for the User Guide's failure ladder (12);
any fallback to the default `data\` from a broken `--data-dir` (13); Expansion Mode, Filter,
Search text or any dialog state persisting (05, 06, 07, 08, 09).

**Out of scope, kept from v0.1.0**: everything its §20 lists — similar-path/typo diagnostics,
the `theme` setting, code signing until there are real users, UI automation, PRD §10's other
variables/sync/plugins/web-CLI/auto-update, non-Windows platforms, 32-bit, screen readers other
than NVDA.

## 21. PRD deviation notes

Where v0.2.0's tickets **override** the PRD outright (rewrites that keep intent are in §1):

1. **The three toolbar placements** (FR-var-expansion-toggle, FR-tree-browser, FR-fix-issues) —
   no toolbar exists; each command has a menu home instead (ticket 02; §12).
2. **FR-filter-bar's "Errors / Warnings" buttons** — no severity classes exist and none are
   minted to power two buttons; the Filter speaks the product's own Issue-type language
   (ticket 07).
3. **FR-var-expansion-toggle's "Warning: Unknown variable" marker** — an undefined `%VAR%`
   stays literal in place and flags `Missing` naturally; no new Issue type, no inline marker
   (ticket 05).
4. **FR-tree-browser's Enter-fills-the-Search-bar and Alt+T** — Enter on a leaf selects the
   Entry's row by identity and closes; the accelerator is Ctrl+T (Alt+letter shortcuts conflict
   with access keys) (ticket 08).
5. **FR-copy-entry's "raw text"** — Ctrl+C copies the currently displayed rendering; the
   default (raw mode) still matches the PRD (ticket 11).
6. **FR-fix-issues' row-per-problem and its network-root default row** — one row per Entry with
   one computed action; network roots are never probed, never flag, and have no row
   (ticket 09).
7. **FR-search's menu home** — View, not the Windows-conventional Edit: what changes the list's
   contents lives in View, chosen knowingly (tickets 02, 06).
8. **The index column's return** is a deviation from the *v0.1.0 spec*, not the PRD — the PRD's
   anchor won by recency; recorded in §2.1.
