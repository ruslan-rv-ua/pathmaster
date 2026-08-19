# 14 — Backups tab and Restore

**Spec:** [spec §8 (FR-backup-ui), §15 (Open Backups Folder)](../../pathmaster-v0-1-0/spec.md) · ADR-0006

**What to build:** The Backups tab lists every Snapshot on disk and Restore brings one back — into the Working Copy as one ordinary, undoable Checkpoint, never straight to the registry. Demoable with hand-placed Snapshot files before Apply exists in a build, and with real ones after.

**Blocked by:** 08 (UI shell + Sessions), 10 (schema + validation).

**Status:** ready-for-agent

- [ ] The Backups tab lists Snapshots: date/time, Scope, entry count; files failing the two-layer validation show `[Corrupted]` as passive list text — never an Announcement — with Restore disabled per-row; foreign files (wrong name pattern, `.tmp`) are silently invisible
- [ ] Restore loads the chosen Snapshot's Entries and Value Type into the target Scope's Working Copy as one ordinary Checkpoint — it never writes the registry directly; no confirm dialog (undo is the safety net)
- [ ] After Restore the target Scope's tab is activated with focus on the restored list, so the operation is heard through focus; undo of a Restore announces "Undone: Restore snapshot"
- [ ] Restore to a non-writable Session (System unelevated; Read-only Data) is a disabled control that reads as disabled
- [ ] Tools → Open Backups Folder opens the directory via a shell invocation, not a file dialog
- [ ] New strings in the Catalogue with Ukrainian translations
