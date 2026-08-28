# 11 — The User Guide and F1

**Spec:** [delta-spec §9](../../pathmaster-v0-2-0/spec.md)

**What to build:** Help → "&User Guide" (F1) opens the browser on a User Guide the executable carries: one page per Interface Language, embedded at build time, rewritten into `data\help.html` on every open. This ticket sits late in the chain on purpose — the guide's content contract (what v0.2.0 adds, the full keyboard table, the Command line subsection) is written once, when every feature it documents exists.

**Blocked by:** 03, 04, 05, 07, 08, 09, 10 (the features the guide documents).

**Status:** done — driven live in both languages, both the file rung and the write-failure rung measured; F1 **as a keystroke** is unproven from the implementation harness and is the one reading left for the Release Checklist, see Comments

- [x] Two purpose-written Markdown documents, `docs/help/en.md` and `docs/help/uk.md` — not the README; content per the contract: what PATH is; the window; editing; what each of the six Status words means; Backups and restore; what v0.2.0 adds; the full keyboard table (mirroring §12's map); Settings; the System PATH and administrator rights; what is written where; troubleshooting; a "Command line" subsection covering `--data-dir`, `--tab`, `--help`. Deliberately absent: installation, release verification, contributing, the licence; no screenshots, zero external requests
- [x] Build: `pulldown-cmark` as a build-dependency converts `docs/help/<code>.md` → `OUT_DIR/help-<code>.html`, embedded via the same `include_bytes!` pattern as the `.mo` files
- [x] The page sets no colours — `:root { color-scheme: light dark; }` plus layout only (`max-width`, `font-family: system-ui`, `line-height`); `<meta charset="utf-8">`, `lang="en"`/`"uk"`, `<title>` "PathMaster {version} — User Guide"
- [x] Opening: `data\help.html` — one file, no language suffix — overwritten unconditionally on every open through the existing atomic `datadir::write_replace`, then `ShellExecuteW`; change language, restart, F1 → the file is rewritten in the new language, no orphan; delete the file, F1 → it returns
- [x] Failure ladder, no Announcement on any rung: write fails → the version-pinned GitHub URL `…/blob/v{version}/docs/help/<code>.md` plus one `WARN` line ({version} from `CARGO_PKG_VERSION`; 404s until the tag exists — named, not a bug); no network → the browser's own offline page; a shell that opens nothing → silence plus a log line (the `open_backups_folder` precedent)
- [x] Menu home: Help → "&User Guide" («Посібник користувача(&U)») carrying `\tF1`, first in the menu, About last; mnemonics U and A; no `…`, no separator; enabled in every state — Backups tab and Read-only Data included
- [x] F1 in dialogs does nothing, as a decision: the dialog stays open, focus does not move, nothing is spoken
- [x] Heading-parity `#[test]` in `pathmaster-core/tests` reading `../../docs/help/*.md`: both documents exist, are non-empty, and carry the same set of headings
- [x] No settings field, no new Announcement, no ADR; the keyboard table stays hand-written (generating it from the menus' source is recorded as not bought)

## Comments

**The two documents are the whole of the content decision, and they are code-true.** Every claim
either quotes the Catalogue or was checked against the module that decides it: the five Status words
against `diagnostics::diagnose_entry` (Empty exclusive; Relative and Missing never co-occurring;
Duplicate flagging the second and later appearances of one normalised path across *both* Scopes;
network roots never probed), the two length limits against `thresholds::classify` (`> 8,191` warns,
`>= 32,767` refuses — the second is "at", not "past"), the settings against the five dialog labels
and `settings.json`'s field names, and the keyboard table against `Command::accelerator` item by
item. §9's **"six Status words"** resolves as five: `Issue::SEVERITY` is five long and §4 fixes
Over-length as Scope-level, flagging no Entry and entering no column. Both documents say five and
give Over-length its own paragraph, naming it as the sixth *problem* — the code-true reading of a
phrase that counts §4's "six Issue types".

**The `<title>` is composed rather than written.** It is the document's own opening `# ` heading with
this build's version spliced in after the product name, so the Ukrainian page's title is Ukrainian
without a second string to keep in step; the shape is asserted in `build.rs`, so a document that
renamed its own title is a build failure rather than a page that announces itself as something else.
The version is read from the environment at build-script *run* time, not `env!`-ed, because that is
the moment it has to be right.

**The heading-parity gate runs over the languages that ship, not over two names.** It reads
`i18n/*.po` — the same enumeration `build.rs` builds the pages from and `catalogue.rs` gates the
msgids from — so a third language cannot build fine and go unchecked. Heading *text* is deliberately
not compared: two languages cannot share a heading's words, so parity is the same headings at the
same levels in the same order, which is the drift worth gating and all that can be. It also refuses
a document that opens below level one or skips a level, since the guide's whole navigation is a list
of levels, and the reading it is built on has its own test — two of that reading's rules (a `#`
inside a fence is not a section; `#word` is not a heading) are invisible in a passing run.

**Driven live against staged copies with private Data Directories, in both languages.** The Help
menu read `Посібник користувача(&U) \t F1` first and `Про програму(&A)` second, both enabled on the
Backups tab as well as on a Scope; the command wrote `data\help.html` with `lang` and `<title>`
matching the Run's language; the file deleted returned on the next press; `data\` held one
`help.html` and no orphan after a language change; and the second rung was measured by holding the
page open with no sharing, which logged
`WARN help: help.html could not be written (os error 5), opening the online copy` and opened the
online copy. The generated pages were grepped for `src=`/`href=`/`<script`/`<link`/`@import`/`url(`
— zero external requests, asserted rather than assumed.

**F1 as a keystroke could not be measured from this harness.** Synthetic keyboard input reached
nothing in the session: with the window confirmed foreground and a row landed by posted mouse
messages, a plain Down did not move the focused row of the **main window's own list**, so the finding
was the harness's and not the accelerator's — the standing rule is to confirm against a known-good
surface first and stop reporting keyboard findings when it fails. What *was* measured is that the
item carries `\tF1`, which is the only mechanism by which any accelerator in this application
exists (there is no `wxAcceleratorTable` in wxdragon at any level), and that the command behind it
does the right thing. **F1 belongs in the Release Checklist's User Guide steps (ticket 12) as an
ear-verified reading.**

**Three things outside the checklist, each with its reason.** The Interface Language moved into
`Run`: F1 picks its page by it, and re-deriving it from the held `SettingsFile` would answer for a
language the Run is not yet speaking — `Decisions` loses the field rather than carrying it twice
(ADR-0010). `ShellExecuteW` got one spelling, `platform::shell::open`, which `snapshots::open_folder`
now goes through as well; it answers *whether anything opened*, which the two callers use
differently on purpose — the Backups folder is silence either way, the guide's bottom rung is a log
line. And spec §17 gained the three modules this ticket lands, which ADR-0010 makes each ticket's own
obligation; the v0.2.0 series has otherwise let that lapse, and ticket 12 is where the rest of it
belongs.

**One correction made in passing:** six doc comments still said "the Announcement catalogue is closed
at seven", which §13 closed at fourteen. They are now correct. `catalogue.rs`'s line is left alone —
it narrates what ADR-0003 *declared*, which is history rather than a stale count.
