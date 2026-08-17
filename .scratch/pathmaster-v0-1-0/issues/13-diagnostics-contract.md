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
