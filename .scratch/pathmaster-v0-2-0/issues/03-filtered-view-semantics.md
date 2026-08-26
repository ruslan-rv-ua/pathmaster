# Filtered view semantics

Type: grilling
Status: open
Blocked by: —

## Question

Search and the Filter bar both make the ListView show a **subset** of the Working Copy — a state
v0.1.0 never had. Pin down what a filtered view *is* in the domain model before either feature is
specified, because every editing command has to answer to it:

- What is the term, and where does it live? (A view over the Working Copy — never a change to it;
  presumably per-Scope, so switching tabs raises: does each Scope keep its own filter/search state?)
- The PRD fixes two anchors: displayed `#` indexes are the **original positions** (no renumbering),
  and Search + Filter compose with AND logic. Confirm or amend.
- **Editing under a filter** — the hard part. What do Move Up / Move Down mean when the adjacent
  entry is hidden? Is Delete allowed? Add — where does the new entry land and is it visible if it
  doesn't match the filter? Does an edit that makes an entry stop matching make it vanish mid-keystroke?
  The cheap, honest option to weigh first: editing commands are disabled while the view is filtered
  (a filtered view is for *finding*, not editing) — measured against what that costs a long-PATH user.
- Undo/redo and Checkpoints: does a Checkpoint restore filter state, or is filter state outside the
  undo history entirely (like the diagnostic results are)?
- Refresh, Restore, Apply while filtered: what does each do to the view?
- Focus and NVDA: when the visible set changes under the user, where does focus land, and what is
  spoken? (The count announcements are each feature's ticket; the *focus rule* is this one.)

Resolved terms go into `CONTEXT.md`.
