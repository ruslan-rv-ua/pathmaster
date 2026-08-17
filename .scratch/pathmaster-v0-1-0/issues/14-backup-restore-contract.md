# Backup and restore contract

Type: grilling
Status: open
Blocked by: 06, 07

## Question

What exactly is a snapshot, and what does Restore do?

- **Rotation.** Does `maxBackups` count files in total or per scope? "Delete the oldest by name" across a mixed
  directory can wipe every System snapshot while sparing fifty User ones — decide deliberately.
- **Naming.** `YYYY-MM-DDTHH-MM-SS.json` carries no scope and no timezone. Two Applies within the same second
  collide. Is the time local or UTC, and does the filename need the scope in it?
- **Restore semantics.** Does Restore write to the registry directly, or load the snapshot into the working
  copy so the user reviews and then Applies? The PRD implies the former; the latter reuses the Apply path,
  inherits the pre-Apply backup, and makes an accidental restore undoable. Decide, and note that this choice
  changes FR-backup-ui's dialog text.
- **Validity.** What makes a snapshot file valid — `timestamp`, `scope`, `entries` present and correctly typed.
  How "[Corrupted]" is announced to NVDA, and whether corrupted files still count toward rotation.
- **Foreign files.** Anything else in `data/backups/` — ignored, listed, or an error?
- **Snapshot contents.** Does it record anything beyond the three fields (app version, machine name)? Each
  extra field must be justified against the portability and no-telemetry promises.
