# Changelog

Every notable change to PathMaster, newest first, in the form
[Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/) describes. The project follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

This file is **developer-facing, and English only** — the document written for users is the User
Guide (`docs/help/en.md` and `docs/help/uk.md`, which the executable carries and F1 opens), and the
repository's bilingual rule covers that and the README. An entry here is written when the change
is, not reconstructed from `git log` at release time, and the section for a version is what the
release page publishes as its body.

## [Unreleased]

## [0.2.0] - 2026-08-28

### Added

- A permanent `#` column on both Scope lists, carrying an Entry's position in the Working Copy and
  never renumbering — so an Entry's place stays readable exactly where reorder is disabled.
- **Search**: a permanent field above each Scope's list, `Ctrl+F`, matching the rendering the list
  is currently showing — case-insensitive, slash-folded, never trimmed — with the match count
  spoken once the typing pauses.
- **Filter**: an exclusive per-Scope choice among All, With issues, and each of the five Issue
  types, in a View → Filter submenu of radio items; `Ctrl+I` toggles the coarse axis.
- **Expansion Mode**, `Ctrl+E`: an app-wide toggle between an Entry's raw text and its expanded
  reading, governing the list, what Search matches and what Copy puts on the clipboard. Per-Run,
  starting raw; never an edit, and invisible to Undo.
- **Tree View**, `Ctrl+T`: a modal prefix tree over the active Scope's Filtered View, snapshotted
  at open; Enter on a leaf selects that Entry in the main list, by identity rather than by text.
- **Fix Issues** (Edit → Fix Issues…): one checkable row per repairable Entry with one computed
  action — delete for Missing, Duplicate and Empty, remove the quotes for Quoted — applied as a
  single Checkpoint one Ctrl+Z takes back.
- **Copy entry**, `Ctrl+C`: the focused Entry's currently displayed rendering onto the clipboard,
  where it outlives the Run.
- **The User Guide** (Help → User Guide, `F1`): one page per Interface Language carried in the
  executable, written into the Data Directory as `help.html` on every open and handed to the
  default browser; the source is `docs/help/<code>.md`, gated for heading parity.
- `--data-dir <path>`, substituting only the locate step of the startup tree: an unusable target
  lands the Run in Read-only Data through a fourth reason rather than falling back to the default
  `data\`. With it the application's whole argument posture — `--help` and `-?` show the usage and
  exit, an unrecognised argument gets a dialog and then a normal start, and a self-relaunch
  forwards its parsed state rather than the verbatim command-line tail.

### Changed

- The menu bar becomes File / Edit / View / Tools / Help. **View** is new, and holds every command
  that changes what a list shows.
- `settings.json` gains three fields, each with a Settings-dialog control: `speakFilteredCount`,
  `filteredCountDelayMs` (0–5000, default 250) and `searchEscapeReturnsFocus`.
- The StatusBar's first field names the narrowing while one is active — "{n} of {m} entries", and
  the Filter's own name when it is not All. The count in parentheses is unmoved: it counts that
  Scope's Issues, not the view's.
- The UI's borrow discipline is structural. Every cell reached by more than one kind of call goes
  behind a scoped `with`/`with_mut` whose guard cannot escape; every modal passes through one door
  that makes the Timer's tick inert while it is open; and a source-scan test fails the build if
  `show_modal` appears anywhere else (ADR-0011).

## [0.1.0] - 2026-08-25

First release.

[Unreleased]: https://github.com/ruslan-rv-ua/pathmaster/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/ruslan-rv-ua/pathmaster/releases/tag/v0.2.0
[0.1.0]: https://github.com/ruslan-rv-ua/pathmaster/releases/tag/v0.1.0
