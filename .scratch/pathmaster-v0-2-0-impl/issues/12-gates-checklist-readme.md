# 12 — Gates, Release Checklist and README: integrate and verify

**Spec:** [delta-spec §12 (mnemonics), §13–§14 (catalogue audit), §15 (inventory), §17, §18](../../pathmaster-v0-2-0/spec.md)

**What to build:** The closing slice: fold the v0.2.0 Checklist delta into the living Release Checklist, re-run the gates the menu growth voided, and bring the README up to date — so a tagged v0.2.0 build can walk the Checklist with nothing missing. No new product behaviour; this ticket verifies and documents what tickets 01–11 built.

**Blocked by:** 02, 03, 04, 05, 06, 07, 08, 09, 10, 11 — everything.

**Status:** ready-for-agent

- [ ] The Checklist delta is folded into `docs/release-checklist.md` and numbered there: steps 2–4 gain the leading position number in the row reading, step 15 gains the Search field in the Tab cycle, step 31 is rewritten for the two-item Help menu, and the new step groups land — Search, Filter, Expansion Mode, Tree View, Fix Issues, Copy, User Guide (eight steps), Command line (seven steps) — each with its expected speech per §13
- [ ] The Copy group gains a **failure-path row** that §17's group does not have: hold the clipboard open from another process and press Ctrl+C — Announcement 14 "Could not copy to clipboard" and **nothing else**, no dialog of any kind. Ticket 08 left `wxdragon::Clipboard` for a plain Win32 write precisely because wx raised its own untranslated «Pathmaster Error» box on that road (deviation accepted 2026-08-28); with no such row, a regression back to it passes every release gate. §8's last two bullets are superseded by that ticket — read them that way in the audit
- [ ] The i18n per-menu mnemonic-uniqueness gate re-runs over all of v0.2.0's menu growth at once, in both languages, and passes — the proposed English View set (S, F, I, E, T) confirmed or amended, the Ukrainian set fixed under the same gate; steps 31 and B12 re-run once
- [ ] Catalogue audit against the spec: the Announcement set closes at fourteen with both languages exactly as §13 tabulates; every §14 non-Announcement string exists; the i18n completeness gate (registry, plural presence, placeholder integrity, mnemonic uniqueness) is green over the whole growth
- [x] README: the keyboard table gains §12's additions (Ctrl+F, Ctrl+I, Ctrl+E, Ctrl+T, Ctrl+C, F1, Space in Fix Issues, Down/Tab/ESC from the Search field); the `data\` inventory grows `help.html`; the portability section's `--data-dir` documentation (landed in ticket 10) reads consistently with the User Guide
- [ ] `TC-file-structure`'s `data\` inventory in the test suite covers `help.html`
- [ ] The wxdragon pin is confirmed still 0.9.18 and the full test gate is green on CI
- [ ] A spot pass over the release mechanics: §18’s "nothing to decide" **no longer holds in full** — ticket 13 makes F2 three files and edits the workflow. Confirm the rest of §18 stands (no workflow edit needed for `pulldown-cmark`, scoop’s Excavator needs no manifest edit, winget stays deferred) and leave F2 and `release.yml` to 13

## Comments

**The README slice landed early, on this branch, at the user's request (2026-08-28)** — out of the
ticket's own order, so that it is recorded here rather than discovered as a surprise. All three
halves of that box are done in **both** languages: the keyboard table takes §12's additions (F1,
Ctrl+C, Ctrl+F, ↓/Tab, Esc, Ctrl+I, Ctrl+E, Ctrl+T, Space) with the row wording lifted verbatim
from the User Guide's own table, so the two documents say the same words and there is one text to
keep in step rather than two; the `data\` inventory grows `help.html`; and the `--data-dir`
paragraph was compared against the guide's "Command line" subsection line by line and needed no
change. `--tab` is still undocumented in the README, as it was in v0.1.0 — that section is about
where the application writes, and it is not a v0.2.0 regression.

**Two things went in beyond that box, both because the release would otherwise ship a README that
is wrong rather than merely short.** Features gained two bullets — narrowing (Search, Filter, Tree,
Ctrl+E, Ctrl+C) and Fix Issues — because a front door that omits the release's headline features
misleads the person it exists for; and the Settings paragraph said the file "holds the interface
language, how many saved copies to keep per PATH, and where the window was last left. Tools →
Settings… changes the first two", which three new fields and a five-control dialog had made false.

**The screenshots and their alt text are deliberately untouched.** The alt text still describes the
picture that is actually in `docs/images/`, which is the v0.1.0 window — it is accurate about its
image and stale only about the product. F1 regenerates both and rewrites the alt text together;
doing the alt text first would leave it describing a picture that is not there. Expect it to need
the `#` column and the search field when F1 runs.

**Still open on this ticket:** the Checklist delta (§17's new step groups, steps 2–4, 15 and 31),
the Copy failure-path row, the mnemonic-uniqueness gate over the whole menu growth in both
languages, the Catalogue audit, `TC-file-structure`'s `data\` inventory, and the wxdragon pin
check.
