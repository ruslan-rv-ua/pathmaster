# 15 — Close-confirm, geometry persistence, clean shutdown

**Spec:** [spec §5 (FR-close-confirm), §12 (geometry), §14 (shutdown line)](../../pathmaster-v0-1-0/spec.md)

**What to build:** Closing the application is safe and remembered: dirty Sessions raise one dialog naming them, Save routes through the full Apply path with partial failure aborting the close, and a clean shutdown persists window geometry and leaves its log line. Reopening restores the window where it was, clamped to real monitors.

**Blocked by:** 07 (settings write path), 13 (Save goes through Apply), [20](20-startup-decisions-module.md) (so geometry clamping lands in a tested module).

**Status:** ready-for-agent

- [ ] One dialog for the application, title naming the dirty Scopes ("Unsaved changes in: User PATH, System PATH — save before closing?"), buttons [Save] [Discard] [Cancel]; clean Sessions close with no dialog
- [ ] Save applies each dirty Session in turn, User first, each through the full Apply path (external-change detection, backup, taxonomy included)
- [ ] Partial failure aborts the close: window stays open, focus moves to the failed tab, the reason is announced
- [ ] Geometry (position, size, maximised state) written to settings.json on clean shutdown only, via the atomic-replace helper, preserving unknown fields; not written in Read-only Data
- [ ] On startup geometry is restored clamped to the connected monitors' work area; fully off-screen → default size centred on primary
- [ ] Clean shutdown logs `INFO shutdown: clean`
- [ ] Dialog strings in the Catalogue with Ukrainian translations
