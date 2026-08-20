# 09 — Diagnostics rules (core)

**Spec:** [spec §7](../../pathmaster-v0-1-0/spec.md) · research/13

**What to build:** The complete diagnostic rulebook in `pathmaster-core`, verified by `cargo test`: Normalisation, the six Issue types, coexistence and severity ordering, and the merged-length threshold logic. Issues are a derived view of the Working Copies — never part of them, excluded from Checkpoints. (The async pass, Status column, and StatusBar wiring are ticket 12.)

**Blocked by:** 02 — operates on the core Entry/Working Copy types.

**Status:** resolved

- [x] Split rule: the raw value splits on every `;`; quotes never protect a separator
- [x] Normalisation is comparison-only, never stored, never written, never touches the filesystem: strip one pair of surrounding `"` → expand `%VAR%` (unknown names stay literal) → `/`→`\` → trim trailing `\` unless that leaves a bare root (`C:\` stays) → compare ordinal case-insensitively; property test: Normalisation idempotence
- [x] `Duplicate`: equal Normalisations; evaluation order is runtime order — System Working Copy first, then User, left to right; first occurrence canonical and clean, every later copy flags, cross-scope included (the User copy carries it)
- [x] `Missing`: local-rooted Entries only (root classified via drive type / UNC prefix, no network round trip); flags when the quote-stripped expanded path does not name an existing directory (not-found and is-a-file both flag; access-denied does not); network-rooted Entries are never probed and never flag; an undefined `%VAR%` flags naturally (the filesystem probe itself is injected/adapted so the rules stay unit-testable)
- [x] `Relative`: any Entry not fully qualified (qualified: `X:\…`, `\\server\share…`, `\\?\…`; flagged: `.`, `..`, bare names, rooted `\foo`, drive-relative `C:foo`); Relative Entries skip the existence check
- [x] `Empty`: zero-length or whitespace-only Entry; an Absent or empty Scope reports nothing; a trailing `;` produces a genuine empty Entry and does flag
- [x] `Quoted`: any Entry containing `"`
- [x] Coexistence: Empty is exclusive; Relative and Missing never co-occur; Quoted co-occurs freely; severity order Missing > Relative > Quoted > Duplicate > Empty
- [x] Over-length is scope-level, never per-entry: merged length = `len(expand(System WC) + ";" + expand(User WC))` in UTF-16 code units; threshold logic for 8,191 (warn) and 32,767 (hard cap) unit-tested

## Comments

Implemented 2026-08-20. Three modules land in the pure core — `normalize`, `diagnostics`,
`thresholds` — held by 66 tests at the crate boundary, none of which links wx, touches the
registry or reads this machine.

- **`normalize`** is the five-step comparison pipeline and nothing else. `Normalised::of` runs
  all of it; `Normalised::of_expanded` runs the tail for the caller that already needed the
  quote-stripped expanded text for its filesystem probe, so nothing expands twice.
- **Expansion was measured against `ExpandEnvironmentStringsW`**, not assumed (2026-08-20): a
  failed lookup emits its opening `%` and rescans from the very next character, so
  `%NOPE%SystemRoot%` expands its second half; `%%` is not a reference; an unterminated `%` is
  ordinary text; the pass is single, so a value that itself carries `%VAR%` is not expanded
  again; the lookup ignores case. `Expansion` reports one derived fact besides the text — whether
  the text *begins* with a name this run does not define — which is what the Relative rule turns on.
- **The environment and the filesystem are injected**, as the ticket asks: `Environment`
  (name → value, case-insensitive as `GetEnvironmentVariableW` is) and `Filesystem`
  (`root_kind`, then `probe` — two questions on purpose, so the root is classified without a
  network round trip and only a local root is ever probed). The fake in the tests records what
  it was asked, so "network-rooted Entries are never probed" is asserted, not assumed. The
  Windows adapters over `GetDriveTypeW`/`GetFileAttributesW` belong to the pass that runs them
  (ticket 12).
- **A road the spec left implicit.** Two of its sentences pull against each other: an undefined
  `%VAR%` "flags Missing naturally", yet `%NOPE%\bin` is not fully qualified either, and
  Relative skips the existence check. Taken road: **text that *begins* with an unresolved
  reference is not judged for shape** — what the path would have started with is exactly what is
  missing — so the Entry goes to the probe and the literal text fails there, as FR-diag-missing
  (D10) describes. The guard is that narrow on purpose: `tools\%NOPE%` is a bare name whatever
  `%NOPE%` was, so it stays Relative and stays unprobed. Qualification is otherwise judged
  *after* expansion, without which `%SystemRoot%\System32` — the commonest Entry there is —
  would read as Relative.
- **Either separator qualifies.** `C:/tools` and `//server/share` are fully qualified, though §7
  spells the three shapes with backslashes only. The .NET path taxonomy §7 cites answers the same
  way, `GetFileAttributesW` resolves `C:/Windows` (measured), and FR-diag-normalise already treats
  the two as one — flagging `C:/tools` as Relative would be a false positive, which is the one
  thing this rulebook was told not to generate.
- **The probe reads the expanded text verbatim**, slashes and all: measured,
  `GetFileAttributesW` answers for `C:/Windows` exactly as for `C:\Windows`, and converting
  would change what a `\\?\` path means. Slash direction belongs to the comparison key, which is
  the only thing FR-diag-normalise describes.
- **Two more measurements**, both confirming rules rather than changing them: `"C:\Windows"`
  fails `GetFileAttributesW` with `ERROR_INVALID_NAME` (why the probe reads past the quotes, and
  why `Quoted` is a finding at all), and so does ` C:\Windows` — a leading space genuinely breaks
  an Entry, so calling it `Relative` is not a false positive. A *trailing* space is trimmed by
  Win32 but not by Normalisation, so `C:\Windows ` and `C:\Windows` are not duplicates: five
  steps, deliberately, and no sixth.
- **Severity order is structural.** `Issue::SEVERITY` is the one list, and each Entry's findings
  are built by filtering it, so "most-severe-first" cannot drift from the declared order. Empty
  returns early and alone; a Relative Entry never reaches the probe.
- **Over-length never enters the per-entry rules.** `thresholds` holds the two measured numbers,
  the UTF-16 count, and `Overlength::may_proceed` — the one place "8,191 is walkable, 32,767 is
  a wall" is written down. `Diagnosis::merged_length()` rides along with every pass, since
  ticket 12's StatusBar needs it after each one.
- **Split** (FR-diag-split) landed with ticket 02; the case the rule exists for — a `;` inside
  quotes still separating — now has its own test in `tests/path.rs`.
- **The idempotence property carries one carve-out, in writing.** Quote stripping takes one pair
  by spec, so a doubly quoted Entry loses a pair per pass; the property assumes that shape away
  and says why beside the assumption. No pass produces it, so nothing reachable is unproved.
- **Spec §18 and ticket 23 amended in the same pass** (the repo's own convention — cf. b1945ac):
  the three property tests live in the test file of the module they constrain, not in a shared
  `properties.rs`. Ticket 02 landed the first that way without recording it; this ticket landed
  the second and wrote it down. The cap of exactly three is untouched.

Reviewed on both axes (`/code-review`) before the commit. The Spec axis found the unresolved-`%VAR%`
guard too wide — it read "any reference failed" where the rule needs "the leading one did" — and
that is fixed above, with two tests (`tools\%NOPE%`, `..\%NOPE%`) that would have caught it. The
Standards axis found the property-file divergence (amended, above) and a module header that said
"six Issue types" over a five-variant enum (corrected: five name an Entry, the sixth is
scope-level, and CONTEXT.md's Issue covers both). `Diagnosis` lost `system()`/`user()`; `scope()`
alone answers.

Hand-offs to ticket 12: the `Environment`/`Filesystem` adapters over `GetEnvironmentVariableW`,
`GetDriveTypeW` and `GetFileAttributesW`; the Issue-type words and their Ukrainian translations
(nothing here touches the Catalogue — the rulebook names no user-visible string); and the worker
thread plus Timer drain that turn a `Diagnosis` into the Status column.
