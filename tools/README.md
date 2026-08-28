# tools

Scripts the repository needs but the executable never carries. **Nothing here ships**, and nothing
here runs as part of a build: each is invoked by hand, and what it produces is committed.

The measurement half backs the Release Checklist's Sanity Check
([docs/release-checklist.md](../docs/release-checklist.md)). It was built for the v0.1.0
wayfinding effort's accessibility tickets and promoted to the repo root by ticket 23, because the
Checklist is a permanent document. The ticket-24 `WM_GETOBJECT` watcher joins this directory when
built.

## Encoding

**Every `.ps1` here is UTF-8 with a BOM, and the BOM is load-bearing.** `just` runs these through
`powershell.exe`, which is Windows PowerShell 5.1, and 5.1 reads a BOM-less script in the machine's
ANSI code page rather than as UTF-8. On a Ukrainian Windows that code page is 1251, where the first
two bytes of «Відновити» read back as `Р’` — and `’` is U+2019, which PowerShell accepts as a
single-quote delimiter. The literal ends in the middle of itself and the file does not parse at all.

Comments survive it, because a comment runs to the end of its line whatever its bytes decode to.
That is why only the one script carrying a Ukrainian string in *code* ever sprang the trap, and why
the BOM is declared on all three instead: the next non-ASCII string will not announce itself.

## `make-icon.ps1`

Rasterises `crates/pathmaster/resources/icon.svg` into the multi-resolution
`crates/pathmaster/resources/app.ico` the exe carries as a resource — the second of the **two
assets from one source design** spec §12 asks for. Run it after editing the SVG, and commit both:

```powershell
.\make-icon.ps1
```

It is a hand-run script rather than a build step deliberately. A build that rasterised the icon
would need ImageMagick on every machine that compiles this application, to produce a file that
changes about once a year.

## `make-screenshots.ps1`

Regenerates the READMEs' main-window screenshots, one per language, into `docs/images/`. This is
checklist step F1, which asks for them to be refreshed whenever the README changes.

```powershell
.\make-screenshots.ps1
```

It exists because the staging is not obvious. The list in the picture must not be anyone's real
`PATH`, so it is filled through the Backups tab's **Restore** — which loads a Snapshot into the
Working Copy and writes nothing, Apply being what would write. And it must not steal the
foreground, because synthetic keystrokes go wherever focus is and somebody may be using the
machine: the Backups tab comes from the app's own `--tab backups`, the Snapshot row is focused by
posting `WM_KEYDOWN` to the list, and Restore is pressed with `BM_CLICK`. All three carry handles
and scalars, never a pointer this process owns — which is the one thing a cross-process
`SendMessage` cannot dereference.

The demo `PATH` carries one Entry of every Issue type. Its **clean** rows have to be real
directories that the machine's System `PATH` does not also hold, or they flag `Duplicate`
(evaluation runs across both Scopes, System first) and the picture shows a list where nothing is
healthy. The script asserts that before it launches anything.

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
.\nvda-drive.ps1 -Launch -Exe ..\.scratch\pathmaster-v0-1-0\prototypes\02-nvda-baseline\target\release\nvda-baseline.exe

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

Ticket 02 (`../.scratch/pathmaster-v0-1-0/research/02-nvda-baseline.md`) — the baseline measurement.
Ticket 08 used the same loop to verify the announcement mechanism.
Ticket 18 used it to try to reproduce the state where NVDA stops announcing rows.

**Do not add accessibility calls to `../.scratch/pathmaster-v0-1-0/prototypes/02-nvda-baseline/`.** It is the baseline that
later measurements are compared against. Copy it first.
