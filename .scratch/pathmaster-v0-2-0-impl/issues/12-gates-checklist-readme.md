# 12 — Gates, Release Checklist and README: integrate and verify

**Spec:** [delta-spec §12 (mnemonics), §13–§14 (catalogue audit), §15 (inventory), §17, §18](../../pathmaster-v0-2-0/spec.md)

**What to build:** The closing slice: fold the v0.2.0 Checklist delta into the living Release Checklist, re-run the gates the menu growth voided, and bring the README up to date — so a tagged v0.2.0 build can walk the Checklist with nothing missing. No new product behaviour; this ticket verifies and documents what tickets 01–11 built.

**Blocked by:** 02, 03, 04, 05, 06, 07, 08, 09, 10, 11 — everything.

**Status:** done

- [x] The Checklist delta is folded into `docs/release-checklist.md` and numbered there: steps 2–4 gain the leading position number in the row reading, step 15 gains the Search field in the Tab cycle, step 31 is rewritten for the two-item Help menu, and the new step groups land — Search, Filter, Expansion Mode, Tree View, Fix Issues, Copy, User Guide (eight steps), Command line (seven steps) — each with its expected speech per §13
- [x] The Copy group gains a **failure-path row** that §17's group does not have: hold the clipboard open from another process and press Ctrl+C — Announcement 14 "Could not copy to clipboard" and **nothing else**, no dialog of any kind. Ticket 08 left `wxdragon::Clipboard` for a plain Win32 write precisely because wx raised its own untranslated «Pathmaster Error» box on that road (deviation accepted 2026-08-28); with no such row, a regression back to it passes every release gate. §8's last two bullets are superseded by that ticket — read them that way in the audit
- [x] The i18n per-menu mnemonic-uniqueness gate re-runs over all of v0.2.0's menu growth at once, in both languages, and passes — the proposed English View set (S, F, I, E, T) confirmed or amended, the Ukrainian set fixed under the same gate; steps 31 and B12 re-run once
- [x] Catalogue audit against the spec: the Announcement set closes at fourteen with both languages exactly as §13 tabulates; every §14 non-Announcement string exists; the i18n completeness gate (registry, plural presence, placeholder integrity, mnemonic uniqueness) is green over the whole growth
- [x] README: the keyboard table gains §12's additions (Ctrl+F, Ctrl+I, Ctrl+E, Ctrl+T, Ctrl+C, F1, Space in Fix Issues, Down/Tab/ESC from the Search field); the `data\` inventory grows `help.html`; the portability section's `--data-dir` documentation (landed in ticket 10) reads consistently with the User Guide
- [x] `TC-file-structure`'s `data\` inventory in the test suite covers `help.html`
- [x] The wxdragon pin is confirmed still 0.9.18 and the full test gate is green on CI
- [x] A spot pass over the release mechanics: §18’s "nothing to decide" **no longer holds in full** — ticket 13 makes F2 three files and edits the workflow. Confirm the rest of §18 stands (no workflow edit needed for `pulldown-cmark`, scoop’s Excavator needs no manifest edit, winget stays deferred) and leave F2 and `release.yml` to 13

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

**The rest of the ticket landed 2026-08-28, on the same branch.** What follows is the audit's
result rather than a description of the code, because that is what this ticket produces.

**The Checklist delta is in `docs/release-checklist.md`, numbered there.** Steps 2–4 read
"{#}; Path: {path}; Status: …" (§2.1 as amended by §19's round three — NVDA prefixes every column
but the leftmost with its header, so promoting `#` into column 0 is what put "Path:" in front of
the path); step 2 also carries §2.1's other half, that the `#` never renumbers under a narrowing,
because a filtered list is the only place that rule can be seen to break. Step 15's Tab cycle is
spelled out — tabs → Search field → list → buttons. Step 31 is the two-item Help menu. The eight
new groups are steps **34–86**, one `####` heading each, continuing section A's own numbering
rather than opening a section: they are the same unelevated NVDA pass, and a filled copy that
counts to 86 is easier to check than one with two numbering schemes.

**The Copy failure-path row is step 71**, and it says why it exists: no dialog of any kind, in
either language, because `wxdragon::Clipboard` raised its own untranslated «Pathmaster Error» box
on that path and ticket 08 left it for a plain Win32 write. §8's last two bullets are read as
superseded by that ticket, per this ticket's instruction.

**One amendment to §17's own wording, made deliberately.** §17's User Guide group says the
unwritable-`data\` run "sends the browser to the online URL and leaves one `WARN` line". It cannot
leave one there: that run is Read-only Data, and a Read-only Data run has **no log at all** — L6
already says so, and `open_user_guide` composes the record only for `Run::log` to drop it. Step 77
keeps the online-URL rung in the step-17 run and names a second staging — `data\` writable,
`help.html` not — as where the line is actually read. Without that split the step would be
unpassable as written, which is worse than the step not existing.

**Mnemonics, re-run over the whole growth in both languages.** The English View set is
**confirmed unamended**: S, F, I, E, T. Ukrainian carries the same five letters in the
parenthesised form the rest of its menus use — «Пошук(&S)», «Фільтр(&F)», «Перемкнути фільтр
проблем(&I)», «Розгорнуті значення(&E)», «Дерево PATH(&T)…». Elsewhere the growth took the
letters that were free: Edit → Copy is **p**, not the C every other Windows Edit menu gives it,
because C is already Cancel Changes' here; Edit → Fix Issues… is **I**, free in that menu because
the View menu's Ctrl+I item is another menu's; Help → User Guide is **U**, beside About's A.

The gate that proves this had two holes, both closed rather than noted. The Ukrainian walk
`continue`d past any label a catalogue happened to lack, so uniqueness could be "proved" over four
of five siblings and read as green; it now counts what it checked against what it should have.
And the groups were derived from the registry alone, so a group key typed one way in `msgids.rs`
and another on the entry would split one menu into two internally-unique groups with nobody the
wiser — `every_declared_menu_group_is_one_the_registry_fills` now holds the declared set equal to
the derived one, and refuses a group of fewer than two, which is the only size a typo can hide in.
`the_view_menu_carries_the_letters_the_spec_proposed` is where the confirmed set is written down:
uniqueness alone would pass just as happily on five other letters, so it cannot be the record of
*which* five were chosen.

**Catalogue audit.** The Announcement set closes at fourteen and is already gated —
`the_announcement_catalogue_is_the_specs_items_and_nothing_else` walks thirteen variants for items
1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13, 14, item 5 being item 4's text plus the suffix, and the
`match` on the enum is what stops a fifteenth being added quietly. Both languages carry every one:
`every_msgid_is_present_and_usable_in_every_language` reads the `.po` sources the way `msgfmt`
does, so an untranslated or fuzzy string reads as missing rather than as present. Every §14
non-Announcement string exists; the list is now written down in
`every_string_the_v0_2_0_delta_adds_is_in_the_registry` as a cross-reference to the prose, since
prose is where a string goes missing without anything failing. The deletion's Action cell is
deliberately **absent** from that list: it reuses Announcement 4's "Delete entry" (ADR-0004), and
a second msgid for it would be the defect.

**`TC-file-structure`.** There was no inventory test at all — each writer gated its own file and
none could see a fourth appear beside it. `a_data_directory_holds_the_file_structure_and_nothing_else`
now drives every write the application makes into `data\` (the set-aside, `settings.json`,
`backups\`, a log past its 1 MB rotation so both generations land, and the guide) and reads the
directory back: `settings.json`, `settings.json.bad`, `backups`, `pathmaster.log`,
`pathmaster.log.old`, `help.html`, and **nothing else**. E2's Process Monitor session was the only
thing standing where that test now stands, and it runs on a released build.

**The pin is still wxdragon 0.9.18**, `Cargo.toml` at `"0.9.17"` and `Cargo.lock` resolving to
0.9.18 — 0.9.20's `get_item_text` fix touches nothing here, because the application renders from
the Working Copy and never reads text back out of a list. `just ci` — the push-CI gate run
locally, same flags and same order — is green over all of it: `cargo fmt --check`, 42 test binaries
with no failure, and `cargo clippy` under `-D warnings`. CI itself answers on push.

**§18, spot pass.** F2 and `release.yml` were ticket 13's and are done — F2 already names three
files, and the workflow already extracts the release body from `CHANGELOG.md` beside the
three-way version gate. The rest stands: the release workflow runs a plain
`cargo build --release --locked`, which compiles `pulldown-cmark` in the same run with no step to
add; scoop's manifest is `checkver: github` with a `$version`/`$url.sha256` autoupdate block, so
the Excavator bumps `version`, `url` and `hash` and needs no manual edit for v0.2.0; and the
winget block (F7–F8) is intact and still deferred.
