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
