# F1 and Help → Documentation

Type: grilling
Status: resolved (2026-08-26)
Blocked by: —

## Question

The first v0.2.0 candidate, raised 2026-08-25: `F1` — Windows' own help key — does nothing, and the
standing rule says every shortcut has a menu home, so the fix is a Help → Documentation item. Small
item, one real question:

- **The offline question.** The README lives online, so a shell-open of its URL does nothing on a
  machine with no network — a real caveat for an exe carried on a stick. Options to weigh: online
  URL with a named failure story; shipping a document beside the exe (ends "one portable file");
  embedding help text in the exe (a dialog or generated local file — the Catalogue question of a
  *long* text, and does it get translated?); or F1 opening the existing About/shortcuts surface
  instead. Decide the shape and its failure story.
- Menu home wording and both-language Catalogue entries; whether the existing Help menu's one-item
  structure (About) grows to two or more.
- What F1 does in *dialogs* (nothing? same target?) — say it, don't leave it to chance.
- The menu-structure steps of the Release Checklist (31, B12, the mnemonic gate) are voided by any
  menu change — note for the assembly ticket, which re-runs them once, for all of v0.2.0's menu
  growth together.

## Resolution (2026-08-26)

Researched first: [research/12-f1-help-best-practices.md](../research/12-f1-help-best-practices.md),
per the map's standing directive 7. Decisions:

1. **The answer is a User Guide the executable carries** (now in `CONTEXT.md`): one page per
   Interface Language, embedded in the exe, written into the Data Directory when opened, handed to
   the browser. The browser *is* the help viewer — that is what NVDA itself does (Help → User Guide
   opens local HTML in the default browser), and it is the only route that buys browse mode:
   heading navigation, a headings list, find. It keeps "One executable" (nothing new ships beside
   the exe) **and** works with no network, which is the whole point for an exe carried on a stick.
   Ruled out **by name**, so they are not re-proposed: **CHM** (HTML Help Workshop is discontinued,
   and Mark-of-the-Web or a network share renders it "Navigation to the webpage was canceled" — the
   one format whose documented failure mode is this application's home ground); **eWriter/MSHC**
   (the reader is a download); **`wxHtmlWindow`** (generic, not native — NVDA cannot read it);
   **`%TEMP%`** (breaks `CONTEXT.md`'s "Nothing the application writes lives anywhere else", which
   the README states publicly); **shell-opening a `.md`** (Windows registers no handler out of the
   box — the user gets the "How do you want to open this file?" picker); and **F1 → About**
   (F1 means documentation everywhere in Windows, never identity).
2. **Source: two purpose-written Markdown documents**, `docs/help/en.md` and `docs/help/uk.md` —
   written for the person **using** the application, in the plain vocabulary of a Windows user, with
   no installation, contribution, licence or release-verification matter. **Not** the README: that
   is the front door for someone *choosing* the application, and it carries badges (four remote
   images — a local help page that phones `img.shields.io` on every open contradicts the offline
   document it is supposed to be), install instructions and repo-relative links. The cost of two
   documents is drift, and decision 10 buys the gates for it.
3. **What it must cover**, settled as a contract (the prose is written during implementation, not
   here): what `PATH` is and what PathMaster does; the window (tabs, list and columns, StatusBar,
   Banner); editing (add/edit/delete/reorder, Working Copy, Apply, Undo/Redo, Cancel); **what each
   of the six words in the Status column means**; Backups and restore; what v0.2.0 adds (Search,
   Filter, Tree View, Fix Issues, Copy, Expansion Mode); **the full keyboard table**; Settings; the
   System PATH and administrator rights; what is written where; and what to do when something is
   wrong. Deliberately absent: installation (scoop, direct download, SmartScreen, hash
   verification — the reader is already running it), release verification, contributing, and the
   licence, which About carries. **No screenshot** — the document is pure text, so it is
   self-contained by construction and makes zero external requests. One page, not a topic set: one
   list of headings, one search.
4. **`data\help.html` — one file, no language suffix, overwritten unconditionally on every open**
   through the existing atomic `datadir::write_replace`. A language suffix would leave an orphan
   file behind the first time the user changes language; "write only if missing" is **poisoned**,
   because scoop persists `data\` as a junction and a v0.2.0 binary would show v0.1.0's guide
   forever; a version stamp inside the file cures that at the price of reading and parsing on every
   open. Unconditional overwrite makes staleness **structurally impossible** rather than handled.
   Spec §3's and the README's by-name inventory of `data\` (`settings.json`, `backups\`,
   `pathmaster.log`) each grow by this file.
5. **The failure ladder, and no Announcement for any rung of it.** Write succeeds → `ShellExecuteW`
   on `data\help.html`. **Write fails** (Read-only Data, full disk, an exe on read-only media) →
   `ShellExecuteW` on the version-pinned URL `…/blob/v{version}/docs/help/<code>.md`, which GitHub
   renders, with `{version}` from the same `CARGO_PKG_VERSION` that fills About and that §16's
   three-way gate keeps honest; plus one `WARN` log line. No network either → the browser shows its
   own offline page, which is the terminal failure and is **visible**. Nothing is announced because
   nothing is silent: every rung opens the browser, focus moves to it, and NVDA names the window.
   Whether the document came from disk or from GitHub is invisible and uninteresting — it is the
   same document. The one genuinely silent case, a shell that opens nothing at all, takes the
   `open_backups_folder` precedent: silence plus a log line. In a development build the pinned URL
   is a 404 until the tag exists; the Release Checklist runs on a tagged build, so the step is
   sound, and the dev-time 404 is named in the spec rather than discovered as a bug.
6. **Menu home: Help → "&User Guide"**, uk «Посібник користувача(&U)», with `"\tF1"` appended by
   code — the menu home F1 has lacked, and the only mechanism there is (ADR-0004). Wording follows
   NVDA's own menu, which is the muscle memory of the intended user; "Documentation" is more
   technical than the register decision 3 sets, and "View Help" would stutter inside a menu already
   called «Довідка». **First in the menu, About last** (§15 already calls About the last item on the
   bar); the Help menu thereby grows from one item to **two**, mnemonics **U** and **A**, unique.
   **No `…`** — the house rule reserves it for items that open a dialog *asking* something.
   **No separator** — the application uses none anywhere. **Enabled in every state**: it is the
   sixth item that does not follow the active Session and the second, after About, that follows
   nothing at all — how to use the application is true in every state the application can be in, and
   Read-only Data already has its own rung on the ladder.
7. **F1 in dialogs does nothing, as a decision.** With no accelerator table, F1 is a menu-item label
   suffix and menu accelerators do not fire while a modal dialog owns input, so this is also the
   default — the question is only whether to pay for the opposite. The price is `EVT_CHAR_HOOK` in
   every dialog (nine today, two more in v0.2.0) as a **standing obligation** no gate would catch a
   future dialog breaking; what it buys is thin, because the guide is one page about the whole
   application with no per-dialog topics. The silence is correct: this application's rule against
   silence governs **commands the user asked for that failed**, and F1 in a dialog is an unbound key.
   Stated so the Release Checklist can assert it.
8. **The HTML wrapper sets no colours** — and "no stylesheet at all" would **not** satisfy that:
   browsers paint bare HTML black-on-white whatever the OS theme says, which would make the guide
   the one surface in this product that forces light on a user in dark mode or High Contrast. The
   HTML equivalent of §12's rule is `:root { color-scheme: light dark; }` — it declares no colour and
   hands the choice to the browser and the OS, and a page declaring no colours gets system colours
   under forced-colors automatically. With it, layout only: `max-width`, `font-family: system-ui`
   (Segoe UI, the font the application's own controls use), `line-height`. Forced, not chosen:
   `<meta charset="utf-8">`; `<html lang="en">` / `lang="uk"`, without which NVDA may read the
   Ukrainian guide in an English voice; and a `<title>` carrying the identity About carries —
   "PathMaster {version} — User Guide" — because the title is the first thing NVDA speaks when the
   page loads.
9. **Build: the `.mo` mechanism, mirrored.** `crates/pathmaster/build.rs` already compiles
   `i18n/*.po` into `OUT_DIR` and generates a table of `include_bytes!`; help does the same with
   `pulldown-cmark` (one new build-dependency beside `polib`), `docs/help/<code>.md` →
   `OUT_DIR/help-<code>.html`. The crate is `publish = false`, so reading `docs/` from `build.rs`
   costs nothing but a `rerun-if-changed`. `docs/help/` rather than a directory inside the crate
   because the same files are the fallback URL's target, and a person browsing the repository
   should find the guide where the other documents live.
10. **Drift is gated twice, and the structural cure is deliberately not bought.** Prose describing
    menu labels, keys and Issue names has nothing tying it to the Catalogue, so v0.3.0 renaming an
    item would leave the guide quietly lying. (a) A **Release Checklist step** catches drift against
    the product — the project's own idiom, since the UI has no automated test and the Checklist is
    what stands in its place. (b) A **language-parity `#[test]`**: both documents exist, are
    non-empty, and carry the **same set of headings** — this catches the drift that will actually
    happen, one language updated and the other forgotten, and it is the same mirror the Catalogue
    already has in its completeness gate. It lives in `pathmaster-core/tests/` reading
    `../../docs/help/*.md`, on `versioninfo.rs`'s precedent: a pure-text check belongs where the
    tests do not link wxWidgets. **Not bought**: generating the keyboard table into the guide from
    the menus' own source. It is the only thing that cannot drift, but accelerators live in code
    rather than in the registry, so it would mean introducing a (command, msgid, accelerator) table
    the product does not have. Recorded here so it is not rediscovered as an oversight.
11. **Nothing persists and nothing is configurable** — no `settings.json` field, as with 07, 08, 09
    and 11. **No new Announcement** — the catalogue stays at fourteen. **No ADR**: the one notable
    fact, that the product hands its own content to another application to render, is neither hard
    to reverse nor surprising once `wxHtmlWindow` is ruled out by name. This is §15/§16 delta-spec
    material.
12. **`CONTEXT.md` gains "User Guide"** — the artifact, not the menu item: one page per Interface
    Language, embedded and rewritten into the Data Directory on each open, never the README and
    never a topic set.

**Handed to the assembly ticket (15).** The Help menu grows to two items — "&User Guide" carrying
`F1` (uk «Посібник користувача(&U)»), then "&About" — so **Release Checklist steps 31 and B12 and
the mnemonic gate are voided by this change**, along with every other v0.2.0 menu change; 15 re-runs
them once, for all of the menu growth together. The Checklist **gains** eight steps: (1) `Alt+H`
speaks two items, the first carrying `F1`; (2) `F1` opens the browser, NVDA speaks
"PathMaster {version} — User Guide", `H` walks the headings; (3) `data\help.html` exists afterwards,
in the Interface Language; (4) change language, restart, `F1` — the file is **rewritten**, no orphan
appears; (5) delete the file, `F1` — it returns; (6) the unwritable-`data\` run (step 17's staging)
sends the browser to the **online** URL and leaves one `WARN` line; (7) `F1` in the Edit dialog says
nothing, the dialog stays open, focus does not move; (8) the item is available on the Backups tab
and in a Read-only Data run. Exact label strings and the final accelerator table → 15, as with every
other feature.
