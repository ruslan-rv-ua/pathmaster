# Editing session model

Type: grilling
Status: resolved
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

## Carried in from ticket 05

The "what is an Entry" question now has a hard constraint under it, and it reaches the domain model:

- A Scope's registry value carries **both raw bytes and a value type** (`REG_EXPAND_SZ` or `REG_SZ`), and the
  type must be **preserved, never normalised** — normalising either turns a literal `%` in a real directory
  name into an expansion, or silently denies the user new `%VAR%` entries. So the Working Copy owns a value
  type, not just a list of strings, and the model must say what happens when a user types `%VAR%` into a
  `REG_SZ` scope.
- A Scope also has a **third state beyond present-and-empty**: the `Path` value can be *absent*
  (`ERROR_FILE_NOT_FOUND`) on a fresh profile. Name it; several downstream behaviours differ.
- Normalisation is a **comparison-time concept only** — the raw substring is what gets written back. Ticket 05
  catalogues 15 ways a naive implementation produces a *successful* registry write with wrong content, and
  most of them start with normalising or decoding at the wrong moment.

## Answer

Resolved by grilling, 2026-08-18, two rounds. Every recommendation was put to the user and accepted as put.

**The model in one sentence:** two independent **Editing Sessions**, one per **Scope**, each a **Working Copy**
over a **Baseline**, with undo as whole-copy **Checkpoints** and *dirty* defined as a comparison rather than a flag.

Ubiquitous language written to [CONTEXT.md](../../../CONTEXT.md); the undo shape earned
[ADR-0001](../../../docs/adr/0001-checkpoint-based-undo.md).

### 1. Session and Scope

- One **Editing Session per Scope** (User, System), fully independent: separate Working Copy, Baseline, dirty
  state and Undo/Redo stack. Apply, Cancel and Refresh act on the active tab's Session and never touch the other.
  The facts all pointed one way — FR-backup-auto already says each Apply produces exactly one backup *for its
  scope*, System needs elevation and User does not, and ticket 05 established that these are two separate values
  with separate types and a separate Absent state. A shared session would let System's read-only state infect
  User editing.
- **The Backups tab is not a Scope** and has no Session.
- A Session carries **`writable`**, decided at load: User always, System only when the process is elevated. A
  non-writable Session disables **every** editing action — Add, Delete, Move, Edit *and* Apply — not Apply alone,
  because a Working Copy that can never be applied is a trap, and an expensive one without visual cues.
  US-admin-elevation already says the System list is read-only without elevation.
- **A Session never survives a process boundary.** Elevation is a relaunch, therefore fresh Sessions.

### 2. Entry, and the round-trip invariant

- An **Entry** is the raw substring between `;` separators, byte-for-byte as read or as typed — whitespace, case,
  trailing `\` and quotes preserved. No parsed structure in the model.
- **Normalisation is a pure comparison-time function** (case-fold, trailing `\`, `/` to `\`, `%VAR%` expansion).
  Its result is never stored and never written. This is the invariant most of ticket 05's 15 hazards violate.
- Each Entry carries an **opaque id** surviving Move and Edit. It exists for focus restoration after Undo, not
  for the registry.
- **Round-trip invariant, testable as stated:** for an unedited Working Copy, split-on-`;` then join-on-`;`
  reproduces the decoded value exactly — including a trailing `;`, which simply means the last Entry is empty.

### 3. Value Type, and the one deliberate exception

- The **Working Copy owns the Value Type** alongside the Entry list. It is therefore part of the Baseline, part
  of the dirty comparison, and part of every Checkpoint — so a type change undoes like any other edit.
- Committing an Entry containing a `%…%` pair into a `REG_SZ` Scope is **never resolved silently in either
  direction**. That one combination raises a dialog: *"This entry uses %VAR%. The current value type (REG_SZ)
  does not expand variables. [Change type to REG_EXPAND_SZ] [Keep as literal text]"*. Both outcomes are legal,
  both are one Checkpoint, both are announced. This is exactly the "explicit, named convert action" that
  research/05 §2.1 proposed instead of normalising, and it is the only exception to *never change the type* —
  user-triggered, visible and reversible. **The type is never automatically downgraded.**

### 4. Absent

- `ScopeValue = Absent | Present { value_type, raw }`. The name **Absent** matches research/05's own vocabulary.
- Over an Absent Scope the Working Copy is an empty list and the Session is clean.
- The first Apply **creates** the value as **`REG_EXPAND_SZ`** — the type Windows itself uses for every `Path`
  on the research machine, and the choice that avoids hazard H5 permanently denying `%VAR%` in a Scope the app
  just created.
- Apply with zero Entries over a *Present* Scope writes an **empty string**; it never deletes the value.
  Deleting is a bigger hammer than the user asked for and is not a feature.
- **One derived special case, settled without a question because the alternative has no defenders:** an empty
  value decodes to **zero Entries, not one empty Entry**. The naive split would make every empty `PATH` report a
  spurious `Empty entry` Issue on first sight. Joining zero Entries yields the empty string, so the round-trip
  invariant holds.

### 5. Undo

- A **Checkpoint** is a *complete captured Working Copy* (Entries with ids, Value Type, and a focus hint), not an
  invertible command. Rationale and consequences: [ADR-0001](../../../docs/adr/0001-checkpoint-based-undo.md).
- **One Checkpoint per user-visible operation**: Add, Delete, Move Up, Move Down, one confirmed Edit, one type
  change, one Cancel. An Edit abandoned with Escape produces none. A multi-entry batch is one Checkpoint, so
  v0.2.0's Fix Issues needs no extra mechanism.
- Every Checkpoint carries a **focus hint** — the id of the Entry the change concerned — so Ctrl+Z places focus
  where something actually happened. For a screen-reader user this is not cosmetic; it is the only way to learn
  what was undone.
- **Derived rule, settled without a question as the universal convention:** a new operation after an Undo
  truncates the Redo stack.

### 6. Dirty

- **Dirty = Working Copy content differs from Baseline content**, compared exactly (order, raw strings, Value
  Type). Not a flag. Add-then-Delete is clean; Move Down then Move Up is clean; undoing back to the Baseline is
  clean and re-disables Apply.
- The **Baseline** is what was read at Load/Refresh or what was successfully written at the last Apply — which
  makes FR-cancel's "the state after the last Apply, or the initial state at startup" fall out for free, with no
  separate state to store or keep in sync.
- One predicate drives the Apply button, the Cancel button, close-confirm, and the warnings on Refresh and Restore.

### 7. Apply is a barrier, not a stack flush

- FR-undo-redo's "Undo/Redo do not apply to already-applied changes" is read as: **Undo never un-writes the
  registry.** Apply discards nothing; it only moves the Baseline onto what was just written.
- **Ctrl+Z immediately after Apply is legal.** It moves the Working Copy back one Checkpoint, touches the
  registry not at all, and simply makes the Session dirty again — StatusBar says so, and the user may Apply
  again. Flushing the stack instead would make "unbounded within the session" untrue at exactly the moment it
  is most wanted.

### 8. Apply's internal order, and external modification

Apply runs: **re-read, compare, (dialog if different), back up what was just read, write, move Baseline,
re-run diagnostics.**

- The comparison is on **(value type, raw bytes)**, per ticket 05 — never the key's timestamp.
- **The backup captures the value just re-read from the registry, not the Baseline.** Ordering matters and the
  PRD never addresses it: backing up the stale Baseline before overwriting somebody else's change would preserve
  something other than what the write destroys, making the backup worthless in precisely the scenario it exists for.
- The three buttons of FR-apply's dialog:
  - **Overwrite** — proceed. Baseline moves, Undo stack survives (§7).
  - **Refresh and discard my changes** — Working Copy *and* Baseline both become the newly read value; **the
    Undo stack is cleared**; nothing is written and no backup is taken.
  - **Cancel** — nothing happens at all: no write, no re-read, Working Copy and stack untouched. The Session stays
    dirty and knowingly stale.
- **Detection lives only in Apply.** No registry watcher and no polling in v0.1.0; F5 is the user's manual
  re-sync.

### 9. Cancel

- Acts on the **active tab's Session only**. Confirmation ("Discard changes? [Yes] [No]") only while dirty.
- **Rewrites FR-cancel's second clause:** instead of "with no changes, cancels immediately", **Apply and Cancel
  are both disabled while the Session is clean**, driven by the §6 predicate. A button that does nothing gives a
  screen-reader user no signal that there is nothing to do; a disabled one does.
- **Cancel is itself a Checkpoint**, so Ctrl+Z after an accidental Cancel restores the discarded work.

### 10. Refresh (F5)

- **Re-reads and resets the active tab's Scope only.** The tempting alternative — refreshing both, because the
  over-length check spans both Scopes — was rejected: §11 takes that check off the registry entirely, and
  refreshing both would reach into an invisible tab holding unsaved work.
- Confirmation while dirty, as in §9.
- **The Undo/Redo stack is cleared** for the refreshed Session — Checkpoints describe edits over a Baseline that
  no longer exists, and undoing past a Refresh would silently re-arm an overwrite that §8's dialog would no
  longer catch, the Baseline now being fresh. FR-refresh already calls this "resets the current editing state".
- Announcement stays "PATH refreshed" per FR-refresh.

### 11. Diagnostics: a derived view

- **Issues are derived from the Working Copy and are never part of it** — excluded from Checkpoints, recomputed
  after any restore. Otherwise Undo would reinstate a diagnosis of a state no longer on screen.
- Any change to a Working Copy invalidates that Scope's Issues.
- **The merged over-length check takes both Scopes' Working Copies** (expanded), not the registry: the warning is
  about what the user is *about to create*. For a read-only System Session the Working Copy equals the registry
  value, so nothing changes there.
- The rules, thresholds and timing remain ticket 13's; this fixes only the data-flow boundary.

### 12. Close-confirm speaks for two Sessions

- **One dialog for the application**, naming the dirty Scopes explicitly: *"You have unsaved changes in: User
  PATH, System PATH. Save before closing? [Save] [Discard] [Cancel]"*. A generic "you have unsaved changes"
  makes a blind user hunt across tabs for them.
- **Save** applies each dirty Session in turn, **User first**, each through the full §8 path — its own backup,
  its own external-modification check.
- **Partial failure aborts the close.** If any Apply fails or is cancelled (access denied, elevation refused, the
  §8 dialog cancelled), the window stays open, focus moves to the tab that failed and the reason is announced.
  The app never closes on a half-executed intent.
- **Discard** closes and writes nothing. **Cancel** stays open with focus restored.

### Terminology collision found and resolved

The PRD already spends the word **Snapshot** on a backup file ("Restore this snapshot?", the Backups tab), while
this ticket asked for "Snapshot" in the editing model. The user-facing term wins: a **Snapshot** is a backup file,
and the undo step is a **Checkpoint**. Ticket 14 keeps ownership of what a Snapshot must contain.
