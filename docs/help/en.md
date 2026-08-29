# PathMaster — User Guide

This is the guide for using PathMaster. It is one page, so the browser's own
navigation works on it: jump by heading, list the headings, or search the whole
text at once. Press `F1` in PathMaster at any time to open it again.

## What PATH is

`PATH` is a list of folders Windows searches when you type a command name
without saying where the program lives. Type `python` in a command prompt and
Windows walks the list from the top, folder by folder, and runs the first
`python.exe` it finds. Nothing else about your computer changes when the list
changes — but which program answers a command name does, and so does whether a
command answers at all.

There are two lists, and Windows merges them:

- **User PATH** is yours. It belongs to your Windows account, and changing it
  needs no special permission.
- **System PATH** is the machine's. Every account on the computer gets it, and
  changing it needs administrator rights.

A program launched from a command prompt sees the System list first, then the
User list, joined into one. That merge matters in two ways PathMaster reports
on: the same folder can appear in both lists, and the two lists together have a
length limit.

Changes to `PATH` reach a program when that program starts. Command prompts,
terminals and editors that were already open keep the list they were given, so
open a new one after applying.

## The window

The window is one message line, three tabs, and a status bar.

- **The message line** sits above the tabs. It carries the last thing
  PathMaster said — the entry count when you switch tabs, what an undo undid,
  why a change could not be applied. It is always visible and never carries
  anything that was not also spoken.
- **The tabs** are **User PATH**, **System PATH** and **Backups**. The two PATH
  tabs each hold a search field, a list and a row of buttons; Backups holds the
  saved copies and a Restore button.
- **The status bar** at the bottom holds two fields: the entry and issue counts
  for both PATHs, and the length of the merged PATH. It is never spoken on its
  own — read it on demand with `NVDA+End`, whenever you want the counts again
  after the message line has moved on.

The list has three columns:

- **`#`** — the entry's position in the PATH, counting from 1. It is the
  entry's real position and never changes when a search or a filter hides rows
  around it.
- **Path** — the entry itself.
- **Status** — what is wrong with it, or nothing at all when nothing is.

Pressing `Tab` walks the window in one cycle: tabs, then the search field, then
the list, then the buttons. Nothing traps the focus, and every command has a
menu home as well as a key.

## Editing entries

Nothing you do in the window touches the registry. Edits change a working copy,
the list shows that copy, and **Apply** (`Ctrl+S`) is the only thing that
writes — after saving a copy of the PATH it is about to overwrite.

- **Add** (`Ctrl+N`) appends a new entry at the end of the list.
- **Edit** (`F2`, `Enter`, or double-click) changes the entry the cursor is on.
  The dialog carries a **Browse** button that opens Windows' own folder picker.
- **Delete** (`Del`) removes it. There is no confirmation, because `Ctrl+Z`
  brings it back.
- **Move Up** and **Move Down** (`Alt+↑`, `Alt+↓`) change the order, which
  changes which of two same-named programs wins.
- **Undo** and **Redo** (`Ctrl+Z`, `Ctrl+Y`) walk the whole history of the
  session and say what they undid or redid.
- **Cancel Changes** throws away everything unapplied on that tab and goes back
  to what the registry holds.
- **Refresh** (`F5`) re-reads that PATH from the registry, which is how you pick
  up a change something else made while PathMaster was open.

Apply and Cancel Changes are unavailable while a list has no unsaved changes.
Every command that cannot be used right now reads as unavailable rather than
failing when you press it.

If the registry value changed under you between opening PathMaster and applying,
Apply stops and asks before overwriting it. If the merged PATH has grown past
what `cmd.exe` can use, Apply says so and lets you decide; past the hard limit
Windows itself enforces, it says so and stops.

## What the Status column says

The Status column carries one word per problem found, most serious first, joined
by commas — so an entry can read `Missing, Quoted`. An entry with nothing wrong
has an empty Status. There are five words:

- **Missing** — the folder is not there. Only local drives are checked, and only
  for a folder: a file at that path is missing as far as `PATH` is concerned.
  Network paths (`\\server\share`) are never checked and never reported, because
  a dead network path takes up to a minute to answer.
- **Relative** — the entry is not a full path from a drive root, so what it
  points at depends on where a program happened to be started. `tools\bin` is
  relative; `C:\tools\bin` is not.
- **Quoted** — the entry contains a `"` character. Quotes are not part of any
  Windows folder name, and `PATH` does not want them: they were almost certainly
  typed to protect a space that needs no protecting here.
- **Duplicate** — this folder already appeared earlier, in this list or in the
  other one. The comparison is on the expanded, case- and slash-insensitive
  reading, so `%SystemRoot%\system32` and `C:\Windows\System32` are the same
  folder said twice. Only the second and later appearances are flagged.
- **Empty** — the entry has no text at all, usually a stray `;` in a
  hand-edited value. An empty entry is never checked for anything else.

There is a sixth problem, and it belongs to the PATHs as a whole rather than to
any one entry, so it has no word in this column: **the merged PATH can be too
long**. The status bar's second field always shows its length. Past 8,191
characters, `cmd.exe` ignores the variable entirely — inside a command prompt
your `PATH` is simply absent — and PathMaster warns at Apply. At 32,767
characters it cannot be stored at all, and Apply refuses.

## Backups and restore

Every Apply saves a copy of the PATH it is about to overwrite, before it writes
anything. Those copies live under the **Backups** tab, newest first, each naming
its PATH and when it was taken.

**Restore** loads the copy you chose back into that PATH's working copy. It does
not write the registry: a restore is an ordinary edit, so `Ctrl+Z` undoes it,
and **Apply** is still what makes it real. That is deliberate — you can look at
what you restored before committing to it.

A copy that cannot be read, or that is not a valid saved copy any more, reads as
**[Corrupted]** and cannot be restored. It is still listed, because the file is
still there.

How many copies to keep per PATH is a setting; the oldest are removed past that
number, and each PATH is counted on its own. **Tools → Open Backups Folder**
opens the folder they live in, if you want to keep one somewhere else.

## What version 0.2.0 adds

Some of what is described above arrived with this version too — the `#` column,
the search field over each list, and the extra `Tab` stop it adds. These are the
rest of it, and each has its own key in the table below.

- **Search** (`Ctrl+F`) — the field above each list narrows it to the entries
  containing what you type. It matches what the list is currently showing,
  ignores case, and treats `/` and `\` as the same character; nothing else is
  interpreted, so searching for `"` finds the quoted entries. `Esc` clears the
  search and puts you back in the list.
- **Filter** (**View → Filter**) — narrows the list to one kind of problem: all
  entries, entries with any problem, or exactly one of the five Status words.
  `Ctrl+I` is the coarse switch, without opening the menu: from *All* it goes to
  *With issues*, and from any other state back to *All*.
- **Expanded values** (`Ctrl+E`) — shows every `%VARIABLE%` resolved to what it
  currently stands for, throughout the list. It changes only what is displayed:
  editing always works on the text as stored, and a variable that stands for
  nothing is left as it is written. The mode goes back to raw when you restart.
- **PATH Tree** (`Ctrl+T`) — shows the current PATH as a folder tree, which is
  the quickest way to see that four entries all live under one Java
  installation. `Enter` on an entry closes the tree and puts the cursor on that
  entry in the list.
- **Fix Issues** (`Ctrl+Shift+I`, or **Edit → Fix Issues…**) — one row per
  entry that can be repaired, each with the repair it would get: delete the
  entry, or remove its quotes. `Space` ticks and unticks a row; **Fix
  selected** applies the ticked ones as a single undoable edit. Nothing reaches
  the registry — Apply still does that.
- **Copy** (`Ctrl+C`) — puts the entry the cursor is on onto the clipboard,
  exactly as the list is showing it, which is how you get an expanded value out
  of PathMaster.
- **This guide** (`F1`), and the `--data-dir` command-line switch described
  below.

Search, filter and expanded values are ways of *looking*, never of changing:
they are not part of undo, they are not saved, and every run starts with none of
them in force. While a search or a filter is narrowing a list, **Add**, **Move
Up** and **Move Down** are unavailable — moving an entry among rows you cannot
see would be a change you cannot check.

## Keyboard

Everything is reachable the ordinary way, through menus and arrow keys. These
are the bindings particular to PathMaster, and every one of them also names
itself on its own menu item.

| Keys | What it does |
|---|---|
| `F1` | Open this guide |
| `Ctrl+N` | Add a new entry, appended at the end of the list |
| `F2` | Edit the entry the cursor is on (`Enter` and double-click do the same) |
| `Del` | Delete it — no confirmation, because `Ctrl+Z` brings it back |
| `Alt+↑` / `Alt+↓` | Move it one place earlier / later |
| `Ctrl+C` | Copy it, exactly as the list is showing it |
| `Ctrl+Z` / `Ctrl+Y` | Undo / redo, saying what was undone |
| `Ctrl+S` | Apply — write this PATH to the registry, after saving a copy of it |
| `F5` | Refresh — re-read this PATH from the registry |
| `Ctrl+F` | Move to the search field and select what is already in it |
| `↓` or `Tab` | From the search field into the list |
| `Esc` | In the search field: clear it and return to the list |
| `Ctrl+I` | Coarse filter switch: *All* → *With issues*, anything else → *All* |
| `Ctrl+Shift+I` | Open Fix Issues over this PATH |
| `Ctrl+E` | Switch between expanded and stored values |
| `Ctrl+T` | Open the PATH Tree |
| `Space` | In Fix Issues: tick or untick the row |
| `Alt+F4` | Close PathMaster |

`F1` opens this guide from the main window. Inside a dialog it does nothing —
close the dialog first.

## Settings

**Tools → Settings…** holds five things:

- **Language** — English or Ukrainian, or following whatever Windows is set to.
  It takes effect the next time PathMaster starts; everything else takes effect
  as you leave the dialog.
- **Snapshots to keep per PATH** — how many saved copies survive per PATH before
  the oldest are removed.
- **Speak filtered entry counts** — whether narrowing a list says how many
  entries it now shows. Turn it off if you would rather read the count from the
  status bar.
- **Delay before speaking the count (ms)** — how long typing has to stop before
  that count is spoken. The default is 250; raise it if the count interrupts
  your typing, lower it to 0 if you would rather hear it at once.
- **Escape returns focus to the list** — whether `Esc` in the search field moves
  the cursor into the list as well as clearing the field.

The settings live in `data\settings.json`, which is plain JSON you may edit by
hand. PathMaster writes back only the settings you actually changed, and leaves
anything else in the file — including keys it does not recognise — exactly as it
found them.

If the whole file cannot be read, PathMaster does not overwrite it: it renames it
to `settings.json.bad`, starts on defaults, and says so in a dialog. If one
*value* is wrong — a language code this version does not know, a negative number
— only that setting falls back to its default, in memory, and the file keeps what
you wrote until you change that same setting in the dialog. Either way the log
records what happened.

## The System PATH and administrator rights

Reading the System PATH needs nothing special: the tab is there, the list is
there, and the problems are reported the same way. **Changing** it needs
administrator rights, and PathMaster does not ask Windows for them until you say
so. Until then, every editing command on that tab reads as unavailable.

**Tools → Restart as Administrator** is how you get them. PathMaster asks
Windows for a fresh copy of itself with administrator rights, and Windows asks
you to confirm. The new window opens on the same tab and says in its title that
it is elevated. If you have unapplied changes, you are asked first, because they
do not survive the restart.

One consequence worth knowing before you start: **a portable NVDA cannot read an
elevated window.** Windows does not permit it, and no application can work
around it. Use an installed NVDA if you need to edit the System PATH.

## What is written where

PathMaster is portable, and that is a claim worth being precise about. It writes
to two places and nowhere else.

**A `data` folder beside the executable**, holding:

- `settings.json` — the settings, and `settings.json.bad` if one was ever set
  aside.
- `backups\` — the saved copies of both PATHs.
- `pathmaster.log` — this run's log, and `pathmaster.log.old` the previous
  run's. It records what happened, never what your PATH contains.
- `help.html` — this guide, rewritten every time you press `F1`.

**The two registry values**, when you apply: `HKCU\Environment` → `Path` for the
User PATH, and
`HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment` → `Path` for
the System PATH.

With one named exception: the **Browse** button opens Windows' own folder picker,
and Windows records the folders you visited in its own "recently used" registry
keys. That is the operating system writing, inside this process, and PathMaster
cannot prevent it. If you never press Browse, it never happens.

### Command line

Three switches, and nothing else is recognised:

```
PathMaster.exe [--tab user|system|backups] [--data-dir <path>] [--help]
```

- **`--tab user|system|backups`** opens on that tab instead of on User PATH.
- **`--data-dir <path>`** puts *this launch's* `data` folder where you say —
  `--data-dir=<path>` works too. The folder is created if it is not there yet,
  and a relative path is read from the folder you ran the command in. It
  remembers nothing: the next plain launch is back beside the executable. If the
  location cannot be created or written, PathMaster starts read-only and says
  so; it never quietly falls back to the default `data` folder. **Restart as
  Administrator** carries the location across, so the elevated instance writes
  where the unelevated one did.
- **`--help`** (or **`-?`**) shows that usage line in a dialog and exits without
  starting.

An argument PathMaster does not recognise gets a dialog naming it, a line in the
log, and then a normal start — a mistyped switch never passes silently.

## If something goes wrong

**A screen reader stops reading the list.** Rare, and it has never been
reproduced deliberately. Press `Alt+Tab` away and back; if that does not help,
restart PathMaster; if it still does not, restart the screen reader, which
always fixes it. To tell this state from a genuinely empty list, `NVDA+Tab` on a
focused row should answer with the row ("list item"), not with the list.

**The System PATH cannot be edited.** That tab is read-only until PathMaster is
running with administrator rights — see *The System PATH and administrator
rights* above.

**Everything is read-only, and the message line says so.** PathMaster could not
write its `data` folder, so it cannot save a copy before applying, and it will
not apply without one. It happens when the executable sits somewhere you cannot
write — a read-only drive, a folder needing administrator rights, or the same
folder handed to `--data-dir`. Move the executable somewhere writable, or point
`--data-dir` at a folder you own.

**A change did not take effect.** Programs read `PATH` when they start. Open a
new command prompt; if the change still is not there, check that you applied it
and not merely edited it — the tab remembers unapplied changes until you do.

**This guide did not open.** PathMaster writes it into `data\help.html` and asks
Windows to open it. If it could not write the file, it opens the copy on GitHub
instead, which needs a network connection. If nothing opened at all, Windows has
no program registered for `.html` — the log records that it tried.

**The log.** `data\pathmaster.log` records what each run did: the version, where
it wrote, and every failure with the error code Windows gave. It never records
what your PATH contains. It is the right thing to attach to a bug report.
