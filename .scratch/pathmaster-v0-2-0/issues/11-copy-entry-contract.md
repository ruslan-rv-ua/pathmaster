# Ctrl+C copy entry

Type: grilling
Status: resolved (2026-08-26)
Blocked by: 01

## Question

FR-copy-entry: Ctrl+C on a selected row puts the entry's text on the clipboard. Small, but it has
edges. With the clipboard facts from 01:

- **Raw text** (with `%VAR%`, per PRD) — or does the expansion toggle (05) change what Ctrl+C
  copies? Decide once: copy-what-is-stored, or copy-what-is-shown.
- Ctrl+C's owner: the accelerator must fire only when the list has focus (a text field's own Ctrl+C
  must keep working). How the accelerator is scoped, and its menu home (per 02's model — Edit menu?).
- Multi-select: the v0.1.0 list is single-select — confirm that stands, so "the selected entry" is
  well-defined.
- Confirmation: PRD wants a spoken confirmation — new closed-set Announcement, exact wording both
  languages; and what (if anything) is announced when there is no selection.
- Clipboard failure (locked by another app) is real but rare: announced, or silently retried, or
  ignored? The v0.1.0 failure-taxonomy style says name it or rule it out loud.

## Input from ticket 05 (2026-08-26)

Expansion Mode (app-wide raw/expanded rendering, `CONTEXT.md`) is decided; copy-raw vs
copy-as-shown stays this ticket's call. Finding to weigh, from
[research/05](../research/05-var-expansion-best-practices.md) (cross-cutting): every strong
analogue binds extraction and mutation to the raw text — Excel copies the formula and its Replace
operates on formulas only, while display stays a read-only projection.

## Input from ticket 06 (2026-08-26)

Ticket 06 put a permanent, focusable `TextCtrl` in every Scope tab — so the Ctrl+C scoping bullet is
no longer hypothetical: there is now a text field one Tab stop above the list whose own Ctrl+C must
keep copying the query. The accelerator has to be scoped to the list rather than to the frame, and
the Backups tab (which has no Search field and no Filtered View) is a third state to answer for.

## Resolution (2026-08-26)

Researched first: [research/11-copy-entry-best-practices.md](../research/11-copy-entry-best-practices.md),
per the map's standing directive 7. Decisions:

1. **Copy-what-is-shown.** Ctrl+C puts the focused Entry's **currently displayed rendering** on the
   clipboard — raw text in raw mode, the expanded reading in expanded mode (Expansion Mode's own
   reading: unknown `%VAR%` stays literal). The PRD's "raw text" is thereby amended the same way
   06 amended Search: every Run starts raw, so the default behaviour still matches the PRD, and the
   Expansion toggle becomes the one way to extract an expanded value from the application at all
   (PowerShell does not expand `%VAR%` on paste; Excel's own plain-text clipboard format carries
   the displayed value, not the formula). Exactly one Entry, exact text fidelity — no quotes added
   (Explorer's Copy-as-path quoting is a command-line transform, not a model; an Entry's own quotes
   are content). Under a Filtered View, the focused **visible** Entry (already 03's law).
2. **Scoping is the platform's own** (research §2, the decisive finding): wxMSW text entries claim
   Ctrl+C/X/V/A before accelerator translation (`wxMSWTextEntryShouldPreProcessMessage`, pinned
   3.3.3 source), so the menu-label accelerator — the only mechanism there is — never steals the
   Search field's or a dialog field's copy. No focus-checking handler, no dynamic tables. The
   command is otherwise **frame-wide** like every v0.1.0 Entry command: focus on a button still
   copies the active Scope's focused Entry.
3. **Menu home: Edit → Copy**, accelerator `\tCtrl+C`; enablement by the existing availability
   model — `session: None` disables it on Backups exactly as Edit/Delete. No second Ctrl+Insert
   chord (it would need a hidden duplicate menu item; no recorded need). Exact label strings,
   mnemonics and the final accelerator table → assembly (15).
4. **Confirmation — Announcement 13**: "Copied to clipboard" (uk «Скопійовано до буфера обміну»),
   fixed text, no echo of the payload (design-system practice: GOV.UK, PatternFly; the row was
   just read by focus, and Entries run long). NVDA itself announces nothing for app-side copies
   (nvda#75 open since 2010), so the application must speak or the gesture is silent.
5. **No selection = silent no-op** — the exact precedent of v0.1.0's `edit`/`delete`
   (`let Some(...) else { return }`). No new Announcement, no selection-tracking enablement.
6. **Failure — Announcement 14, no retry**: a failed `set_text` announces "Could not copy to
   clipboard" (uk «Не вдалося скопіювати до буфера обміну») immediately. The failure-taxonomy law
   (name it or rule it out loud) plus the accessibility argument: for a blind user the Announcement
   is the only channel, and a copy that silently did nothing is indistinguishable from a missed
   keystroke. Success speaks, failure speaks; silence only ever means "nothing was selected".
7. **The copy outlives the Run**: after a successful `set_text`, `flush()` — best-effort, its own
   result never announced (a failed flush merely restores stock wx behaviour). Without it the
   clipboard empties when the application exits, and this is precisely the tool a user copies from
   and then closes.
8. **Single-select stands** — reaffirmed (v0.1.0 NVDA baseline chose it; 03 built on it); "the
   selected entry" is well-defined and Ctrl+C always copies exactly one.
9. **Nothing persists, nothing configures** — no `settings.json` field. **No new NVDA
   obligation** — both new Announcements ride the mechanism v0.1.0's ticket 08 proved; ticket 16
   gains nothing.

The Announcement catalogue grows from twelve to **fourteen** — two new msgid pairs, neither
carrying a placeholder, so no plural forms.
