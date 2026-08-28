# PathMaster

*English · [Українська](README.uk.md)*

Portable editor and diagnostics for the Windows `PATH` environment variable.

[![CI status](https://github.com/ruslan-rv-ua/pathmaster/actions/workflows/ci.yml/badge.svg)](https://github.com/ruslan-rv-ua/pathmaster/actions/workflows/ci.yml)
[![Latest release](https://img.shields.io/github/v/release/ruslan-rv-ua/pathmaster)](https://github.com/ruslan-rv-ua/pathmaster/releases/latest)
[![Licence: MIT](https://img.shields.io/badge/licence-MIT-blue)](LICENSE)
![Windows 10 and 11, 64-bit](https://img.shields.io/badge/Windows-10%20%7C%2011%20x64-blue)

![The PathMaster main window: a message line reading "User PATH: 10 entries", three tabs (User PATH, System PATH, Backups), an empty search field, and a three-column list of #, Path and Status, its rows numbered 1 to 10. Healthy entries leave Status empty; the flagged ones read Missing, "Missing, Quoted", Relative, Duplicate and Empty. A row of buttons follows — Add, Edit, Delete, Move Up, Move Down, Apply, Cancel Changes — and the status bar counts entries and issues for each PATH and gives the merged PATH length.](docs/images/main-window-en.png)

*The PATH in that picture is a deliberately broken example: it carries one of every problem
PathMaster reports.*

## Contents

- [Features](#features)
- [Install](#install)
- [Keyboard](#keyboard)
- [What gets written where](#what-gets-written-where)
- [Settings](#settings)
- [What PathMaster deliberately does not do](#what-pathmaster-deliberately-does-not-do)
- [Troubleshooting](#troubleshooting)
- [How releases are verified](#how-releases-are-verified)
- [Contributing](#contributing)
- [Licence](#licence)

## Features

- **Both PATHs in one place.** Yours and the machine's, each as a plain list you can read,
  reorder and correct. Nothing reaches the registry until you apply it.
- **Find your way around a long PATH.** Search as you type, filter the list down to the entries
  with a given problem, or open the whole PATH as a tree to see its shape. `Ctrl+E` reads every
  entry with its `%VARIABLES%` expanded, and `Ctrl+C` copies an entry exactly as shown.
- **It tells you what is broken.** Folders that do not exist, relative paths, entries wrapped in
  quotes, duplicates — across both PATHs, not just within one — empty entries, and a combined
  PATH longer than `cmd.exe` can use.
- **And it offers to fix it.** Fix Issues proposes one action per broken entry — delete it, or
  take the quotes off — you tick the ones you agree with, and a single `Ctrl+Z` takes the whole
  batch back.
- **Nothing is irreversible.** Full undo and redo, and a copy of a PATH is saved before that PATH
  is changed. The Backups tab loads any saved copy back as an ordinary, undoable edit.
- **Accessibility first.** Universal design throughout, not a mode: every action has a keyboard
  route and a home in the menus, every message is shown as well as spoken, and the application
  sets no colours of its own — so your Windows theme, including High Contrast, simply applies.
  `F1` opens a full user guide in your browser, where your screen reader's own browse mode,
  heading navigation and find all work. Tested with NVDA.
- **Portable.** One executable. Everything it writes lives in a `data` folder beside it.
- **English and Ukrainian**, chosen in Settings or followed from Windows.

## Install

### scoop

```powershell
scoop bucket add ruslan-rv-ua https://github.com/ruslan-rv-ua/scoop-bucket
```

```powershell
scoop install pathmaster
```

### Direct download

Take `PathMaster-v<version>-x64.exe` and the `.sha256` file beside it from the
[releases page](https://github.com/ruslan-rv-ua/pathmaster/releases). There is no archive to
unpack and no installer to run — the `.exe` is the whole application. Put it where you want it to
live; it creates its `data` folder next to itself.

PathMaster is **not code-signed**, so Windows SmartScreen shows "Windows protected your PC" the
first time you run it: choose **More info**, then **Run anyway**. Signing costs money every year
and only starts to count once enough people have installed the signed binary, so it is
deliberately deferred.

That warning means "nobody has vouched for this file", not "this file is broken" — so check the
file yourself. The published `.sha256` holds the exact fingerprint of that release:

```powershell
$exe = Get-Item .\PathMaster-v*-x64.exe
$want = ((Get-Content "$exe.sha256") -split ' ')[0]
$got = (Get-FileHash $exe -Algorithm SHA256).Hash
if ($want -and $got -and $want -eq $got) { 'MATCHES the published fingerprint' } else { 'DOES NOT MATCH - do not run it' }
```

It prints **MATCHES the published fingerprint** when the file is byte-for-byte the one that was
published. Anything else — including any error above the answer — means do not run it.

The check says the version nowhere, deliberately: it finds whichever `PathMaster-v…-x64.exe` you
downloaded. And it answers in words rather than with `True`, because the obvious one-line form
of this check compares nothing to nothing when a file is missing, and `True` is exactly what
PowerShell says to that.

## Keyboard

Everything is reachable the ordinary way — tabs, arrows, menus. These are the bindings particular
to PathMaster, and each one also names itself on its own menu item:

| Keys | What it does |
|---|---|
| `F1` | Open the user guide in your browser |
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
| `Ctrl+E` | Switch between expanded and stored values |
| `Ctrl+T` | Open the PATH Tree |
| `Space` | In Fix Issues: tick or untick the row |

Apply and Cancel Changes are unavailable while a list has no unsaved changes, and every menu item
reads as unavailable when it cannot be used.

## What gets written where

PathMaster is portable, and that is a claim worth being precise about.

**The application writes to two places and nowhere else:**

- `data\` beside the executable — `settings.json`, `backups\`, `pathmaster.log`, and `help.html`,
  the user guide, rewritten every time you press `F1`. There is no setting that moves this
  folder.
- The two `PATH` values, when you apply: `HKCU\Environment` → `Path` for your own, and
  `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment` → `Path` for the machine's.

**One launch at a time can point somewhere else.** `PathMaster.exe --data-dir <path>` puts that
run's `data\` where you say, and remembers nothing — the next plain launch is back beside the
executable. A relative path is read from the folder you ran the command in, the folder is created
if it is not there yet, and if it cannot be created or written the application starts in read-only
mode and says so: it never quietly falls back to the default `data\`. "Restart as Administrator"
carries the same location across, so the elevated instance writes where the unelevated one did.

`PathMaster.exe --help` (or `-?`) shows the command line in a dialog and exits. An argument
PathMaster does not recognise gets a dialog naming it, a line in the log, and then a normal start —
a mistyped switch never passes silently.

**With one named exception.** The **Browse** button opens Windows' own folder picker, and Windows
records the folders you visited in its own "recently used" registry keys
(`HKCU\...\Explorer\ComDlg32`). That is the operating system writing, inside this process, and
PathMaster cannot prevent it. If you never press Browse, it never happens.

**What the package manager writes is its own, not ours:**

- **scoop** installs into `~\scoop\apps\pathmaster`, puts a shim on your `PATH`, adds a Start Menu
  shortcut, and links your `data` folder out to `~\scoop\persist\pathmaster\data`, so settings and
  saved copies survive `scoop update`.

## Settings

`data\settings.json` is plain JSON and you may edit it by hand. It holds the interface language,
how many saved copies to keep per PATH, three settings for the search field and the counts it
speaks, and where the window was last left. Tools → Settings… changes all of those but the last.

If the file cannot be read at all, PathMaster **does not overwrite it**: it renames it to
`settings.json.bad` (one copy; the next incident replaces it), starts on defaults, and says so in
a dialog. Whatever you had is still in the `.bad` file.

If one *value* is wrong — a language code this version does not know, a negative number — only
that setting falls back to its default, in memory, and **the file keeps what you wrote** until you
change that same setting in the dialog. Fields PathMaster does not recognise are left alone, as is
the order of the keys.

## What PathMaster deliberately does not do

- **It does not guess at typos.** `C:\Python312` and `C:\Python313` are both perfectly good
  folders, and a diagnostic that cried wolf about them would make every other diagnostic worth
  less.
- **It never touches network paths.** An entry on a `\\server\share` is never checked for
  existence and never reported missing: a dead UNC path blocks for 20 to 60 seconds and cannot be
  cancelled.
- **It has no theme setting.** Your Windows colours, always.
- **It is not signed**, by decision — see [Direct download](#direct-download).

Things that are merely *not here yet* live in the issue tracker, not in this list.

## Troubleshooting

**Editing the machine's PATH** needs administrator rights, and PathMaster gets them by restarting
itself: Tools → Restart as Administrator. A *portable* NVDA cannot read an elevated window —
Windows does not allow it — so use an installed one if you need the System PATH.

**If a screen reader stops reading the list** — rare, seen once and never reproduced — press
`Alt+Tab` away and back; if that does not help, restart PathMaster; if it still does not, restart
the screen reader, which always fixes it. To tell this state from a genuinely empty list,
`NVDA+Tab` on a focused row should answer with the row ("list item"), not with the list.

## How releases are verified

There is no automated test of this application's user interface. Instead there is
[a checklist](docs/release-checklist.md) — a written script of every step, naming the exact words
a screen reader is expected to speak — run by hand before every release. A failed step blocks the
release.

Automated gates run too, on the published file rather than on the build that made it: no
dependency on the Visual C++ runtime, under 40 MB, and the same version in the tag, the source and
the executable's own properties.

## Contributing

Questions, bug reports and feature requests belong in
[Issues](https://github.com/ruslan-rv-ua/pathmaster/issues). Pull requests are welcome; for
anything larger than a fix, open an issue first so the design can be agreed before the work.

## Licence

[MIT](LICENSE). Copyright (c) 2026 Ruslan Iskov.
