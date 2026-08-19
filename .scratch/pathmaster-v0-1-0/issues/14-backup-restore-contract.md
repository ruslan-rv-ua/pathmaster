# Backup and restore contract

Type: grilling
Status: resolved
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

## Answer

**Rotation is per-Scope.** `maxBackups` is an independent budget for System and for User — never a single
count pooled across `data\backups\`. A pooled count lets one Scope's rotation starve the other (fifty User
Applies silently wiping every System Snapshot), which is exactly the failure the industry's own retention
practice (tiered/GFS-style schemes never pool categories) exists to avoid, and exactly what this ticket's
question already flagged as the risk to decide deliberately.

**Filename: `YYYY-MM-DDTHH-MM-SS-<Scope>.json`, local time, numeric-suffix on collision.** Scope goes in the
name (`System`/`User`) so both rotation and the Backups list can identify a file's Scope without parsing its
contents — load-bearing now that rotation is per-Scope. Local time, not UTC: this is a single-machine,
single-user, non-syncing portable tool, and the Backups list is read by that one person, for whom local time
is legible and UTC buys nothing. Two Applies inside the same second append a numeric suffix
(`...-System-1.json`, `...-System-2.json`) rather than adding sub-second precision to the visible name, which
would expose an implementation detail (clock resolution) in a user-facing filename for no benefit.

**Snapshot schema carries Value Type and an explicit Absent marker; entries stay decoded strings, not raw
bytes** — [ADR-0006](../../../docs/adr/0006-snapshot-schema-is-decoded-not-raw.md). This closes ticket 05's
hazard H15 (a backup that cannot reproduce the Value Type it captured cannot be a faithful restore source) and
satisfies ticket 06's requirement that a Snapshot represent Absent and zero-Entries as distinct, real states —
without giving up the human-readable JSON the PRD's file format was chosen for.

```json
{ "timestamp": "2026-08-19T14-32-07", "scope": "System", "valueType": "REG_EXPAND_SZ",
  "entries": ["C:\\Windows", "%JAVA_HOME%\\bin"] }
```
```json
{ "timestamp": "2026-08-19T14-32-07", "scope": "System", "absent": true }
```

**Restore loads into the Working Copy; it never writes the registry directly.** The chosen Snapshot's Entries
and Value Type replace the current Working Copy's, exactly as a Checkpoint would — one ordinary undoable
operation under ticket 06's model, not a new code path. This is also the dominant restore-UX shape elsewhere
(preview/stage before commit, not blind direct-write), and it is what the ticket's own question already leaned
toward: the user reviews and Applies as usual, an accidental Restore is undoable with Ctrl+Z like any other
edit, and Restore inherits the pre-Apply Snapshot Apply always takes — so restoring from a Snapshot is itself
backed up. Restore stays disabled in Read-only Data (ticket 07): loading into the Working Copy changes nothing
about *that*, since the block is on Apply eventually needing to write a fresh Snapshot it cannot.

**Validity is schema validity, checked in two layers, all-or-nothing.** (1) the file parses as JSON; (2) every
required field is present with the right shape — `timestamp` a string, `scope` one of `System`/`User`,
`valueType` one of `REG_SZ`/`REG_EXPAND_SZ`, and exactly one of `entries` (array of strings) or `absent`
(`true`) present. Any failure at either layer — unparsable JSON, a missing field, a field of the wrong shape —
makes the file **Corrupted** (`CONTEXT.md`). There is no partial-recovery attempt and no guessing a missing
field's value: a Snapshot exists to be trusted completely or not used at all.

**Corrupted surfaces as passive list text, never an Announcement.** Ticket 09 already closed the Announcement
catalogue at seven items, none of which is "a Snapshot failed to load" — so `[Corrupted]` is shown in the
Backups list the same way the Status column is: comctl32 reads it for free when the row gets focus, no
`announce()` call, no new catalogue entry. Restore stays available for other rows; the disabled state is
per-row (a Corrupted Snapshot cannot be restored — nothing to load).

**A Corrupted Snapshot still counts toward its Scope's rotation budget.** Its Scope is read from the filename,
which is unaffected by content corruption (naming decided above), so rotation identifies and, when it is the
oldest, deletes a Corrupted file exactly like a valid one. It is not a useful restore source, so exempting it
from rotation would let corrupted files accumulate outside the budget rotation exists to enforce.

**Foreign files are silently ignored; atomic-write temporaries are named to be skipped, not parsed.** Anything
in `data\backups\` that does not match the Snapshot filename pattern is invisible to the app — not listed, not
an error, never treated as Corrupted (Corrupted is reserved for files that *look* like a Snapshot and fail
validation, not for files that were never claiming to be one). The temp files ticket 07's atomic
temp-then-`MoveFileExW` write produces get a non-`.json` extension (`.tmp`) so both the Backups list and
rotation filter them out by extension alone, without ever attempting to parse a file that is, by construction,
mid-write.

**Announcement text (ticket 09, D3, item 3 — Apply-time backup-write failure, not a Restore-time Corrupted
read):** title "Backup failed", body/Announcement text "Could not write a backup before applying — no changes
were made." Dialog discipline (title/buttons only) is unaffected; this is the Announcement text, not a
`MessageDialog`.

## Carried forward

- **Ticket 05 (H15)** is closed: the schema (above) preserves Value Type.
- **Ticket 06**'s requirement that a Snapshot represent Absent and zero-Entries as distinct states is satisfied
  by the explicit `absent` field and `entries: []`.
- **Ticket 07**: rotation now tolerates files another instance already deleted (existing decision, unaffected)
  and, additionally, corrupted ones — deleting a file that fails to parse is still "tolerate not-found," since
  the delete only needs the filename.
- **Ticket 09**: the closed seven-item Announcement catalogue is unaffected; Corrupted is a list-text state,
  not an eighth item. The backup-failure text above is D3's item 3, now drafted.
- **Ticket 13 / 17**: the Backups tab gains a text-only status affordance (`[Corrupted]`) — same shape as the
  main list's Status column, no new UI mechanism.
