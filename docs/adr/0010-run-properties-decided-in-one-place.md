# The properties of a Run are decided in one place, and `main` stays the composition root

Unlike the two before it, this record settles a **boundary** rather than a contested choice: what belongs
in the module that decides a Run's properties, and — more usefully — what deliberately does not. The
instinct it exists to head off is "`main` is a hundred lines, empty it": that is the wrong half to move,
and the distinction it rests on (what is a property of the Run and what merely looks like one) is subtle
enough to be got wrong twice, having already been got wrong once here.

**Most of `main` was never the problem.** Its hundred lines are overwhelmingly calls to functions that
already have tests — `datadir`, `settings::read`, `language::resolve`, `locale`, `Logger`,
`panic_hook`, `ScopeKey::read`, `Session::new`, `Record::startup`. What has none is the glue between
them, and it is exactly seven rules: Read-only Data is a Run without a log; the panic hook installs only
where there is a log to install against; the startup line precedes the settings lines; the three arms of
`Source` decide one dialog and a number of `WARN` records; User writes with the Run while System also
needs elevation; the Read-only reason survives into the UI; and a Scope whose startup read fails becomes
an empty **non-writable** Session, because nothing may be written over a value that was never read. The
last is the sharpest — the spec never names a failed startup read, so that rule is the ticket's own
invention, and it has been carrying the application's data safety untested since impl ticket 08.

**Those seven move; the wiring does not.** `main` is the composition root, and assembling the pieces is
what a composition root is for — installing the Catalogue, wrapping the Sessions, opening the window,
returning the exit code. Moving that too would relocate code without making anything testable, since
none of it decides anything. What comes out is decisions; what stays is assembly.

**The seam is the OS call, not the crate boundary.** `startup::decide` takes the located directory, the
elevation answer and the system language as parameters, and performs the rest — establish, read
settings, resolve the language, decide writability, load the Sessions through the `ScopeKey`s it is
given. That is the shape `datadir::decide` and `locale::from_langid` already have, and the rule behind
it is worth stating plainly: **factor out the one call a test cannot make fail, and keep everything
downstream of it.** A test then aims the whole startup sequence at a temporary directory, a temporary
registry key and both elevation answers, without needing a privilege or a real machine.

**A property of the Run is not the same as a setting.** Three things reached the window as "the Run's
facts" in the first draft of impl ticket 13 — the `Logger`, the Data Directory, and the backup budget —
and the third does not belong: impl ticket 16 changes `maxBackups` while the application is running.
Keeping a copy of it beside genuinely fixed facts would have made ticket 16 remember to update two
places, and would have made a mutable number a member of a set defined by never changing. The window
holds the current `SettingsFile` instead, and an Apply Run reads the budget from it each time.

## Consequences

- **`CONTEXT.md` gains `Run`.** Two existing entries — Read-only Data and Interface Language — already
  lean on "a property of the run" without the glossary defining it, and a third and fourth are arriving
  (elevation, and impl ticket 17's `--tab`). One definition replaces the repetition, and gives the type
  its name.
- **`datadir::startup()` is deleted, not kept as a convenience.** It bundles locate and decide, and
  after this it has no caller. Left standing it would be a second route to the Data Directory decision
  that bypasses the writability rule now built on top of it, and a second route is what someone reaches
  for by mistake. Its two halves stay public and separately tested; `main` calls `current_exe()` and
  `locate` in one line, which is the irreducible OS fact staying at the edge where it belongs.
- **The module returns records rather than writing them**, as the Apply Run does
  ([ADR-0008](0008-apply-sequence-lives-in-platform.md)) — so what a startup logs is assertable without
  a filesystem. It still decides *whether there is a log at all*, because that is rule one.
- **It loads the Sessions.** That drags two registry reads into its span, which a narrower module would
  have avoided — but the failed-read rule lives inside that load and nowhere else, and it is the rule
  most worth having a test.
- **Sequenced after impl ticket 13, before 15.** 13 is what gives the user Ctrl+S and has already slipped
  once behind ticket 19; it builds the Run's facts in `main` and this moves where they are built, which
  is a change of one call site rather than of a shape. Arriving before 15 is what lets that ticket's
  geometry clamping — pure arithmetic over monitor rectangles — land in a tested module rather than in
  `main`.
- **Spec §17 does not name this module, or two others now arriving.** §17 calls its module names
  indicative and fixes only the inter-crate seams, so adding one is not a violation — but impl ticket 12
  treated the reverse gap as worth closing, so each ticket amends §17 with its own module's name as that
  module lands, rather than the spec predicting three names at once.
