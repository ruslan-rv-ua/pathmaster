# Elevation and System PATH writes

Type: grilling
Status: resolved
Blocked by: 05, 07

## Question

What is the elevation model, end to end?

- **Detection.** Which API decides "we are elevated"? Handle the common middle case: an administrator account
  with UAC on, running unelevated.
- **Relaunch.** `ShellExecute("runas", <current exe>)`. Which arguments carry over — the active tab, nothing at
  all? What happens when the user dismisses the UAC prompt: silent return to the same state, or an explicit
  message? (Silence after a security prompt is a bad experience for a screen-reader user especially.)
- **Unsaved changes.** Charting settled that they are lost, behind a confirm dialog. Re-confirm this holds
  against the session model from ticket 06, and decide whether the confirm text names what is being lost.
- **`data/` under elevation.** Backups written as admin, resulting ownership, and whether a later unelevated
  run can still rotate and delete them. Ticket 07 owns the directory; this ticket owns the consequence.
- **Restoring a System snapshot** from an unelevated instance (FR-backup-ui): blocked before the confirm
  dialog, or offered with an elevation prompt?
- **Write failures.** Access denied, key missing, value locked by another writer, broadcast timeout. What the
  user sees, what is logged, and what the working copy looks like afterwards — the PRD only says "changes are
  not applied", which is not enough to implement.
- **Whole app versus write helper.** Single-exe means a helper is the same binary with a flag. Is that worth it,
  or does the whole app relaunch elevated? Name the trade-off; this one may deserve an ADR.

## Carried in from ticket 05

**TC-wm-settingchange's 5000 ms is a spec bug.** The `SendMessageTimeout` timeout applies **per top-level
window and multiplies** — with 226 windows open on the research machine, 5000 ms is a theoretical 18.8-minute
freeze, while a healthy broadcast measured 37 ms. Rewrite to 1000–2000 ms with `SMTO_ABORTIFHUNG`, and run it
**off the UI thread**. Two more rules for the write path:

- The `lParam` string must be **UTF-16LE, NUL-terminated, and outlive the call**. UTF-8 into the W variant
  yields garbage and UTF-16 into the A variant yields `"E"` — and both report success.
- A `0` return from the broadcast is **not** an Apply failure and must not be reported as one. Also be honest
  about what it delivers: Explorer refreshes its block so newly launched processes inherit the new PATH;
  already-open shells do not, and no message can change that.

## Carried in from ticket 06

- **An Editing Session never survives a process boundary.** Elevation is a relaunch, so it starts fresh Sessions
  — which means relaunching elevated **destroys the User Session's unsaved changes**. The elevation path must
  therefore run *through* the close-confirm flow (§12 of ticket 06), never around it. This ticket owns what that
  looks like: whether elevation is offered at all while anything is dirty, and what the dialog says.
- **`writable` is a per-Session property**, decided at load: User always, System only when the process is
  elevated. A non-writable Session disables **every** editing action — Add, Delete, Move, Edit *and* Apply — not
  Apply alone, because a Working Copy that can never be applied is a trap.
- Ticket 06 deliberately did **not** decide how elevation is initiated, how the relaunch is performed, or whether
  the elevated instance re-opens on the System tab. Those stay here.

## Carried in from ticket 07

- **The ACL bullet above is answered, not open.** Access to files in the Data Directory is governed by the
  **inherited DACL, not by ownership**: a directory created under a user-writable parent inherits `FullControl`
  for the user, `Administrators` and `SYSTEM`, so a file written by the elevated instance stays fully
  modifiable by a later unelevated one, and backup rotation keeps working. Measured, not assumed. This ticket
  inherits the consequence rather than deciding it.
- **Two instances are explicitly allowed.** Ticket 07 rejected a single-instance lock precisely because
  elevation-by-relaunch makes a second instance a designed state; a cross-elevation mutex would also need an
  explicit DACL and mandatory label to be visible from medium integrity. So the relaunch design is free of
  locking concerns — but this ticket still owns whether the original instance **exits** after spawning the
  elevated one, or both stay up.
- **Elevation is also a remedy for Read-only Data, and that is a new question for this ticket.** An executable
  in `C:\Program Files\` is Read-only Data unelevated and fully writable elevated. Decide whether the app
  *offers* elevation as the way out of Read-only Data, or merely states the reason and leaves it to the user —
  bearing in mind that a relaunch destroys unsaved changes and must go through the close-confirm flow.
- **Startup predicts, Apply verifies.** The startup writability verdict governs the UI only; Apply always
  begins by writing a Snapshot and treats its failure as an Apply failure. The write-failure taxonomy this
  ticket owns therefore has to cover a *data directory* failure at Apply time, not only a registry one.

## Carried in from ticket 09

Every Apply failure is Announced (ticket 09, D3 — closed catalogue, item 3). This ticket owns the exact
failure texts for registry-write and elevation errors. Dialog discipline applies: all critical
information in a dialog's **title and buttons** — NVDA never speaks a `MessageDialog` body.

## Answer

Resolved 2026-08-19 by grilling, preceded (at the user's standing instruction) by an internet
best-practices pass: Microsoft's UAC UX guide, `TOKEN_ELEVATION_TYPE` semantics, the COM elevation
moniker's own warnings, the NVDA user guide, and precedent (PowerToys Environment Variables, Rapid
Environment Editor). Decision-by-decision:

**D1 — Whole-app relaunch; no write helper.**
[ADR-0005](../../../docs/adr/0005-elevation-by-whole-app-relaunch.md). The single-exe helper
(`--elevated-write` flag) was rejected: it is an EoP surface (a confused deputy accepting arguments
under an elevated token), it prompts on the secure desktop per write against the UX guide's
"stay elevated" rule, and every neighbouring tool relaunches whole. Precedent is unanimous.

**D2 — Detection.** `GetTokenInformation(TokenElevation)` → `TokenIsElevated`, checked once at startup.
Never `TokenElevationType` (returns `Default` for built-in Administrator / UAC-off — exactly the
configurations a writability verdict must not misread). An admin account running unelevated reads as
not elevated, which is correct: the System Session loads non-writable.

**D3 — Entry point.** One menu command, "Restart as Administrator", the *only* way into elevation:
disabled when already elevated (NVDA reads the disabled state for free). No button on the System tab,
no auto-offer dialog on tab switch (a modal surprise during Tab navigation is the worst option for a
screen-reader user), no second offer from Read-only Data (see D7). A menu item cannot carry the UAC
shield icon; the label's words carry the meaning instead.

**D4 — Relaunch semantics.** The command does what it says: close-confirm flow (if any Session is
dirty) → `ShellExecuteEx("runas", <current exe>, "--tab <active>")` → on success the original
instance **exits**; on `ERROR_CANCELLED` it stays, fully functional. Two designed-live instances
(ticket 07) remain *possible*, but the elevation command never *produces* them — two independent
Baselines over one PATH value invite self-inflicted external-edit conflicts.

**D5 — What crosses the boundary: the active tab, nothing else.** One argument. Sessions are dead at
the boundary (ticket 06) and stay dead; carrying selection or working-copy state would be session
state smuggled across. Focus discipline (ticket 09) extends to the relaunch: the user lands on the
tab they left.

**D6 — Declined UAC prompt is never silent.** `ShellExecuteEx` fails with `ERROR_CANCELLED` (1223) —
the app always knows. Response: a `MessageDialog`, all information in the title
("Elevation was cancelled — still running without administrator rights"), OK only, focus returns to
where it was. A dialog, not an Announcement: this answers an explicit user action, and the ticket-09
catalogue stays closed at seven.

**D7 — Read-only Data names its reason and stops.** No inline elevation offer — that would be a
second entry point, contradicting D3. The standing menu command is the remedy; the README says
plainly that a portable app does not belong in `C:\Program Files`.

**D8 — Restoring a System Snapshot unelevated: disabled control, with the standard disabled-state
reading.** Restore-to-System is a System write, and ticket 06 already rules that a non-writable
Session disables every write action. Path out: D3's command → relaunch → Restore.

**D9 — Unsaved changes (charting decision re-confirmed).** Lost, behind a confirm that runs *through*
ticket 06's close-confirm flow and **names what is lost in the title**:
"Discard unsaved User changes and restart as administrator?" — buttons "Discard and Restart" /
"Cancel". All information in title and buttons, per the dialog discipline.

**D10 — Apply failure taxonomy** (user-visible texts are Catalogue entries; exact English fixed at
spec assembly, ticket 16):

| Failure | User sees | Working Copy / Baseline |
|---|---|---|
| Snapshot write fails (Data Directory, at Apply time) | Announcement 3: Apply failed — backup could not be written | unchanged / unchanged; registry untouched (backup-first order, ticket 06) |
| Registry write fails (access denied, key unopenable, value locked) | Announcement 3: Apply failed — access denied (or named cause) | unchanged / unchanged; Session stays dirty |
| External edit detected at re-read | ticket 06's conflict dialog (already decided) | per user's dialog choice |
| `WM_SETTINGCHANGE` broadcast returns 0 / times out | **nothing** — not a failure (ticket 05) | Apply succeeded; Baseline moved |

Invariants: no failure mutates the Working Copy; no failure moves the Baseline; every failure lands
one log record with the raw error code. The broadcast runs off the UI thread, 1000–2000 ms,
`SMTO_ABORTIFHUNG`, UTF-16LE NUL-terminated `lParam` outliving the call — TC-wm-settingchange's
5000 ms is rewritten (spec bug, carried from ticket 05).

**D11 — Elevated window title.** "Administrator: PathMaster" (the cmd.exe convention), a Catalogue
string. Alt+Tab speaks the title first — the cheapest always-available answer to "which instance am
I in".

**Consequence exported to ticket 19:** NVDA interacts with elevated windows only when installed
(uiAccess) or itself elevated — a portable NVDA goes deaf on the elevated instance. The verification
checklist must run against the elevated instance explicitly (comment left on ticket 19).
