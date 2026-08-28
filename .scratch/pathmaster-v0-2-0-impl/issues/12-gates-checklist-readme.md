# 12 — Gates, Release Checklist and README: integrate and verify

**Spec:** [delta-spec §12 (mnemonics), §13–§14 (catalogue audit), §15 (inventory), §17, §18](../../pathmaster-v0-2-0/spec.md)

**What to build:** The closing slice: fold the v0.2.0 Checklist delta into the living Release Checklist, re-run the gates the menu growth voided, and bring the README up to date — so a tagged v0.2.0 build can walk the Checklist with nothing missing. No new product behaviour; this ticket verifies and documents what tickets 01–11 built.

**Blocked by:** 02, 03, 04, 05, 06, 07, 08, 09, 10, 11 — everything.

**Status:** ready-for-agent

- [ ] The Checklist delta is folded into `docs/release-checklist.md` and numbered there: steps 2–4 gain the leading position number in the row reading, step 15 gains the Search field in the Tab cycle, step 31 is rewritten for the two-item Help menu, and the new step groups land — Search, Filter, Expansion Mode, Tree View, Fix Issues, Copy, User Guide (eight steps), Command line (seven steps) — each with its expected speech per §13
- [ ] The Copy group gains a **failure-path row** that §17's group does not have: hold the clipboard open from another process and press Ctrl+C — Announcement 14 "Could not copy to clipboard" and **nothing else**, no dialog of any kind. Ticket 08 left `wxdragon::Clipboard` for a plain Win32 write precisely because wx raised its own untranslated «Pathmaster Error» box on that road (deviation accepted 2026-08-28); with no such row, a regression back to it passes every release gate. §8's last two bullets are superseded by that ticket — read them that way in the audit
- [ ] The i18n per-menu mnemonic-uniqueness gate re-runs over all of v0.2.0's menu growth at once, in both languages, and passes — the proposed English View set (S, F, I, E, T) confirmed or amended, the Ukrainian set fixed under the same gate; steps 31 and B12 re-run once
- [ ] Catalogue audit against the spec: the Announcement set closes at fourteen with both languages exactly as §13 tabulates; every §14 non-Announcement string exists; the i18n completeness gate (registry, plural presence, placeholder integrity, mnemonic uniqueness) is green over the whole growth
- [ ] README: the keyboard table gains §12's additions (Ctrl+F, Ctrl+I, Ctrl+E, Ctrl+T, Ctrl+C, F1, Space in Fix Issues, Down/Tab/ESC from the Search field); the `data\` inventory grows `help.html`; the portability section's `--data-dir` documentation (landed in ticket 10) reads consistently with the User Guide
- [ ] `TC-file-structure`'s `data\` inventory in the test suite covers `help.html`
- [ ] The wxdragon pin is confirmed still 0.9.18 and the full test gate is green on CI
- [ ] A spot pass over the release mechanics: §18’s "nothing to decide" **no longer holds in full** — ticket 13 makes F2 three files and edits the workflow. Confirm the rest of §18 stands (no workflow edit needed for `pulldown-cmark`, scoop’s Excavator needs no manifest edit, winget stays deferred) and leave F2 and `release.yml` to 13
