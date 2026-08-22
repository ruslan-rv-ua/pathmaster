# PathMaster Release Checklist

The canonical, manual verification script that gates every release. Defined by tickets 09 (D8),
10 (D8), 12, 17 (D5), 18/24, 19 (D6) and 22 of the v0.1.0 wayfinder map; the spec
([.scratch/pathmaster-v0-1-0/spec.md](../.scratch/pathmaster-v0-1-0/spec.md), §10.2/§18) points here.

**How it is used.** Run personally, on real NVDA, before each release. Each release attaches a
**filled copy** of this document to the GitHub Release — every step marked pass / fail / void /
skipped-with-reason, plus the header below. **A failed step blocks the release.** A pass is a
record, not a ritual.

```
Release:        v_._._
Date:           ____-__-__
NVDA version:   _____
NVDA install:   installed / portable   (elevated-instance section REQUIRES installed NVDA)
Windows:        _____
Monitors:       _____ (count and scale factors, for step L1)
```

## Gate zero: the Sanity Check

Before any NVDA step, and again whenever a step's silence is suspicious: focus a list row and press
`NVDA+Tab`. NVDA must answer with the **row** ("елемент списку" / "list item"), not the list.

- If it does not, the session is in the ticket-18 deaf state and **every NVDA measurement is void —
  not failed, void**. Recovery ladder: Alt+Tab away and back → restart PathMaster → restart NVDA
  (guaranteed). Re-run the gate, then repeat the voided steps.
- The `WM_GETOBJECT` watcher in `tools/` (when built) **backs** this gate as a diagnostic — it
  never replaces the manual gesture, which stays canonical (the signature misses post-creation
  rejections).

## A. Main NVDA pass (unelevated instance)

Every step presumes gate zero passed. Expected speech is the canonical English; on a Ukrainian
interface expect the Catalogue's Ukrainian equivalents.

| # | Step | Expected speech | ✓ |
|---|------|-----------------|---|
| 1 | Launch the app | Window title "PathMaster" | |
| 2 | Arrow to a healthy Entry | Path text only — no "Status:" | |
| 3 | Arrow to an Entry with one Issue | "{path}; Status: {type}" — type one of Missing / Relative / Quoted / Duplicate / Empty | |
| 4 | Arrow to an Entry with several Issues | All types, comma-joined, in the order Missing > Relative > Quoted > Duplicate > Empty | |
| 5 | Ctrl+Tab to the other Scope | Tab label, then "System PATH: {n} entries" | |
| 6 | Activate an empty Scope | "…: no entries" | |
| 7 | Refresh (F5) | "{scope}: {n} entries"; `NVDA+Tab` confirms focus kept the Entry | |
| 8 | Edit an entry, Ctrl+Z | "Undone: Edit entry"; focus lands on the row | |
| 9 | Ctrl+Y | "Redone: Edit entry" | |
| 10 | Apply | "User PATH applied"; `NVDA+Tab` confirms focus stayed on the Entry | |
| 11 | Ctrl+Z after Apply | "Undone: Edit entry, unsaved changes" | |
| 12 | Cancel | "Changes discarded" | |
| 13 | Close with a dirty Session | Dialog title names the dirty Scopes ("Unsaved changes in: …"); its three buttons spoken as [Save] [Discard] [Cancel], focus starting on Cancel | |
| 14 | Menu with a clean Session | Apply/Cancel items read as unavailable ("недоступно") | |
| 15 | Full Tab cycle | Every control reached and spoken; cycle returns to start, no trap | |
| 16 | `NVDA+End` | Both status bar fields spoken on demand (entry/issue counts; merged PATH length) | |
| 17 | Start with an unwritable `data\` | Read-only Data Announcement at startup, reason named | |
| 18 | Ctrl+Tab to the Backups tab | The tab label, and **nothing else** — it is not a Scope, so no entry count and no other Announcement | |
| 19 | Tab into the list and arrow through it | Each row read by its columns: "{date and time}; Scope: {User PATH \| System PATH}; Entries: {n}" — the date spoken as `2026-08-19 14:32:07`, not as the file name spells it | |
| 20 | Arrow to a Snapshot whose file was corrupted by hand (see below) | The row ends "Entries: \[Corrupted]" — part of the row, never an Announcement; Tab to Restore and it reads as unavailable | |
| 21 | Arrow to a valid User Snapshot and press Restore | **No confirmation dialog**; the User tab is activated and speaks its new entry count; `NVDA+Tab` confirms focus is on the restored list; Apply and Cancel Changes read as available | |
| 22 | Ctrl+Z after step 21 | "Undone: Restore snapshot", and the list is back to what it held | |
| 23 | Arrow to a **System** Snapshot, unelevated | Restore reads as unavailable — the System Session cannot be written, whatever the file holds | |
| 24 | Tools → Open Backups Folder | The menu item is spoken; `data\backups\` opens in Explorer — a folder, **not** a file-picker dialog | |
| 25 | Continue step 17's unwritable-`data\` run: Backups tab, arrow to any Snapshot | The list still **shows** every Snapshot — Read-only Data still reads — and Restore reads as unavailable on all of them, this Scope included: an Apply could not take the backup it must take first | |
| 26 | Answer step 13's dialog with [Cancel] — Escape answers it too | The window stays open, the Session is still dirty, and `data\pathmaster.log` has gained **no** `shutdown: clean` line | |
| 27 | Close again, answer [Discard] | The application closes; the registry value is unchanged and `data\backups\` has no new file; the log ends `INFO  shutdown: clean` | |
| 28 | Dirty **both** Scopes (elevated — see section C), close, answer [Save] | The title names both, User first; "User PATH applied" then "System PATH applied"; the application closes and `data\backups\` holds one new file per Scope | |
| 29 | Stage B11's unwritable `data\backups\`, dirty a Session, close, answer [Save] | "Apply failed — could not write a backup, no changes were made."; the **window stays open** on the failed Scope's tab, `NVDA+Tab` confirms focus is on that tab's list, the Session is still dirty, and the log has no `shutdown: clean` line | |
| 30 | File → Exit, and Alt+F4 | Each is spoken as a menu item carrying `Alt+F4`, and each reaches the same close-confirm as the title bar's [X] | |

Steps 20 and 21 need Snapshots to exist. Apply once to make a real one (step 10), or hand-place
files in `data\backups\`: `YYYY-MM-DDTHH-MM-SS-User.json` holding
`{"timestamp":"…","scope":"User","valueType":"REG_EXPAND_SZ","entries":["C:\\one"]}` for step 21,
and the same name with a truncated body for step 20. A file named anything else — and the `.tmp`
of a write in progress — must not appear in the list at all.

## B. Dialog steps (ticket 10)

| # | Step | Expected speech | ✓ |
|---|------|-----------------|---|
| B1 | F2 on a row | Edit dialog: title "Edit entry", labelled path field, buttons spoken | |
| B2 | OK on an entry containing `<` | Error dialog whose **title is the message**; OK; focus returns to the field, text intact | |
| B3 | Browse in the Edit dialog | Standard Windows folder picker, operable by keyboard; chosen folder replaces the field text, focus returns to the field | |
| B4 | OK on an entry using `%VAR%` in a `REG_SZ` Scope | Convert-or-keep dialog: title spoken, both buttons spoken; either answer commits, one Ctrl+Z takes it back | |
| B5 | Cancel while dirty, then F5 while dirty | Each confirmation's title spoken and its two buttons; focus starts on the safe button, and Escape answers with it | |
| B6 | Edit an Entry, then change `HKCU\Environment\Path` in `regedit`, then Apply | External-change dialog: title "PATH was modified externally since last refresh" and **all three** buttons spoken; Escape answers with [Cancel]; nothing written, Session still dirty | |
| B7 | Stage B6 again, answer [Refresh and discard my changes] | List becomes the value `regedit` left; nothing written and no new file in `data\backups\`; Ctrl+Z brings nothing back — the stacks were cleared | |
| B8 | Stage B6 again, answer [Overwrite] | "User PATH applied"; the newest file in `data\backups\` holds the value `regedit` left, **not** the one the Session remembered | |
| B9 | Add an Entry long enough to take the StatusBar's merged length past 8,191, then Apply | Warning dialog: title names 8,191 and the length this Apply would leave; both buttons spoken; [Cancel] writes nothing, [Apply Anyway] proceeds to "User PATH applied" | |
| B10 | Lengthen it past 32,767, then Apply | Hard-cap dialog: title names 32,767 and the length; **exactly one button**, Cancel, which Escape also answers; nothing written | |
| B11 | Delete `data\backups\`, put a *file* of that name in its place, then Apply | "Apply failed — could not write a backup, no changes were made."; the registry value is unchanged and the Session is still dirty | |
| B12 | Open the Tools menu and arrow through it | **Settings…** first, then Open Backups Folder — §15's order; each spoken, neither carrying an accelerator | |
| B12a | Tools → Settings… | Dialog title "Settings"; Tab through it and hear the selector named "Language (takes effect after restart)", then the field named "Snapshots to keep per PATH", then [OK] and [Cancel] — our own buttons, not stock ones | |
| B13 | Arrow through the language selector | Three items: the auto choice in the current interface language, then "English" and "Українська" **each in its own language**, never translated | |
| B14 | Type `0` in the budget field and press OK | Error dialog whose **title is the message**; OK; focus returns to the field with the text intact, and the Settings dialog is still open. The same for an empty field and for `2.5` | |
| B15 | Change **only** the budget, OK, and open `data\settings.json` | `maxBackups` is the new number; `language` is untouched — including a hand-placed unreadable one (stage `"language": "fr"` beforehand, which must still be there) — as are any unknown fields and the file's key order | |
| B16 | Change **only** the language and OK | `language` is the new code, the running window is **still in the old language** — menu bar, tabs and Banner unchanged — and nothing is spoken. Restart: the new language is in force | |
| B17 | Open Settings three times: change nothing and press OK, then press [Cancel], then press Escape | `settings.json` is byte-identical after all three, and each time `NVDA+Tab` confirms focus is back on the control it was on before the menu was opened | |
| B18 | Tools → Settings… in the step-17 unwritable-`data\` run | The item is **available**, and inside the dialog the selector, the field and [OK] all read as unavailable while the settings are still shown; Cancel is where focus starts and the only way out besides Escape | |
| B19 | Set the budget to 2, OK, then Apply three times **without restarting** | `data\backups\` holds exactly 2 files for that Scope — the new budget is in force from the next rotation, not from the next run | |
| B20 | Set `"language": "fr"` by hand, restart, open Settings and select "Follow the system language" (already selected), OK | Nothing is written, `"fr"` still stands, and the `WARN settings:` line recurs at the next start — the file really does still say something this version cannot read. Choosing a language, OK, reopening and choosing back clears it | |
| B21 | With the app running, hold `data\settings.json` open exclusively (or deny yourself write on that file), then change a setting and press OK | Dialog "Settings could not be written — nothing was changed" with a single OK; the log gains one `WARN settings:` line; reopening Settings shows the **old** values, and once the file is released the same change goes through on a second OK | |

## C. Elevated instance (ticket 12)

**Requires installed NVDA** — a portable NVDA is deaf to elevated windows; record which was used in
the header. Elevate via Tools → Restart as Administrator.

| # | Step | Expected speech | ✓ |
|---|------|-----------------|---|
| C1 | Alt+Tab to the elevated instance | Title "Administrator: PathMaster" spoken first | |
| C2 | Gate zero, elevated | `NVDA+Tab` on a row answers with the row | |
| C3 | Arrow one list row, elevated | Row read with columns, as in step 3 | |
| C4 | One Announcement, elevated (e.g. F5) | "{scope}: {n} entries" | |
| C5 | Decline the UAC prompt (separate attempt) | Dialog title "Elevation was cancelled — still running without administrator rights"; original instance stays functional | |
| C6 | Edit and Apply on the System tab, elevated | "System PATH applied" — Announcement 2's other string, which no unelevated run can reach: unelevated, the System Session is non-writable and Apply reads as unavailable | |
| C7 | Backups tab, elevated: arrow to a System Snapshot and press Restore | Restore now reads as available (step 23's other half); the System tab is activated and speaks its new entry count, and `NVDA+Tab` confirms focus is on the restored list | |
| C8 | Dirty the User Session, then Tools → Restart as Administrator | Dialog title "Discard unsaved User changes and restart as administrator?" and both buttons spoken — [Discard and Restart] [Cancel]; focus starts on [Cancel] and Escape answers with it: the window stays open, the Session is still dirty, nothing was written | |
| C9 | Stage C8 again from the **System tab**, answer [Discard and Restart], accept the UAC prompt | The original instance exits — its log ends `INFO  shutdown: clean` — and the elevated one opens **on the System tab**, titled "Administrator: PathMaster"; the registry value is unchanged, because Discard saved nothing | |
| C10 | Elevated: open the Tools menu | Settings…, Open Backups Folder, then Restart as Administrator reading as **unavailable** («недоступно») — the one entry point, disabled where it could only restart into what already holds | |

## D. Layout and environment

| # | Step | Expected result | ✓ |
|---|------|-----------------|---|
| L1 | Drag the window between monitors with different DPI scale factors | Layout survives — no clipped or misplaced controls. Skippable with a note when only one monitor is available | |
| L2 | Resize to minimum 800×600 and maximise | List fills its tab; Path column takes remaining width; nothing clipped | |
| L3 | Move and resize the window, close cleanly, reopen | It opens exactly where it was left. Maximise, close, reopen: it opens maximised | |
| L4 | Close on a second monitor, unplug it (or hand-edit `window` in `settings.json` to `x: 9000, y: 9000`), reopen | The window opens at the default 900×650, centred on the primary monitor — never off the edge of every screen | |
| L5 | Hand-edit `settings.json` to add an unknown field and an unknown member inside `window`, then close cleanly | Both survive the rewrite, as do `language` and `maxBackups` and the file's key order | |
| L6 | Repeat L3 in the step-17 unwritable-`data\` run | The remembered geometry is still **read** and restored; `settings.json` is byte-identical afterwards, and the run has no log at all to hold a shutdown line | |
| L7 | Move the window somewhere memorable and close cleanly; reopen, **minimise** it, and close it from the taskbar | It reopens where it was left before minimising, never centred: a minimised window is not written at all, so `window` in `settings.json` is unchanged by that second close | |

## E. Non-NVDA release checks

| # | Step | Expected result | ✓ |
|---|------|-----------------|---|
| E1 | `README.uk.md` is in sync with `README.md`, or the release did not change the README | drift guard (ticket 22) | |
| E2 | Process Monitor, filtered to `PathMaster.exe`: normal session incl. one Browse use | No file/registry write outside `<exe dir>\data\`, the two PATH values, and ComDlg32 MRU after Browse | |
| E3 | Clean-VM run (no VC++ redistributable) | App starts and lists PATH. Required once for v0.1.0, then only when packaging changes; note "not repeated — packaging unchanged" otherwise | |

CI gates (version gate, `cargo test`, dumpbin imports, exe ≤ 40 MB) run automatically and are not
part of this manual pass.
