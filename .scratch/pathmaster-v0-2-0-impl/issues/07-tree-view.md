# 07 — Tree View

**Spec:** [delta-spec §6, §14 (strings)](../../pathmaster-v0-2-0/spec.md)

**What to build:** View → "PATH Tree…" (Ctrl+T) opens a modal, per-Scope comprehension surface: the active Scope's Filtered View snapshotted at open, merged by the expanded reading into a prefix tree with compressed chains and audible three-part leaf labels. Enter on a leaf (or "Go to entry") selects that Entry's row in the main list — by identity, never by text — and closes.

**Blocked by:** 03 (the Filtered View the snapshot is taken of).

**Status:** ready-for-agent

- [ ] Menu item View → "PATH Tree…" with Ctrl+T, disabled on the Backups tab; dialog title names the Scope ("PATH Tree — User PATH" / "… — System PATH"), both languages
- [ ] Content: the active Scope's Filtered View snapshotted at open (whole Working Copy when unnarrowed); the dialog never touches the narrowing criteria; snapshot — no live diagnostics, no refresh affordance, no timer in the modal's event loop; reopening is the refresh
- [ ] The merge algorithm lives in `pathmaster-core` and is unit-tested: Entries merged by the expanded reading (Normalisation's own, undefined `%VAR%` literal, independent of Expansion Mode) into a prefix tree; single-child chains compress into one node with the joined label; siblings sort alphabetically case-insensitive; "Unresolved variables" and "Relative entries" top-level groups sort after the drive roots and hide when empty; no artificial super-root; one leaf per Entry — duplicates are sibling leaves
- [ ] Leaf label is the whole audible payload: segment/joined chain + raw form in parentheses only when it differs + Issue suffix in the exact Status-column words only when an Issue exists (`bin (%JAVA_HOME%\bin) — Missing`); inner nodes and groups carry no suffixes
- [ ] Interaction: Enter on a leaf selects that Entry's row in the main list by Entry identity and closes; Enter on an inner node expands/collapses via the native default action — the `ITEM_ACTIVATED` handler is the single home of the commit logic; the landed row speaks in full and Cancel speaks the restored focus
- [ ] Buttons "Go to entry" (default; disabled while an inner node or group is selected) + Cancel; Esc closes; no OK, no Close; tab order tree → Go to entry → Cancel; initial focus on the first top-level node
- [ ] Widget: wxdragon `TreeCtrl` — the native `SysTreeView32`
- [ ] No new Announcements, no `settings.json` fields; the dialog remembers nothing — expansion state not preserved
- [ ] Catalogue strings shipped in both languages: the two titles, "Go to entry", the two group names; Cancel reuses the existing msgid; i18n gate green
