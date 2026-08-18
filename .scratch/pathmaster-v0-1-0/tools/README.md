# tools

Measurement harness for the accessibility tickets. Nothing here ships.

## `nvda-drive.ps1`

Drives a prototype with synthetic keystrokes and returns what NVDA said, so a screen-reader
measurement can be run without a human at the keyboard.

### Why it works

At logging level **Input/Output**, NVDA writes every utterance to `%TEMP%\nvda.log` as a
`Speaking [...]` line and every keystroke as an `Input:` line. That turns "what does the screen
reader announce" into a diffable text artifact. The script sends one key, waits for the log to go
quiet, sends the next, and finally returns only the bytes the run appended.

Keys are injected with `keybd_event`. NVDA logs them as ordinary gestures
(`Input: kb(desktop):downArrow`), so from its side they are indistinguishable from typing — and NVDA's
speech follows accessibility events, not keystrokes, so injection cannot by itself explain a silence.

### Before running

1. **Raise NVDA's logging level**: `NVDA+Ctrl+G` → General → Logging level → **Input/Output**.
   The script warns if it cannot find `Speaking`/`Input:` lines in the recent log.
2. At that level NVDA logs all speech and keystrokes **system-wide**. Close anything sensitive,
   run the pass, and **put the level back to Info afterwards**.

### Usage

```powershell
# start the app under measurement and remember its pid + a log offset
.\nvda-drive.ps1 -Launch -Exe ..\prototypes\02-nvda-baseline\target\release\nvda-baseline.exe

# send keys, one at a time, waiting for speech after each; prints the log slice
.\nvda-drive.ps1 -Keys 'TAB,DOWN,DOWN,DOWN,CTRL+HOME'

# ask the control what its own state is - the cross-check that distinguishes
# "the screen reader said nothing" from "nothing actually happened"
.\nvda-drive.ps1 -Probe

.\nvda-drive.ps1 -Keys 'ALT+F4'
```

Key syntax is `MOD+MOD+KEY`, comma-separated: `TAB`, `SHIFT+TAB`, `CTRL+HOME`, `ALT+F4`, `INS+TAB`
(`INS` is the NVDA modifier), `F6`, `A`–`Z`, arrows, `PGUP`/`PGDN`, `F1`–`F12`.

### Check NVDA is sane before trusting a pass

`NVDA+Tab` while a list row is focused must answer `'елемент списку'`. If it answers `'список'` —
NVDA reporting the list itself as the focused object — NVDA is in the state described in **ticket 18**
and will announce nothing for row movement. Results from that state are void. Ticket 02 lost a whole
measurement to it and reached the opposite conclusion.

### Things that cost time to learn once

- **`-Launch` and `-Keys` belong in one call.** Only the process that started the app is granted the
  right to take the foreground, and that right does not survive into a later PowerShell process. A
  separate `-Keys` call works only while the window is still foreground; otherwise it aborts rather
  than typing into whatever the user is doing.
- **`-Probe` is the point.** A silent log proves nothing on its own. Reading
  `LVM_GETNEXTITEM(LVNI_FOCUSED)` and MSAA `accFocus` straight out of the control is what turned
  ticket 02's silence from "maybe the script failed" into a finding.
- **Extended-key flag matters.** Arrows, Home/End, Insert must carry `KEYEVENTF_EXTENDEDKEY` or
  Windows reads them as their numpad twins. NVDA's default modifier is the *extended* Insert.
- **Focus is re-checked before every key.** Synthetic input goes wherever focus is, so a window that
  steals focus mid-run aborts the pass instead of typing into it.
- **Only the appended slice is read**, never the whole log — the level being system-wide means
  everything before the run is unrelated speech.
- **Pace on log quiet, not on a fixed sleep.** NVDA speaks asynchronously; a fixed delay either
  truncates an utterance or wastes the run's wall-clock.

### Where it has been used

Ticket 02 (`../research/02-nvda-baseline.md`) — the baseline measurement.
Ticket 08 needs the same loop for verifying whichever announcement rung it picks.
Ticket 18 needs it to try to reproduce the state where NVDA stops announcing rows.

**Do not add accessibility calls to `../prototypes/02-nvda-baseline/`.** It is the baseline that
later measurements are compared against. Copy it first.
