# NVDA baseline for a stock wxdragon shell

Type: prototype
Status: open
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
