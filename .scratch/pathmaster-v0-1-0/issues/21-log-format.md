# Log format

Type: grilling
Status: open
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
