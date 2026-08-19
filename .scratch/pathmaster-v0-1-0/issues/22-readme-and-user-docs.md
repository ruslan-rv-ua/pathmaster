# README and user-facing docs

Type: grilling
Status: open
Blocked by: —

## Question

What does the README (and any other user-facing document) promise, warn about, and explain?

Graduated out of the map's **Not yet specified** on 2026-08-19, once the packaging ticket it waited
on resolved. The resolved tickets have already accumulated a list of things the README *owes* the
user — this ticket decides structure, tone and completeness, and closes the list:

- The honest description of what winget/scoop themselves write to the machine: winget's ARP key
  under `HKCU`, the Links directory on the user PATH, the exe rename via `Commands`; scoop's shim
  and `persist: data` junction (ticket 15).
- The one named exception to "nothing outside the app's directory": ComDlg32 MRU writes from the
  Browse folder picker (tickets 07/10).
- SmartScreen on an unsigned exe: what the user will see and why it is expected (charting
  constraint 10).
- The installed-NVDA requirement for the elevated instance — portable NVDA is deaf to it
  (ticket 12).
- The ticket-18 anomaly's user-facing workaround: if NVDA goes silent on the list, restart NVDA.
- `winget uninstall` deletes `data\`, backups included; `winget upgrade` keeps it (ticket 07).

To decide: single README or split docs; language (English only, or a Ukrainian section given the
Interface Language work); whether the Release Checklist's existence is user-visible documentation
or internal; and what, if anything, of the spec's cut/deferred list is worth telling users.

## Comments

**2026-08-19, from ticket 20 (failure taxonomy):** one more item the README owes the user —
`settings.json` is hand-editable, and when it cannot be parsed the app sets it aside as
`settings.json.bad` (single copy) and starts on defaults, telling the user via a startup dialog.
Document the `.bad` file: what it is, that the previous content is recoverable from it, and that
bad *values* of individual fields are tolerated per-field (raw value kept in the file) rather than
resetting the whole file.
