# 10 — The `--data-dir` switch and the argument posture

**Spec:** [delta-spec §10, §13 (item 7), §14 (command-line strings)](../../pathmaster-v0-2-0/spec.md) · ADR-0002 · ADR-0005

**What to build:** `PathMaster.exe --data-dir <path>` substitutes only the locate step of the startup tree — everything downstream runs unchanged — and an unusable target lands the Run in Read-only Data through a fourth reason naming the switch, never a fallback to the default `data\`. The ticket also lands the whole-app argument posture: unknown switch → dialog-and-continue, `--help`/`-?` → usage dialog and exit, and elevation that re-serializes the override so the elevated instance writes in the same place.

**Blocked by:** 01 (retrofit lands first).

**Status:** done — built, gated and driven live in both languages; the **UAC leg** of the elevated relaunch is the Release Checklist’s, where the delta-spec put it, **accepted by the user 2026-08-28** (see Comments)

- [x] Both spellings work: `--data-dir <path>` and `--data-dir=<path>`; before resolution the value is stripped of trailing `"` and trailing path separators; both handled before elevation forwarding
- [x] Relative paths resolve against the CWD once at startup, make-absolute — never `fs::canonicalize`; the resolved absolute path is the single truth every downstream surface uses; a missing directory is `create_dir_all`-created like the default one
- [x] An unusable target (cannot create or write) → Read-only Data through a **fourth reason naming the switch**; Announcement 7 gains the reason text in both languages; never a fallback to the default `data\`
- [x] The startup log line grows `dataDir: <resolved path>` on every override Run (the audited exception to the no-PII path rule, per the ticket's decision)
- [x] Elevation forwards by re-serialization from parsed state (`--tab <active> --data-dir <resolved>`), never the verbatim command-line tail, through one ArgvQuote writer/reader type in `pathmaster-platform` (Colascione's 2n/2n+1 rules), unit-tested round-trip including spaced and quote-adjacent paths; unknown arguments die at the boundary — no second dialog in the elevated instance; "Restart as Administrator" during a spaced-path override Run demonstrably lands the elevated instance in the same directory — **the line and its landing are verified live; the UAC prompt itself is Checklist-only, see Comments**
- [x] Whole-app posture: unknown switch → dialog-and-continue — title "Unknown argument {arg} was ignored", the shared usage line in the body, [OK], one `WARN` line, then a normal start; a valueless or malformed `--data-dir` is a broken override → the fourth reason, not an unknown argument; `--tab`'s v0.1.0 leniency stays; `--help` and `-?` → a dialog with the same usage line, then exit
- [x] Catalogue strings shipped in both languages: the unknown-argument title, the usage line, the `--help` dialog title, the fourth Read-only reason; i18n gate green
- [x] The README's portability section documents the switch (space form); CONTEXT.md's Data Directory grows its one sentence; the User Guide's "Command line" subsection is ticket 11's to write
- [x] A fresh-path `--data-dir` launch creates the directory there with `data\` beside the exe untouched — verified live

## Comments

**2026-08-27 (implementation)** — The switch substitutes **one step**, and the code says so in one
place: `datadir::decide` now takes a `Location` — beside the executable, unknown, an override, or a
broken override — and the override arm calls the *same* `establish` the default road calls, then
renames only the reason a failure earns. Nothing about creating, probing or reading is a second
road, so there is no second road to keep in step. `establish` itself is untouched.

**The fourth reason is one reason for three failures** — a target that could not be created, one
whose probe failed, and a switch carrying nothing that resolves — because there is one thing to say
about all three and one thing to do about it, and what needs saying is *which location* failed
rather than how. `ReadOnlyReason::OverrideUnusable(Option<PathBuf>)` keeps the directory where
there is one, so a `settings.json` sitting in an unwritable override is still read and its language
still obeyed: the existing "two of the three reasons carry a directory" rule extends rather than
bends.

**`Location` is a separate question from `DataDirState`**, and holding both on the `Run` is what the
elevation forwarding needs. Two Runs can end in the same Read-only Data for different reasons, and
only `Location` says which of the two places a Run was *aimed at* — which the log line names and
which a self-relaunch has to re-serialize. A broken override crosses the boundary as the bare
switch it was: dropping it would land the elevated instance in the default `data\`, which is
writing where it was not pointed, so the odd-looking `--tab user --data-dir` is the only honest
line there.

**Arguments are `OsString` end to end**, including the `=` split, which is done on UTF-16 units.
`to_string_lossy` turns an unpaired surrogate into U+FFFD, and for the one argument that is a
filesystem path that means creating a directory *near* where the user pointed — the exact hazard
the no-fallback rule exists for. `--tab`'s v0.1.0 reader could afford lossy (a mangled value is not
one of three known words, so it degrades to a plain launch) and this one cannot. The one relative
shape `locate_override` refuses is the drive-relative `C:foo`: it names a current directory *per
drive* that only the OS knows, and a guess is not a pointing, so it is a broken override.

**`CommandLine` is the writer and the reader in one type**, which is `StartTab::argument`'s v0.1.0
trick extended from one argument to the whole line. `split` is deliberately a *model* of
`CommandLineToArgvW` rather than a call to it: what parses the elevated instance's line is Windows,
and the point of having the reader is that the pair can be round-tripped in a unit test — spaced
paths, a trailing backslash (which would otherwise escape the closing quote), a quote inside the
value, and the empty argument. Unknown arguments cannot ride along, because the line is built from
parsed state and there is nowhere for one to sit.

**The log grows one audited exception and one new bounded inlet.** `dataDir: <resolved path>` rides
the startup line on override Runs only — the log is the only diagnostic artifact, and a Run that
wrote elsewhere is otherwise unreconstructable — and an unknown argument is quoted and truncated at
the same 100 characters a rejected settings value is, since it is the user's own text and may be
anything. Both are named in `logfmt`'s module doc beside the prohibitions they qualify. One `WARN`
per unknown argument, where the dialog names only the first: a garbled launcher line would
otherwise stack a dialog per token in front of a screen-reader user at startup, so the dialog
points at the usage line and the log is the inventory.

**`--help` locates nothing**, which is why it speaks the *system* language rather than the stored
choice: the stored choice lives in a Data Directory this query would have to go looking for, and
`--data-dir` may have pointed that directory somewhere that does not exist yet. Creating it to
answer a question about switches is what "a query, not a launch" rules out. A `MessageDialog` needs
a parent all the same, so an unshown `Frame` is one — and destroying it is also what ends the event
loop, the same door every other exit leaves by.

**Driven live on staged copies, both languages** (cross-process probes; the machine's own PATH is
read but never written, and every Data Directory in the run is under the scratch tree):

- `--help` → one dialog, `#32770` titled «Командний рядок PathMaster», body «Використання:
  PathMaster.exe [--tab user|system|backups] [--data-dir <шлях>] [--help]», one OK; the process
  **exits**, and no `data\` appears beside the executable. `<шлях>`/`<path>` renders literally —
  the angle brackets survive the TaskDialog path.
- `--data-dir "…\pm data dir"` (a fresh, spaced path) → the directory is created there, the Banner
  is **empty** (not Read-only Data), and `data\` beside the exe does **not** exist. Log:
  `INFO startup: PathMaster 0.1.0, elevated: no, data: writable, language: uk, dataDir: …\pm data
  dir`.
- The line `CommandLine::relaunch` writes for that Run — `--tab system --data-dir "…\pm data dir"`
  — handed to a fresh instance: the same directory (its log grows a second startup line, no third
  directory anywhere), tab index **1** (System PATH), and **no second dialog**.
- An unusable target (a file squatting on the path) → Banner «Лише читання: розташування, вказане
  в --data-dir, неможливо використати», the file byte-identical afterwards, and no `data\` beside
  the exe. A **valueless** `--data-dir` gives the same Banner with **no dialog** — a broken
  override is not an unknown argument.
- `--datadir=C:\typo --data-dir "…" --nonsense` → one dialog, «Невідомий аргумент
  --datadir=C:\typo проігноровано» with the usage line in its body; [OK] leaves the application
  running with no dialogs left, and the log carries **two** `WARN startup: unknown argument …` lines
  under a startup line that still names the override. The English copy shows "Unknown argument
  --nonsense was ignored".
- `--data-dir "relative data"` from a working directory of `…\from here` → `…\from here\relative
  data`, and the log's `dataDir:` names that absolute path.
- A plain launch afterwards writes beside the executable again and its startup line carries **no**
  `dataDir:` — the switch remembers nothing.

**The UAC prompt is the one leg an unattended session cannot pull**, and it is not left undone: the
delta-spec names "'Restart as Administrator' during an override Run with a path containing spaces
lands the elevated instance in the same directory" as one of its seven Release Checklist steps, and
ticket 12 folds that group in. What is covered here is everything on this side of the prompt — the
line is built by `CommandLine::relaunch(tab, location)` and handed to `ShellExecuteExW` verbatim,
the write→split→parse round trip is asserted over spaced and quote-adjacent paths, and that exact
line was run live and landed in the same directory. What is not covered is `ShellExecuteEx("runas")`
returning, which needs a human at the secure desktop.

**No ADR, no settings field, no new Announcement** — the fourth reason rides Announcement 7 and the
set stays at fourteen, exactly as §10 specifies. The Release Checklist's "Command line" group is
ticket 12's; nothing this change touches falsifies an existing step.

**2026-08-28 (review closed)** — The deferral is **accepted**, and it was checked rather than taken
on the ticket's word. §17's Command line group names the step as its fourth of seven, written before
implementation began, so the assignment is the spec's and not a retrofit. What is left to a human is
only `ShellExecuteEx("runas")` returning: the *line* that leg carries is asserted by
`a_spaced_override_reaches_the_next_instance_unchanged`, and was additionally run live into a fresh
instance that landed in the same directory.

No gap of ticket 08's kind was found. Everything mechanically checkable sits in `cargo test` on every
CI run — both spellings, the valueless switch as a broken override rather than an unknown argument,
the switch typo, ArgvQuote's backslash rule, the splitter against Windows' own, both boundary-crossing
rules, the quoting artifacts, the root that the separator strip never eats, and drive-relative `C:foo`
among the values that resolve to nothing — and the remaining six Checklist steps cover the rest.
