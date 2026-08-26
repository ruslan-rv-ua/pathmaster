# --data-dir switch

Type: grilling
Status: open
Blocked by: —

## Question

Parked by v0.1.0's ticket 07 as the one relocation-adjacent feature that survives the no-relocation
principle: a command-line switch carries the Data Directory location **per launch** rather than
remembering it. Specify:

- Syntax and validation: `--data-dir <path>` — what happens on a missing directory (create? refuse?),
  a relative path (resolve against what?), a file, an unwritable target (Read-only Data applies?).
  Every branch lands in the v0.1.0 startup failure taxonomy — extend it, don't fork it.
- **Elevation must forward it.** ADR-0005 relaunches the whole app elevated; the relaunch command
  line has to carry the switch or the elevated run silently writes elsewhere. Same for any future
  restart path (language change is restart-based).
- Interaction with the "located from the executable" rule (ADR-0002): the switch overrides it for
  one Run — confirm the CONTEXT.md "Data Directory" and "Run" entries absorb this without a new term.
- Unknown/malformed arguments generally: v0.1.0 has no CLI at all, so this ticket incidentally
  decides the app's whole argument-handling posture (ignore? dialog? log?).
- Does it appear in --help output… which doesn't exist for a GUI app — decide how the switch is
  documented (README only?).
