# Research: `--data-dir` — how portable Windows GUI apps take a directory from the command line

Supporting ticket [13-data-dir-switch](../issues/13-data-dir-switch.md).
Researched 2026-08-26, per the map's standing directive 7 (research before grilling).

## 1. Precedents — what the established apps actually do

| App | Switch & syntax | Missing directory | Relative path | Invalid target (file / unwritable) |
|---|---|---|---|---|
| **Chromium/Chrome** | `--user-data-dir=c:\foo` ([docs](https://chromium.googlesource.com/chromium/src/+/main/docs/user_data_dir.md)) | **Created recursively** (`PathService::OverrideAndCreateIfNeeded`, [chrome_main_delegate.cc](https://github.com/chromium/chromium/blob/main/chrome/app/chrome_main_delegate.cc)) | Made absolute via `_wfullpath()` → **against CWD** ([install_static/user_data_dir.cc](https://github.com/chromium/chromium/blob/main/chrome/install_static/user_data_dir.cc)) | **Falls back to the default dir**, records the bad path, browser shows a **warning dialog** and keeps running ("The browser process will handle the error later; other processes that need the dir crash here") |
| **VS Code** | `--user-data-dir <dir>` ([CLI docs](https://code.visualstudio.com/docs/configure/command-line)) | Created (not documented; standard Electron/Code behavior — **undocumented**, unverified here) | Not documented | Not documented. Note: **portable mode's `data` folder beside the exe overrides the switch entirely** ([portable docs](https://code.visualstudio.com/docs/editor/portable)) |
| **Firefox** | `-profile <path>` (also `--`, and `/` on Windows) ([wiki](https://wiki.mozilla.org/Firefox/CommandLineOptions)) | **Created** — `EnsureDirExists` → `aPath->Create(nsIFile::DIRECTORY_TYPE, 0700)` ([nsToolkitProfileService.cpp](https://github.com/mozilla-firefox/firefox/blob/main/toolkit/profile/nsToolkitProfileService.cpp)) | `XRE_GetFileFromPath` → `_wfullpath()` on Windows → **against CWD** | Console error `"Error: argument --profile requires a path to a directory"` and startup fails |
| **Telegram Desktop** | `-workdir <path>` ([launcher.cpp](https://github.com/telegramdesktop/tdesktop/blob/dev/Telegram/SourceFiles/core/launcher.cpp)) | **Created recursively** — `cForceWorkingDir` does `QDir().mkpath()` + user-only permissions ([settings.h](https://github.com/telegramdesktop/tdesktop/blob/dev/Telegram/SourceFiles/settings.h)) | `QDir(...).absolutePath()` → **against CWD**; a trailing `/` is appended by the app itself | No existence check at parse time; no error surface found (silent) |
| **qBittorrent** | `--profile=<dir>` ([portable-mode wiki](https://github.com/qbittorrent/qBittorrent/wiki/How-to-use-portable-mode)) | Not documented (wiki implies it reads configs "if the config files already exist inside it") | Not documented; companion `--relative-fastresume` exists for making stored paths relative | n/a for the switch itself; see §2 for its strict argument posture |
| **KeePass 2.x** | `-cfg-local:<path>` — overrides the **config file**, not a data dir; `-`/`--`/`/` prefixes all accepted, values after `:`, quotes required for spaces ([cmdline](https://keepass.info/help/base/cmdline.html)) | Not documented | Not documented | Not documented. Portable/installed is decided by the `PreferUserConfiguration` flag in the config beside the exe, not by a switch ([configuration](https://keepass.info/help/base/configuration.html)) |
| **Notepad++** | `-settingsDir="d:\your settings dir\"` — "Override the default settings dir" ([manual](https://npp-user-manual.org/docs/command-prompt/)) | **Refused with an error dialog**: "The given path … via command line \"-settingsDir=\" is not a valid directory. This argument will be ignored." — then **falls back to the default** ([Parameters.cpp](https://github.com/notepad-plus-plus/notepad-plus-plus/blob/master/PowerEditor/src/Parameters.cpp)). Not created. | Not documented | Same dialog-and-ignore path. Portable mode itself is a zero-byte `doLocalConf.xml` beside the exe ([config-files manual](https://www.npp-user-manual.org/docs/config-files/)) |
| **SumatraPDF** | `-appdata <directory>` — "set custom directory where we'll store SumatraPDF-settings.txt file and thumbnail cache" ([docs](https://www.sumatrapdfreader.org/docs/Command-line-arguments)) | Not documented | Not documented | Not documented |

Three convergences worth naming:

1. **`--long-name` with a directory value is the shape**; `=`-joined (Chromium, qBittorrent, Notepad++)
   and space-separated (Firefox, Telegram, SumatraPDF, VS Code) both have major precedents. KeePass's
   `-cfg-local:` colon syntax is the outlier no one else copied.
2. **Relative paths resolve against the CWD everywhere it is verifiable** — Chromium, Firefox and
   Telegram all make the path absolute with `_wfullpath()`/`QDir::absolutePath()` semantics, i.e.
   against the process CWD, *not* the exe directory. No precedent resolves against the exe dir.
3. **On a missing directory the browsers create it recursively; the editors refuse.** Chromium,
   Firefox and Telegram create (the switch is their multi-instance mechanism, so the target is
   expected not to exist yet). Notepad++ — the closest analogue to "override a settings dir that
   should already be somewhere" — validates, shows an error dialog, ignores the switch and starts
   with defaults. Nobody found silently writes into a half-broken location without either creating
   it properly or telling the user.

**Chromium's invalid-target story in full**, since it is the most engineered: `install_static`
first strips *all* trailing separators ("On Windows, trailing separators leave Chrome in a bad
state" — [user_data_dir.cc](https://github.com/chromium/chromium/blob/main/chrome/install_static/user_data_dir.cc)),
makes the path absolute, and if the directory cannot be created, swaps in the default and remembers
the rejected path; the browser process then shows a warning box — the string users see is
"Chromium cannot read and write to its data directory: …" ([issue 41288900](https://issues.chromium.org/issues/41288900),
call site `MaybeShowInvalidUserDataDirWarningDialog()` in
[chrome_browser_main.cc](https://github.com/chromium/chromium/blob/main/chrome/browser/chrome_browser_main.cc))
— and continues on the default. So even the "create it for me" school refuses to *run degraded
silently*.

## 2. Unknown / malformed argument posture

There is **no Microsoft guidance** on this for GUI apps — none was found on Microsoft Learn, and the
old Windows UX guidelines do not address command lines. Practice splits three ways:

| Posture | Who | Evidence |
|---|---|---|
| **Silently ignore** | Chromium | `base::CommandLine` treats *anything* prefixed `--`/`-`/(`/` on Windows) as a switch and stores it; "no mechanism … for rejecting unrecognized switch names" ([command_line.h](https://github.com/chromium/chromium/blob/main/base/command_line.h)). Unknown switches are simply never read. |
| **Warn (console only) and continue** | Firefox, VS Code | Firefox: `console.error("Warning: unrecognized command line flag", curarg)` — and it deliberately *also skips the next argument* ([BrowserContentHandler.sys.mjs](https://github.com/mozilla-firefox/firefox/blob/main/browser/components/BrowserContentHandler.sys.mjs)). VS Code: "Warning: 'x' is not in the list of known options, but still passed to Electron/Chromium" ([argv.ts](https://github.com/microsoft/vscode/blob/main/src/vs/platform/environment/node/argv.ts) `onUnknownOption`; observed in [vscode#128279](https://github.com/microsoft/vscode/issues/128279)). Invisible unless launched from a console. |
| **Error dialog, refuse to start** | qBittorrent | Unknown parameter → `CommandLineParameterError` → on Windows a Critical `QMessageBox` titled "Bad command line" containing the message *plus the full help text*, then `EXIT_FAILURE` ([main.cpp](https://github.com/qbittorrent/qBittorrent/blob/master/src/app/main.cpp)). |

A fourth family — **unknown token = file to open** — covers the document apps: SumatraPDF ("Anything
that is not recognized as a known option is interpreted as a file path"), Notepad++ (leftover
params become files to open), and Firefox for *non*-dash leftovers (resolved as URIs). That family
is only coherent for apps whose primary argument *is* a document; it does not transfer to an app
with no file-open semantics.

Malformed-but-known is a separate branch: Firefox errors ("argument --profile requires a path"),
qBittorrent dialogs, Chromium takes the last of duplicate switches ("If a switch is specified
multiple times, only the last value is used" — command_line.h).

## 3. `--help` for a GUI-subsystem app

The constraint is structural: the PE header names one subsystem, decided before the process runs —
"you can't write a program that's both", per Raymond Chen
([The Old New Thing](https://devblogs.microsoft.com/oldnewthing/?p=19643%2F)). A GUI-subsystem
process has no console and its std handles "will likely be invalid on startup until AttachConsole
is called" ([AttachConsole](https://learn.microsoft.com/en-us/windows/console/attachconsole)).
The escape hatches, each with its cost:

- **`AttachConsole(ATTACH_PARENT_PROCESS)`** — works only when a parent console exists
  (`ERROR_INVALID_HANDLE` otherwise); and because the shell did not wait for a GUI process, the
  prompt has already been printed, so output lands *after* the prompt and the user must press Enter
  to get their prompt back. Mozilla's own bug about this ("Output stdout to the console on Windows",
  [bug 1257155](https://bugzilla.mozilla.org/show_bug.cgi?id=1257155)) is why Firefox grew a
  dedicated `-attach-console` switch rather than doing it implicitly.
- **Dual binary** (`devenv.com` + `devenv.exe`) — the Visual Studio pattern (same Old New Thing
  post). Two artifacts; instantly disqualified by a one-executable promise.
- **Message box** — what the GUI apps in scope actually do:
  - **Firefox**: `DumpHelp()` is literally `#if defined(XP_WIN) && !MOZ_WINCONSOLE` → **`MessageBoxW`**
    ([nsAppRunner.cpp](https://github.com/mozilla-firefox/firefox/blob/main/toolkit/xre/nsAppRunner.cpp)) —
    help text in a dialog on GUI builds.
  - **Notepad++**: `--help` shows the argument list "before Notepad++'s launch", and the same text
    is a menu item, **? → Command Line Arguments** — a MsgBox
    ([manual](https://npp-user-manual.org/docs/command-prompt/), [issue #4067](https://github.com/notepad-plus-plus/notepad-plus-plus/issues/4067)).
  - **qBittorrent**: embeds the full usage text *inside the bad-argument error dialog* (§2), so the
    help surface exists exactly when a CLI user needs it.
- **Website/README only** — SumatraPDF (no help switch at all, docs on the website) and Chrome
  (no `--help` on Windows; nothing in its docs even claims one — undocumented, and nothing to cite).
- **VS Code is not a counterexample**: `code --help` prints to a console because `code` is a
  separate CLI wrapper process, not the GUI `Code.exe`
  ([CLI docs](https://code.visualstudio.com/docs/configure/command-line)).

Notepad++'s pairing is the standout convention for a menu-bar app: the same help text reachable as
a dialog from the command line **and** as a menu item — which also solves discoverability for the
user who never opens a terminal.

## 4. Forwarding the command line through an elevated relaunch

`ShellExecuteExW` with `runas` takes parameters as **one flat string** (`lpParameters`,
[SHELLEXECUTEINFOW](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ns-shellapi-shellexecuteinfow)) —
so the relaunching app must serialize, and the elevated instance will re-parse. Two defensible
strategies:

1. **Forward the tail of `GetCommandLineW()` verbatim** (everything after argv[0]). What the parent
   received is by definition a string its own parser accepted; passing it through unchanged cannot
   introduce new quoting bugs, and it round-trips arguments that the parsed-argv view has already
   normalized. The cost: argv[0]'s extent must be found by scanning the raw string (quoted or
   unquoted program name — "the program name can be enclosed in quotation marks or not",
   [CommandLineToArgvW](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-commandlinetoargvw)).
2. **Re-serialize parsed args** — correct **only** with a real `ArgvQuote`: quote if the argument
   contains space/tab/quote; inside, emit `2n+1` backslashes before an embedded `"` and `2n` before
   the closing quote. The reference implementation and rationale are Microsoft-hosted:
   ["Everyone quotes command line arguments the wrong way"](https://learn.microsoft.com/en-us/archive/blogs/twistylittlepassagesallalike/everyone-quotes-command-line-arguments-the-wrong-way)
   (Daniel Colascione). "Simply add quotes around command line arguments without any further
   processing" is its named **Do not** #1.

**The trailing-backslash pitfall is documented on both sides.** Parser side
([Parsing C command-line arguments](https://learn.microsoft.com/en-us/cpp/c-language/parsing-c-command-line-arguments?view=msvc-170)):
an even number of backslashes before `"` halves and the quote delimits; an odd number halves and
produces a literal `"`. So the natural user input

```
app.exe --data-dir "C:\my data\"
```

parses as one argument `--data-dir` and one argument `C:\my data"` — trailing quote swallowed into
the value, and any following switch swallowed into it too (Colascione's own example:
`"\some\directory with\spaces\" argument2` → `[\some\directory with\spaces" argument2]`).
CommandLineToArgvW's remarks add that its backslash special-casing "assumes that any preceding
argument is a valid file system path, or else it may behave unpredictably."
Real apps defend at two points:

- **Sanitize the received value**: Chromium strips *all* trailing `\` and `/` from the user data
  dir; Notepad++ strips a stray surrounding quote pair off the `-settingsDir` value in
  [winmain.cpp](https://github.com/notepad-plus-plus/notepad-plus-plus/blob/master/PowerEditor/src/winmain.cpp)
  (`if (path[0]=='"' && path[len-1]=='"') path = path.substr(1, len-2)`). A `--data-dir` value
  ending in `"` or with unbalanced quotes is a recognizable artifact of this rule and can be
  trimmed rather than rejected.
- **Serialize correctly on the way out** (the ArgvQuote rules above).

**Rust specifics.** `std::env::args` on Windows implements the compatible modern rules ("A quote
can be escaped if preceded by an odd number of backslashes … the number of backslashes is halved" —
[library/std/src/sys/args/windows.rs](https://github.com/rust-lang/rust/blob/master/library/std/src/sys/args/windows.rs)),
and `std::process::Command` quotes outgoing args correctly by the same rules ("Add n+1 backslashes
to total 2n+1 before internal `"` … Add n backslashes to total 2n before ending `"`" — same file;
`raw_arg` exists precisely to bypass that escaping for non-conforming parsees like `cmd.exe /c`,
[CommandExt::raw_arg](https://doc.rust-lang.org/std/os/windows/process/trait.CommandExt.html)).
But a `ShellExecuteExW("runas")` relaunch does **not** go through `Command` — `lpParameters` is
built by hand, so the app needs its own ArgvQuote (or the verbatim-tail strategy) regardless of
what std would have done. The cmd.exe metacharacter hazard in Colascione's article does not apply
here: nothing passes through a shell.

## 5. Accepting a directory path from the command line — security/robustness

- **Canonicalize before validating** — CERT FIO02-C: "Canonicalizing file names makes it much
  easier to verify a path"; risks named are symlinks, `..` traversal, and TOCTOU between check and
  use. Its Windows verdict is sobering: `GetFullPathName()` removes `..`/`.` "but there are
  numerous other canonicalization issues that are not addressed" (UNC shares, short 8.3 names,
  trailing dots, shortcuts), and "the best advice is to try to avoid making decisions based on a
  path" — validate by *opening/creating and using* the directory, not by inspecting the string
  ([FIO02-C](https://wiki.sei.cmu.edu/confluence/display/c/FIO02-C.+Canonicalize+path+names+originating+from+tainted+sources)).
- **Reserved device names** — CON, PRN, AUX, NUL, COM1–9, LPT1–9 *and* the superscript forms
  COM¹/²/³, LPT¹/²/³, **with or without an extension** ("NUL.txt … equivalent to NUL"), are
  reserved in every directory
  ([Naming Files, Paths, and Namespaces](https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file)).
  A `--data-dir NUL` or `--data-dir C:\x\CON` must fail *somewhere*; the question is only whether
  the failure message is comprehensible.
- **Trailing spaces and periods** — "Do not end a file or directory name with a space or a period.
  Although the underlying file system may support such names, the Windows shell and user interface
  does not" (same doc). A create-then-use validation catches the shell-visible breakage only if the
  name is normalized first.
- **Path length** — MAX_PATH is 260; exceeding it requires the `\\?\` prefix *and* Unicode APIs, or
  the Windows 10 1607+ opt-in (registry `LongPathsEnabled` **plus** a `longPathAware` manifest
  element); "relative paths are always limited to a total of MAX_PATH"
  ([Maximum Path Length Limitation](https://learn.microsoft.com/en-us/windows/win32/fileio/maximum-file-path-limitation)).
  Rust's `std::fs::canonicalize` returns `\\?\`-prefixed paths on Windows and its docs warn the
  result "may be incompatible with other applications (if passed to the application on the
  command-line, or written to a file another application may read)"
  ([canonicalize](https://doc.rust-lang.org/std/fs/fn.canonicalize.html)) — relevant twice here:
  once for displaying the resolved dir, once because an elevated relaunch would put that string
  *back on a command line*.
- **UNC paths** — `\\server\share\…` is a fully qualified path form and needs no special-casing to
  *accept* (naming doc, "Fully Qualified vs. Relative Paths"); the long-path form is `\\?\UNC\…`.
  Whether a data dir on a share is *wanted* is a product question, not a parsing one.
- **Symlinks/junctions** — NTFS links "follow file naming conventions and rules just as a regular
  file or directory would" (naming doc → Hard Links and Junctions); CERT's position is that
  resolving them is exactly what canonicalization is for, and that the check must happen at use
  time (TOCTOU). An app that simply creates-and-writes through whatever the path resolves to — as
  every precedent app here does — is taking the FIO02-C "use the OS mechanism" branch, not the
  validate-the-string branch.
- **Trailing-quote artifacts** — see §4; the two deployed mitigations are Chromium's
  trailing-separator strip and Notepad++'s quote strip.

## Implications for PathMaster (options, not decisions)

Decision points the grilling has to close, with the defensible sides of each:

1. **Spelling/syntax.** `--data-dir` with a value is fully within convention. Both `--data-dir <path>`
   (Firefox/VS Code/Telegram style) and `--data-dir=<path>` (Chromium/qBittorrent style) have
   first-tier precedents; accepting both is what Chromium's parser does by construction. No
   precedent supports inventing a `/datadir:` or `-cfg:`-style form.
2. **Missing directory.** Two defensible schools: *create recursively* (Chromium, Firefox,
   Telegram — the switch as a "point me at a fresh sandbox" tool) vs *refuse with an error dialog
   and name the path* (Notepad++ — the switch as "use this existing settings home"). Silent
   fallback-without-telling exists in no verified precedent; Chromium falls back *and* warns.
   Whichever side is chosen, the ticket's own rule applies: the branch must land in the existing
   startup-failure taxonomy, and Notepad++'s dialog text (path named, switch named, consequence
   named) is the template worth stealing.
3. **Relative paths.** Every verifiable precedent resolves against the **CWD**. For a portable app
   whose home ground is "carried on a stick", exe-relative would be *self-consistent with ADR-0002*
   but contrary to every precedent; if CWD-relative is chosen, the resolved absolute path should be
   what all downstream surfaces (About, errors, elevation forwarding) display.
4. **Invalid target (file, unwritable, reserved name).** Options: refuse-and-exit (Firefox),
   refuse-and-continue-on-default with a dialog (Notepad++, Chromium). Continuing *silently* on the
   default is the one posture with no precedent. Per FIO02-C, the validation itself should be
   "try to create/open and use it", not string inspection — with the reserved-name and
   trailing-space cases caught only because the create-or-open fails or the name is normalized first.
   Read-only Data (spec §3) already defines what an unwritable data dir means at startup; the switch
   only needs to route into it.
5. **Unknown/malformed arguments.** Three precedented postures: ignore (Chromium), console-warn
   (Firefox/VS Code — invisible for a GUI-only launch), dialog-and-refuse (qBittorrent). The
   "unknown = file to open" family is unavailable: PathMaster opens no documents. For an app with
   exactly one switch, qBittorrent's refuse-with-help-text and Chromium's ignore are both coherent;
   half-measures (accept `--data-dir` but ignore a typo'd `--datadir` silently) combine the worst
   properties of each.
6. **Help surface.** The GUI-subsystem constraint is real (§3); the in-convention options are:
   README/website only (SumatraPDF, KeePass), a `--help`/`-?` message box (Firefox, Notepad++), and
   Notepad++'s pairing of that box with a Help-menu item. `AttachConsole` output is the only option
   with a documented degraded UX and no precedent among the surveyed apps as the *sole* mechanism.
   Ticket 12's F1/browser machinery may already own the "where is the documentation" answer; if so,
   the switch needs only a README section and (optionally) the error dialog naming the switch, the
   qBittorrent way.
7. **Elevation forwarding.** Both strategies are defensible: verbatim `GetCommandLineW()` tail
   (no re-quoting risk, must locate argv[0]'s end) or re-serialization through a correct ArgvQuote
   (§4 rules; std::process::Command's own escaping does not cover `ShellExecuteExW`). Either way,
   the *resolved absolute* path — not the user's original relative spelling — is what must be
   forwarded, because the elevated process's CWD is not guaranteed to match. And whichever branch
   trims trailing separators/quote artifacts must run *before* forwarding, or the artifact is
   re-serialized into the elevated command line.

## Sources

- Chromium: [user_data_dir.md](https://chromium.googlesource.com/chromium/src/+/main/docs/user_data_dir.md) · [install_static/user_data_dir.cc](https://github.com/chromium/chromium/blob/main/chrome/install_static/user_data_dir.cc) · [chrome_main_delegate.cc](https://github.com/chromium/chromium/blob/main/chrome/app/chrome_main_delegate.cc) · [chrome_browser_main.cc](https://github.com/chromium/chromium/blob/main/chrome/browser/chrome_browser_main.cc) · [base/command_line.h](https://github.com/chromium/chromium/blob/main/base/command_line.h) · [issue 41288900 (data-dir dialog)](https://issues.chromium.org/issues/41288900)
- VS Code: [command line docs](https://code.visualstudio.com/docs/configure/command-line) · [portable mode](https://code.visualstudio.com/docs/editor/portable) · [argv.ts](https://github.com/microsoft/vscode/blob/main/src/vs/platform/environment/node/argv.ts) · [vscode#128279](https://github.com/microsoft/vscode/issues/128279)
- Firefox: [CommandLineOptions wiki](https://wiki.mozilla.org/Firefox/CommandLineOptions) · [nsToolkitProfileService.cpp](https://github.com/mozilla-firefox/firefox/blob/main/toolkit/profile/nsToolkitProfileService.cpp) · [nsAppRunner.cpp (DumpHelp → MessageBoxW)](https://github.com/mozilla-firefox/firefox/blob/main/toolkit/xre/nsAppRunner.cpp) · [BrowserContentHandler.sys.mjs](https://github.com/mozilla-firefox/firefox/blob/main/browser/components/BrowserContentHandler.sys.mjs) · [bug 1257155 (-attach-console)](https://bugzilla.mozilla.org/show_bug.cgi?id=1257155)
- Telegram Desktop: [core/launcher.cpp](https://github.com/telegramdesktop/tdesktop/blob/dev/Telegram/SourceFiles/core/launcher.cpp) · [settings.h (cForceWorkingDir)](https://github.com/telegramdesktop/tdesktop/blob/dev/Telegram/SourceFiles/settings.h)
- qBittorrent: [portable mode wiki](https://github.com/qbittorrent/qBittorrent/wiki/How-to-use-portable-mode) · [src/app/main.cpp](https://github.com/qbittorrent/qBittorrent/blob/master/src/app/main.cpp)
- KeePass: [cmdline](https://keepass.info/help/base/cmdline.html) · [configuration](https://keepass.info/help/base/configuration.html)
- Notepad++: [command-prompt manual](https://npp-user-manual.org/docs/command-prompt/) · [config-files manual (doLocalConf.xml)](https://www.npp-user-manual.org/docs/config-files/) · [winmain.cpp](https://github.com/notepad-plus-plus/notepad-plus-plus/blob/master/PowerEditor/src/winmain.cpp) · [Parameters.cpp](https://github.com/notepad-plus-plus/notepad-plus-plus/blob/master/PowerEditor/src/Parameters.cpp) · [issue #4067 (help MsgBox)](https://github.com/notepad-plus-plus/notepad-plus-plus/issues/4067)
- SumatraPDF: [command-line arguments](https://www.sumatrapdfreader.org/docs/Command-line-arguments)
- Microsoft: [Parsing C command-line arguments](https://learn.microsoft.com/en-us/cpp/c-language/parsing-c-command-line-arguments?view=msvc-170) · [CommandLineToArgvW](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-commandlinetoargvw) · [SHELLEXECUTEINFOW](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ns-shellapi-shellexecuteinfow) · [AttachConsole](https://learn.microsoft.com/en-us/windows/console/attachconsole) · [Naming Files, Paths, and Namespaces](https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file) · [Maximum Path Length Limitation](https://learn.microsoft.com/en-us/windows/win32/fileio/maximum-file-path-limitation) · ["Everyone quotes command line arguments the wrong way" (Colascione)](https://learn.microsoft.com/en-us/archive/blogs/twistylittlepassagesallalike/everyone-quotes-command-line-arguments-the-wrong-way) · [Raymond Chen — console or GUI](https://devblogs.microsoft.com/oldnewthing/?p=19643%2F)
- CERT: [FIO02-C — Canonicalize path names originating from tainted sources](https://wiki.sei.cmu.edu/confluence/display/c/FIO02-C.+Canonicalize+path+names+originating+from+tainted+sources)
- Rust: [sys/args/windows.rs](https://github.com/rust-lang/rust/blob/master/library/std/src/sys/args/windows.rs) · [CommandExt::raw_arg](https://doc.rust-lang.org/std/os/windows/process/trait.CommandExt.html) · [std::fs::canonicalize](https://doc.rust-lang.org/std/fs/fn.canonicalize.html)
