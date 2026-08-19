# NVDA went deaf on the list — unexplained

Type: research
Status: open
Blocked by: —

## Question

Why did NVDA stop announcing list rows in a running wxdragon app, and can it happen to a user?

On 2026-08-18, between 16:24 and 16:31, ticket 02's prototype was measured and found completely silent
on its main list. The same binary, measured again an hour later, announces every row correctly. The
silent window is real, evidenced, and does not reproduce.

This matters more than an odd log entry: the failure mode is a screen-reader user arrowing through
their PATH and hearing **nothing**, with no error, no warning, and no way to tell that the app rather
than their own attention is at fault. If it can happen in the field, no amount of correct accessibility
code prevents it.

## What was observed

During the silent window:

- **Fourteen arrow presses across an 11-row list produced no speech at all.**
- `NVDA+Tab` answered `['список', 'у фокусі', 'з 11 рядків і 2 стовпців']` — NVDA reporting the **list**
  as the focused object and never descending to a row. In the healthy state the same gesture answers
  `['C:\scoop\shims; Status: OK', 'елемент списку', 'у фокусі', 'виділено']`.
- `NVDA+End` answered `['Рядок стану невиявлено']` — "status bar not found" — on a frame that answers
  the same gesture correctly now.

Cross-checks taken at the time, which is what makes this a finding rather than a harness failure:

- `LVM_GETNEXTITEM(LVNI_FOCUSED)` read straight out of the control showed the focused row moving 0 → 3
  while nothing was spoken.
- MSAA on that list returned 11 `ROLE_SYSTEM_LISTITEM` children with correct names, and `accFocus`
  named the exact row the arrows had reached, with `selected + focused` set.
- NVDA's log showed no error. Its in-process `sysListView32` helper was attached and running (it logged
  a benign `LVM_GETGROUPINFOBYINDEX failed`, which the healthy runs also produce).

Constant across both states: same executable, same **NVDA process** (the log is continuous — no restart,
no rotation to `nvda-old.log`), same `nvda.ini`, same machine, same session.

Full evidence: `../research/02-nvda-baseline.md`, section "The anomaly".

## What has already been ruled out

Replaying the exact key sequence that preceded the silence — including an accidental unpaced triple
`Tab` and a `Shift+Tab` re-entry into the list — on a fresh instance announces every row. So the
sequence itself is not the trigger.

## Hypotheses, cheapest first

1. **A race at window creation.** In the silent run the first keys arrived ~2.6 s after launch in an
   unpaced burst (~65 ms apart) while NVDA's in-process helper was still attaching. NVDA may have built
   a degraded object for the list and cached it. **Test:** launch and hammer keys immediately and
   unpaced, repeatedly, varying the delay from launch; watch whether `NVDA+Tab` answers `'список'`
   instead of `'елемент списку'`.
2. **A stale NVDA session.** NVDA had been up ~12 h. **Test:** repeat after a long uptime, and after
   NVDA has visited many other applications.
3. **Something specific to that process instance's injection** — e.g. the helper attaching to a window
   that was created before the notebook page it lives on. **Test:** compare NVDA's log around window
   creation between a healthy and a silent run.

## What a conclusion has to produce

Not just a cause, but a **detectable signature and a mitigation**:

- Can the app tell it is in this state? If NVDA has the list as a leaf, is there anything observable
  from inside the process?
- Does anything the app can do — recreating the control, firing an explicit `EVENT_OBJECT_FOCUS`,
  toggling focus away and back — restore it? A recovery that costs one line is worth having even for a
  rare state.
- If the cause turns out to be NVDA-side and unfixable from here, say so plainly and record the
  workaround a user would need (restart NVDA), so support answers exist before anyone hits it.

Until then, every accessibility measurement in this repo carries a precondition: **`NVDA+Tab` on a list
row must answer `'елемент списку'`, not `'список'`.** If it answers the latter, NVDA is in the bad state
and the pass is void. This is already noted in tickets 02 and 08 and in `../tools/README.md`.

Findings → `../research/18-nvda-deaf-on-listctrl.md`.

## Comments

**2026-08-19, from ticket 19 (test and verification strategy):** two consumers now wait on this
ticket's conclusion. (1) The no-NVDA-automation decision (19 D3) is explicitly conditioned on the
deaf state having **no detectable signature** — if this investigation produces one, the automation
question reopens cheaply. (2) The spec (ticket 16) will record this anomaly as a documented open
risk with restart-NVDA as the user-facing workaround — it does **not** block the spec, so a
conclusion here amends the spec's risk note rather than gating it.
