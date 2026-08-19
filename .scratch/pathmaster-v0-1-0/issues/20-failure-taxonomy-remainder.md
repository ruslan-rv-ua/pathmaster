# Failure taxonomy: settings load and log write

Type: grilling
Status: resolved
Blocked by: —

## Question

What does the user see, what is logged, and what (if anything) is announced when **`settings.json`
fails to load** or **a log write fails**?

Graduated out of the map's **Not yet specified** on 2026-08-19. The rest of the taxonomy is already
settled and constrains this ticket hard:

- **Apply-time failures**: ticket 12's four-row taxonomy and its two invariants (no failure mutates
  the Working Copy, none moves the Baseline).
- **Snapshot/restore-read failures**: ticket 14's Corrupted — schema-validity, all-or-nothing,
  passive list text, never an Announcement.
- The Announcement catalogue is **closed at seven** (ticket 09) and has survived four attempts to
  grow it. Any answer here that needs an eighth Announcement is probably wrong.
- Read-only Data (ticket 07) already covers the *unwritable* Data Directory; this ticket is about a
  directory that is writable but whose contents are bad (malformed `settings.json`) or turn bad
  mid-run (log write starts failing).

Open questions to settle:

- Malformed or partially-valid `settings.json`: all-or-nothing like Corrupted, or per-field fallback
  to defaults? Is the bad file overwritten on next clean shutdown, and is that a data-loss hazard?
- Out-of-range values (a `maxBackups` of `-3`, an unknown `language`): same mechanism or separate?
- A log write failing mid-run: silent, one-time notice, or degrade to no-logging for the rest of the
  run? (A failing log must never take the app down with it.)
- Does any of this appear in the UI at all, or is the log the only witness?

## Answer

Resolved 2026-08-19 in a grilling session, informed by an industry-practice survey (never crash on a
bad config; set the broken file aside rather than overwrite; per-field tolerance for bad values;
logging failures never disrupt the application — the Serilog reliability policy).

**1. Two layers, mirroring ticket 14's Corrupted.** The parse layer is all-or-nothing: unparsable
JSON **or a root that is not an object** → the whole file is treated as unreadable and the run uses
full defaults. A parsable object root proceeds to the per-field layer below. One mental model for
both of the app's JSON file kinds.

**2. Unreadable file: set aside, never overwritten.** The bad file is renamed to `settings.json.bad`
(temp+rename discipline, single copy — the next incident overwrites it), and the run proceeds on
defaults; a fresh `settings.json` then appears through the normal write path (clean shutdown or a
settings change). In Read-only Data no rename happens — no writes — but the dialog below still shows.
This **overrides FR-settings-file**, which wanted the file silently reset in place plus a StatusBar
warning: the StatusBar is unhearable (ticket 02), overwriting a hand-edited file is a data-loss
hazard, and the ticket wins over the PRD.

**3. The user is told about an unreadable file — by dialog, not Announcement.** One startup dialog,
message carried in the title per the ticket 09 discipline: "Settings could not be read — defaults are
in use". Same catalogue-preserving move as ticket 12's declined-UAC dialog; the Announcement
catalogue stays closed at seven. Rationale: the dominant cause of an unparsable file is a hand edit,
and the person whose edit silently didn't take will hunt for why the language never changed.

**4. Invalid values of known fields: per-field default in memory, raw value preserved in the file.**
`maxBackups: -3`, `language: "fr"`, a non-numeric geometry field — each falls back to its own default
for this run, and the file keeps the raw value on rewrite **until the user changes that very setting
in the UI**. This extends ticket 11's "record the choice, not its outcome": a value from a future
version (v0.2's `language: "fr"`) survives a v0.1 run instead of being silently downgraded. Clamping
was rejected — it invents a value the user never chose, and `-3 → 0` would mean "no backups", a
dangerous surprise. Valid domain for `maxBackups` is **≥ 1** (default 50, PRD FR-backup-rotation);
`0` and negatives are invalid because rotation at 0 would silently delete the pre-Apply safety net
the product exists to provide. Invalid `language` falls back to `auto`.

**5. Unknown fields: ignored and preserved.** v0.1 never deletes from the file what it does not
understand — unknown fields ride through every rewrite untouched (cheap in Rust: a flattened raw
map). Same downgrade-hazard reasoning as point 4.

**6. Per-field fallbacks are witnessed by the log only.** One line per fixed field; no dialog, no
Announcement, nothing in the UI. An unreadable file is "your edit did not take" — a dialog; a bad
field is noise — a log line.

**7. A failing log never takes the app down, and every record is an independent attempt.** Axiom
first: no logging failure crashes, blocks, or degrades the application. Each record is tried on its
own — no disabled-for-the-rest-of-the-run latch, because the condition may heal (disk space freed).
A failed write is silently dropped and counted in memory; on the next successful write one extra
line records `N log records were lost`, so the log honestly witnesses its own gap. A log file that
cannot be opened at startup means the run simply has no log — it does **not** trigger Read-only
Data, which remains a property of the Data Directory, not of one file.

**8. An absent `settings.json` is not a failure.** First run: defaults, no dialog, no log line; the
file materialises on the first natural write. Absent, unreadable, and bad-field are three distinct
states — the same discipline as Absent vs empty vs unreadable for a Scope (ticket 05).

Hand-offs: the `N log records were lost` line and the per-field fallback lines are log-format
material for ticket 21; `settings.json.bad` is README material for ticket 22; the FR-settings-file
override goes into ticket 16's PRD-deviation notes.
