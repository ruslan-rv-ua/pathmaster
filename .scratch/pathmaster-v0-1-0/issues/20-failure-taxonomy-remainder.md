# Failure taxonomy: settings load and log write

Type: grilling
Status: open
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
