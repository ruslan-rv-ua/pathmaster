# --data-dir switch

Type: grilling
Status: resolved (2026-08-26)
Blocked by: —

## Question

Parked by v0.1.0's ticket 07 as the one relocation-adjacent feature that survives the no-relocation
principle: a command-line switch carries the Data Directory location **per launch** rather than
remembering it. Specify:

- Syntax and validation: `--data-dir <path>` — what happens on a missing directory (create? refuse?),
  a relative path (resolve against what?), a file, an unwritable target (Read-only Data applies?).
  Every branch lands in the v0.1.0 startup failure taxonomy — extend it, don't fork it.
- **Elevation must forward it.** ADR-0005 relaunches the whole app elevated; the relaunch command
  line has to carry the switch or the elevated run silently writes elsewhere. Same for any future
  restart path (language change is restart-based).
- Interaction with the "located from the executable" rule (ADR-0002): the switch overrides it for
  one Run — confirm the CONTEXT.md "Data Directory" and "Run" entries absorb this without a new term.
- Unknown/malformed arguments generally: v0.1.0 has no CLI at all, so this ticket incidentally
  decides the app's whole argument-handling posture (ignore? dialog? log?).
- Does it appear in --help output… which doesn't exist for a GUI app — decide how the switch is
  documented (README only?).

## Resolution (2026-08-26)

Researched first: [research/13-data-dir-best-practices.md](../research/13-data-dir-best-practices.md),
per the map's standing directive 7. Decisions:

- **The switch substitutes the locate step and nothing else.** `--data-dir` replaces "where is the
  directory" in the v0.1.0 §3 startup tree (locate → create → probe → …); everything downstream runs
  unchanged: a missing directory is `create_dir_all`-created like the default one, and a target that
  cannot be created or written lands the Run in **Read-only Data** through a **fourth reason** naming
  the switch — "the --data-dir location cannot be used" (en/uk pair). **No fallback to the default
  `data\`, ever**: the application never writes where it was not pointed. (The browser school —
  Chromium/Firefox/Telegram create recursively — harmonized with v0.1.0's own tree; Notepad++'s
  refuse-and-fall-back was rejected precisely because falling back *is* the silent-wrong-place hazard
  this ticket exists to prevent.)
- **Both spellings**, `--data-dir <path>` and `--data-dir=<path>`; the README documents the space
  form (consistent with `--tab`). Before resolution the value is stripped of trailing `"` and
  trailing path separators — recognizable artifacts of the documented backslash-before-quote parsing
  rule (`--data-dir "C:\x\"` parses to `C:\x"`); the deployed mitigations are Chromium's separator
  strip and Notepad++'s quote strip, and both run **before** elevation forwarding.
- **Relative paths resolve against the CWD** (every verifiable precedent: `_wfullpath` semantics in
  Chromium and Firefox, `QDir::absolutePath` in Telegram) — make-absolute, not `fs::canonicalize`,
  whose `\\?\` result must not ride a command line. Resolution happens **once at startup**; the
  resolved absolute path is the single truth every downstream surface uses: the log, the Read-only
  reason's record, the elevated relaunch.
- **The startup log line grows `dataDir: <resolved path>`** on every override Run, success included —
  the log is the only diagnostic artifact (§14), and a Run that wrote elsewhere is otherwise
  unreconstructable after the fact.
- **Argument posture (whole app, decided here).** Unknown switch → **dialog-and-continue**: message
  in the title ("Unknown argument {arg} was ignored" [assembly]), one usage line in the body (the
  qBittorrent trick — help appears exactly when a CLI user needs it), [OK], one `WARN` log line,
  then a normal start. A valueless or malformed `--data-dir` is not an unknown argument but a broken
  override → Read-only Data, fourth reason (no fallback, per the first bullet). `--tab`'s v0.1.0
  leniency (unrecognized value = plain launch, never a guess) is spec'd foundation and stays.
  Silent ignoring (Chromium) rejected: a typo'd `--datadir` silently landing data in the default
  directory is the exact hazard above; refuse-to-start (qBittorrent) rejected against ADR-0002's
  open-and-explain philosophy.
- **`--help` and `-?` are recognized**: a dialog carrying the same usage line (one shared Catalogue
  string), then **exit** — it is a query, not a launch; Firefox's GUI-build `DumpHelp()` is
  literally a `MessageBoxW`, the dominant convention. AttachConsole rejected (documented degraded
  UX, no precedent as sole mechanism). Documentation homes: a command-line note in the README's
  portability section and a "Command line" subsection in the User Guide (ticket 12's surface) — the
  Notepad++ pairing of CLI dialog + menu-reachable text, assembled from parts we already have.
- **Elevation forwards by re-serialization, never the verbatim `GetCommandLineW` tail.** The
  CWD-resolution decision forces it: the elevated process's CWD is not guaranteed, so the *resolved
  absolute* path — not the user's original spelling — must cross. The relaunch line is built from
  parsed state (`--tab <active> --data-dir <resolved>`) through one ArgvQuote implementation
  (Colascione's 2n/2n+1 backslash rules; `std::process::Command`'s quoting does not apply to
  `ShellExecuteExW`'s hand-built `lpParameters`) living beside the `--tab` type in
  `pathmaster-platform` — writer and reader one type, the v0.1.0 trick, extended. Unknown arguments
  die at the boundary (no second dialog in the elevated instance). The rule is general: **any
  future self-relaunch carries the override**, or it silently writes elsewhere.
- **Glossary absorbed without a new term**: one sentence added to CONTEXT.md's Data Directory
  ("…unless a single Run was pointed elsewhere by the `--data-dir` command-line switch, which
  carries the location for that Run only and remembers nothing"); Run already covers it ("where the
  Data Directory is" is a Run property, decided at startup). "There is no setting that moves it"
  stays true. **No ADR**: nothing irreversible, and ADR-0005's consequence list plus ADR-0002's
  "per launch instead of remembering it" already record the surprising parts.
- **No settings field, no new Announcement** — the fourth reason rides Announcement 7
  ("Read-only: {reason}"); the set stays at fourteen. Catalogue additions for assembly (→ 15): the
  reason-4 pair, the unknown-argument dialog title, the shared usage line, the `--help` dialog
  title.
- **Release Checklist steps named (seven)**: a fresh-path launch creates the directory there and
  leaves `data\` beside the exe untouched; a relative path resolves against the shell's CWD; an
  unusable target speaks the fourth Read-only reason and writes nothing anywhere; "Restart as
  Administrator" during an override Run with a path containing spaces lands the elevated instance
  in the same directory; an unknown argument shows the dialog and the app continues, with one
  `WARN` line; `--help` shows the usage dialog and exits; the startup log line carries `dataDir:`
  on override Runs.
