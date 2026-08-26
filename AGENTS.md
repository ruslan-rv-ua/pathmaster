# PathMaster2

## Agent skills

### Issue tracker

Issues live as markdown files under `.scratch/<feature-slug>/` in this repo — there is no remote issue tracker. See `docs/agents/issue-tracker.md`.

### Triage labels

The five canonical roles, using the default label strings unchanged. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context — `CONTEXT.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.

## Commands

The `justfile` at the repo root is the canonical command list — `just` alone prints it.
`just ci` is the push-CI gate run locally, same flags and same order.

## Git workflow

This repo uses git-flow (classic preset, via git-flow-next). `main` is the release branch, `develop` is the integration branch, and work happens on `feature/`, `bugfix/`, `release/`, and `hotfix/` prefixed branches. Start work with `git flow feature start <name>` rather than branching off `main` directly.
