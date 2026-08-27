# 08 — Copy entry

**Spec:** [delta-spec §8, §13 (items 13, 14)](../../pathmaster-v0-2-0/spec.md)

**What to build:** Edit → Copy (Ctrl+C) puts the focused visible Entry's currently displayed rendering on the clipboard — raw in raw mode, expanded in expanded mode — and NVDA hears "Copied to clipboard", or the failure message when the clipboard write fails.

**Blocked by:** 04 (copy-what-is-shown needs Expansion Mode to exist).

**Status:** ready-for-agent

- [ ] Menu home Edit → Copy `\tCtrl+C`, joining the per-Entry group after Delete Entry; `session: None` disables it on the Backups tab exactly as Edit/Delete; msgid "Copy" («Копіювати») shipped
- [ ] Copies the focused visible Entry's currently displayed rendering with exact text fidelity — no quotes added, an Entry's own quotes are content; always exactly one Entry (single-select reaffirmed)
- [ ] Scoping is the platform's own: no focus-checking handler, no dynamic accelerator tables — with focus in the Search field or a dialog field, Ctrl+C copies that field's text, not the Entry (wxMSW text entries claim it before accelerator translation); the command is otherwise frame-wide
- [ ] Success speaks Announcement 13 "Copied to clipboard" — fixed text, no echo of the payload; failure speaks Announcement 14 "Could not copy to clipboard" immediately on a failed `set_text`, no retry; both pairs shipped in both languages
- [ ] No selection = silent no-op (the edit/delete precedent) — silence only ever means "nothing was selected"
- [ ] After a successful `set_text`, `flush()` — best-effort, its own result never announced; the copy demonstrably outlives the Run (close the app, paste still works)
- [ ] No Ctrl+Insert twin, no settings field, no new NVDA obligation
