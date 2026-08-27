# 09 — Fix Issues dialog

**Spec:** [delta-spec §7, §13 (item 12), §14 (strings)](../../pathmaster-v0-2-0/spec.md)

**What to build:** Edit → "Fix Issues…" opens a modal, per-Scope repair surface: one checkbox row per fixable Entry with one computed action (Delete entry, or Remove quotes), Disk-Cleanup defaults, and one Checkpoint on [Fix selected] — "Fixed {n} entries" spoken after focus lands. Nothing reaches the registry; only the Working Copy changes.

**Blocked by:** 01 (retrofit), 02 (the `#` column meaning the dialog reuses).

**Status:** ready-for-agent

- [ ] Fixable = three deletions + one repair: Missing, Duplicate, Empty propose **Delete entry**; Quoted proposes **Remove quotes — every `"` in the Entry**; Relative gets no repair and Relative-only Entries are excluded; Over-length is excluded entirely — no row, no reminder text
- [ ] One row per Entry, one computed action: the Issue column carries the comma-joined Status string; the action is Delete entry when any of Missing/Duplicate/Empty is flagged (deletion cures Quoted too), else Remove quotes
- [ ] Columns # / Path / Issue / Action; `#` is the original position (§2.1 convention); Path is always the raw text, whatever the Expansion Mode
- [ ] Checkboxes are native `LVS_EX_CHECKBOXES` through the raw-`LVM_*` hatch; check state is read once, by `LVM_GETITEMSTATE` at apply time — no check events; Space toggles with the change announced in place (rides the native path, measured in wayfinder ticket 16)
- [ ] Defaults — the Disk Cleanup principle: ON for Remove quotes, Delete via Duplicate or Empty, and Delete via Missing on a `DriveType=Fixed` local root with no `%VAR%` in the raw text; OFF for Delete via Missing when the raw text contains `%VAR%` or the root is a non-Fixed drive; network roots are never probed, never flag, and have no row
- [ ] Buttons [Fix selected] [Cancel] — "Apply" nowhere in a label; title names the Scope; initial focus on the first row; [Cancel] keeps default and Escape; no Select-all/Clear-all; zero rows checked at activation = Cancel — no Checkpoint, no Announcement, button never dynamically disabled
- [ ] Applying = one Checkpoint in the active Session, operation name "Fixing issues" («Виправлення проблем»); focus first (Delete's law: same index clamped to the new last row), then Announcement 12 "Fixed {n} entries", plural by {n}, both languages; one Ctrl+Z restores every fixed Entry; re-diagnosis follows the existing recompute-after-every-change law
- [ ] Enablement: Edit → "Fix Issues…" (after the Move pair, before the history block, **no accelerator**), disabled on the Backups tab; enabled iff the active Scope has ≥ 1 fixable row AND its Session is writable (System unelevated and Read-only Data disable it); menu enablement is the only indicator
- [ ] Staleness, both halves: at open, the dialog builds only from a diagnostic pass whose generation stamp equals the current Working-Copy generation — if none exists yet, the command waits for the outstanding pass (< 1 s budget, no spinner); after open, modality is the fence — apply resolves checked rows to Entries by id, never by index, and asserts the generation unchanged
- [ ] Catalogue strings shipped in both languages: the two titles, "Fix selected", "Issue", "Action", "Remove quotes", the "Fix Issues…" menu item; "Delete entry" reuses Announcement 4's operation msgid; Cancel, Path, `#` reuse existing msgids; i18n gate green
- [ ] No `settings.json` field — nothing about the dialog persists
