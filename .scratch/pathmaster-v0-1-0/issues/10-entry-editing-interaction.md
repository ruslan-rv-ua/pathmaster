# Entry editing interaction

Type: grilling
Status: open
Blocked by: 03, 06

## Question

How does the user edit an entry, given what wxdragon can actually do?

The PRD asserts inline editing (FR-edit-f2) and a Browse button in both Add and Edit (FR-browse-folder).
These two are in tension even if the binding exists: a `wxListCtrl` label editor is a bare text field with
nowhere to put a Browse button.

- If in-place label editing **is** bound: can it carry validation and a Browse affordance, and what does NVDA
  do when an edit field opens over a list row?
- If it is **not** bound, or cannot host Browse: an edit dialog — path field, Browse button, OK / Cancel —
  opened by F2 and double-click. This is very likely both more accessible and simpler, at the price of
  contradicting the PRD's "inline". Decide, and rewrite the requirement rather than pretending both hold.
- **Validation** on confirm: forbidden characters `< > | "`, empty value. Where the error appears, whether it
  blocks the commit, and how it is announced.
- **Add.** The PRD appends an empty entry and opens the editor immediately. If the user cancels, does the empty
  row stay behind — and immediately become an "Empty entry" issue? (It should not.)
- **Duplicates on entry.** Warn while typing, warn on commit, or let diagnostics flag it afterwards? Only one
  of these is consistent with diagnostics being asynchronous.
- Does editing a path that does not exist yet stay legal? (It must — creating the directory later is a normal
  workflow.)

Output: rewritten FR-edit-f2, FR-add-delete and FR-browse-folder.
