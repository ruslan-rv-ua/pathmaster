# 10 — The `--data-dir` switch and the argument posture

**Spec:** [delta-spec §10, §13 (item 7), §14 (command-line strings)](../../pathmaster-v0-2-0/spec.md) · ADR-0002 · ADR-0005

**What to build:** `PathMaster.exe --data-dir <path>` substitutes only the locate step of the startup tree — everything downstream runs unchanged — and an unusable target lands the Run in Read-only Data through a fourth reason naming the switch, never a fallback to the default `data\`. The ticket also lands the whole-app argument posture: unknown switch → dialog-and-continue, `--help`/`-?` → usage dialog and exit, and elevation that re-serializes the override so the elevated instance writes in the same place.

**Blocked by:** 01 (retrofit lands first).

**Status:** ready-for-agent

- [ ] Both spellings work: `--data-dir <path>` and `--data-dir=<path>`; before resolution the value is stripped of trailing `"` and trailing path separators; both handled before elevation forwarding
- [ ] Relative paths resolve against the CWD once at startup, make-absolute — never `fs::canonicalize`; the resolved absolute path is the single truth every downstream surface uses; a missing directory is `create_dir_all`-created like the default one
- [ ] An unusable target (cannot create or write) → Read-only Data through a **fourth reason naming the switch**; Announcement 7 gains the reason text in both languages; never a fallback to the default `data\`
- [ ] The startup log line grows `dataDir: <resolved path>` on every override Run (the audited exception to the no-PII path rule, per the ticket's decision)
- [ ] Elevation forwards by re-serialization from parsed state (`--tab <active> --data-dir <resolved>`), never the verbatim command-line tail, through one ArgvQuote writer/reader type in `pathmaster-platform` (Colascione's 2n/2n+1 rules), unit-tested round-trip including spaced and quote-adjacent paths; unknown arguments die at the boundary — no second dialog in the elevated instance; "Restart as Administrator" during a spaced-path override Run demonstrably lands the elevated instance in the same directory
- [ ] Whole-app posture: unknown switch → dialog-and-continue — title "Unknown argument {arg} was ignored", the shared usage line in the body, [OK], one `WARN` line, then a normal start; a valueless or malformed `--data-dir` is a broken override → the fourth reason, not an unknown argument; `--tab`'s v0.1.0 leniency stays; `--help` and `-?` → a dialog with the same usage line, then exit
- [ ] Catalogue strings shipped in both languages: the unknown-argument title, the usage line, the `--help` dialog title, the fourth Read-only reason; i18n gate green
- [ ] The README's portability section documents the switch (space form); CONTEXT.md's Data Directory grows its one sentence; the User Guide's "Command line" subsection is ticket 11's to write
- [ ] A fresh-path `--data-dir` launch creates the directory there with `data\` beside the exe untouched — verified live
