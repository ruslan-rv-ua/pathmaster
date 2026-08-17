# Editing session model

Type: grilling
Status: open
Blocked by: —

## Question

What is the editing model — the working copy, the dirty state, and the exact boundaries of Undo?

The PRD assumes this model without ever stating it. Everything downstream (diagnostics, backups, elevation,
close-confirm) hangs off the answer, which is why it sits on the frontier.

Open:

- Is there **one working copy per scope** (User, System) or one shared session? Does Apply on the User tab
  touch the System tab's dirty state? Does the close-confirm dialog speak for both at once?
- **Undo stack**: per scope or global? FR-undo-redo says undo "does not apply to already-applied changes" —
  does Apply *clear* the stack or merely mark a boundary? What happens on Ctrl+Z immediately after Apply?
- **Cancel**: per tab or global? FR-cancel restores "the state after the last Apply" — per scope, presumably,
  but say so.
- **Refresh (F5)**: both scopes or the active one?
- **Granularity**: is each Add / Delete / Move / Edit exactly one undo step? The model must also admit a
  multi-entry batch as a single step, because Fix Issues (v0.2.0) will need it.
- **What is an Entry?** The raw registry substring, or a parsed value? FR-diag-duplicates requires the original
  string be written back byte-for-byte, so normalisation must be a comparison-time concept only — confirm and
  name it.

Use `/domain-modeling`. Output: the ubiquitous language in `CONTEXT.md` at the repo root — Entry, Scope,
Working Copy, Snapshot, Issue, Session — plus the state model in the ticket answer.
