# 06 — Catalogue and i18n mechanism

**Spec:** [spec §11](../../pathmaster-v0-1-0/spec.md) · ADR-0004

**What to build:** The one Catalogue every later ticket puts its strings into: `wxTranslations` with embedded `.mo`, a single `translate()` shared by visible labels and Announcements, English and Ukrainian catalogues, and the automated completeness gate. Demoable: the shell from ticket 01 shows its tab labels in Ukrainian when forced to `uk`.

**Blocked by:** 01 — the bin crate and build.rs host the loader and the smoke test.

**Status:** resolved

- [x] `.po` files committed under the bin crate's `i18n/`; `.mo` generated at build time by `polib` (pure Rust, no `msgfmt`); `build.rs` enumerates `i18n/*.po`; `.mo` embedded via `include_bytes!` from `OUT_DIR` through a custom `TranslationsLoader`
- [x] `add_std_catalog()` is never called; one `translate()` used by labels and Announcements alike; a miss returns the msgid (msgids are English source text, and where two strings mean different things their English differs)
- [x] Placeholders are named braces (`{n}`, `{operation}`) with one explicit substitution helper — no printf-style codes
- [x] Interface Language resolution: system language Ukrainian → `uk`, everything else → `en` (English is the fallback, not the default); the stored choice domain is `auto`/`en`/`uk` (wiring to settings.json lands in ticket 07)
- [x] Accelerators belong to the code, never the Catalogue: the Catalogue holds `"&Undo"`, the code appends `"\tCtrl+Z"`; Ukrainian mnemonics keep the Latin letter in parentheses (`"Файл(&F)"`); languages listed by endonym
- [x] Ukrainian `.po` carries `Plural-Forms: nplurals=3`
- [x] Completeness gate as a plain `#[test]` over a registry of msgid constants in core: presence via `get_string(…).is_some()` (never `translate(s) != s`), plural presence, placeholder integrity, per-menu mnemonic uniqueness, self-sensitivity; fuzzy entries read as missing for free
- [x] One `get_string` smoke test in the binary (the wx half of the gate), running in CI
- [x] Never translated: registry paths, file names, the `WM_SETTINGCHANGE` payload, the entire log

## Comments

Implemented 2026-08-20, TDD at the crate boundary (ADR-0007): 14 tests in
`crates/pathmaster-core/tests/msgids.rs`, 9 in `tests/catalogue.rs` (the gate), 6 in
`tests/language.rs`, 3 in `crates/pathmaster-platform/tests/locale.rs`, and the one
wx-linking smoke test in the binary. `cargo test -p pathmaster-core` still finishes in under
two seconds and links no wxWidgets.

- **Split across crates per §17**: `core::msgids` holds the msgid constants, the `REGISTRY`
  of `CatalogueEntry` rows, and the pure checks the gate is built from (`placeholders`,
  `fill`, `mnemonic`, `duplicate_mnemonic`); `core::language` holds the stored choice
  (`auto|en|uk`), the two shipped languages with their endonyms, and the resolution branch;
  `pathmaster::catalog` holds the `TranslationsLoader`, `install()` and the one `translate()`;
  `i18n/uk.po` and `build.rs` sit with the binary that embeds them.
- **The gate is two halves that meet in the middle.** The `.po` half runs in core (no wx):
  presence, plural forms against the declared `nplurals`, placeholder integrity in both
  directions, per-menu mnemonic uniqueness, no tab character anywhere, nothing in a catalogue
  the registry does not name, and self-sensitivity. The wx half is the binary's single test:
  every registered msgid answered by `get_string(...).is_some()` out of the **embedded** `.mo`,
  plural selection at n = 1 / 2 / 5 giving Ukrainian's three different words, and a
  never-registered msgid answering `None`. Both were verified to bite: a sabotaged `uk.po`
  (broken placeholder, `#, fuzzy`, a stale entry, a tab accelerator, a deleted entry, a
  deleted plural form) failed the gate every time with the msgid named in the message.
- **Demoed live** on this `uk-UA` machine: the shell's accessibility tree — the layer NVDA
  reads — reports `PATH користувача`, `PATH системи`, `Резервні копії`, `Шлях`, `Стан`, and
  the startup line records `language: uk`. Forced to `LanguageChoice::English` the same probe
  reports the English msgids and `language: en`. There is no runtime forcing path yet: the
  choice is hardcoded `Auto` until ticket 07 reads it from `settings.json`, so on an English
  machine the demo needs that one-line edit.

### Three decisions a reviewer should see

- **No `en.po` ships.** "English and Ukrainian catalogues" is satisfied by the msgid registry
  plus `uk.po`: msgids **are** the English source text and English is the fallback, not a
  catalogue of its own (§11, D5). An identity `en.po` would be a second source of English
  needing a gate rule forbidding it from ever differing from the first. Nothing forbids one
  either — D10's "drop `xx.po` in" stays true for `en`, and the gate was rewritten to stop
  asserting its absence after review flagged that as invented policy.
- **The system language is read from Windows, not from wx.** §11 D3 specifies
  `Locale::get_system_language() == Ukrainian`; that comparison **can never be true** on this
  stack. wxdragon 0.9.18's `Language` enum mirrors wxWidgets **3.2** (it stops at
  `UserDefined = 234`, and `from_i32` answers `None` above that), while the vendored
  wxWidgets **3.3.3** renumbered `wxLanguage` to roughly nine hundred entries —
  `wxLANGUAGE_UKRAINIAN` sits near ordinal 859, and 3.3 also added
  `wxLANGUAGE_UKRAINIAN_UKRAINE`, which is what a `uk-UA` machine actually matches.
  **Measured**: on this `uk-UA` machine `Locale::get_system_language()` answers `Unknown`,
  inside `wxdragon::main` and outside it alike. Ticket 11's fact table ("`Language::Ukrainian
  = 217`, a single variant") was read out of *wxdragon's* `language.rs`, not out of wxWidgets
  3.3.3's `language.h`; the two disagree. D3's decision is untouched (Ukrainian → `uk`,
  everything else → `en`) — only its source moved, to `platform::locale::system_language()`
  over `GetUserDefaultUILanguage`, which is also the more faithful home under §17 ("platform
  — no wx") and is testable, unlike a wx call in the bin. Tested against an independent
  oracle (.NET's UI culture), so the test is honest on a Ukrainian developer machine and an
  English CI runner alike. The same enum drift makes `set_language(...)` unusable;
  `set_language_str("uk")` is a string and is unaffected. Both facts are recorded at their
  call sites.
- **Fuzzy is not free with `polib`.** `mo_file::write` writes every message it is given,
  fuzzy and untranslated included — and an untranslated entry would answer with an *empty
  string* where a miss should have fallen back to English. `build.rs` therefore performs the
  two exclusions `msgfmt` performs. That leaves two filters that could drift (D7 claimed
  drift was structurally impossible), so the gate closes it rather than trusting it: every
  registered msgid must be usable **and** a catalogue may hold nothing the registry does not
  name, between them leaving no `.po` entry the two filters could read differently.

### Scope, stated

- **Strings seeded**: the five the shell shows (three tab labels, two column headers) and
  Announcement 1 (spec §10.1) — the plural pair per Scope plus its own zero-case msgid. The
  plural entries were pulled forward from ticket 08 deliberately: `nplurals=3` and the gate's
  plural checks have nothing to measure without them, and the wx smoke test is where three
  Ukrainian forms are proven to come back. Ticket 08 wires them to `announce()`.
- **Menu labels stay with the tickets that build their menus** (11, 13, 16, 17). The registry
  carries the per-menu grouping and the mnemonic checks are pinned against fixtures in
  `msgids.rs`, so the first `menu_item` entry is gated on arrival — but the gate's menu walk
  is over an empty set today, and the test says so. That is also the honest reading of the
  accelerator criterion: the *rule* is enforced now (a tab character in the Catalogue fails
  the gate, `Файл(&F)` parses to `F`, endonyms live outside the Catalogue), while the code
  that appends `"\tCtrl+Z"` arrives with the menu it appends to.
- **The log is outside the Catalogue structurally, not by discipline**: every log line is
  built in `pathmaster-core`, which cannot reach `pathmaster::catalog` without reversing the
  dependency direction.
- **Accepted silence**: `add_catalog` returning `false` is not reported. For English that is
  the designed path (no `en.mo`, none wanted); for Ukrainian the gate and the smoke test make
  it unreachable, and reporting it would mean growing `logfmt`'s deliberately closed
  constructor set (ticket 05, §14).

Two-axis review (Standards / Spec) run before commit. Fixes applied: `Entry` renamed to
`CatalogueEntry` (it collided with the domain Entry in the same crate) and `CATALOGUE` to
`REGISTRY`; the duplicated brace scan behind `placeholders`/`fill` shared as one `next_braces`;
the repeated catalogue walk in the gate shared as `each_message`; the invented "no `en.po`"
assertion dropped; the "adding a language edits one branch" claim corrected to name every edit
it actually needs; the `GetUserPreferredUILanguages` equivalence claim narrowed to what is
true. Noted, not changed: the gate reads `../pathmaster/i18n` across a crate boundary (a
test-time path the §17 split requires, not a dependency edge) and repeats `build.rs`'s `.po`
glob (sharing it would mean giving the pure core a filesystem). Rustfmt churn from
`cargo fmt --all` in ticket 02's core tests was reverted again to keep the commit scoped.
