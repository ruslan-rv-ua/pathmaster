# Issue tracker: Local Markdown

Issues and specs for this repo live as markdown files in `.scratch/`.
There is no remote issue tracker for this project.

## Conventions

- One feature per directory: `.scratch/<feature-slug>/`
- The spec is `.scratch/<feature-slug>/spec.md`
- Implementation issues are one file per ticket at `.scratch/<feature-slug>/issues/<NN>-<slug>.md`, numbered from `01` — never a single combined tickets file
- Triage state is recorded as a `Status:` line near the top of each issue file
- Comments and conversation history append to the bottom of the file under a `## Comments` heading
- Every **implementation** ticket carries a `- [ ] CHANGELOG.md's [Unreleased] gains its line` checkbox, and that line is written as part of the work — the root `CHANGELOG.md` is never reconstructed from `git log`, whose subjects here are deliberately oblique. Nothing else holds this: there is no CI gate requiring a `crates/` commit to touch the file (it would noise on refactors and test-only commits, and with no pull requests there is no label to opt out with), and Release Checklist step F2 only renames `[Unreleased]` to the version being released

## When a skill says "publish to the issue tracker"

Create a new file under `.scratch/<feature-slug>/` (creating the directory if needed).

## When a skill says "fetch the relevant ticket"

Read the file at the referenced path. The user will normally pass the path or the issue number directly.

## Wayfinding operations

Used by `/wayfinder`. The **map** is a file with one **child** file per ticket.

- **Map**: `.scratch/<effort>/map.md` — the Destination / Notes / Decisions-so-far / Not-yet-specified / Out-of-scope body.
- **Child ticket**: `.scratch/<effort>/issues/NN-<slug>.md`, numbered from `01`, with the question in the body. A `Type:` line records the ticket type (`research`/`prototype`/`grilling`/`task`); a `Status:` line records `open`/`claimed`/`resolved`.
- **Blocking**: a `Blocked by: NN, NN` line near the top. A ticket is unblocked when every file it lists is `resolved`.
- **Frontier**: scan `.scratch/<effort>/issues/` for files that are open, unblocked, and unclaimed; first by number wins.
- **Claim**: set `Status: claimed` and save before any work.
- **Resolve**: append the answer under an `## Answer` heading, set `Status: resolved`, then append a context pointer (gist + link) to the map's Decisions-so-far in `map.md`.

### Research findings

Research tickets write their findings to `.scratch/<effort>/research/<NN>-<slug>.md` on the current branch
(this repo has no parallel-branch workflow — parallel research agents write to distinct files instead).
