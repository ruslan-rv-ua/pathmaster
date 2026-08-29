# PathMaster Release Checklist

The canonical, manual verification script that gates every release. Defined by tickets 09 (D8),
10 (D8), 12, 17 (D5), 18/24, 19 (D6) and 22 of the v0.1.0 wayfinder map; the spec
([.scratch/pathmaster-v0-1-0/spec.md](../.scratch/pathmaster-v0-1-0/spec.md), §10.2/§18) points here.
Steps 34–86, and the amendments to steps 2–4, 15 and 31, are the v0.2.0 delta
([.scratch/pathmaster-v0-2-0/spec.md](../.scratch/pathmaster-v0-2-0/spec.md), §17) folded in here:
this is the document that gets filled, and a delta left in the spec is one nobody walks with.

**How it is used.** Run personally, on real NVDA, before each release, filling a **copy** of this
document — every step marked pass / fail / void / skipped-with-reason, plus the header below. The
filled copy stays in the maintainer's records; it is not published on the release. **A failed step
blocks the release.** A pass is a record, not a ritual.

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
| 1 | Launch the app | Window title "PathMaster", then the User list and its **first row**, read without a keystroke — "1; Path: {path}" and **no** "Status:", because no diagnostic pass has completed yet. `NVDA+Tab` confirms focus is on the row, not the tab strip. Record verbatim what is spoken between the title and the row: whether the Scope is named there is what decides if a launch ever needs to say so itself | |
| 2 | Arrow to a healthy Entry | "{#}; Path: {path}" — the position, then the path under its column header, and no "Status:". The `#` is the Entry's place in the **full** list and never renumbers under a narrowing | |
| 3 | Arrow to an Entry with one Issue | "{#}; Path: {path}; Status: {type}" — type one of Missing / Relative / Quoted / Duplicate / Empty | |
| 4 | Arrow to an Entry with several Issues | The same "{#}; Path: {path}; Status: …" opening, then all types, comma-joined, in the order Missing > Relative > Quoted > Duplicate > Empty | |
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
| 15 | Full Tab cycle | Every control reached and spoken, in the order tabs → **Search field** → list → buttons; cycle returns to start, no trap | |
| 16 | `NVDA+End` | Both status bar fields spoken on demand (entry/issue counts; merged PATH length) | |
| 17 | Start with an unwritable `data\` | Read-only Data Announcement at startup, reason named — and **after** the landing row of step 1, spoken **whole**. A reason cut off mid-sentence is a failure: focus landing after the Announcement instead of before it is what cancels it | |
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
| 31 | `Alt+H` on an English interface — on a Ukrainian one `F10`, then `→` to «Довідка», which carries no mnemonic (ADR-0012), then arrow through the Help menu | **Two** items: **User Guide** first, carrying `F1`, then **About**, carrying no accelerator. Neither is separated from the other, and the menu is still the last on the bar | |
| 32 | Activate Help → About | Dialog whose **title** is "PathMaster {version} — MIT License" («PathMaster {version} — ліцензія MIT»), with the version of the build under test; a single [OK] which Escape also answers; `NVDA+Tab` afterwards confirms focus is back where it was | |
| 33 | Open Help → About in the step-17 unwritable-`data\` run, and again on the Backups tab | The item is **available** in both: it names the build, which is true in every state | |

Steps 20 and 21 need Snapshots to exist. Apply once to make a real one (step 10), or hand-place
files in `data\backups\`: `YYYY-MM-DDTHH-MM-SS-User.json` holding
`{"timestamp":"…","scope":"User","valueType":"REG_EXPAND_SZ","entries":["C:\\one"]}` for step 21,
and the same name with a truncated body for step 20. A file named anything else — and the `.tmp`
of a write in progress — must not appear in the list at all.

### The v0.2.0 surfaces (spec v0.2.0 §17)

The same pass, continuing the same numbering. Gate zero still stands over every step that
speaks, and the expected speech is still the Catalogue's — the Command line group is the one
that mostly does not speak, and its column header says so. Each group is one surface, so a
voided group can be repeated on its own without re-running the whole section.

#### Search (v0.2.0 §3)

| # | Step | Expected speech | ✓ |
|---|------|-----------------|---|
| 34 | Type a few characters into the Search field | Only the typing echo while the rows rebuild — no chatter, and no deaf signature — then the count **once**, after the pause: "{n} of {m} entries". Re-run gate zero if the echo itself stops | |
| 35 | Type a query nothing matches | "No matching entries", over a list with no rows | |
| 36 | From the field press `Tab`; from the field again press `↓` | Each lands on a row, and NVDA reads that row in full | |
| 37 | Press `Enter` in the field | Nothing at all: no Announcement, no default button, and the focus does not move | |
| 38 | Press `ESC` in a field with text | The field clears and focus returns to the list. With "Escape returns focus to the list" cleared (B24) the field clears and focus **stays** in it; either way, ESC on an already-empty field says nothing | |
| 39 | Press `Ctrl+F` from the list, from a button, and with the menu bar just closed | Focus lands in that Scope's Search field with its text **selected**, from every one of them | |
| 40 | Narrow one Scope, then `Ctrl+Tab` onto it from the other | "User PATH: {n} of {m} entries" — Announcement 10, the Scope named; never step 5's bare count. Narrow it to nothing and repeat: "User PATH: no matching entries", the Scope still named | |
| 41 | View → Search on the Backups tab | The item reads as unavailable, and `Ctrl+F` does nothing | |
| 42 | `NVDA+End` while a Scope is narrowed | Field 0 reads the narrowed form — "User PATH: {n} of {m} entries ({k} issues)" — and the parenthetical is still that **Scope's** issue count, not the view's. Then Delete a visible row, and Ctrl+Z it back: the visible set recomputes and **nothing is spoken** either time. A Working-Copy change is not a change of criteria, and only criteria speak | |

#### Filter (v0.2.0 §4)

| # | Step | Expected speech | ✓ |
|---|------|-----------------|---|
| 43 | View → Filter, and arrow through the submenu | Seven items, and NVDA names the selected one **as selected** — a radio group, walked with the arrow keys | |
| 44 | Choose a type, `Ctrl+Tab` to the other Scope, and open the submenu there | The checked item is that Scope's own: each Editing Session keeps its own Filter, and the mark follows the tab | |
| 45 | Choose a type, e.g. Missing | "Missing: {n} of {m} entries" — one Announcement composed with the Search text, never two; a type that matches nothing speaks "Missing: no matching entries", the state still named. `NVDA+End` then reads field 0 as "User PATH: Missing — {n} of {m} entries ({k} issues)" | |
| 46 | `Ctrl+I` from All, from With issues, and from a type state | All → With issues, With issues → All, and any type state → All; each speaks its own count, and the submenu's mark follows every one of them | |
| 47 | Clear both narrowings — ESC in the field, Filter → All | Announcement 1, the plain "{scope}: {n} entries": the view is no longer filtered, and `NVDA+End` confirms field 0 is back to its unnarrowed form | |

#### Expansion Mode (v0.2.0 §5)

| # | Step | Expected speech | ✓ |
|---|------|-----------------|---|
| 48 | View → Expanded Values with the mode off, and again with it on | The item reads its **checked state** both ways — the mark is what says which way it went, and the label itself never changes | |
| 49 | `Ctrl+E`, then `Ctrl+E` again | "Showing expanded values", then "Showing raw values" | |
| 50 | Narrow a Scope, then `Ctrl+E` | **Both**, in order and through the same debounced path: the mode message, then the count one `filteredCountDelayMs` later. Neither swallows the other at the 250 ms default | |
| 51 | With expanded values showing, `F2` on an Entry holding `%VAR%` | The dialog's field carries the **raw** text: the expansion is a rendering, never the Working Copy | |

#### Tree View (v0.2.0 §6)

| # | Step | Expected speech | ✓ |
|---|------|-----------------|---|
| 52 | `Ctrl+T` on the User tab | A dialog titled "PATH Tree — User PATH", and its first node speaks on open | |
| 53 | Arrow to a compressed chain node | Its whole joined label in one reading, with the level and position — never the head segment alone | |
| 54 | Arrow to a three-part leaf | Segment, raw parenthetical **and** Issue suffix, all three, untruncated | |
| 55 | Arrow to the group nodes | "Unresolved variables" and "Relative entries", each spoken by name | |
| 56 | `Enter` on a leaf | The dialog closes, and the landed row speaks in full — "{#}; Path: {path}; Status: …" | |
| 57 | Reopen, select a leaf, and press [Go to entry] | The same as `Enter`: closes, lands, and the row speaks | |
| 58 | Move to an inner node or a group node and Tab to [Go to entry] | It reads as **unavailable** — there is no Entry to go to | |
| 59 | Reopen and answer with `Esc`; reopen and answer with [Cancel] | Each closes, and `NVDA+Tab` confirms focus is back where it was. View → PATH Tree… on the Backups tab reads as unavailable, and `Ctrl+T` there does nothing | |

#### Fix Issues (v0.2.0 §7)

| # | Step | Expected speech | ✓ |
|---|------|-----------------|---|
| 60 | Edit → Fix Issues… on a Scope with no fixable row | Reads as unavailable, and `Ctrl+Shift+I` does nothing | |
| 61 | The same on the Backups tab, on the System tab **unelevated**, and in the step-17 unwritable-`data\` run | Unavailable in all three, and the keystroke does nothing in any of them | |
| 62 | Open it with `Ctrl+Shift+I` on a Scope that has fixable rows | A dialog titled "Fix issues — User PATH"; arrowing the rows reads each as "checked" / "not checked" with its `#`, Path, Issue and Action columns | |
| 63 | Arrow to a Missing row whose Entry carries `%VAR%` | It starts **not checked**: an unresolved variable is not proof the directory is absent | |
| 64 | `Space` on a row, twice | The new state is announced **in place** each time | |
| 65 | Check a known number of rows and press [Fix selected] | Focus lands first, then "Fixed {n} entries" **last** — and {n} is the number that was checked | |
| 66 | One `Ctrl+Z` after step 65 | "Undone: Fixing issues", and **every** fixed Entry is back: one Checkpoint for the whole apply. Reopen, check nothing, press [Fix selected] — it closes in silence, and `Ctrl+Z` has nothing new to undo | |

#### Copy (v0.2.0 §8)

| # | Step | Expected speech | ✓ |
|---|------|-----------------|---|
| 67 | `Ctrl+C` on a row, then paste into a text editor | "Copied to clipboard", and what pastes is exactly what the list is showing | |
| 68 | `Ctrl+E`, then `Ctrl+C` on an Entry holding `%VAR%` | The clipboard holds the **expansion**: the copy follows the rendering, not the stored text | |
| 69 | Put focus in the Search field, select its text, `Ctrl+C` | The **query** is copied, not the Entry, and nothing is announced — wxMSW's text-entry preprocessing claims Ctrl+C, which is intended | |
| 70 | `Ctrl+C` on the Backups tab, and Edit → Copy there | The item reads as unavailable and the keystroke does nothing. Then close the application entirely and paste again — the clipboard still holds what step 67 put there | |
| 71 | **Failure path.** Hold the clipboard open from another process (one that opens it and does not close it), then `Ctrl+C` on a row | "Could not copy to clipboard" — Announcement 14 — and **nothing else**: no dialog of any kind, in either language. This row is why Copy writes through Win32 directly rather than through `wxdragon::Clipboard`, which raised its own untranslated «Pathmaster Error» box on exactly this path; with no such row, a regression back to it passes every other gate | |

#### User Guide and F1 (v0.2.0 §9)

| # | Step | Expected speech | ✓ |
|---|------|-----------------|---|
| 72 | `Alt+H` on an English interface — on a Ukrainian one `F10`, then `→` to «Довідка», which carries no mnemonic (ADR-0012), then arrow the menu | Two items: **User Guide** first, carrying `F1`, then **About** — step 31 read from the other side | |
| 73 | `F1` | The browser opens the guide; NVDA speaks "PathMaster {version} — User Guide" as the document title, and `H` walks its headings | |
| 74 | Look in `data\` afterwards | `help.html` is there, in the Interface Language | |
| 75 | Change the Interface Language, restart, `F1` | The **one** file is rewritten in the new language — no second file, and no orphan of the old one | |
| 76 | Delete `data\help.html`, then `F1` | It returns, and the guide opens as before | |
| 77 | `F1` in the step-17 unwritable-`data\` run | The browser opens the **online** copy, pinned to this build's tag, and nothing is announced. That run has no log to record it (L6); to read the `WARN` line itself, use a run whose `data\` is writable but whose `help.html` is not — hold the file open, or put a directory of that name in its place — where `F1` goes online just the same and `data\pathmaster.log` gains exactly one line | |
| 78 | `F1` with the Edit dialog open | Nothing at all: no Announcement, the dialog stays open, and focus does not move | |
| 79 | Help → User Guide on the Backups tab, and in the unwritable-`data\` run | **Available** in both — how to use the application is true in every state | |

#### Command line (v0.2.0 §10)

Launch checks: each starts the application afresh, from a shell, with the switch under test.

| # | Step | Expected result | ✓ |
|---|------|-----------------|---|
| 80 | `--data-dir` pointing at a path that does not exist yet | The directory is created **there** and used; `data\` beside the exe is untouched — no new file in it, and none created if there was none | |
| 81 | A **relative** `--data-dir`, from a shell whose current directory is somewhere known | It resolves against that shell's current directory, never against the exe's | |
| 82 | `--data-dir` pointing somewhere unusable — a *file* of that name, or a directory that denies writes | Announcement 7 with the **fourth** reason: "Read-only: the --data-dir location cannot be used". Nothing is written anywhere — not there, and not beside the exe | |
| 83 | In an override Run whose path contains a space, Tools → Restart as Administrator | The elevated instance comes up on the **same** Data Directory: the switch and its spaced path survive the relaunch | |
| 84 | An unknown argument, e.g. `--colour` | The dialog "Unknown argument --colour was ignored", the usage line in its body; the application **continues** after [OK], and the log gains one `WARN` line | |
| 85 | `--help` | The dialog titled "PathMaster command line", the usage line in its body, and the application **exits** when it is answered — no main window at all | |
| 86 | Read `data\pathmaster.log` after an override Run and after a default Run | The startup line carries `dataDir:` on the override Run, and does not on the default one | |

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
| B12a | Tools → Settings… | Dialog title "Settings"; Tab through it and hear the selector named "Language (takes effect after restart)", then the field named "Snapshots to keep per PATH", then the checkbox "Speak filtered entry counts" **with its checked state**, then the field named "Delay before speaking the count (ms)", then the checkbox "Escape returns focus to the list" with its state, then [OK] and [Cancel] — our own buttons, not stock ones. Space toggles either checkbox and the new state is spoken in place | |
| B13 | Arrow through the language selector | Three items: the auto choice in the current interface language, then "English" and "Українська" **each in its own language**, never translated | |
| B14 | Type `0` in the budget field and press OK | Error dialog whose **title is the message**; OK; focus returns to the field with the text intact, and the Settings dialog is still open. The same for an empty field and for `2.5` | |
| B14a | Type `5001` in the delay field and press OK | The delay's own rejection — "Delay before speaking the count must be a whole number, 0 to 5000" — never the budget's; focus returns to the delay field with `5001` intact and the dialog stays open. `0` and `5000` are both accepted. With **both** fields wrong, the budget's message comes first: one dialog per press, in the order the fields are laid out | |
| B15 | Change **only** the budget, OK, and open `data\settings.json` | `maxBackups` is the new number; `language` is untouched — including a hand-placed unreadable one (stage `"language": "fr"` beforehand, which must still be there) — as are any unknown fields and the file's key order | |
| B16 | Change **only** the language and OK | `language` is the new code, the running window is **still in the old language** — menu bar, tabs and Banner unchanged — and nothing is spoken. Restart: the new language is in force | |
| B17 | Open Settings three times: change nothing and press OK, then press [Cancel], then press Escape | `settings.json` is byte-identical after all three, and each time `NVDA+Tab` confirms focus is back on the control it was on before the menu was opened | |
| B18 | Tools → Settings… in the step-17 unwritable-`data\` run | The item is **available**, and inside the dialog the selector, both fields, both checkboxes and [OK] all read as unavailable while the settings are still shown — the checkboxes still speaking their state; Cancel is where focus starts and the only way out besides Escape | |
| B19 | Set the budget to 2, OK, then Apply three times **without restarting** | `data\backups\` holds exactly 2 files for that Scope — the new budget is in force from the next rotation, not from the next run | |
| B20 | Set `"language": "fr"` by hand, restart, open Settings and select "Follow the system language" (already selected), OK | Nothing is written, `"fr"` still stands, and the `WARN settings:` line recurs at the next start — the file really does still say something this version cannot read. Choosing a language, OK, reopening and choosing back clears it | |
| B21 | With the app running, hold `data\settings.json` open exclusively (or deny yourself write on that file), then change a setting and press OK | Dialog "Settings could not be written — nothing was changed" with a single OK; the log gains one `WARN settings:` line; reopening Settings shows the **old** values, and once the file is released the same change goes through on a second OK | |
| B22 | Set the delay to `3000`, OK, then type one character in a Search field **without restarting** | The rows narrow and the count speaks about three seconds later, not a quarter of one — the new delay is in force from the next keystroke. Back to `250` and the same gesture speaks at once | |
| B23 | Clear "Speak filtered entry counts", OK, then narrow a Scope and clear it again | Narrowing says nothing at all — items 9, 10 and 11 are silent, on typing, on a tab switch and on an Expansion toggle — while the rows still narrow and the StatusBar still counts them. Clearing both narrowings still speaks Announcement 1: the switch silences the filtered counts and nothing else | |
| B24 | Clear "Escape returns focus to the list", OK, then press ESC in a Search field | The field clears and focus **stays in it**; with the box checked, focus returns to the list. Either way ESC on an already-empty field says nothing | |
| B25 | `Ctrl+N` on a Scope's list | The Add-entry dialog opens — the same one the **Add** button opens, its title and its field spoken. Then the same keystroke on the Backups tab, and again with a Search text or a Filter narrowing the list: **Edit → Add Entry…** reads as unavailable and the key does nothing, both times | |

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
| L8 | Look at the title bar, the taskbar button and `Alt+Tab` | Each shows the PathMaster icon, never the generic Windows one. **Two separate mechanisms**: the exe resource governs Explorer and pinned shortcuts, `Frame::set_icon` governs these three — a build can get one right and the other wrong | |
| L9 | Look at the `.exe` in Explorer, at Large icons and at Details, and open its Properties → Details tab | The icon is the same design at both sizes; the tab reads product PathMaster, company Ruslan Iskov, and the version being released | |

## E. Non-NVDA release checks

| # | Step | Expected result | ✓ |
|---|------|-----------------|---|
| E1 | `README.uk.md` is in sync with `README.md`, or the release did not change the README | drift guard (ticket 22) | |
| E2 | Process Monitor, filtered to `PathMaster.exe`: normal session incl. one Browse use | No file/registry write outside `<exe dir>\data\`, the two PATH values, and ComDlg32 MRU after Browse | |
| E3 | Clean-VM run (no VC++ redistributable) | App starts and lists PATH. Required once for v0.1.0, then only when packaging changes; note "not repeated — packaging unchanged" otherwise | |

## F. Release-time actions (the pre-release checklist)

Sections A-E are about the application. This one is about **the release itself**, and its steps
run in this order: nothing below can be done before the one above it. **F6 and F9 are the
packaging half**, and like E3 they are required once for v0.1.0 and thereafter only when packaging
changes — F9 in particular is the one-time seeding of the bucket, and F10 is what replaces it
every release after. F5 is not one of them and runs every release: it is the step that proves the
instruction given to users still works. F7 and F8 — the winget submission — are **deferred
indefinitely** and
live in their own block after the table: scoop and direct download are the release channels until
that decision is revisited.

> **Precondition for the whole section: the repository must be public.** Not a preference — a
> release from a private repository does not work at all. The asset URLs both manifests download
> from are unreachable, `LicenseUrl` 404s, and the README's own CI and release badges render
> broken. Check it once, before F1, and the rest of this section is about the software rather
> than about permissions.

| # | Step | Expected result | ✓ |
|---|------|-----------------|---|
| F1 | If this release changed the window: `tools\make-screenshots.ps1`, then check the alt text in `README.md` and `README.uk.md` still describes what the picture shows | Both images regenerate at 900×650 and E1's sync guard passes. The script refuses to run if a row meant to read as healthy would flag `Duplicate` against this machine's System PATH | |
| F2 | Bump the version in **three** files — `Cargo.toml`, `crates/pathmaster/resources/app.rc`, and `CHANGELOG.md`, where `[Unreleased]` is renamed to the version being released with today's date, a fresh empty `[Unreleased]` is opened above it, and the link references at the foot are updated — then `cargo update --workspace` and `just ci` | `the_versioninfo_carries_the_crate_version` and `the_newest_released_section_carries_the_crate_version` pass — the two of the three version legs a tag cannot check, plus the section the release page is about to publish as its body; `every_version_heading_carries_its_link_reference` covers the foot of the file, the one maintenance point the format adds beyond the headings. **`Cargo.lock` is a fourth file, and it is not hand-edited**: it records the workspace version, and every gate that matters here — `just ci`, push CI, the release workflow — runs `--locked`, which fails on a stale lock rather than quietly refreshing it. `cargo update --workspace` moves the three members and nothing else; commit it with the other three | |
| F3 | Push that commit to `develop`, let push CI go green, then cut the release with `git flow release start` / `finish`, which merges into `main` and tags it there — **then check the tag's name before pushing it** | The release workflow re-runs the whole gate itself — `cargo fmt --check`, `cargo test`, `cargo clippy` — before it builds anything, and only then the three-way version gate. **The tag has to land on `main`**, because a hotfix branches from `main`: tag `develop` instead and the first urgent fix has an empty branch to start from. **And it has to be named `v<version>`**: `git-flow-next` tags with the branch name and has no version-tag prefix setting at all (`git flow config edit topic release` knows only `--tag=true \| false`), so `finish` leaves `0.2.0` where `release.yml` waits on `v*`. A tag without the `v` starts nothing — no build, no release page, and no red anything; you find out by waiting. Rename it before the push: `git tag -d <version>` then `git tag -a v<version> -m "PathMaster v<version>" <commit>`. Push CI watches `develop` while the tag lands on `main`, so the tagged commit may be one push CI never saw — going through `develop` first gets you the answer sooner, it is not the only thing standing between a release and untested code | |
| F4 | Watch the release workflow | Every gate green — imports (no `VCRUNTIME`/`MSVCP`/`api-ms-win-crt`), size ≤ 40 MB, VERSIONINFO read back out of the linked artifact. The release page carries **exactly two** assets: the `.exe` and its `.sha256`. The PDB is a workflow artifact and is **not** on the release. The release page's **body** is that version's own `CHANGELOG.md` section | |
| F5 | Download both assets afresh and run the README's own `Get-FileHash` comparison | `True`. Every release: this is the step that proves the instruction given to users actually works, not just that a hash was written | |
| F6 | Clean VM, no VC++ redistributable: run the downloaded `.exe` (this is E3, done here on the released file) | SmartScreen shows the warning the README describes and "More info → Run anyway" gets past it; the app starts and lists PATH. **Note the VM's Windows build** — the deferred winget manifest claims a floor of 10.0.19044 (the spec §1 pin) and nothing has tested below it | |
| F9 | **Once, for the first release only** — copy `packaging/scoop/pathmaster.json` into the bucket's `bucket/` by hand, with the hash from F5 and the matching `version` and `url`, and push. Then `scoop install pathmaster` | The bucket's CI validates the manifest and does **not** revert the push. Installs; the shim launches the app with **no console flash**; a Start Menu shortcut exists; `~\scoop\persist\pathmaster\data` holds the Data Directory, and `scoop update` leaves it alone. By hand because the Excavator only updates manifests it already sees — F10 can bump a manifest, never seed it | |
| F10 | **Every release after that** — in the **bucket** repository, Actions → Excavator → Run workflow. Then `scoop update pathmaster` | The run itself is what must be green: it commits `pathmaster: Update to version <version>`, and **the bucket's own CI does not run on that commit at all** — the Excavator pushes with `github.token`, and GitHub does not trigger workflows on such a push, so there is no run to go looking for. Nothing is lost by that: the Excavator validates the manifest itself, and the bucket's CI is there for the manifests that arrive by hand (F9, and the override workflow). The log names every manifest it checked, `pathmaster` among them — `SKIP_UPDATED` is deliberately off there so "it did not see my app" and "my app was already current" cannot look alike. The upgrade keeps `~\scoop\persist\pathmaster\data` with its settings and Snapshots. **A red run is the expected way to learn something is wrong** (`THROW_ERROR` is on): a `checkver` failure means the release page or the tag shape moved, and a hash failure means the sidecar and the asset disagree — go back to F5 in that case, and do not hand-edit the bucket's manifest to get past it. Skipping this step delays nothing more than a day: the Excavator's daily run picks the release up on its own | |

### Deferred: winget (F7-F8)

The winget submission is postponed indefinitely. The manifests in `packaging/winget/` stay
finished and identity-guarded, so taking this up again costs only the version and the F5 hash of
whichever release is being submitted; these two steps then run once, after that release's F5,
exactly as written. Until then they are not part of any release pass — not even as
skipped-with-reason rows.

| # | Step | Expected result | ✓ |
|---|------|-----------------|---|
| F7 | Submit `packaging/winget/` (hash filled in from F5) as a PR to microsoft/winget-pkgs, then `winget install RuslanIskov.PathMaster` on a clean machine | Installs; `pathmaster` is a command; **`data\` is created beside the real exe in the package folder, not in winget's shared `Links\` directory** — the symlink-resolve rule (spec §3), and the one thing only a live install can show | |
| F8 | On that machine: make a change, Apply, then `winget upgrade`, then `winget uninstall` | `upgrade` keeps `data\` with its settings and Snapshots; `uninstall` deletes the package folder and `data\` with it — both exactly as the README said while the winget section stood; restore that section as part of this block | |

CI gates (version gate, `cargo test`, dumpbin imports, exe ≤ 40 MB, VERSIONINFO) run
automatically and are not part of this manual pass.
