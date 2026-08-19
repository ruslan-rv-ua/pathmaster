# 13 — Apply end-to-end

**Spec:** [spec §5 (FR-apply), §4 (TC-wm-settingchange), §7 (over-length dialogs), §8 (FR-backup-auto), §9 (failure taxonomy), §10.1 items 2–3](../../pathmaster-v0-1-0/spec.md) · ADR-0001, ADR-0006

**What to build:** Ctrl+S actually changes the machine, safely: Apply re-reads the Scope, detects external edits, writes a Snapshot of what it just re-read, writes the registry preserving Value Type, moves the Baseline, broadcasts the change, re-runs diagnostics, and announces the outcome — with the full failure taxonomy where no failure mutates the Working Copy or moves the Baseline.

**Blocked by:** 03 (registry adapter), 08 (Sessions in UI, announce), 09 (merged-length thresholds), 10 (Snapshot writing).

**Status:** ready-for-agent

- [ ] Order fixed: re-read → compare `(vtype, bytes)` → (external-change dialog) → back up the re-read value, never the Baseline → write → move Baseline → re-run diagnostics; detection lives only in Apply — no watcher, no polling
- [ ] External-change dialog: title "PATH was modified externally since last refresh", buttons [Overwrite] (proceed; Undo stack survives) / [Refresh and discard my changes] (Working Copy and Baseline become the new value, stacks cleared, nothing written, no backup) / [Cancel] (nothing happens; Session stays dirty and knowingly stale)
- [ ] Over-length gates at Apply: past 8,191 post-write merged length → warning dialog (title per spec, [Apply Anyway] [Cancel]); at ≥ 32,767 → hard-cap dialog, single [Cancel], no proceed
- [ ] Snapshot written to `data\backups\` per the ticket-10 schema and filename rules, temp+rename with `.tmp` mid-write; rotation runs per-Scope after the write
- [ ] Registry write raw via the adapter; first Apply over an Absent Scope creates `REG_EXPAND_SZ`; zero Entries over Present writes an empty string
- [ ] Broadcast off the UI thread: `SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, 0, L"Environment", SMTO_ABORTIFHUNG, 1000–2000, …)`, lParam UTF-16LE NUL-terminated and outliving the call; a 0 return / timeout is not a failure — one `WARN` line, never surfaced
- [ ] Failure taxonomy with exact texts: snapshot-write failure → "Apply failed — could not write a backup, no changes were made."; registry-write failure → "Apply failed — {cause}"; no failure mutates the Working Copy or moves the Baseline; every failure lands one log record with the raw error code
- [ ] Success announces "User PATH applied" / "System PATH applied" (Announcement 2); focus stays on the current Entry; Apply disabled while clean; one `INFO apply:` audit line (Scope, entry count, chars, Value Type — no PATH text)
- [ ] Undo after Apply re-dirties the Session with the ", unsaved changes" suffix (barrier behaviour observable in the UI)
- [ ] Apply never consults the startup writability prediction — it verifies at write time (startup predicts, Apply verifies)
