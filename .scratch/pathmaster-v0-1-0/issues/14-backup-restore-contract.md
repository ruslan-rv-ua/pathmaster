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

## Carried in from ticket 06

- **Apply's order is now fixed, and it decides what a Snapshot contains**: re-read → compare → (external-change
  dialog) → **back up the value just re-read from the registry, not the Baseline** → write → move Baseline.
  Backing up the stale Baseline before overwriting somebody else's change would preserve something other than
  what the write destroys — worthless in exactly the scenario the backup exists for.
- **No backup is taken on "Refresh and discard my changes"** — nothing is written, so there is nothing to
  protect.
- **A Snapshot must be able to represent states the PRD's three fields cannot.** Beyond H15's value type, it
  needs to distinguish an **Absent** Scope from a present-but-empty one, and **zero Entries** from one empty
  Entry — otherwise restore cannot reproduce what it saved.
- **Restore-into-the-Working-Copy just got cheaper.** Under the Checkpoint model it is one ordinary undoable
  operation, which strengthens the option this ticket was already weighing against a direct registry write.
- **Terminology.** **Snapshot** is the backup file — the term is reserved here, and the editing model's undo step
  is called a **Checkpoint** instead. See [CONTEXT.md](../../../CONTEXT.md).

## Carried in from ticket 07

- **Snapshots are written temp+rename, in the same directory.** Even though a Snapshot never overwrites an
  existing file, an interrupted write must not leave a **half-written Snapshot that still looks restorable** —
  which makes this ticket's "validity" bullet about *corrupt* files, not *truncated* ones.
- **Rotation runs in a world with two instances.** It must tolerate a file another instance has already
  deleted (treat not-found as success), and the `maxBackups` decision has to hold when two processes rotate
  concurrently.
- **Foreign files now include our own.** The atomic-write temporaries live in `data\backups\` for the duration
  of a write. This ticket's "foreign files — ignored, listed, or an error?" bullet must give them a
  recognisable name and skip them in the listing, rather than showing a transient half-file to the user.
- **Read-only Data**: the Backups tab still **lists** existing Snapshots — reading is unaffected — but Restore
  is disabled, because restoring leads to an Apply that cannot take a backup first. This ticket owns what the
  disabled state says.
- **`winget uninstall` deletes the Data Directory, Snapshots included.** Relevant if this ticket wants to say
  anything about the durability of a backup; the README honesty paragraph is ticket 15's.

## Carried in from ticket 09

Backup failures abort Apply and are Announced (ticket 09, D3 — closed catalogue, item 3); this ticket owns
the exact texts. Dialog discipline applies: all critical information in a dialog's **title and buttons** —
NVDA never speaks a `MessageDialog` body.
