# NVDA baseline for a stock wxdragon shell

Status: **measured**. All seven questions answered.

**Headline: a stock wxdragon shell is announced well, including the list.** Rows speak both columns —
`'C:\scoop\shims; Status: Warning: Duplicate'` — with no accessibility code of any kind. The status bar
is readable. What is genuinely silent is narrow and listed at the end.

> **Correction, 2026-08-18.** An earlier version of this file reported the opposite — that arrowing the
> list announced nothing. That measurement was taken during a ~7-minute window in which NVDA treated
> this control as a leaf, and it does not reproduce. The window itself is real and is recorded below
> under "The anomaly", because a screen reader going deaf on the main control is worth its own
> investigation — but it is **not** the baseline. Everything above and below it is re-measured in the
> healthy state.

## Provenance

| | |
|---|---|
| Prototype | `../prototypes/02-nvda-baseline/`, release build, no accessibility code |
| Pass 1 | 2026-08-18 15:25–15:27, driven by hand — launch, tabs, empty list, menus, Tab traversal |
| Pass 2 | 2026-08-18 16:24–16:31, script-driven — **discarded, see "The anomaly"** |
| Pass 3 | 2026-08-18 ~17:30, script-driven — the populated list, status bar, empty list, re-verification |
| Harness | `../tools/nvda-drive.ps1` |
| Source | `%TEMP%\nvda.log` at logging level Input/Output |
| Env | NVDA 2025.3.3 x86, RHVoice `Natalia`; NVDA UI language follows OS, so **roles are spoken in Ukrainian** |

Utterances are copied verbatim from `Speaking [...]` lines. `CancellableSpeech` markers are dropped;
`CharacterModeCommand(True), 'a', CharacterModeCommand(False)` is written `‹a›` — NVDA spelling out an
access key.

## 1. Launch and window title — spoken

```
['PathMaster — NVDA baseline prototype']
['вкладка']
['User PATH', 'вкладка', 'виділено']
```

Focus starts on the notebook page, not the frame.

## 2. Ctrl+Tab between tabs — spoken

```
['System PATH', 'вкладка', 'виділено']
['Backups',     'вкладка', 'виділено']
```

Label, role and selection state, free. No position ("2 of 3") — expected, this machine has
`reportObjectPositionInformation = False`, so its absence measures the config, not the control.

## 3. Arrowing a populated list — **fully announced, both columns**

All eleven rows of the User PATH list, `Ctrl+Home` then ten `downArrow`:

```
['C:\Users\Ruslan\AppData\Local\Microsoft\WindowsApps; Status: OK']
['C:\Program Files\Git\cmd; Status: OK']
['C:\scoop\shims; Status: OK']
['%USERPROFILE%\.cargo\bin; Status: OK']
['C:\Program Files\nodejs; Status: OK']
['C:\scoop\shims; Status: Warning: Duplicate']
['C:\Tools\NoSuchFolder; Status: Error: Path does not exist']
['.\relative\bin; Status: Warning: Relative path']
['Status: Error: Empty entry']
['C:\Program Files\PowerShell\7; Status: OK']
['C:\Program Files\dotnet; Status: OK']
```

This answers the ticket's three sub-questions at once:

- **Is the Status column read with the Path?** Yes, every time, in one utterance.
- **Are column headers announced?** Yes, and in the useful form: NVDA speaks the first column bare and
  prefixes every later column with its header name — hence `; Status: `. The header is what makes the
  second value intelligible, and it arrives free.
- **Is the row position announced?** No. Config, not control — `reportObjectPositionInformation` is
  off on this machine, so this stays unanswerable here.

`NVDA+Tab` on a row reports it as an item, with state:

```
['C:\scoop\shims; Status: OK', 'елемент списку', 'у фокусі', 'виділено']
```

Two details worth carrying forward:

- **An Entry with an empty Path announces only `['Status: Error: Empty entry']`.** NVDA omits an empty
  column rather than saying "blank", so the row is identified purely by its Issue. Whether that is
  good enough is ticket 09's call, not a defect of the control.
- `%VAR%` and `\` are spoken as part of the path without symbol names, consistent with `symbolLevel = 0`.

## 4. The empty list — near-silent

Entering the empty Backups list announces the role and nothing else; arrowing in it is silent (there is
nothing to move to); `NVDA+Tab` gives the shape without a row count:

```
tab into it  → ['список']
downArrow    → (nothing)
NVDA+Tab     → ['список', 'у фокусі', 'з 2 стовпців']
```

No count, no "порожньо" — while NVDA does say `'порожньо'` for an empty edit field elsewhere in the
same log. **This is the one place the free ride is genuinely thin**: a user landing here learns they
are in a list, not that it is empty.

## 5. Menus — rich

```
['File',               'підменю', 'Alt+', ‹f›]
['Apply\tCtrl+S',      ‹a›]
['Refresh\tF5',        ‹r›]
['Undo\tCtrl+Z',       'недоступно', ‹u›]
['Redo\tCtrl+Y',       ‹r›]
['Add Entry…\tInsert', ‹a›]
['Expand %VAR%',       'відсоток']
['Show Status Bar',    'позначено', ‹s›]
['View',               'підменю', 'Alt+', ‹v›]
```

- **Disabled state announced** — `'недоступно'` on `Undo`. Free.
- **Checked state announced** — `'позначено'` on the check-item. Free.
- **Accelerators are spoken because they are inside the label.** `Apply\tCtrl+S` is one string and NVDA
  reads the string. Not a separate shortcut property — so the `\t` convention in menu labels is
  load-bearing and must survive any refactor. Separately `reportKeyboardShortcuts = True` yields the
  access key.

## 6. Tab / Shift+Tab traversal — clean, no trap

```
tab bar → список → Add… → Delete → Move Up → Move Down → (wraps to tab bar)
```

Confirmed in both directions. Buttons announce name, role and access key:

```
['Add…', 'кнопка', 'Alt+', ‹a›]   ['Delete',    'кнопка', 'Alt+', ‹d›]
['Move Up','кнопка', 'Alt+', ‹u›] ['Move Down', 'кнопка', 'Alt+', ‹o›]
```

Nothing traps focus.

## 7. Status bar — readable by command, not by traversal

```
NVDA+End → ['User PATH: 11 entries (4 issues) Total length: 486 chars']
```

Both fields arrive in one utterance, and `NVDA+End` is the standard desktop-layout gesture for it, so a
screen-reader user has a normal way to reach it.

But it is **not in the Tab order** — Tab from `Move Down` wraps straight back to the tab bar — and
**`F6` is silent**, producing no utterance at all. So it is reachable only by someone who thinks to ask
for it.

Caveat about the prototype, not about NVDA: its status bar text did not change when the tab changed
(still "User PATH: 11 entries" while on Backups). The prototype sets it once; nothing is implied about
how the real app would behave.

## The anomaly — NVDA went deaf on this control for ~7 minutes

Recorded because it is a genuine risk, not because it is the baseline.

Between 16:24 and 16:31 the same binary, the same NVDA process (the log is continuous — no restart, no
rotation to `nvda-old.log`) and the same config produced:

- **Total silence** on fourteen arrow presses across the populated list.
- `NVDA+Tab` → `['список', 'у фокусі', 'з 11 рядків і 2 стовпців']` — NVDA reporting the **list** as the
  focused object, never descending to a row. Compare the healthy state, where the same gesture returns
  the row and `'елемент списку'`.
- `NVDA+End` → `['Рядок стану невиявлено']` — "status bar not found", on the very frame that answers
  the same gesture correctly now.

It was cross-checked at the time, so the silence was real and not a harness failure:

- `LVM_GETNEXTITEM(LVNI_FOCUSED)` read straight out of the control showed the focused row moving 0 → 3.
- MSAA on that list returned 11 `ROLE_SYSTEM_LISTITEM` children with correct names, and `accFocus`
  named the exact row the arrows had reached, with `selected + focused` set.

So the content was present and correct while NVDA announced none of it.

**It does not reproduce.** Replaying the identical key sequence on a fresh instance — including the
accidental triple-`Tab` and the `Shift+Tab` re-entry that preceded the silence — announces every row.

Hypotheses, cheapest first, none tested:

1. **A race at window creation.** In the silent run the first keys arrived ~2.6 s after launch, sent
   in an unpaced burst (~65 ms apart) by a harness bug, while NVDA's in-process helper was still
   attaching. NVDA may have cached a degraded object for the list and never re-built it. Test: launch
   and hammer keys immediately, unpaced, several times.
2. **A stale NVDA session.** NVDA had been running ~12 h. Test: reproduce after a long uptime.
3. Something specific to that process instance's injection.

Until it is understood, the honest statement is: *this shell announces its list correctly, and has been
observed once not to, for reasons unknown.* Anyone re-measuring should check `NVDA+Tab` on a row first
— if it answers `'список'` instead of `'елемент списку'`, NVDA is in the bad state and the pass is void.

## What this means

**Free, and enough to build on:** window title, tab labels and selection, **list rows with both columns
and their header names**, button names + roles + access keys, menu names, accelerator text, disabled and
checked states, a focus order with no traps, and a status bar that answers `NVDA+End`.

**Not free — the short list:**

1. **Empty states.** An empty list says only "список", with no count and no "empty". Ticket **09**
   should require it.
2. **The status bar is command-only.** Not in the Tab order and `F6` is silent. Tickets **09** and
   **17** decide whether that is acceptable or whether its content needs a second home.
3. **Row position.** Never spoken on this machine, because the user's config has it off. Anything the
   design wants heard must not rely on it.
4. **Everything not tied to a focus change** — the banner, "PATH refreshed", "Copied to clipboard".
   Untouched by this measurement and still ticket **08**'s question.

Any later ticket that adds an accessibility call must re-measure against this file rather than assume
it only added — the first `set_accessibility_*` call moves the control off the comctl32 path that
produced everything above.
