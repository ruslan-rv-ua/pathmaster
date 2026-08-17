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

## Carried in from ticket 05

**The PRD's snapshot shape may be too lossy to restore from.** `{ timestamp, scope, entries: [...] }` stores a
decoded, split list of strings — which discards the value **type** (`REG_EXPAND_SZ` vs `REG_SZ`) and any
byte-level detail of the original value. Ticket 05 lists this as hazard H15: a backup that cannot reproduce
what was actually in the registry makes every other corruption mode unrecoverable, which defeats the entire
point of backing up before Apply.

Decide what a snapshot must carry to be a **faithful** restore source — at minimum the value type alongside the
entries, and possibly the raw value itself — and weigh that against the file staying human-readable, which is
part of why JSON was chosen. This is the ticket where those two goals are traded off explicitly.
