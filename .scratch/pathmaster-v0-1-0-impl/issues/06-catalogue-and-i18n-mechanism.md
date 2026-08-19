# 06 — Catalogue and i18n mechanism

**Spec:** [spec §11](../../pathmaster-v0-1-0/spec.md) · ADR-0004

**What to build:** The one Catalogue every later ticket puts its strings into: `wxTranslations` with embedded `.mo`, a single `translate()` shared by visible labels and Announcements, English and Ukrainian catalogues, and the automated completeness gate. Demoable: the shell from ticket 01 shows its tab labels in Ukrainian when forced to `uk`.

**Blocked by:** 01 — the bin crate and build.rs host the loader and the smoke test.

**Status:** ready-for-agent

- [ ] `.po` files committed under the bin crate's `i18n/`; `.mo` generated at build time by `polib` (pure Rust, no `msgfmt`); `build.rs` enumerates `i18n/*.po`; `.mo` embedded via `include_bytes!` from `OUT_DIR` through a custom `TranslationsLoader`
- [ ] `add_std_catalog()` is never called; one `translate()` used by labels and Announcements alike; a miss returns the msgid (msgids are English source text, and where two strings mean different things their English differs)
- [ ] Placeholders are named braces (`{n}`, `{operation}`) with one explicit substitution helper — no printf-style codes
- [ ] Interface Language resolution: system language Ukrainian → `uk`, everything else → `en` (English is the fallback, not the default); the stored choice domain is `auto`/`en`/`uk` (wiring to settings.json lands in ticket 07)
- [ ] Accelerators belong to the code, never the Catalogue: the Catalogue holds `"&Undo"`, the code appends `"\tCtrl+Z"`; Ukrainian mnemonics keep the Latin letter in parentheses (`"Файл(&F)"`); languages listed by endonym
- [ ] Ukrainian `.po` carries `Plural-Forms: nplurals=3`
- [ ] Completeness gate as a plain `#[test]` over a registry of msgid constants in core: presence via `get_string(…).is_some()` (never `translate(s) != s`), plural presence, placeholder integrity, per-menu mnemonic uniqueness, self-sensitivity; fuzzy entries read as missing for free
- [ ] One `get_string` smoke test in the binary (the wx half of the gate), running in CI
- [ ] Never translated: registry paths, file names, the `WM_SETTINGCHANGE` payload, the entire log
