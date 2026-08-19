# Deaf-state detection: does v0.1.0 act on the signature?

Type: grilling
Status: resolved
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

## Resolution

2026-08-19. Grilled with best-practice research first (map Notes 13); all recommendations accepted.

**v0.1.0 ships zero deaf-state code — the answer is C + A combined: harness plus documentation.**

- **Nothing in-app.** No subclass proc, no screen-reader detection, no log line for this state in the
  shipped binary. The spec records the signature and the support ladder (Alt+Tab → restart app →
  restart NVDA) as a documented risk note; the README already carries the user-facing workaround
  (ticket 22).
- **The harness gains the watcher.** `nvda-drive.ps1` / the measurement prototype get the
  `WM_GETOBJECT` subclass watcher (research/18 §8.3) as an automated *diagnostic backing* for the
  Sanity Check. It **backs, never replaces**, the manual `NVDA+Tab` step: the signature misses
  post-creation rejections (research/18 §5 — `WM_GETOBJECT` arrives, NVDA still silent), so the
  manual gesture stays the canonical gate, and ticket 19's no-automation-in-the-gate decision holds.
  What the watcher buys: any future recurrence self-documents (focus→`WM_GETOBJECT` latency log)
  into a tracker-worthy NVDA report.
- **Threshold:** recorded as "about one second, tune in the harness" — no pre-spec live measurement.
  A false positive in the harness costs one glance at a log, never user experience.
- **Banner hint rejected on its own terms:** in the deaf state `announce()` rides the same dead
  pipeline (research/18 §7) and the target user is a screen-reader user — a visual-only hint fires
  exactly when its reader cannot hear it, and whether the review cursor reaches the Banner is
  unmeasured (map constraint 5: unmeasured is not working).
- **In-app passive detection parked as a v0.2.0 candidate** in the map's Out of scope — the
  signature is measured and the false-positive analysis done, so the decision is half-made if the
  state ever recurs in the field; revisit only then.
- **New term** — **Sanity Check** — added to [CONTEXT.md](../../../CONTEXT.md).

Evidence behind the recommendation: desktop AT-presence detection is precedented and legitimate
(Chromium enables accessibility off a `WM_GETOBJECT` probe —
[Chromium accessibility docs](https://www.chromium.org/developers/design-documents/accessibility/));
no precedent exists anywhere for a shipping app detecting a *broken* AT pipeline and hinting the
user — the ecosystem's answer to "NVDA randomly stops speaking" is user-side recovery
([nvaccess/nvda#8162](https://github.com/nvaccess/nvda/issues/8162),
[#2003](https://github.com/nvaccess/nvda/issues/2003),
[nvda.groups.io](https://nvda.groups.io/g/nvda/topic/nvda_randomly_stops_speaking/96034362));
the web-world anti-detection arguments
([Roselli](https://adrianroselli.com/2014/03/on-screen-reader-detection.html),
[Groves](https://karlgroves.com/should-we-detect-screen-readers-is-the-wrong-question/)) apply to
forking the experience, which nothing here does. No ADR: shipping no code is cheap to reverse, so
the decision fails the hard-to-reverse test.
