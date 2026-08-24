# PathMaster

*English · [Українська](README.uk.md)*

PathMaster edits the Windows `PATH` — the list of folders Windows searches when you type a
command. It shows your `PATH` and the machine's `PATH` as plain lists you can read, reorder and
correct, tells you which entries are broken and why, and saves a copy of a list before it changes
it. It is built for a screen-reader user first, and it is one portable executable that keeps
everything it writes in a folder beside itself.

<!-- Screenshot of the main window, with full alt text, goes here at release time. -->

## Accessibility

This is the part of PathMaster that is not a feature. Everything else was decided after it.

- **NVDA is the screen reader it is tested with.** Every release is verified by a manual NVDA
  pass — see [How releases are verified](#how-releases-are-verified).
- **JAWS and Narrator are not deliberately broken and are not tested.** PathMaster uses stock
  Windows controls and makes no custom accessibility calls at all, so anything that reads a
  native list and a native menu should read this application. Nobody has measured that.
- **Nothing is audio-only.** Every message the application speaks is also shown, in one message
  line above the tabs. Nothing sets a colour anywhere, so High Contrast works because there is
  nothing punching through it.
- **Every action has a keyboard route and a menu home.** No scenario requires a mouse — see the
  [keyboard map](#keyboard).
- **An entry's problems are read as part of the row.** Arrowing onto an entry reads the path and
  then, only when something is wrong with it, "Status: " and the problem words.

### If NVDA goes quiet on the list

There is one known state — seen once, never reproduced — in which NVDA treats the list as a
single object and announces nothing as you arrow through it. It is not a state PathMaster can
currently detect, and while it lasts the spoken message line is very likely silent too.

**Check for it like this:** focus a row in a list and press `NVDA+Tab`. NVDA must answer with the
**row** ("list item"). If it answers with the list, you are in this state.

**Getting out of it, in order:**

1. `Alt+Tab` away to another window and back.
2. Close PathMaster and start it again.
3. Restart NVDA. This always works.

Nothing is lost by any of these except unsaved edits, and PathMaster asks before discarding
those.

### The Administrator window needs an installed NVDA

Editing the machine's `PATH` needs administrator rights, and PathMaster gets them by restarting
itself (Tools → Restart as Administrator). **A portable NVDA cannot read that window** — Windows
does not let it — so use an installed NVDA if you need to edit the System `PATH`.

## Install

### winget

```powershell
winget install RuslanIskov.PathMaster
```

### scoop

```powershell
scoop bucket add ruslan-rv-ua https://github.com/ruslan-rv-ua/scoop-bucket
scoop install pathmaster
```

### Direct download

Take `PathMaster-v<version>-x64.exe` and the `.sha256` file beside it from the
[releases page](https://github.com/ruslan-rv-ua/pathmaster2/releases). There is no archive to
unpack and no installer to run: the `.exe` is the whole application. Put it wherever you want it
to live — it will create its `data` folder next to itself.

#### Windows will warn you, and that is expected

PathMaster is **not code-signed**. Signing costs money every year and buys reputation only after
enough people have installed the signed binary, so it is deliberately deferred until this
application has users worth the trouble. Because of that, Windows SmartScreen shows
"Windows protected your PC" the first time you run it. Choose **More info**, then
**Run anyway**.

That warning means "nobody has vouched for this file", not "this file is broken" — so here is how
to check the file yourself.

#### Verify what you downloaded

The `.sha256` file published beside each release holds the exact fingerprint of that release's
`.exe`. Compare it with the file you have:

```powershell
Get-FileHash .\PathMaster-v0.1.0-x64.exe -Algorithm SHA256
Get-Content .\PathMaster-v0.1.0-x64.exe.sha256
```

The long hexadecimal string must be the same in both. (Upper and lower case do not matter.) To
have PowerShell answer with one word instead of two lines:

```powershell
(Get-FileHash .\PathMaster-v0.1.0-x64.exe -Algorithm SHA256).Hash -eq ((Get-Content .\PathMaster-v0.1.0-x64.exe.sha256) -split ' ')[0]
```

`True` means the file is byte-for-byte the one that was published. Anything else means do not run
it.

## Keyboard

The full map. Menu items name their own shortcuts, so a screen reader reads them out as you move
through the menus — this table is the same information in one place.

| Keys | What it does |
|---|---|
| `Tab` / `Shift+Tab` | Move through every control on the tab, in a cycle with no traps |
| `Ctrl+Tab` / `Ctrl+Shift+Tab` | Next / previous tab: User PATH, System PATH, Backups |
| `↑` `↓` | Move through the list |
| `F2`, `Enter`, or double-click | Edit the entry the cursor is on |
| `Del` | Delete the entry the cursor is on — no confirmation, because `Ctrl+Z` brings it back |
| `Alt+↑` / `Alt+↓` | Move the entry one place earlier / later |
| `Ctrl+Z` / `Ctrl+Y` | Undo / redo, announcing what was undone |
| `Ctrl+S` | Apply — write this PATH to the registry, after saving a backup of it |
| `F5` | Refresh — re-read this PATH from the registry |
| `Alt+F4` | Close, asking first if anything is unsaved |
| `Alt+F` / `Alt+E` / `Alt+T` / `Alt+H` | Open the File / Edit / Tools / Help menu |
| `NVDA+End` | Read the status bar: the entry and problem counts, and the merged PATH length |
| `NVDA+Tab` | Not a PathMaster shortcut — the check described under [If NVDA goes quiet](#if-nvda-goes-quiet-on-the-list) |

Apply and Cancel Changes are unavailable while a list has no unsaved changes, and every menu item
reads as unavailable when it cannot be used, which is how a screen reader tells you so.

## What gets written where

PathMaster is portable, and that is a claim worth being precise about.

**The application itself writes to two places and nowhere else:**

- `data\` beside the executable — `settings.json`, `backups\` (the saved copies), and
  `pathmaster.log`. Nothing it writes lives anywhere else, and there is no setting that moves
  this folder.
- The two `PATH` values, when you Apply: `HKCU\Environment` → `Path` for your own, and
  `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment` → `Path` for the machine's.

**With one named exception.** The **Browse** button in the Add/Edit dialog opens Windows' own
folder picker, and Windows records the folders you visited in its own "recently used" registry
keys (`HKCU\...\Explorer\ComDlg32`). That is the operating system writing, in this process, and
PathMaster cannot prevent it. If you never press Browse, it never happens.

**What the package managers write is theirs, not ours:**

- **winget** copies the exe to its own package folder under `%LOCALAPPDATA%`, **renames it to
  `pathmaster.exe`**, adds an entry under `HKCU` so it appears in Apps & features, and puts a
  symbolic link in its Links folder — which is on your user `PATH`. That is how `pathmaster`
  becomes a command you can type.
  - `winget upgrade` **keeps** your `data` folder, settings and backups included.
  - `winget uninstall` **deletes** the package folder, and your `data` folder with it —
    **backups included**. Copy anything you want to keep first.
- **scoop** installs into `~\scoop\apps\pathmaster`, puts a shim on your `PATH`, adds a Start
  Menu shortcut, and — because the manifest asks it to — moves your `data` folder into
  `~\scoop\persist\pathmaster\data` and links it back. Your settings and backups therefore
  survive `scoop update`.

## Settings

`data\settings.json` is a plain JSON file and you may edit it by hand. It holds your interface
language, how many backups to keep per PATH, and where the window was last left. Tools →
Settings… changes the first two.

If the file cannot be read at all — broken JSON, or not text — PathMaster **does not overwrite
it**. It renames it to `settings.json.bad` (one copy; the next incident replaces it), starts on
defaults, and tells you so in a dialog. Whatever you had is still there in the `.bad` file.

If one *value* is wrong — a language code this version does not know, a negative number — only
that setting falls back to its default, in memory, and **the file keeps what you wrote** until
you change that same setting in the Settings dialog. Fields PathMaster does not recognise are
left alone and survive every rewrite, as does the order of the keys.

## What PathMaster deliberately does not do

Each of these is a decision, not an omission.

- **It does not guess at typos.** `C:\Python312` and `C:\Python313` are both perfectly good
  folders, and a diagnostic that cried wolf about them would make every other diagnostic worth
  less. It reports what it can be sure of: missing folders, relative paths, quoted entries,
  duplicates, empty entries, and a merged PATH over the length `cmd.exe` can use.
- **It never touches network paths.** An entry on a `\\server\share` is never checked for
  existence and never flagged as missing: a dead UNC path blocks for 20 to 60 seconds and cannot
  be cancelled, and a file manager that freezes is worse than one that says less.
- **It has no theme setting.** It uses your Windows colours, always, and sets none of its own.
- **It is not signed**, by decision — see [above](#windows-will-warn-you-and-that-is-expected).

Features that are merely *not here yet* are in the issue tracker, not in this list.

## How releases are verified

There is no automated test of this application's user interface, and there is deliberately no
badge in this README claiming otherwise. What there is instead is
[a checklist](docs/release-checklist.md): a written script of every step, each one naming the
exact words NVDA is expected to speak, run personally on real NVDA before every release.

**Every release attaches a filled-in copy of that checklist**, naming the NVDA version it was run
with and marking every step passed, failed or skipped. A failed step blocks the release. For an
unsigned binary, that recorded manual pass is the honest trust signal — and you can read the
script before you decide whether to trust it.

Automated gates run on every release too, on the published file rather than on the build that
made it: it must have no dependency on the Visual C++ runtime, must be under 40 MB, and must
carry the same version in the tag, the source and the executable's own properties.

## Licence

[MIT](LICENSE). Copyright (c) 2026 Ruslan Iskov.
