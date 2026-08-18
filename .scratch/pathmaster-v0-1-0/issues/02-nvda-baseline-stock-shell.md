# NVDA baseline for a stock wxdragon shell

Type: prototype
Status: resolved
Blocked by: —

## Question

What does NVDA announce for a stock wxdragon UI, with **no accessibility code written at all**?

This is the most load-bearing fact in the effort: it decides how much of US-accessibility arrives free from
native Win32 controls and how much has to be engineered.

Build the smallest throwaway app (`/prototype`) that carries the app's real shape:

- A frame with a menubar — File / Edit / View / Tools / Help — with accelerators (`Ctrl+Z`, `F5`) and one
  deliberately disabled item.
- A notebook with three tabs: User PATH / System PATH / Backups.
- A `wxListCtrl` in report mode, columns `Path` and `Status`, ~10 rows, some with `Warning: Duplicate` /
  `Error: Path does not exist` text in the Status column, and one empty-list tab.
- Add / Delete / Move Up / Move Down buttons, and a status bar with two fields.

No accessibility work of any kind. Then the user runs NVDA and reports, **verbatim**, what is spoken for:

1. App launch and window title.
2. Switching tabs with Ctrl+Tab.
3. Arrowing through the list — is the Status column read along with Path? Are column headers announced? Is the
   row position ("3 of 12") announced?
4. Landing on the empty list.
5. Opening a menu, moving through items, hearing a shortcut and a disabled state.
6. Tab / Shift+Tab between panes and buttons; where focus goes and whether anything traps it.
7. Status bar — is it reachable and readable at all?

Record what is spoken **and what is silent**. Silence is the finding that matters.

Findings → `../research/02-nvda-baseline.md`, with the prototype's location noted.

## Carried in from ticket 01

- **Pin wxdragon ≥ 0.9.17.** Before PR #155 the `AccRole` discriminants were mis-ordered, so an older version
  reports wrong MSAA roles. That PR and #158 were authored by a core NVDA developer, so recent versions are the
  ones with real screen-reader attention.
- **Measure the baseline before touching any accessibility call, and keep it that way.** Confirmed at source:
  `wxWindow::CreateAccessible()` returns `nullptr` by default and no wx control overrides it, so `WM_GETOBJECT`
  goes unhandled and comctl32's *own* IAccessible serves the list rows — that is what "free" means here. But
  the first `set_accessibility_*` call on a widget flips it onto the wx-mediated path. So this ticket measures
  the stock behaviour, and any later ticket that adds labels must re-measure rather than assume it only added.

## Comments

### 2026-08-18 — prototype built, measurement deliberately not run

**Status: still open, and deliberately so.** The prototype exists and runs; the NVDA pass was not
performed. The user judged the measurement unnecessary ("я впевнений в wxdragon") after running the
prototype informally and reporting that it worked well. That is a real verdict about the app being
usable, but it is not this ticket's question: the ticket asks **which strings are spoken and where
there is silence**, because tickets 08 and 09 are sized entirely by that. So the ticket stays open
rather than being closed on an impression.

Note the ticket is also asking a narrower question than "is wxdragon good" — ticket 01 settled that.
It asks what **comctl32's own IAccessible** delivers with no accessibility code present.

**Everything below is done, so a later attempt costs minutes, not an hour.**

**The prototype.** `../prototypes/02-nvda-baseline/` — a wxdragon 0.9.18 app carrying the real shape
(File/Edit/View/Tools/Help menubar with `\t` accelerators, a disabled Undo and a checked check-item,
a 3-tab notebook, report-mode ListCtrl with Path/Status columns, 11 mixed User rows / 6 System rows /
an empty Backups list, four buttons per page, a two-field status bar). It contains **no accessibility
code of any kind** — that prohibition is the measurement, and it must survive any future edit.

```
cd .scratch/pathmaster-v0-1-0/prototypes/02-nvda-baseline
LIBCLANG_PATH="C:\scoop\apps\llvm\current\bin" cargo build --release
```

Smoke-tested: process stays up and `MainWindowTitle` reads `PathMaster — NVDA baseline prototype`.

**Two build-shape decisions baked in, both deliberate.**

- **comctl32 v6 manifest is embedded** via `build.rs` + `app.manifest` (linker `/MANIFEST:EMBED` +
  `/MANIFESTINPUT`, no extra crate). `wxdragon-sys` links `comctl32` but embeds **no manifest at
  all**, so without this the process gets legacy v5 common controls — different theming *and*
  different MSAA behaviour, which would make the whole baseline measure the wrong thing. Verified
  present in the built exe.
- **`ListCtrlStyle::SingleSel`** — not stock-by-omission. Delete / Move Up / Move Down each act on
  one entry, so single selection is the app's real shape. It changes what NVDA says about selection
  state, so it is called out rather than left implicit.

**The run sheet.** `../prototypes/02-nvda-baseline/NVDA-RUN-SHEET.md` — a 42-step keystroke script,
nothing to transcribe by hand. The method that makes it cheap: set NVDA's logging level to
**Input/Output** (`NVDA+Ctrl+G` → General), and NVDA writes every utterance to `%TEMP%\nvda.log` as
`Speaking [...]` lines, with interleaved `Input:` lines to align keystrokes to speech. Bracket the
run with a doubled `NVDA+T` to mark the segment. Caveat to repeat to the user: at that level NVDA
logs all speech and keystrokes **system-wide**, so the level must go back to Info afterwards.

**Environment, read straight out of the user's NVDA install (no need to ask again).**

| | |
|---|---|
| NVDA | 2025.3.3 x86, Python 3.11.9, comtypes 1.4.11 |
| Synth | RHVoice 1.18.2, voice `Natalia`, rate 55 + rateBoost |
| NVDA UI language | `Windows` (follows OS) |
| OS | Windows 11 25H2 (10.0.26200.9168) AMD64 |
| Keyboard layout | `desktop` — so status bar is `NVDA+End` |
| Config | `C:\Users\Руслан\AppData\Roaming\nvda\nvda.ini` |

**The config finding that any future pass must handle — this is the real content of this comment.**

Two settings in `[presentation]` are **off**, and NVDA's default for both is **on**:

- `reportObjectPositionInformation = False`
- `reportObjectDescriptions = False`

So on this machine, "3 of 12" is never spoken and descriptions are never spoken — **regardless of
what the control offers**. Question 3 of this ticket ("is the row position announced?") is therefore
unanswerable from a pass on this config: a silence there measures the config, not comctl32.

Also relevant: `symbolLevel = 0` (backslashes and `%` not spoken as symbols),
`reportKeyboardShortcuts = True` (so accelerators *should* be spoken), `speakCommandKeys = False`.

The user agreed the right shape is **two passes** — pass A on their real config, pass B with those
two settings temporarily on — because the **delta between them** is what separates "the control
cannot" from "this user's config suppresses". Both halves matter: pass B sizes the engineering work
for tickets 08/09, pass A is the input to ticket 09's verbosity policy, since the primary user runs
with position info off and the app must therefore not *depend* on it.

**Blast radius while this stays open.** Tickets 08 and 09 are blocked by 02, and 09 blocks 16, so
the entire accessibility spine of the spec is blocked. Proceeding without the measurement means
accepting an assumption where the Destination currently promises proof — which is a redraw of the
Destination, not a resolution of this ticket.

### 2026-08-18 — measurement declined; ticket stays open by decision, not by oversight

The user ran the prototype informally **twice** with NVDA and reported both times that it works well,
then stated plainly they do not plan to run the instrumented pass. Recorded as their decision, and
not raised again.

**What that is worth, stated honestly.** Two independent informal runs by a person who uses NVDA
daily are real evidence that the shell is *usable* — not nothing. What they do not provide is the
thing tickets 08 and 09 are sized by: the **verbatim strings**, and specifically the silences. Both
runs also happened on a config with `reportObjectPositionInformation` and `reportObjectDescriptions`
off, so they could not have answered the position question either way.

Verified from `%TEMP%\nvda.log` after the second run: `loggingLevel` was still `INFO`, zero speech
lines, zero references to the prototype window. So there is no recorded data from either run, only
the two verdicts above.

**Consequence, so nobody has to rediscover it.** This ticket blocks 08, 08 blocks 09, and 09 blocks
16 — so as long as it stays open, the accessibility spine of the spec cannot advance and the map
cannot reach its Destination. The Destination currently promises that every mechanism the product's
accessibility depends on is "proven against real NVDA rather than assumed", and map decision 5 says
NVDA verification ends in a real verdict rather than a guess. **Either this ticket eventually gets
its pass, or the Destination and decision 5 have to be redrawn to accept an assumption.** That is a
scoping call for the user, deliberately left open here rather than resolved silently in either
direction.

The cost of changing their mind later is low: everything in the previous comment still stands, and the
missing step is one setting (`NVDA+Ctrl+G` → General → Logging level → Input/Output) plus one pass.

### 2026-08-18 — partial pass captured; question 3 still unmeasured

A run was started and abandoned part-way ("я не закінчив, але впевнений що все працюватиме чудово").
Because logging was already at Input/Output, everything pressed before it stopped was recorded, so the
partial pass was harvested rather than discarded. Findings: `../research/02-nvda-baseline.md`.

**Answered from real data:** launch and title (1), Ctrl+Tab between tabs (2), the empty list (4), menus
(5), Tab/Shift+Tab traversal (6), and half of (7) — the status bar is not in the Tab order. Notably free:
`'недоступно'` on a disabled menu item, `'позначено'` on a check-item, access keys on every button, and
accelerator text spoken because `\t` puts it inside the label. Also confirmed from NVDA's own injected
helper (`sysListView32.cpp`) that the list is served by comctl32's SysListView32 support, not by wx.

**Still unmeasured — and it is the one the ticket exists for:** question 3, arrowing a **populated**
list. Whether the Status column is spoken with the Path, whether headers are announced, what the
Duplicate / does-not-exist / empty-entry rows sound like. The empty list said only `['список']` — no
count, no "порожньо" — which points away from the control volunteering context, so the answer should
not be assumed. Tickets 08 and 09 are sized by this and stay blocked on it.

Ticket remains **open**. Remaining cost: run-sheet sections C (11–22) and G (39–40) plus `F6` — about
three minutes, prototype already built, capture method already proven.

### 2026-08-18 — measured; the list is silent

Section C was run by script rather than by hand: keys injected with `keybd_event`, NVDA logging them as
ordinary gestures, and the control's own `LVM_GETNEXTITEM(LVNI_FOCUSED)` read back afterwards to prove
they landed. Full findings: `../research/02-nvda-baseline.md`.

**The answer to question 3 is silence.** Ten arrows moved the focused row 0 -> 3 in an 11-row list and
NVDA spoke nothing at all. `NVDA+Tab` at that moment reports `['список', 'у фокусі', 'з 11 рядків і 2
стовпців']` — the list, never the row. There is no NVDA error involved.

The rows are not missing. MSAA on the focused list returns 11 `ROLE_SYSTEM_LISTITEM` children with
correct names, and `accFocus` names the very row the arrows moved to, with `selected + focused` set. So
the content is present and correct; **the event announcing the change is what does not arrive**. That
reshapes ticket 08: not "add a live region" but "make row focus reach the screen reader at all".

Also measured: the status bar is unreachable — not in the Tab order, and `NVDA+End` answers `['Рядок
стану невиявлено']` though the frame does own an `msctls_statusbar32`. `F6` is silent. And the empty
list says only `['список']`, no count.

Everything else in the shell is genuinely free and good — title, tabs, buttons with access keys, menus
with `'недоступно'` and `'позначено'`, no focus traps. The earlier impression that the app "works well"
was accurate for all of that; it just did not cover the one surface the application is built around.

Ticket **resolved**. Two questions remain open but are downstream of the silence and unmeasurable until
it is fixed: whether the Status column is read with the Path, and whether column headers are announced.
Both belong to ticket 08's re-measurement.
