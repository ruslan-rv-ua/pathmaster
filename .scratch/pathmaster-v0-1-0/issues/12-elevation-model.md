# Elevation and System PATH writes

Type: grilling
Status: open
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
