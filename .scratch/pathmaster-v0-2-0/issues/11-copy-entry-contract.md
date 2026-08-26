# Ctrl+C copy entry

Type: grilling
Status: open
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
