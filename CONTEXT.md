# PathMaster

A portable Windows desktop application that reads, edits and diagnoses the `PATH` environment variable,
built for a screen-reader user first. This glossary is the ubiquitous language of that domain — what the
concepts *are*, not how they are implemented.

## Language

### The run

**Run**:
One execution of the application, from launch to exit. A handful of things are decided once at its start
and never change while it lasts — where the Data Directory is and whether it can be written, whether the
process is elevated, and the Interface Language — and those are properties of the Run rather than of a
Scope or of anything the user is editing. A setting the user can change while the application is running
is not one of them, however much it reads like configuration
([ADR-0010](docs/adr/0010-run-properties-decided-in-one-place.md)).
_Avoid_: Session (that is one Scope's editing state), Startup (that is when a Run's properties are
decided, not the Run itself), Launch

### The values being edited

**Scope**:
One of the two places Windows stores a `PATH` — the current user's, or the machine's. Each has its own
registry value, its own Value Type, its own permissions, and its own Editing Session.
_Avoid_: Tab, Level, Target, Environment

**Value Type**:
Whether a Scope's stored value expands `%VAR%` references or holds them as literal text. It is part of the
data, carried through editing and written back with it, never changed as a side effect of an unrelated edit.
_Avoid_: Format, Kind, Encoding

**Absent**:
The state of a Scope whose value does not exist at all — distinct both from a Scope holding an empty value
and from a failure to read it. Restoring the two requires different writes, so they are never conflated.
_Avoid_: Missing, Null, Empty, Unset

**Entry**:
One `PATH` element: the raw substring between `;` separators, exactly as read or as typed. It has no parsed
structure — whitespace, letter case and a trailing `\` are all part of it and all survive a round trip.
_Avoid_: Path, Item, Row, Segment, Directory

**Normalisation**:
The comparison-time reading of an Entry — one pair of surrounding quotes stripped, letter case
folded, trailing `\` and slash direction reconciled, `%VAR%` expanded. It exists only to answer
questions like "are these two the same?"; its result is never stored and never written, and it never
consults the filesystem.
_Avoid_: Canonicalisation, Cleaning, Sanitising

### Editing

**Editing Session**:
The unit of editing: one Scope's Working Copy, its Baseline, and its Undo/Redo history, held for as long as
the application runs. There is one per Scope and they are independent — one may be dirty while the other is
clean, and neither is writable unless its Scope permits it. A Session never survives a process boundary.
_Avoid_: Session, Context, Document, Buffer

**Working Copy**:
The list of Entries and the Value Type an Editing Session is currently working on — what the user sees and
what every edit changes. Nothing reaches the registry until it is applied.
_Avoid_: Draft, Model, Buffer, Current state

**Baseline**:
The Scope's value as the Editing Session last saw it in the registry — set when the value is read, and moved
forward each time a write succeeds. It is what the Working Copy is compared against and what Cancel returns to.
_Avoid_: Original, Saved state, Last known, Snapshot

**Dirty**:
The property of an Editing Session whose Working Copy differs from its Baseline. It is a comparison of
content, not a record that something happened — an edit and its exact reversal leave the Session clean.
_Avoid_: Modified, Changed, Unsaved (as a flag), Touched

**Checkpoint**:
One entry in an Editing Session's undo history: a complete captured state of its Working Copy, the Entry the
change concerned so focus can return there, and the name of the operation it stands for so undoing it can be
announced — the one thing focus landing on a row cannot say. One user-visible operation produces exactly one
Checkpoint, however many Entries it touched.
_Avoid_: Snapshot (that is a backup file), Undo step, Command, Transaction, Revision

**Apply**:
Writing an Editing Session's Working Copy to its Scope's registry value and moving the Baseline to match.
_Avoid_: Save, Commit, Write, Persist

**Apply Run**:
One pass of the Apply sequence over one or more Scopes, in the order it is given them, stopping at the first
Scope that does not complete — whether it failed or the user cancelled it. Ctrl+S is a run over one Scope; the
close-confirm's Save is one over every dirty Scope, User first. A Scope a run completes is not necessarily one
it applied: the external-change dialog's middle answer adopts the value that was just read and writes nothing
([ADR-0008](docs/adr/0008-apply-sequence-lives-in-platform.md)).
_Avoid_: Save, Batch, Transaction, Apply-all

### Diagnosis and recovery

**Issue**:
A single diagnostic finding about one Entry or about a Scope as a whole — a duplicate, a path that does not
exist, and so on. Issues are derived from a Working Copy and recomputed whenever it changes; they are never
part of it.
_Avoid_: Error, Warning, Problem (those name severity, not the finding), Diagnostic

**Snapshot**:
One saved copy of a single Scope's value, written to a file before that Scope is applied and restorable
later — a JSON file recording the Scope and either its Entries with the Value Type they were stored under,
or that it was Absent, which has neither
([ADR-0006](docs/adr/0006-snapshot-schema-is-decoded-not-raw.md)). Restoring loads a Snapshot into the
Working Copy rather than writing the registry directly, so a Restore is one ordinary Checkpoint and Apply is
what actually writes it.
_Avoid_: Backup (reserve that for the act of taking one and for the directory they live in)

**Corrupted**:
The state of a Snapshot file that fails schema validation — unparsable JSON, or a missing or mistyped field.
Shown as passive text in the Backups list, the same free ride NVDA already gets on other list columns; never
spoken as an Announcement. A Corrupted Snapshot still counts toward its Scope's rotation budget.
_Avoid_: Invalid, Broken, Damaged

### Speaking to the user

**Announcement**:
One of a closed set of messages the application both speaks through the screen reader and shows in
the Banner. Nothing outside that set is spoken unprompted, and no Announcement is audio-only — hearing
and seeing carry the same information.
_Avoid_: Notification, Toast, Alert, Message

**Banner**:
The single visible home of Announcements — one message line in the main window. It never carries information
that is not also spoken, and like everything else it never sets its own colours.
_Avoid_: Status bar (that is command-only and separate), InfoBar, Message area

**Catalogue**:
The single source of every string the user reads or hears — control labels, Announcements, Issue names,
dialog titles and buttons alike. There is exactly one, so what is shown and what is spoken can never drift
apart. The log is deliberately outside it: it is written for a developer reading a machine they cannot see,
not for the user.
_Avoid_: Resources, Strings, Locale files, Translations (those are what a Catalogue holds, not what it is)

**Interface Language**:
The language the application speaks and shows, decided once when it starts — from the user's stored choice,
or from the system when they have made none. Like Read-only Data it is a property of the run, not of a
Scope, and it never changes while the application is running.
_Avoid_: Locale (that is the system's, not ours), Language setting, Culture

### Verification

**Release Checklist**:
The manual, personally-run verification script that gates every release: each step names the action
and the exact text NVDA is expected to speak, and every NVDA step is void unless the sanity check
passes first. A release produces a filled copy — results and the NVDA used — kept as a release
artifact, so a pass is a record, not a ritual.
_Avoid_: Test plan, QA pass, Smoke test, Manual testing (the Checklist is one specific, recorded script)

**Sanity Check**:
The precondition gate on every NVDA measurement: `NVDA+Tab` on a focused list row must answer with the row,
not the list. A step run in a failed-sanity state is void — not failed, void — because a deaf screen reader
is indistinguishable from a regression.
_Avoid_: Smoke test, Health check, Precondition (too generic — this is one specific gesture with one specific expected answer)

### The application's own storage

**Data Directory**:
The single place the application writes: a directory beside the executable, holding the settings, the
Snapshots and the log. It is located from the executable itself rather than from the path the process was
launched through, so it follows the binary wherever a launcher, shim or package manager points at it. Nothing
the application writes lives anywhere else, and there is no setting that moves it.
_Avoid_: Data folder, App data, Working directory, Install directory

**Read-only Data**:
The state of a run whose Data Directory cannot be written. The application still reads, diagnoses and lists,
but every write path is closed and the reason is named. It is decided once at startup and is a property of
the run, not of a Scope — a Scope the user could otherwise edit is uneditable in this state, because nothing
can be backed up before it is applied.
_Avoid_: Read-only mode, Safe mode, Degraded mode, Locked
