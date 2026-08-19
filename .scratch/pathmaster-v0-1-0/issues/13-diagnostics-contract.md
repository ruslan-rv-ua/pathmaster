# Diagnostics contract

Type: grilling
Status: open
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
