# 07 — settings.json read path and failure taxonomy

**Spec:** [spec §13](../../pathmaster-v0-1-0/spec.md)

**What to build:** The application reads `settings.json` at startup and survives every way the file can be wrong: absent file = silent first run with defaults; unparsable file = set aside as `.bad` with one startup dialog; a bad field = in-memory default with a `WARN` line, raw value preserved. The Interface Language the run actually uses comes out of this path. (The Settings dialog UI is ticket 16; geometry persistence is ticket 15.)

**Blocked by:** 04 (Data Directory), 05 (WARN lines), 06 (the dialog title lives in the Catalogue; language resolution consumes the stored choice).

**Status:** resolved

- [x] Core `settings` module parses `language` (`"auto"|"en"|"uk"`, default `auto`), `maxBackups` (int ≥ 1, default 50; 0 outlawed), and window geometry; unit tests for parse + per-field fallback
- [x] Absent file = first run: defaults, no dialog, no log line; the file is created on first natural write, not at startup
- [x] Parse layer, all-or-nothing: unparsable JSON or non-object root → rename to `settings.json.bad` (atomic, single copy, next incident overwrites; no rename in Read-only Data), full defaults, one startup dialog titled "Settings could not be read — defaults are in use", [OK]
- [x] Field layer, per-field: an invalid known-field value falls back to its default in memory while the file keeps the raw value until the user changes that setting in the UI (choice-not-outcome; a v0.2 value survives a v0.1 run); no clamping; one `WARN` log line each, no dialog, no Announcement
- [x] Unknown fields are ignored and preserved through every rewrite
- [x] Settings are read in both data modes; written only in Writable Data
- [x] Startup order holds: Data Directory → settings → translations (language resolved from the stored choice) → UI

## Comments

Implemented 2026-08-20, TDD at the crate boundary (ADR-0007): 23 tests in
`crates/pathmaster-core/tests/settings.rs`, 13 in `crates/pathmaster-platform/tests/settings.rs`,
plus one logfmt test, two `datadir` tests for the accessors this ticket added, and the Catalogue
gate picking up the new dialog title. `cargo test -p pathmaster-core` still finishes in under two
seconds and links no wxWidgets.

- **Split across crates per §17.** `core::settings` owns both layers of the taxonomy and the
  document a rewrite amends; `platform::settings` owns the file — which of the three states it was
  found in, the `.bad` set-aside, and the atomic write; `main.rs` owns the order and the one
  dialog. `datadir` gained `rename_replacing` (the rename half of `write_replace`, which now calls
  it) and two accessors, `dir()` and `is_writable()`, so the settings reader asks the Data
  Directory state what it knows instead of re-deriving it.
- **The rewrite is an amendment, not a serialisation.** `SettingsFile` holds the typed values *and*
  the parsed document, and the setters are the only way either changes. That makes
  choice-not-outcome structural rather than disciplinary: a `language: "fr"` a v0.1 run cannot read
  survives every rewrite, and only `set_language` can replace it — there is no code path that
  serialises the settings over the file. Unknown fields ride through the same way, nested ones
  included (writing geometry amends the `window` record it finds rather than replacing it).
- **Demoed live** on this `uk-UA` machine, against a `data\` beside a copied binary. A stored
  `"language": "en"` produced `language: en` in the startup line — the choice beating the system,
  which is the whole point of the ticket. A `maxBackups: 0` alongside it produced exactly one
  `WARN settings: field "maxBackups" invalid (raw: "0"), using default 50` and left the file
  byte-identical, unknown field included. A hand-broken file produced the set-aside, the `WARN`
  line, and — in the accessibility tree, the layer NVDA reads — a dialog whose Window name,
  TitleBar and Text all read `Не вдалося прочитати налаштування — використовуються типові
  значення`, with a stock [OK]. An absent file left the log with its startup line alone and created
  no `settings.json` at all.
- Release exe **7.08 MB** against the 40 MB gate, so the new dependency costs nothing that matters.

### Four decisions a reviewer should see

- **The pure core takes its first dependency: `serde_json`, with `preserve_order`.** Both of this
  application's file kinds are JSON a *user hand-edits*, and the Snapshot schema (ticket 10) will
  put arbitrary Entry text — backslashes, quotes, `%VAR%` — through the same encoder. A hand-rolled
  parser would have to get `\u` escapes, surrogate pairs, number grammar and nesting depth right to
  earn nothing; this repo already reaches for a correct crate over a hand-rolled one (`polib` over
  an `msgfmt` pin). `preserve_order` costs indexmap/hashbrown/equivalent and buys a hand-written
  file keeping its author's key order through the rewrites that preserve its unknown fields —
  "ignored and preserved" reads poorly if every clean shutdown re-sorts the file alphabetically.
  Core stays pure: no I/O, no OS calls, still builds on any OS.
- **The set-aside answers bad *contents*, never a file this run could not open.** The first cut
  renamed on any read error, which the spec review caught: two instances are a designed state (§3,
  no single-instance lock), so a momentary sharing violation on a perfectly good `settings.json`
  would have moved it onto the single `.bad` copy — destroying exactly what the set-aside exists to
  preserve. The read is now `fs::read` + `String::from_utf8`, which separates "could not get at the
  file" (defaults, dialog, **no rename**) from "the bytes are bad" (defaults, dialog, set aside —
  non-UTF-8 counts, since that is the file's own content). Tested by holding the file open with
  `share_mode(0)`. Spec §13 carries the rule as a dated amendment.
- **An unreadable file earns one `WARN settings:` line, which §13 did not ask for.** The spec's
  parse-layer bullet enumerates the dialog and the defaults and no log line, while its field-layer
  bullet names one explicitly — so this is an addition, deliberately made and recorded rather than
  slipped in. The reason: without it a `settings.json` can be *moved on disk* while the log — the
  only artifact a developer ever gets from a machine they cannot see (§14) — shows nothing but
  `INFO startup`. The line names whether the file was set aside or left in place, carries file
  names rather than a location (PII prohibition #2 is about paths), and grows `logfmt`'s
  deliberately closed constructor set by exactly one. §13 is amended to name it.
- **Geometry is one field, not five.** `window` falls back as a unit under its own name, because
  half a position is not a place to put a window and the members have no individual defaults to
  fall back to (there is no number that means "centred"). The `WARN` line's raw value carries the
  whole record, so a developer still sees which member was wrong. A non-positive `width`/`height`
  is invalid like any other out-of-domain value and is not clamped — §12's clamping is about
  placing a *valid* geometry on the connected monitors, which stays ticket 15's job.

### Scope, stated

- **The write path exists and is tested; nothing calls it yet.** Ticket 15 (geometry on clean
  shutdown) and ticket 16 (the Settings dialog) are its callers, and both are blocked on this
  ticket for exactly that. `write` takes a directory rather than the `DataDirState` on purpose: the
  only way to get one is to match `DataDirState::Writable`, so "written only in Writable Data" is
  visible at the call site instead of trusted. Reading takes the whole state, because it behaves
  differently in each.
- **`main.rs` uses the stored language and drops the rest.** Nothing yet consumes `maxBackups`
  (rotation is ticket 10/13) or the geometry (ticket 15), so carrying the `SettingsFile` into the
  UI would be dead weight with no consumer; the ticket that needs it plumbs it.
- **A leading UTF-8 BOM is dropped rather than treated as unparsable.** Several Windows editors
  leave one, RFC 8259 §8.1 lets a parser ignore it, and setting a hand-edited file aside over an
  invisible byte would be the least explicable failure this application could produce. A UTF-16
  file — what PowerShell's `>` writes — is a different matter and *is* set aside.
- **The dialog is one msgid used as both title and body**, with the stock [OK] and a warning icon.
  NVDA speaks a `MessageDialog`'s title and buttons and never its body (§10 D6), so the title
  carries everything and the body repeats it for the eyes; [OK] is the one button in the
  application whose text carries no meaning we would have to own (§11). One consequence worth
  naming: an unreadable `settings.json` means the dialog announcing it speaks the *system*
  language, because the choice that would have overridden it was in the file we could not read.
- **`spec.md` was amended twice**: §13 gained the three rules above, and §17's platform module list
  gained `settings` — the same move impl ticket 06 made for `locale`.

Two-axis review (Standards / Spec) run before commit. Fixes applied: the rename-on-any-read-error
defect above; `Parsed::Object` renamed to `Parsed::Readable` so both arms name the outcome rather
than the JSON shape; the `DataDirState` walk moved out of `platform::settings` onto the state
itself as `dir()` and `is_writable()`, replacing a bare `(Option<&Path>, bool)` tuple; the double
dispatch over `Source` in `main.rs` collapsed into one match that yields the dialog flag; the three
public setters and `FILE_NAME` given the *why* docs the house style expects; and each default named
once as a constant (`DEFAULT_LANGUAGE`, `DEFAULT_MAX_BACKUPS`) so the value the run uses and the
value the log reports cannot drift. Noted, not changed: geometry's all-or-nothing unit, the BOM
strip, the `width`/`height` domain, `preserve_order` and the warning icon — each is a decision
above rather than an oversight.
