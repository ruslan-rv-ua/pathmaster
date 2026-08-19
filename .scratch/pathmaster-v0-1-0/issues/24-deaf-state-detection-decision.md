# Deaf-state detection: does v0.1.0 act on the signature?

Type: grilling
Status: open
Blocked by: —

## Question

Ticket 18 found a detectable signature for the NVDA deaf-list state: a processed focus event
necessarily sends `WM_GETOBJECT (OBJID_CLIENT)` to the list HWND, so *screen reader present + item
focus changed + no `WM_GETOBJECT` within ~1 s* identifies the state from inside the process via
`SetWindowSubclass`, with no accessibility code. This is exactly the condition under which
ticket 19's D3 (no NVDA automation, Release Checklist only) said the question reopens.

Decide what v0.1.0 does with it:

- **Nothing in-app** — the spec records the signature and the support ladder
  (Alt+Tab → restart app → restart NVDA) as a documented risk note only. Zero code, zero risk of
  false positives; a rare unreported state may not earn a mechanism.
- **Passive detection** — the subclass listens and, when the signature fires, logs one line
  (interacts with ticket 21's closed log-record list) and/or shows a passive Banner/StatusBar hint
  ("screen reader may have lost this list — press Alt+Tab"). Must not speak: in the deaf state
  `announce()` is very likely dead too (ticket 18), and a false positive that *announces* would be
  worse than silence.
- **Detection in the measurement harness only** — `nvda-drive.ps1` and the Release Checklist gain
  the signature as an automated precondition check (replacing or backing the manual `NVDA+Tab`
  sanity step), while the shipped app stays clean. Reopens only the *harness* half of 19 D3, not the
  UI-automation half.

Also decide: does the ~1 s threshold need a live measurement before the spec can state it, or is it
recorded as "about one second, tune during implementation"?

Constraints already settled around this: ticket 09's Announcement catalogue is closed at seven;
ticket 21 (in flight) is deciding whether the log-record list is closed; ticket 19 keeps
`nvda-drive.ps1` a measurement tool, never a CI gate — nothing here may turn it into one.

Findings that ground this decision: [research/18](../research/18-nvda-deaf-on-listctrl.md).
