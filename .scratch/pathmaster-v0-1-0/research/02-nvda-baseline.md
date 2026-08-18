# NVDA baseline for a stock wxdragon shell

Status: **measured**. All seven questions answered.

**Headline: arrowing through a populated list is completely silent.** Everything else in the shell
speaks well without a line of accessibility code. The one surface the whole application is built
around is the one that says nothing.

## Provenance

| | |
|---|---|
| Prototype | `../prototypes/02-nvda-baseline/`, release build, no accessibility code |
| Pass 1 | 2026-08-18 15:25–15:27, driven by hand — launch, tabs, empty list, menus, Tab traversal |
| Pass 2 | 2026-08-18 16:24–16:31, driven by script — the populated list, status bar, `F6` |
| Source | `%TEMP%\nvda.log` at logging level Input/Output |
| Env | NVDA 2025.3.3 x86, RHVoice `Natalia`; NVDA UI language follows OS, so **roles are spoken in Ukrainian** |

Utterances are copied verbatim from `Speaking [...]` lines. `CancellableSpeech` markers are dropped;
`CharacterModeCommand(True), 'a', CharacterModeCommand(False)` is written `‹a›` — NVDA spelling out an
access key.

**On pass 2 being script-driven.** Keys were injected with `keybd_event` and NVDA logged each one as an
ordinary gesture (`Input: kb(desktop):downArrow`), so from NVDA's side they were indistinguishable from
typing. NVDA's speech follows accessibility events, not keystrokes, so the injection path cannot by
itself explain silence. It was also cross-checked at the control: `LVM_GETNEXTITEM(LVNI_FOCUSED)` was
read directly out of the list after the arrows and had moved (see below), proving the keys landed and
the list responded.

## 1. Launch and window title — spoken

```
['PathMaster — NVDA baseline prototype']
['вкладка']
['User PATH', 'вкладка', 'виділено']
```

`NVDA+Tab` on the settled window: `['User PATH', 'вкладка', 'у фокусі', 'виділено']`. Focus starts on
the notebook page, not the frame.

## 2. Ctrl+Tab between tabs — spoken

```
['System PATH', 'вкладка', 'виділено']
['Backups',     'вкладка', 'виділено']
```

Label, role and selection state, free. No position ("2 of 3") — expected, this machine has
`reportObjectPositionInformation = False`, so its absence measures the config, not the control.

## 3. Arrowing a populated list — **SILENT**

The User PATH list, 11 rows, focused. Ten `downArrow` presses, then `Ctrl+Home`, then three more:

```
Input: kb(desktop):downArrow      (×10)
Input: kb(desktop):control+home
Input: kb(desktop):downArrow      (×3)
```

**Not one `Speaking` line.** Nothing at all — not the path, not the status, not a row number, not even
a sound.

The keys landed. Read straight out of the control immediately after the three arrows:

```
focusedIndex now = 3   selectedCount = 1
```

So the focused row moved 0 → 3 while NVDA said nothing. Asking NVDA what it thinks is focused, at that
moment and again after another arrow, gives the same answer both times:

```
NVDA+tab → ['список', 'у фокусі', 'з 11 рядків і 2 стовпців']
```

NVDA reports **the list**, and knows its shape — 11 rows, 2 columns — but never the row. Its notion of
the focused object never descends into the list.

There is no NVDA error behind this. The only in-process complaint during the run is benign and fires on
entry, not on movement:

```
RPC process 9240 (nvda-baseline.exe) — sysListView32.cpp,
nvdaInProcUtils_sysListView32_getGroupInfo, 43: LVM_GETGROUPINFOBYINDEX failed
```

(The list has no groups, so the group query fails. It also confirms NVDA is injecting and running its
dedicated **SysListView32** support against this control, exactly as ticket 01 predicted from source.)

### The rows are not missing — only the announcement is

MSAA was queried directly on the focused list window (`AccessibleObjectFromWindow`, `OBJID_CLIENT`),
with the focus sitting on row 4:

```
hr=0  childCount=12  role=33 (ROLE_SYSTEM_LIST)
accFocus -> 4
child 1 : name='C:\Users\Ruslan\AppData\Local\Microsoft\WindowsApps' role=34 state=0x300000
child 2 : name='C:\Program Files\Git\cmd'                            role=34 state=0x300000
child 3 : name='C:\scoop\shims'                                      role=34 state=0x300000
child 4 : name='%USERPROFILE%\.cargo\bin'                            role=34 state=0x300006
child 5 : name='C:\Program Files\nodejs'                             role=34 state=0x300000
child 6 : name='C:\scoop\shims'                                      role=34 state=0x300000
```

Role 34 is `ROLE_SYSTEM_LISTITEM`; `0x300000` is selectable + focusable; `0x300006` adds **selected +
focused**. So every row exists as a proper accessible object with a name, and the control answers
`accFocus` correctly, naming the very row the arrows moved to.

**The data is there and correct. What is missing is the event that tells NVDA it changed.** That is a
more precise and much cheaper problem than "the list is inaccessible" — but it is not free, and it is
not something this baseline gives us.

Two things this does *not* settle, both downstream of the silence:

- **Whether the Status column would be read.** `accName` carries the **Path column only** — no Status
  text. That is expected for MSAA and is not evidence either way: NVDA reads further columns itself via
  `LVM_GETITEMTEXT` in its in-process helper, not from `accName`. Since no row is ever announced, this
  stays unmeasured.
- **Whether column headers would be announced.** Same reason.

## 4. The empty list — near-silent

Entering the empty Backups list announces the role and nothing else:

```
['список']
```

No count, no "порожньо" — while NVDA *does* say `'порожньо'` for an empty edit field elsewhere in the
same log. Consistent with question 3: this control volunteers nothing at item level.

## 5. Menus — the richest free surface

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
- `%` in `Expand %VAR%` is spoken as `'відсоток'` despite `symbolLevel = 0`. Noted, unexplained; it
  matters because Entry text is full of `%VAR%` and `\`.

## 6. Tab / Shift+Tab traversal — clean, no trap

```
tab bar → список → Add… → Delete → Move Up → Move Down → (wraps to tab bar)
```

Confirmed in both directions. Buttons announce name, role and access key:

```
['Add…',      'кнопка', 'Alt+', ‹a›]
['Delete',    'кнопка', 'Alt+', ‹d›]
['Move Up',   'кнопка', 'Alt+', ‹u›]
['Move Down', 'кнопка', 'Alt+', ‹o›]
```

Nothing traps focus. The list announces only `['список']` on entry.

## 7. Status bar — **not reachable, not readable**

Two independent results:

- It is **not in the Tab order** — Tab from `Move Down` wraps straight back to the tab bar.
- `NVDA+End` answers **`['Рядок стану невиявлено']`** — "status bar not found" — even though the frame
  really does own an `msctls_statusbar32` child window (confirmed by enumerating the frame's children).

`F6` produces silence: no utterance at all.

So the status bar is, for a screen-reader user, absent. Anything the design puts there is invisible
unless something is done about it.

## What this means

**Free, and genuinely good:** window title, tab labels and selection, button names + roles + access
keys, menu names, accelerator text, disabled and checked states, a focus order with no traps. The
impression that the app "works well" is accurate for all of this.

**Not free, and load-bearing:**

1. **List row announcement (the big one).** Rows are fully formed accessible objects with correct names
   and states, but moving between them produces no speech. Every core task in PathMaster — reviewing
   entries, finding a duplicate, landing on a broken path — happens by arrowing this list. Ticket **08**
   is now sized by this: it is not "add a live region", it is "make row focus reach the screen reader at
   all".
2. **Status column and headers.** Still unmeasured, and unmeasurable until (1) is fixed. Ticket **13**
   must not assume Issues are heard just because they sit in a column.
3. **Status bar.** Unreachable and unreadable as it stands. Ticket **17** cannot treat it as an
   information channel; ticket **09** must decide whether it becomes one or its content moves.
4. **Empty states.** An empty list says only "список". Ticket **09** should require the count.

The measurement is now the baseline it was meant to be, and any later ticket that adds an accessibility
call must re-measure against it rather than assume it only added.
