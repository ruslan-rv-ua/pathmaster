# 20 — The Run's properties, decided in one place

**Spec:** [spec §11 (startup order), §3 (Data Directory, Read-only Data), §9 (writability and elevation), §13 (settings taxonomy), §14 (a Run without a log), §17 (module layout)](../../pathmaster-v0-1-0/spec.md) · ADR-0002, ADR-0007, [ADR-0010](../../../docs/adr/0010-run-properties-decided-in-one-place.md)

**What to build:** The seven rules `main` currently holds between its already-tested calls move into `platform::startup`, which decides everything a **Run** is — where its data lives and whether it can be written, whether it is elevated, what language it speaks, whether it has a log, and which Scopes it may write — and hands back the Sessions, the records to log and the facts the window needs. `main` keeps the wiring and nothing else.

**Blocked by:** 13 (which builds the Run's facts in `main`; this moves where they are built).

**Blocks:** 15, so its geometry clamping lands in a tested module rather than in `main`.

**Status:** ready-for-agent

- [ ] `platform::startup` decides all seven rules: Read-only Data is a Run without a log; the panic hook installs only where there is a log path; the startup record precedes the settings records; `Source`'s three arms decide one dialog flag and the `WARN` records; User writes with the Run and System also needs elevation; the Read-only reason survives to the UI; a Scope whose read fails becomes an empty **non-writable** Session
- [ ] Its parameters are the irreducible OS facts — the located directory, the elevation answer, the system language, and the two `ScopeKey`s — and it performs everything downstream: establish, read settings, resolve the language, decide writability, load the Sessions. The pattern `datadir::decide` and `locale::from_langid` already set
- [ ] It returns one struct whose fields `main` destructures; one of them **is** ticket 13's Run-facts struct, so this ticket changes where that is built and not what it holds
- [ ] It returns the `Logger` and the `Record`s; `main` writes them (ADR-0008's shape). Deciding *whether there is a log* stays here — that is rule one
- [ ] The backup budget leaves the Run's facts: the window holds the current `SettingsFile`, and an Apply Run reads `maxBackups` from it each time, which is what makes ticket 16's "applies immediately" ordinary rather than special
- [ ] `datadir::startup()` is **deleted**. `main` calls `current_exe()` and `locate` in one line and passes the result; `decide` and `establish` stay public and separately tested
- [ ] `main` keeps only assembly: the located directory, the wx entry, `catalog::install`, wrapping the Sessions, building the window, the settings dialog, the exit code
- [ ] Tests aim the whole sequence at a temporary directory, a temporary registry key under `HKCU\Software\PathMasterTest` and **both** elevation answers — no privilege, no real machine. The writability table and the failed-read rule are the two that matter most
- [ ] `CONTEXT.md` gains **Run** (done ahead of this ticket, with ADR-0010) and the type is named for it
- [ ] Spec §17's `pathmaster-platform` module list gains `startup`

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
