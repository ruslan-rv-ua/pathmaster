# 20 — The Run's properties, decided in one place

**Spec:** [spec §11 (startup order), §3 (Data Directory, Read-only Data), §9 (writability and elevation), §13 (settings taxonomy), §14 (a Run without a log), §17 (module layout)](../../pathmaster-v0-1-0/spec.md) · ADR-0002, ADR-0007, [ADR-0010](../../../docs/adr/0010-run-properties-decided-in-one-place.md)

**What to build:** The seven rules `main` currently holds between its already-tested calls move into `platform::startup`, which decides everything a **Run** is — where its data lives and whether it can be written, whether it is elevated, what language it speaks, whether it has a log, and which Scopes it may write — and hands back the Sessions, the records to log and the facts the window needs. `main` keeps the wiring and nothing else.

**Blocked by:** 13 (which builds the Run's facts in `main`; this moves where they are built).

**Blocks:** 15, so its geometry clamping lands in a tested module rather than in `main`.

**Status:** resolved

- [x] `platform::startup` decides all seven rules: Read-only Data is a Run without a log; the panic hook installs only where there is a log path; the startup record precedes the settings records; `Source`'s three arms decide one dialog flag and the `WARN` records; User writes with the Run and System also needs elevation; the Read-only reason survives to the UI; a Scope whose read fails becomes an empty **non-writable** Session
- [x] Its parameters are the irreducible OS facts — the located directory, the elevation answer, the system language, and the two `ScopeKey`s — and it performs everything downstream: establish, read settings, resolve the language, decide writability, load the Sessions. The pattern `datadir::decide` and `locale::from_langid` already set
- [x] It returns one struct whose fields `main` destructures; one of them **is** ticket 13's Run-facts struct, so this ticket changes where that is built and not what it holds
- [x] It returns the `Logger` and the `Record`s; `main` writes them (ADR-0008's shape). Deciding *whether there is a log* stays here — that is rule one
- [x] The backup budget leaves the Run's facts: the window holds the current `SettingsFile`, and an Apply Run reads `maxBackups` from it each time, which is what makes ticket 16's "applies immediately" ordinary rather than special
- [x] `datadir::startup()` is **deleted**. `main` calls `current_exe()` and `locate` in one line and passes the result; `decide` and `establish` stay public and separately tested
- [x] `main` keeps only assembly: the located directory, the wx entry, `catalog::install`, wrapping the Sessions, building the window, the settings dialog, the exit code
- [x] Tests aim the whole sequence at a temporary directory, a temporary registry key under `HKCU\Software\PathMasterTest` and **both** elevation answers — no privilege, no real machine. The writability table and the failed-read rule are the two that matter most
- [x] `CONTEXT.md` gains **Run** (done ahead of this ticket, with ADR-0010) and the type is named for it
- [x] Spec §17's `pathmaster-platform` module list gains `startup`

## Comments

Designed 2026-08-20 with tickets 13 and 19, out of the same architecture review. The reasoning is in
[ADR-0010](../../../docs/adr/0010-run-properties-decided-in-one-place.md); two things are worth repeating
where the work happens.

**The review's own card overstated this one, and the correction is the ticket.** It said `main` was "a
hundred untested lines". It is a hundred lines, but almost all of them call functions that already have
tests — `datadir`, `settings::read`, `language::resolve`, `locale`, `Logger`, `panic_hook`,
`ScopeKey::read`, `Session::new`, `Record::startup`. What has no test is the glue, and the glue is seven
rules. That is a smaller finding, and it is why this ticket is scoped to decisions rather than to
emptying `main`: the composition root is doing its job, and moving the assembly out would relocate code
without making anything testable.

**Two of the seven are load-bearing and the rest ride along.** `data_writable && elevated` is one `&&`
that ADR-0002 calls a trap when wrong — a Working Copy that can never be applied. And a failed startup
read producing an empty *non-writable* Session is a rule the spec never states: impl ticket 08 invented
it, correctly, on the grounds that nothing may be written over a value that was never read, and it has
been carrying that weight untested ever since.

---

Implemented 2026-08-21 on `feature/startup-decisions-module`. `pathmaster-platform::startup` now owns
the seven rules and the two types they produce; `main.rs` is 200 lines lighter at 91, of which the
decision half is one destructuring `let` over five arguments. Fifteen tests behind the module, none
of them needing a privilege or the real `PATH`.

**The Run-facts struct is called `Run`, and that is the whole of the naming change.** `RunFacts` was
the placeholder ticket 13 used before `CONTEXT.md` had the word; the glossary has it now, and
`apply.rs`'s own doc already explains why the Apply sequence's per-pass type had to be `ApplyRun` —
it was avoiding a collision with a name nothing yet held. It holds exactly what it held: the
`Logger`, the Data Directory, the log path.

**`LoadedScope` split in two, which was not a rename.** What startup returns holds a bare `Session`;
what the window takes holds an `Rc<RefCell<Session>>` and is called `SharedScope`. The wrap is the
one line of "wrapping the Sessions" the ticket leaves in `main`, and putting it there is what keeps
`Rc` — a decision about the window's lifetime — out of a crate that has no window. Two types rather
than one because the seam is real: `startup` cannot name the sharing, and the window cannot use the
Session without it.

**Rule two is the one that fought the test harness, and the fight is the reason for two of the
fifteen tests.** `panic_hook::install` replaces the process-wide hook, and a hook that appends to a
log file prints nothing — so a `decide` left un-guarded inside a test binary silently swallows
libtest's own failure reporting for every test after the first, turning a later assertion failure
into a bare `FAILED` with no message. The in-process tests therefore put the harness's hook back the
moment `decide` returns (under a lock, since the hook is global and the tests run in parallel), and
rule two itself is asserted where clobbering the hook is the *point*: two child processes, one per
arm, using the same re-run-the-test-binary harness `tests/panic_hook.rs` already had. The writable
arm finds the `ERROR panic:` line in the log and **no** `panicked at` in the child's output; the
read-only arm finds the opposite, which is what "installs only where there is a log path" means when
read from the outside.

**Two smaller choices worth naming:**

- **The version stays `env!("CARGO_PKG_VERSION")` rather than becoming a parameter.** The ticket
  fixes the parameter list at the irreducible OS facts, and the workspace pins one version across
  all three crates — so the constant read in `pathmaster-platform` is the binary's version, and the
  test asserts the startup record carries it.
- **`datadir::startup()`'s test did not go with it.** It was the only live check that `locate` and
  `decide` agree against a real executable, so it is now `decide(locate(&exe))` under a name that
  says so — the same coverage, minus the convenience ADR-0010 deletes.

**Verified live on this machine.** Launched the debug build against a deleted `data\`: the directory
and `pathmaster.log` were created, the one line in it reads `INFO startup: PathMaster 0.1.0,
elevated: no, data: writable, language: uk`, both Scope lists loaded (43 User Entries, 19 System),
and `WM_CLOSE` exited 0 with nothing further logged. That is every rule but the read-only ones
exercised end to end through the real composition root.
