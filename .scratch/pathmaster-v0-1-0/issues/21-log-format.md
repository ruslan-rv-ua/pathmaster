# Log format

Type: grilling
Status: resolved
Blocked by: 20

## Question

What does one log record contain, and how does the file read?

Graduated out of the map's **Not yet specified** on 2026-08-19. Already settled around it:

- **Rotation**: one generation, rotated only at open (ticket 07); append one line per record.
- **Language**: always English, deliberately outside the Catalogue (ticket 11) — written for a
  developer reading a machine they cannot see.
- **Scope**: logging in v0.1.0 is *minimal* (charting constraint 9).

To decide, once ticket 20 fixes what failures are log-worthy:

- Line shape: timestamp format, level, message — structured (JSON lines) or human-first plain text?
- What is logged at all in a healthy run (startup facts? Apply outcomes?) versus only on failure —
  "minimal" needs a boundary.
- What must **never** be logged (the PATH value itself is user data on a shared machine — does it
  appear verbatim, truncated, or never?).
- Does a crash/panic reach the log, and how (the `panic=abort` build profile from ticket 04
  constrains what a panic hook can do)?

## Answer

Resolved 2026-08-19 in a grilling session (two rounds, all recommendations accepted). Per map
note 13, an internet best-practices pass preceded the questions; the industry's JSON-first advice
was found to rest entirely on aggregation pipelines this app will never have, so it was set aside
with reasons rather than followed.

**Line shape.** Human-first plain text, one record per line:

```
<timestamp> <LEVEL> <area>: <message>
```

The prefix (timestamp, level, area) is machine-stable so `grep` and eyes both work; the message
tail is free-form English. No JSON, no key-value framing — the log's only readers are a developer
in a text editor and a bug report on GitHub.

**Timestamp.** RFC 3339 in **local time with numeric offset** (`2026-08-19T15:36:31+03:00`).
Unambiguous (the offset is present), matches the local-time Snapshot filenames from ticket 14, and
matches how the user reports time. UTC only pays when correlating machines; there is one machine.

**Levels.** Exactly three, padded to five characters so columns align:

- `INFO ` — the healthy-run skeleton (startup, successful Apply, clean shutdown).
- `WARN ` — anything the app survived by itself: per-field settings fallback, `settings.json.bad`
  set-aside, `N log records were lost`, external registry edit detected, broadcast timeout.
- `ERROR` — a user-requested operation failed: Apply failure, panic.

**Healthy-run boundary (the "minimal" line).** A healthy session writes a 3–5 line skeleton, not
an empty file (an empty file is indistinguishable from a log that failed to open):

- Startup: `INFO startup: PathMaster 0.1.0, elevated: no, data: writable, language: uk` —
  version, elevation, Data Directory state, Interface Language. The version line is the only way a
  pasted log identifies its build.
- Apply success: `INFO apply: User scope written, 14 entries, 512 chars, REG_EXPAND_SZ` — Scope,
  Entry count, length, Value Type. Apply is the one system-mutating operation; it earns an audit line.
- Apply failure: `ERROR apply: User scope failed, <taxonomy row>` — the ticket-12 row name verbatim
  (snapshot-write / registry-write / external-edit). Broadcast timeout is not a failure (ticket 12):
  `WARN apply: settings broadcast timed out`.
- Clean shutdown: `INFO shutdown: clean`. A killed process shows as the line's absence; a panic
  shows as the ERROR above it.

**Never logged.** Two absolute prohibitions, both PII-driven (`PATH` and filesystem paths carry
`C:\Users\<name>`):

1. **No Entry text and no PATH value, in any record.** Only derived facts: Entry counts, lengths,
   Value Type, Scope name.
2. **No absolute filesystem paths, in any record.** The startup line reports the Data Directory's
   *state* (`data: writable` / `data: read-only`), never its location — the local reader is already
   in that directory, and a bug report should not leak the username.

Rejected raw `settings.json` values ARE logged (ticket 20 makes the log their only witness) but
**truncated to ~100 characters** with a truncation marker, guarding against a pathological file
putting a megabyte on one line.

**Panic.** A `std::panic::set_hook` hook (runs before abort even under `panic=abort`) writes one
`ERROR panic:` line — panic message plus `file:line`, **no backtrace** (the PDB is not shipped, so
frames would be bare addresses). The hook appends directly to the file, best-effort, swallowing
errors — deliberately bypassing the logger infrastructure so a panic inside the logger cannot recurse.

**File and rotation.** `pathmaster.log` in `data\`. At open only (ticket 07): if the file exceeds
**1 MB**, rename to `pathmaster.log.old`, overwriting any previous `.old`. One generation; at
minimal-logging rates 1 MB is years of history and both files open instantly in anything.

**Ticket 20's three record kinds, mapped.** Per-field fallback → `WARN settings: field
"maxBackups" invalid (raw: "0"), using default 50`; unreadable settings → `WARN settings:
settings.json unreadable, renamed to settings.json.bad, defaults in use`; lost records →
`WARN log: 3 records were lost`. The logger is just another area (`log`), which is how the format
satisfies "must not assume every record originates in application code" — no special syntax exists.

No ADR: the format is cheap to change before 1.0, so it fails the hard-to-reverse test. No new
glossary term: "the log" already appears in CONTEXT.md's Catalogue entry as deliberately outside it.

## Comments

**2026-08-19, claim takeover:** the 13:43 session claimed this ticket but never resolved it while
two later sessions closed tickets 22 and 24; treating that claim as stale. This session takes over,
starting with an internet best-practices pass (map note 13) before grilling.

**2026-08-19, from ticket 20 (failure taxonomy):** now unblocked. Ticket 20 hands this ticket three
record kinds the format must carry: (1) one line per settings field that fell back to its default
(name the field, the rejected raw value, and the default used — the log is the *only* witness of
these); (2) the unreadable-settings event (file renamed to `settings.json.bad`, defaults in use);
(3) the `N log records were lost` line written on the first successful write after a run of
failures — note this line is generated by the logger itself, so the format must not assume every
record originates in application code. Also settled there: a failed log write is silently dropped
and counted, never retried in a loop and never surfaced in the UI, so the format needs no
error-reporting channel of its own.
