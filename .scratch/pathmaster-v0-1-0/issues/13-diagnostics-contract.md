# Diagnostics contract

Type: grilling
Status: resolved
Blocked by: 05, 06

## Question

What are the exact rules for the five issue types, and how does each surface?

Settled at charting: similar-path/typo detection is cut; five types remain — duplicate, non-existent,
over-length, relative, empty.

- **Duplicate.** The normalisation used for comparison: case, trailing `\`, `/` versus `\`, `%VAR%` expanded or
  raw, 8.3 short names. Which entry carries the warning — every copy, or all but the first? And the raw string
  must still be written back untouched.
- **Non-existent.** A UNC path on a disconnected share can block for tens of seconds, so this check needs a
  per-entry timeout and must run off the UI thread (FR-auto-diagnose promises a full pass in under a second for
  200 entries — that promise is only keepable with a timeout). Decide: is a timed-out check a **third state**
  rather than an Error? What about a path that exists but is a file, or exists but is unreadable?
- **Over-length.** Entry-level status, banner, or both? The PRD puts it in both places without saying which
  entry would even carry it.
- **Relative.** Which shapes are flagged: `.`, `..`, a bare name, and drive-relative `\foo` — the last is not
  obviously the same category as the others.
- **Multiple issues on one entry.** Which status wins in the Status column, and what does NVDA announce?
- **Cross-scope duplicates.** An entry present in both User and System is genuinely a duplicate at process
  start, since Windows concatenates them. Is it reported, and on which tab?
- **`%VAR%` expansion.** Diagnostics need expansion even though the display toggle is deferred to v0.2.0 —
  where does that happen, and what if the variable does not exist?

Output: the rewritten FR-diag-* family with rules precise enough to write tests from.

## Carried in from ticket 03

FR-auto-diagnose's "asynchronous, does not block the UI" must name a mechanism, because the obvious one does
not exist and the near-miss fails silently:

- There is no `CallAfter` and no `QueueEvent`/custom event with a payload; `EventType` is a closed set.
  `call_after` is a Rust-side queue drained only from the idle handler, at most 10 per tick, and it does **not**
  call `wake_up_idle()` — so a callback queued while the app is truly idle may wait for the next UI activity.
- Widget handles are auto-`Send` but resolve through a **thread-local** registry: calling a widget from the
  worker thread compiles, silently no-ops, and updates nothing. The rule to write into the spec is *widgets may
  be captured across threads but only called on the UI thread.*
- The upstream-recommended shape is worker thread → `mpsc` → drained inside `on_idle` with
  `request_more(has_more)`, or a `Timer`. Decide which, and state it in the acceptance criteria.

## Carried in from ticket 05

- **The over-length check runs on the expanded, merged string**, not the raw sum of the two scopes — measured
  2207 raw versus 2198 expanded on the research machine. The 32767 is the documented limit for **one
  environment variable**; the old environment-*block* limit was lifted after Server 2003, and `setx`'s 1024
  crop is `setx`'s own. What actually breaks on overflow is UNKNOWN, so the requirement must **warn at a
  threshold** rather than assert a failure mode.
- **Diagnose the working copy, never the process environment.** This process's own `PATH` was 1796 chars while
  a fresh merge computes 2198 — an app that reads `std::env::var("PATH")` diagnoses a stale snapshot of
  whatever launched it.
- **Expansion for diagnostics must not leak into what gets written.** Comparison and existence checks work on
  expanded values; the raw substring is what goes back to the registry, byte for byte.

## Carried in from ticket 06

- **Issues are a derived view of the Working Copy and are never part of it** — excluded from Checkpoints and
  recomputed after any undo or restore, so Undo can never reinstate a diagnosis of a state no longer displayed.
  Any change to a Working Copy invalidates that Scope's Issues.
- **The merged over-length check takes both Scopes' Working Copies** (expanded), not the registry values — the
  warning is about what the user is *about to create*. For a read-only System Session the two coincide.
- **Two edge cases the Entry model hands straight to this ticket.** An empty value decodes to **zero Entries**,
  so a fresh empty `PATH` must not report `Empty entry` — but a **trailing `;` does** produce a genuine empty
  Entry, and that one is a real finding. Decide whether they read differently to the user. Separately, a
  whitespace-only Entry (`"   "`) is a legal Entry preserved verbatim — decide whether it counts as empty.
- **Diagnostics is the only consumer of Normalisation**, which is a comparison-time function whose result is
  never stored and never written. This ticket therefore owns its exact definition (case, trailing `\`, slash
  direction, `%VAR%` expansion) — ticket 06 fixed only that it exists and where it may not leak.

## Carried in from ticket 09

The Status column is the per-entry carrier (ticket 09, D1): issue types only, no severity prefix; several
Issues comma-joined in a fixed severity order; empty column for a healthy Entry (never "OK"). This ticket
owns the **exact word for each of the five types** and the **severity order** used for joining. Keep each
word short — it is spoken on every arrow key over an affected row.

## Answer

Resolved 2026-08-19 through a twelve-question grilling round, each question grounded first in web
research against primary sources — facts in
[research/13-diagnostics-facts.md](../research/13-diagnostics-facts.md). The charting-time "five
types" constant is consciously amended: **six Issue types**, `Quoted` added on measured evidence
(D7). Decisions D1–D10 are recorded in the Comments below as they were made; D11–D12 complete them:

- **D11 Async mechanism** — one worker thread computes a diagnostic pass and sends results over an
  `mpsc` channel; a wx Timer (~100 ms, running **only while a pass is outstanding**) drains the
  channel on the UI thread and stops. The idle-handler route was rejected on ticket 03's measured
  trap (idle may not fire while the app is truly idle; `request_more` busy-spins). Widgets are
  never called off the UI thread.
- **D12 Status column words, order, coexistence** — msgids `Missing`, `Relative`, `Quoted`,
  `Duplicate`, `Empty` (uk: Відсутній, Відносний, У лапках, Дублікат, Порожній), comma-joined
  most-severe-first in exactly that order. Coexistence rules: **Empty is exclusive** (an Empty
  Entry carries nothing else — two empties are not also duplicates); **Relative skips the
  existence check** (its resolution depends on process state, so Relative and Missing never
  co-occur); **Quoted co-occurs freely** (all checks run on the quote-stripped text). Over-length
  never appears in the column (D6). One-word labels because the text is spoken on every arrow-key
  over an affected row; RapidEE's colour-only flagging (no words at all) is exactly the pattern a
  screen-reader user cannot use.

### The rewritten FR-diag family

- **FR-diag-split.** The raw value splits into Entries on **every** `;`; quotes never protect a
  separator. (Matches `CreateProcessW`/`SearchPathW`, PowerShell, Python, PowerToys.)
- **FR-diag-normalise.** Normalisation is comparison-only — never stored, never written: strip one
  pair of surrounding `"` → expand `%VAR%` (`ExpandEnvironmentStrings`, process environment,
  unknown names stay literal) → `/`→`\` → trim trailing `\` unless that leaves a bare drive root
  (`C:\` stays `C:\`, never `C:`) → compare ordinal case-insensitively. Never touches the
  filesystem: no 8.3, no symlinks, no canonical casing.
- **FR-diag-duplicate.** Entries with equal Normalisations are duplicates. Evaluation order is the
  runtime order: System Working Copy first, then User, each left to right. The first occurrence is
  canonical and clean; **every later occurrence flags `Duplicate`** — cross-scope included, where
  the User copy carries the flag. Editing either Working Copy recomputes both Scopes' duplicates.
- **FR-diag-missing.** Local-rooted Entries only (root classified via `GetDriveTypeW` / UNC prefix
  — no network round trip): flag `Missing` when the quote-stripped expanded path does not name an
  existing **directory** — not-found and exists-but-is-a-file both flag (a file entry is inert:
  path search appends `\name.exe` to the entry-as-directory); `ERROR_ACCESS_DENIED` does **not**
  flag (the object exists). Network-rooted Entries are never probed in v0.1.0 and never flag —
  documented in the README. An undefined `%VAR%` flags Missing naturally (the literal text fails).
- **FR-diag-relative.** Flag `Relative` on any Entry that is not fully qualified. Qualified:
  `X:\…`, `\\server\share…`, `\\?\…`. Flagged: `.`, `..`, bare names, rooted `\foo`,
  drive-relative `C:foo`. Relative Entries skip the existence check.
- **FR-diag-empty.** Flag `Empty` on a zero-length or whitespace-only Entry. An Absent or empty
  Scope decodes to zero Entries and reports nothing; a trailing `;` produces a genuine empty Entry
  and does flag.
- **FR-diag-quoted.** Flag `Quoted` on any Entry containing `"`. Rationale: measured consumer
  lottery — the quoted spelling is dead for `CreateProcessW`/`SearchPathW`, PowerShell, `where`,
  Python, alive for cmd/CRT/Rust/Node; the fix is trivial and the breakage otherwise silent.
- **FR-diag-overlength.** Scope-level, not per-entry: the merged length is
  `len(expand(System WC) + ";" + expand(User WC))` in UTF-16 code units, shown in a **passive
  StatusBar field** (queried via NVDA+End). At Apply, if the post-write merged length exceeds
  **8,191** → warning dialog, title carries the message ("cmd.exe will ignore this PATH" —
  KB 830473), proceed allowed; **≥ 32,767** → same dialog, no proceed button (hard cap,
  `SetEnvironmentVariableW`). No per-entry finding, no Announcement (catalogue stays at seven),
  no 2,047 warning (Vista-era folklore).
- **FR-diag-async.** A pass runs on one worker thread; results reach the UI thread via the D11
  Timer-drained channel. Issues are a derived view of the Working Copies: recomputed after any
  edit, undo/redo, Refresh or restore; excluded from Checkpoints (ticket 06).
- **FR-diag-status.** The Status column carries the flagged types' words, comma-joined in the D12
  order; an empty column is the only healthy state (never "OK") — per ticket 09.

### Hand-offs

- **Ticket 17**: the StatusBar gains a merged-length field; the layout must place it (comment added
  there). Diagnostics claims no Banner use.
- **Ticket 19**: every rule above is pure logic over `(raw string, per-path filesystem verdict)`
  and unit-testable; only the Timer drain and the spoken column need the manual NVDA script.
- **Map Notes**: charting constraint 7 amended — six diagnostic types, not five.

## Comments

**2026-08-19, pre-grilling research (at the user's request):** four web investigations against
primary sources — quoted-entry consumer behavior (measured empirically), duplicate-normalisation
practice in five tools (source code read), existence-check blocking and mitigation, and real
over-length thresholds. Facts: [research/13-diagnostics-facts.md](../research/13-diagnostics-facts.md).
Headlines: quotes are a consumer lottery (cmd/CRT/Rust/Node strip them, the OS itself/PowerShell/
Python do not) and even *splitting* on `;` is quote-aware for half the ecosystem; no surveyed tool
touches the filesystem for duplicate comparison; a dead UNC path blocks 20–60 s and cannot be
cancelled, so "timed out" must be a distinct state; the honest over-length numbers are 8,191
(cmd drops the variable entirely) and 32,767 (hard cap) — 2,047 is folklore.

**2026-08-19, grilling round in progress — decisions so far (D-numbers for the resolution):**

- **D1 Splitting is naive** — every `;` separates, quotes never protect it. Matches the OS's own
  `CreateProcessW`/`SearchPathW`, PowerShell, Python, and PowerToys; ticket 06's Entry definition
  ("raw substring between `;` separators") stands unchanged. The rare quoted-semicolon entry shows
  as two odd Entries — the same two the OS itself sees.
- **D2 Normalisation for duplicate comparison** — case folded, trailing `\` trimmed (except a bare
  root `C:\`, which must not become drive-relative `C:`), `/`→`\`, one pair of surrounding double
  quotes stripped, `%VAR%` expanded. Never touches the filesystem: no 8.3 resolution, no symlinks,
  no canonical casing (no surveyed tool does either). `%SystemRoot%\system32` **is** a duplicate of
  `C:\Windows\system32`.
- **D3 Duplicate carrier** — the first copy in execution order (System scope first, then position)
  is canonical and stays clean; **every later copy is flagged**, including cross-scope, where the
  User copy carries the flag. Accepted consequence: the User tab's Issues depend on the System
  Working Copy, so a System edit recomputes the User scope's diagnostics too.
- **D4 Network paths are never probed in v0.1.0** — `GetDriveTypeW`/UNC-prefix classification (no
  network round trip) splits entries into local (fully checked, ~10 ms for 200) and network (never
  checked, no prober threads, no zombie hangs). A deadline prober is a v0.2.0 candidate.
- **D5 Existence verdicts** — the check means "the Entry names an existing *directory*": a path that
  exists but is a file → **Non-existent** (a file entry is inert — search appends `\name.exe` to the
  entry-as-directory); access denied → **no Issue** (the object exists; not repeating the .NET
  `File.Exists` mistake); network → no Issue and no Status text (documented in the README). The
  Status column stays "empty = healthy" per ticket 09.
- **D6 Over-length leaves the per-entry Issue set entirely** — it is a property of the merged
  expanded PATH (System + `;` + User, both Working Copies), carried by two scope-level surfaces:
  a passive StatusBar field with the current merged length (visible always, queried via NVDA+End),
  and an Apply-time dialog when the result would exceed **8,191** (cmd.exe drops the variable —
  KB 830473; title carries the message, user may proceed) or **≥ 32,767** (hard cap — same dialog,
  no proceed button). 2,047 is Vista-era folklore and is not warned about. The Announcement
  catalogue stays closed at seven.
- **D7 Quoted is a sixth Issue type.** An Entry containing `"` is flagged: measured, the quoted
  spelling is the same directory for cmd/CRT/Rust/Node but **dead for `CreateProcessW`/`SearchPathW`,
  PowerShell, `where.exe` and Python** — a silent breakage with a trivial fix (F2, remove quotes).
  The charting-time "five types" constant is consciously amended; no MS documentation legitimises
  quotes in PATH, and Rust's `join_paths` refuses to write them.
- **D8 Relative = not fully qualified** ([.NET path taxonomy](https://learn.microsoft.com/en-us/dotnet/standard/io/file-path-formats)):
  flags `.`, `..`, bare names, rooted `\foo` **and** drive-relative `C:foo` — everything that
  resolves against the process's current state. Clean forms: `X:\…`, `\\server\share…`, `\\?\…`.
  (`PathIsRelativeW` semantics rejected — it passes both hazardous forms.)
- **D9 Empty covers whitespace-only.** Empty means "no usable path text": zero-length Entries (from
  `;;` or a trailing `;`) and whitespace-only alike. A fresh empty value is zero Entries and reports
  nothing (ticket 06). Unlike Unix, an empty Windows PATH element has no current-directory
  semantics — it is dead weight, not a hazard.
- **D10 Undefined `%VAR%` → Non-existent naturally.** Expansion runs against the process
  environment via `ExpandEnvironmentStrings` (documented: unknown names stay literal, lookup is
  case-insensitive); the literal `%FOO%\bin` then fails the existence check. No seventh type; the
  stale-inherited-environment limitation is documented in the spec. Building a fresh merged
  environment from the registry is out of v0.1.0.

**2026-08-19, from ticket 10 (resolved):** two hand-offs.

1. **Whitespace-only Entries commit.** The editor blocks only length-zero values — blocking `"   "` would
   smuggle a trim into validation, and the editor never trims (ticket 06). So whether a whitespace-only
   Entry reads as `Empty entry` is now entirely this ticket's call, as its existing bullet suspected.
2. **Quoted entries are a real Normalisation input.** The editor forbids typing a new `;` (ticket 10, D5),
   but values arriving from the registry may contain **quoted entries** (`"C:\Program Files\foo"`, and
   historically quotes were how a `;` was embedded in one entry). A raw existence check on the quoted string
   false-positives `Non-existent`. Decide whether Normalisation strips surrounding quotes for existence and
   duplicate comparison — the raw string still round-trips untouched — and verify first *which* Windows
   consumers actually honour quotes in `PATH` (a fact to check, not assume).
