# README and user-facing docs

Type: grilling
Status: resolved
Blocked by: —

## Question

What does the README (and any other user-facing document) promise, warn about, and explain?

Graduated out of the map's **Not yet specified** on 2026-08-19, once the packaging ticket it waited
on resolved. The resolved tickets have already accumulated a list of things the README *owes* the
user — this ticket decides structure, tone and completeness, and closes the list:

- The honest description of what winget/scoop themselves write to the machine: winget's ARP key
  under `HKCU`, the Links directory on the user PATH, the exe rename via `Commands`; scoop's shim
  and `persist: data` junction (ticket 15).
- The one named exception to "nothing outside the app's directory": ComDlg32 MRU writes from the
  Browse folder picker (tickets 07/10).
- SmartScreen on an unsigned exe: what the user will see and why it is expected (charting
  constraint 10).
- The installed-NVDA requirement for the elevated instance — portable NVDA is deaf to it
  (ticket 12).
- The ticket-18 anomaly's user-facing workaround: if NVDA goes silent on the list, restart NVDA.
- `winget uninstall` deletes `data\`, backups included; `winget upgrade` keeps it (ticket 07).

To decide: single README or split docs; language (English only, or a Ukrainian section given the
Interface Language work); whether the Release Checklist's existence is user-visible documentation
or internal; and what, if anything, of the spec's cut/deferred list is worth telling users.

## Comments

**2026-08-19, from ticket 20 (failure taxonomy):** one more item the README owes the user —
`settings.json` is hand-editable, and when it cannot be parsed the app sets it aside as
`settings.json.bad` (single copy) and starts on defaults, telling the user via a startup dialog.
Document the `.bad` file: what it is, that the previous content is recoverable from it, and that
bad *values* of individual fields are tolerated per-field (raw value kept in the file) rather than
resetting the whole file.

## Answer

Resolved 2026-08-19 with the user, per the standing directive: internet research on README best
practices first, informed options second. Research consulted: Standard Readme / community README
guides (structure, non-technical opening, copy-paste install commands), the GitHub community
practice for multilingual READMEs (canonical English + `README.<lang>.md` translations,
cross-linked at the top, prose-only translation), and how unsigned-exe projects communicate
SmartScreen (explanation + bypass steps + published-hash verification as one unit).

**Shape: one `README.md`, mirrored in full by `README.uk.md`.** No split docs — warnings scattered
across files go unread, and a screen-reader user navigates one file's headings faster than a file
tree. English is canonical; the Ukrainian translation is complete (the app has a Ukrainian
Interface Language and Ukrainian screen-reader users are a target audience, so they get the whole
document, not a half). Cross-language links sit at the top of both files; code blocks and commands
stay untranslated. Drift guard: the Release Checklist (ticket 19) gains one non-NVDA step —
"`README.uk.md` is in sync with `README.md`, or the release did not change the README."

**Section skeleton** (both language versions mirror it):

1. **Title + description** — 1–3 non-technical sentences; **no badges** (noisy images read first
   by a screen reader). One screenshot of the main window with a full alt text, added at release
   time (the spec only reserves its place).
2. **Accessibility** — the headline section: NVDA is the tested screen reader; JAWS/Narrator are
   not deliberately broken but not tested; the elevated instance requires *installed* NVDA
   (portable NVDA is deaf to it — ticket 12); the ticket-18 workaround: if NVDA goes silent on
   the list, restart NVDA.
3. **Install** — winget, scoop, direct download, copy-paste commands. Direct download carries the
   SmartScreen block: the warning is expected on an unsigned exe and why, "More info → Run
   anyway", and verification via PowerShell `Get-FileHash` against the released `.sha256` sidecar
   (chosen over `certutil` for readability). Signing is deferred by decision, stated plainly.
4. **Keyboard reference** — a short table (~10 rows) mirroring the bindings fixed by tickets
   10/17 (F2/Enter/double-click edit, Delete without confirm — undo is the safety net, Ctrl+Z/Y,
   Apply, tab order, NVDA+End for the status bar). For a blind user this is the difference
   between reading one screen and exploring by touch.
5. **Portability: what gets written where** — `data\` beside the exe; the one exception: ComDlg32
   MRU writes from the Browse folder picker (tickets 07/10); what the package managers themselves
   write: winget's ARP key under `HKCU`, the Links directory on the user PATH, the exe rename via
   `Commands`; scoop's shim and `persist: data` junction; `winget uninstall` deletes `data\`,
   backups included, `winget upgrade` keeps it.
6. **Settings** — `settings.json` is hand-editable; unparsable file is set aside as
   `settings.json.bad` (single copy, previous content recoverable) and the app starts on
   defaults; bad values of known fields fall back per-field while the file keeps the raw value.
7. **What PathMaster deliberately does not do** — by-design cuts with one-line reasons:
   similar-path/typo detection (false-positive generator), a theme setting (system colours
   always), probing network paths (a dead UNC blocks 20–60 s and cannot be cancelled). **No
   v0.2.0 promise list** — deferred features live in the issue tracker, not the README.
8. **How releases are verified** — the Release Checklist is user-visible trust documentation:
   link to `docs/release-checklist.md` and note that every release attaches a filled copy naming
   the NVDA version used. For an unsigned exe this recorded manual NVDA pass is the honest trust
   signal no badge replaces.
9. **License** — **MIT** (user decision; also closes ticket 15's open `License` field in the
   winget manifest — `CompanyName`/`PackageIdentifier` unaffected).

This closes the list the resolved tickets accumulated: every item named in the Question and the
ticket-20 comment has a home in the skeleton above.

## Amended 2026-08-25, by the user, during impl ticket 18

The section skeleton above shipped once and was then **redirected**, on the ground that the Answer
had got the positioning wrong: PathMaster is built on **universal design**, and its users are
sighted as well as blind. A headline Accessibility section with two screen-reader subsections
under it reads as an application *for* blind users rather than one that works for everyone — and
front-loading it pushes what the thing actually does below the fold.

**What changed.**

- **"Accessibility first" is one item in a Features list**, not section 2. It says universal
  design is a principle rather than a mode, names the three things that make it true (a keyboard
  route and a menu home for every action, every message shown as well as spoken, no colour ever
  set), and notes that NVDA is what it is tested with. One bullet, where it used to be a screen.
- **The keyboard table lists only PathMaster's own bindings** — F2, Del, Alt+arrows, Ctrl+Z/Y,
  Ctrl+S, F5. Tab, Shift+Tab, Ctrl+Tab, the arrows, Enter/Space and the Alt mnemonics came out:
  they are Windows, everybody knows them, and printing them made the six that matter hard to find.
  `NVDA+End` came out too — it is the screen reader's shortcut, not ours.
- **Nothing required was dropped.** The ticket-18 deaf-list ladder (spec §19 requires it in the
  README) and the installed-NVDA requirement for the elevated window (ticket 12) are now two short
  paragraphs in a **Troubleshooting** section, where somebody who has hit the problem will look —
  rather than in front of somebody who has not.
- **The structure follows Standard Readme and current practice**, checked against them rather than
  invented: a short description under 120 characters on its own line, matching the package
  managers' (which is why `ShortDescription` in the winget manifest and `description` in the scoop
  one lost "screen-reader-friendly" — discoverability for the people who search on that word lives
  in `Tags`, which is what winget actually searches); the screenshot immediately under it; a table
  of contents, since both files run past 100 lines; then Features, Install, Usage, the rest, and
  **Contributing** and **License** last. Contributing is new and was missing.

**Unchanged:** one `README.md` mirrored in full by `README.uk.md`, English canonical, no badges,
code blocks untranslated, and E1 of the Release Checklist as the drift guard. Both files are now
1,388 and 1,306 words and mirror each other section for section.
