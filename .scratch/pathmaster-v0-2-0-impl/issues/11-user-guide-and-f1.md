# 11 — The User Guide and F1

**Spec:** [delta-spec §9](../../pathmaster-v0-2-0/spec.md)

**What to build:** Help → "&User Guide" (F1) opens the browser on a User Guide the executable carries: one page per Interface Language, embedded at build time, rewritten into `data\help.html` on every open. This ticket sits late in the chain on purpose — the guide's content contract (what v0.2.0 adds, the full keyboard table, the Command line subsection) is written once, when every feature it documents exists.

**Blocked by:** 03, 04, 05, 07, 08, 09, 10 (the features the guide documents).

**Status:** ready-for-agent

- [ ] Two purpose-written Markdown documents, `docs/help/en.md` and `docs/help/uk.md` — not the README; content per the contract: what PATH is; the window; editing; what each of the six Status words means; Backups and restore; what v0.2.0 adds; the full keyboard table (mirroring §12's map); Settings; the System PATH and administrator rights; what is written where; troubleshooting; a "Command line" subsection covering `--data-dir`, `--tab`, `--help`. Deliberately absent: installation, release verification, contributing, the licence; no screenshots, zero external requests
- [ ] Build: `pulldown-cmark` as a build-dependency converts `docs/help/<code>.md` → `OUT_DIR/help-<code>.html`, embedded via the same `include_bytes!` pattern as the `.mo` files
- [ ] The page sets no colours — `:root { color-scheme: light dark; }` plus layout only (`max-width`, `font-family: system-ui`, `line-height`); `<meta charset="utf-8">`, `lang="en"`/`"uk"`, `<title>` "PathMaster {version} — User Guide"
- [ ] Opening: `data\help.html` — one file, no language suffix — overwritten unconditionally on every open through the existing atomic `datadir::write_replace`, then `ShellExecuteW`; change language, restart, F1 → the file is rewritten in the new language, no orphan; delete the file, F1 → it returns
- [ ] Failure ladder, no Announcement on any rung: write fails → the version-pinned GitHub URL `…/blob/v{version}/docs/help/<code>.md` plus one `WARN` line ({version} from `CARGO_PKG_VERSION`; 404s until the tag exists — named, not a bug); no network → the browser's own offline page; a shell that opens nothing → silence plus a log line (the `open_backups_folder` precedent)
- [ ] Menu home: Help → "&User Guide" («Посібник користувача(&U)») carrying `\tF1`, first in the menu, About last; mnemonics U and A; no `…`, no separator; enabled in every state — Backups tab and Read-only Data included
- [ ] F1 in dialogs does nothing, as a decision: the dialog stays open, focus does not move, nothing is spoken
- [ ] Heading-parity `#[test]` in `pathmaster-core/tests` reading `../../docs/help/*.md`: both documents exist, are non-empty, and carry the same set of headings
- [ ] No settings field, no new Announcement, no ADR; the keyboard table stays hand-written (generating it from the menus' source is recorded as not bought)
