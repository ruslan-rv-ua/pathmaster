# 13 — CHANGELOG.md, back-filled and wired into the release

**Spec:** amends [delta-spec §18](../../pathmaster-v0-2-0/spec.md) — release mechanics · [docs/release-checklist.md](../../../docs/release-checklist.md) step F2 · [docs/agents/issue-tracker.md](../../../docs/agents/issue-tracker.md)

**What to build:** A `CHANGELOG.md` at the repository root in Keep a Changelog form, back-filled so v0.2.0 ships with a real entry, gated at release time by a test in the shape of `versioninfo.rs`, and made the source of the release page's notes — which today are empty. The file is **developer-facing by decision**: the user-facing document is the User Guide (ticket 11), and this one is the convention a repository keeps for the people working in it.

**Blocked by:** nothing. Tickets 01–11 are done or in review, and the back-fill reads the delta-spec rather than the code. Runs beside 12; the two touch `docs/release-checklist.md` in different sections (12 in A/B, this one in F).

**Status:** done — the file, its gate and the release body are built, and the extractor was exercised in `pwsh` against the real `CHANGELOG.md` in all four of its cases; **the one leg no local run can press is the release page itself**, which lands at F4 of the next release's Checklist pass, see Comments

**This ticket amends the locked delta-spec.** §18 says "the version bump is F2's two files plus the tag" and "nothing to decide" about release mechanics. Both become false here: F2 bumps **three** files, and `release.yml` gains a workflow edit. §18 was right about what v0.2.0's *features* needed; this is a decision taken after it locked, and it is recorded here rather than by reopening the spec.

- [x] `CHANGELOG.md` at the repository root, [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/) form: `## [Unreleased]` on top, then released versions newest-first as `## [X.Y.Z] - YYYY-MM-DD`, with `### Added` / `### Changed` / `### Deprecated` / `### Removed` / `### Fixed` / `### Security` in that order and only where non-empty. **English only** — no `CHANGELOG.uk.md`, and no E1-style sync step: the bilingual rule covers the README and the User Guide, which are what users read
- [x] `## [0.1.0] - 2026-08-25` is **one line** — the first release, no category headings. It is written retroactively and says nothing it would have to be true about
- [x] `## [Unreleased]` is **back-filled in full** from the delta-spec, not from `git log` — the commit subjects here are deliberately oblique and would not survive being read as a change list. `### Added`: the permanent `#` column (§2.1), Search (§3, Ctrl+F), Filter (§4, Ctrl+I), Expansion Mode (§5, Ctrl+E), Tree View (§6, Ctrl+T), Fix Issues (§7), Copy entry (§8, Ctrl+C), the User Guide and F1 (§9), `--data-dir` with the whole-app argument posture (§10). `### Changed`: the menu bar grows View (§12), three new `settings.json` fields (§15), the StatusBar's field 0 (§16), and the UI's borrow discipline made structural (§11, ADR-0011). **No `### Removed`** — the drag & drop reorder was cut before it existed, so it never shipped and cannot be removed; if that line is written, the entry is wrong
- [x] `#[test]` in `crates/pathmaster-core/tests/`, reading `../../../CHANGELOG.md` through `include_str!`: the newest heading that is **not** `[Unreleased]` carries exactly `CARGO_PKG_VERSION`. Pure text, no filesystem, in the crate that links no wxWidgets — `versioninfo.rs`'s reasoning verbatim (ADR-0007, ADR-0009), and its own doc comment should say so. Green through development (`Cargo.toml` 0.1.0, newest released `[0.1.0]`, `[Unreleased]` growing above it) and red at F2 the moment the version moves and the heading does not
- [x] The same test asserts every version heading has a matching link reference at the foot of the file — `[Unreleased]: …/compare/vX.Y.Z...HEAD` and `[X.Y.Z]: …/releases/tag/vX.Y.Z` — so the one maintenance point the format adds beyond the headings is not left ungated
- [x] `release.yml`: `--generate-notes` becomes `--notes-file`, the body extracted from `CHANGELOG.md` for the tag's version. The extractor and the test read **one** heading grammar — `^## \[<version>\]` up to the next `^## \[` or end of file — and the workflow **throws** on a missing or whitespace-only section, so an unwritten entry blocks the release instead of publishing an empty page. Today's `--generate-notes` produces nothing at all here (it lists pull requests, and this repository has none: v0.1.0's page is a bare `**Full Changelog**` link), so this is the first release body the project will have
- [x] `docs/release-checklist.md` step F2: two files become three — `Cargo.toml`, `crates/pathmaster/resources/app.rc`, and `CHANGELOG.md`, where `[Unreleased]` is renamed to the version being released with today's date and a fresh empty `[Unreleased]` opened above it, link references updated. The step's expected-result column names the new test beside `the_versioninfo_carries_the_crate_version`. F4's expected result gains one clause: the release page carries the version's section as its body
- [x] `docs/agents/issue-tracker.md` gains the standing rule that every **implementation** ticket carries a `- [ ] CHANGELOG.md's [Unreleased] gains its line` checkbox — this is what holds the discipline, and it is the whole of what holds it. Deliberately **not bought**: a push-CI gate requiring a `crates/` commit to touch `CHANGELOG.md`. It would be the only thing catching real per-commit drift, but it noises on refactors and test-only commits, and with no pull requests there is no label to opt out with — only a magic commit-message hole that would be used
- [x] No ADR: the decision is easy to reverse and surprises nobody. No Help-menu home, no `data\` copy, nothing embedded in the executable — the file is not the User Guide and must not become a second one

## Comments

**The back-fill is the delta-spec's product surface, and nothing else.** Nine `### Added` lines and
four `### Changed` ones, in the ticket's own order, each written from the section it names rather
than from the commit that landed it — the subjects in this repository's log are deliberately oblique
and would have produced a change list nobody could read. There is **no `### Removed`**: the drag &
drop reorder exercised its right to die before it existed, and a version can only remove what a
previous one shipped. `## [0.1.0]` is the one retroactive line the ticket asked for — "First
release." — which is the only claim about that tag that needs no evidence.

**The entries carry no repository-relative links, deliberately.** The same text is published as the
release page's body, where a relative path is only as good as whatever renders it there; ADR-0011 is
named in words, and the number is what finds the file. The preamble, which is never extracted, is
where the links live.

**The heading grammar is one rule read in two places, and the test's self-sensitivity is what proves
the reading.** `version_headings` matches a line beginning `## [` and takes the text to the `]`; the
workflow's extractor matches the same and adds only where a section *ends*. The Rust side is
exercised against a sample carrying the three things it must **not** match — a `###` category
heading, prose that begins `## `, and a `[0.2.0]:` link reference, which spells a version in
brackets exactly as a heading does. Both gates were then mutated to confirm they fail: moving
`Cargo.toml` to 0.2.0 with the heading left at `[0.1.0]` reddens
`the_newest_released_section_carries_the_crate_version`, and pointing `[Unreleased]`'s compare base
at a version that is not the newest reddens `every_version_heading_carries_its_link_reference`.

**Two assertions beyond the ticket's two.** The test asserts the **first** version heading is
`[Unreleased]`, which turns "the newest heading that is not `[Unreleased]`" from a search into a
reading and is the only thing holding F2's "a fresh empty `[Unreleased]` opened above it" — that
otherwise fails silently and leaves the next change nowhere to be written. And a heading that never
closes its `]` **panics** rather than being passed over, with a `#[should_panic]` guard on it: the
extractor ends a section at `## [` and never looks for the `]`, so a line this reader dropped
quietly would be a heading to the release and not to the gate. That is the divergence the shared
grammar exists to prevent, and skipping was how it got in. Deliberately **not** added: a
non-empty-section assertion. The section for the version being released does not exist
until F2 creates it, so during development the check could only look at `[0.1.0]`, whose body picks
up the foot's link references and is therefore never empty — a green that measures nothing. The
workflow's throw is where that guard belongs, and it is there.

**The extractor was exercised in `pwsh` in six cases**, since the workflow step itself cannot be run
without cutting a release: the real file with `VERSION=0.1.0` (extracts); `VERSION=0.2.0` with no
such section (throws, named); the file after a simulated F2 rename (extracts 41 lines, stopping
cleanly at `## [0.1.0]` and dropping the link references); the same with the section blanked
(throws); a one-line file, which `Get-Content` hands back as a bare string — hence the `@()`, without
which indexing walks characters and the failure is a method-not-found rather than the message; and
the output's first bytes, confirming `Set-Content -Encoding utf8` writes no BOM under the job's
`pwsh` (it would under Windows PowerShell 5.1). It is one pass, with `## [` written once, so the
grammar is stated rather than repeated. What no local run can press is the release page itself —
that the body arrives with the section's markdown intact — and it is F4's, where the ticket put it.

**The one wart in the grammar, named rather than papered over.** A section runs to the next `## [`
or to the end of the file, so the **oldest** section's body carries the link-reference block at the
foot. It is unreachable — the version being released is always the newest, never the oldest — and
closing it would have cost a second rule for the extractor to disagree with the test about. The
workflow comment says so where a reader of that step will find it.

**This ticket adds no `[Unreleased]` line of its own.** The standing rule it writes into
`docs/agents/issue-tracker.md` begins with the next implementation ticket; the back-fill above is
fixed by the delta-spec's feature list, and a changelog is not one of that list's entries. Ticket 12,
which predates the rule and changes no product behaviour, is in the same position.

**Two corrections came out of the review, both real.** The Copy entry line claimed the rendering was
"flushed so it outlives the Run" — §8's words, and superseded: impl ticket 08 landed a plain Win32
write precisely because wx could not be quiet on that road, `crates/pathmaster/src/clipboard.rs`
says "**it needs no flush**", and there is no flush call in the shipped binary. The outcome is true
and the mechanism was not, so the mechanism is gone from the line. The second is the grammar
divergence above. Neither would have been caught by any gate in this ticket, which is the argument
for the review rather than against the gates.

**F2 and F4 are the only Checklist rows touched**, both in section F, which is where the ticket
placed the split with ticket 12 (A/B). F3's description of the workflow's step order was left as it
stands: it already reads as though the version gate follows the `fmt`/`test`/`clippy` re-run when in
the file it precedes it, and that is a pre-existing inaccuracy this ticket has no business fixing
under 12's nose.
