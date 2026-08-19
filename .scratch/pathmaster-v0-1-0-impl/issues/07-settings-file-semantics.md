# 07 — settings.json read path and failure taxonomy

**Spec:** [spec §13](../../pathmaster-v0-1-0/spec.md)

**What to build:** The application reads `settings.json` at startup and survives every way the file can be wrong: absent file = silent first run with defaults; unparsable file = set aside as `.bad` with one startup dialog; a bad field = in-memory default with a `WARN` line, raw value preserved. The Interface Language the run actually uses comes out of this path. (The Settings dialog UI is ticket 16; geometry persistence is ticket 15.)

**Blocked by:** 04 (Data Directory), 05 (WARN lines), 06 (the dialog title lives in the Catalogue; language resolution consumes the stored choice).

**Status:** ready-for-agent

- [ ] Core `settings` module parses `language` (`"auto"|"en"|"uk"`, default `auto`), `maxBackups` (int ≥ 1, default 50; 0 outlawed), and window geometry; unit tests for parse + per-field fallback
- [ ] Absent file = first run: defaults, no dialog, no log line; the file is created on first natural write, not at startup
- [ ] Parse layer, all-or-nothing: unparsable JSON or non-object root → rename to `settings.json.bad` (atomic, single copy, next incident overwrites; no rename in Read-only Data), full defaults, one startup dialog titled "Settings could not be read — defaults are in use", [OK]
- [ ] Field layer, per-field: an invalid known-field value falls back to its default in memory while the file keeps the raw value until the user changes that setting in the UI (choice-not-outcome; a v0.2 value survives a v0.1 run); no clamping; one `WARN` log line each, no dialog, no Announcement
- [ ] Unknown fields are ignored and preserved through every rewrite
- [ ] Settings are read in both data modes; written only in Writable Data
- [ ] Startup order holds: Data Directory → settings → translations (language resolved from the stored choice) → UI
