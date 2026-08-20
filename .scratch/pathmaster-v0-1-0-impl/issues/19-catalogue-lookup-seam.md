# 19 — The Catalogue lookup seam, and Announcements as a type

**Spec:** [spec §11 (FR-i18n), §10 + §10.1 (the closed seven), §12 (StatusBar), §7 (Status column), §18 (test tiers)](../../pathmaster-v0-1-0/spec.md) · ADR-0003, ADR-0004, ADR-0007, [ADR-0009](../../../docs/adr/0009-catalogue-lookup-is-injected.md)

**What to build:** The Catalogue stops being a wx call that every composed string has to reach past. A lookup interface `pathmaster-core` owns, a `Catalogue` holding it, every composing function moved down beside the msgids it fills, and §10.1's seven Announcements as an enum the one voice takes — so ADR-0003's closed set is closed by the compiler instead of by memory, and the composition rules become testable without linking wxWidgets.

**Blocked by:** 06 (the Catalogue mechanism), 12 (the StatusBar and Status column texts this moves) — both resolved.

**Blocks:** 13, so that its Announcements 2–3 and the five taxonomy texts land in a tested crate rather than being retrofitted.

**Take before 14, though nothing enforces it.** The frontier rule is "open, unblocked, first by number wins", and it has no way to express priority — with 13 blocked, it hands out 14 next. But 14's Backups list composes its own rows (a date and Scope read off the file name, plus Corrupted), so taken first it adds to exactly the pile this ticket exists to clear. 16 is safe either way: its dialog strings are plain labels.

**Status:** ready-for-agent

- [ ] A lookup interface in `pathmaster-core` with two adapters: the binary's, calling the free `catalog::translate` / `translate_plural` and never holding a `Translations` (`set_global` transfers ownership to wx); and the tests', answering with the msgid and picking `n == 1 ? singular : plural` — wxdragon's own documented no-catalogue fallback
- [ ] A `Catalogue` in core holding the injected lookup, with composition as its methods rather than free functions each taking an adapter — `CONTEXT.md`'s "there is exactly one". Never a global: a global is the trap being left
- [ ] `Announcement` is a data-carrying enum — **six variants for §10.1's seven**, because item 5 is item 4's ", unsaved changes" suffix and `UndoOutcome::crossed_apply` already models it. Announcements 2 and 3 are defined here and wired by ticket 13
- [ ] No platform type appears in a variant: `ReadOnlyReason` and ticket 13's typed Apply failure contribute a **msgid**, which `catalogue_msgid()` already returns
- [ ] `Announcer::announce` takes an `Announcement`, not a `&str`, so nothing outside the catalogue can be spoken. It loses `Copy`, which no closure ever used — every one holds `Rc<App>`
- [ ] Everything that composes moves: the six builders at the tail of `ui/mod.rs`, `status_text` in `scope_page.rs`, `rejection_text` in `entry_dialog.rs`. Bare widget lookups stay on `catalog::translate` — a label has no rule to test
- [ ] Core tests cover what composition can get wrong, all through the identity adapter and linking no wx: `{operation}` filled, the suffix appended only across the Apply barrier, the zero msgid chosen over a plural form, the Status column's severity-ordered join, StatusBar field 1's conditional threshold warning, and field 0's Read-only substitution
- [ ] One test asserts the catalogue is §10.1's seven and nothing else
- [ ] The wx smoke test runs **through** the wx adapter rather than past it, and asserts one composed Announcement in real Ukrainian — the undo line with its suffix. It remains the only test that links wxWidgets
- [ ] No new dependency: `gettext-ng`, `gettext-rs` and a hand-rolled plural evaluator were all considered and rejected ([ADR-0009](../../../docs/adr/0009-catalogue-lookup-is-injected.md))
- [ ] No Catalogue text changes and no `.po` changes; the completeness gate passes unchanged. This ticket moves code, not strings
- [ ] Deferred, not built here: `Command::menu_label` (which appends the accelerator, so it *is* composition) and `Command::enabled` (pure logic over a core type, in the untested crate) both stay in the binary — moving them means moving `Command`, which is separate work

## Comments

Designed 2026-08-20, before any code, out of the same architecture review that produced ticket 13's
amendment and [ADR-0008](../../../docs/adr/0008-apply-sequence-lives-in-platform.md). This was the
review's second candidate and it is sequenced ahead of 13 deliberately: ticket 13 adds Announcements 2
and 3 plus the five taxonomy texts, and without the seam every one of them lands in the crate ADR-0007
leaves untested, to be moved later by hand.

**The finding was not that the composition functions are badly placed.** It is that they *cannot* be
placed anywhere else. `translate()` is a wx call, so anything that composes a user-facing string is
pinned to the wx-linking crate, however pure its logic. Core already owns the msgids and `fill()`; only
the lookup is on the wrong side. That is a seam with two adapters waiting for it, not an abstraction
invented for testing.

**The rejected alternative is recorded in [ADR-0009](../../../docs/adr/0009-catalogue-lookup-is-injected.md)**
and is worth naming here too, because it will be proposed again: give core *real* translations in its
tests instead of the identity. `polib` is already a dev-dependency and could read the shipped `.po`
files — but it stores the plural expression as a string and never evaluates it, and the pure-Rust crates
that do are either incomplete (`gettext-ng` 0.4.1, self-declared) or an FFI binding NFR-portable forbids
(`gettext-rs`). The decisive objection is not the crates: at runtime the plural form is chosen by **wx**,
so a test choosing it with any other implementation asserts behaviour the product does not have. What
composition can actually get wrong does not depend on the language at all.

**One consequence worth expecting while reading the diff.** `ui/mod.rs` loses roughly a hundred lines
from its tail and gains nothing, and `pathmaster-core` gains a module of about the same size — but the
core version arrives with tests, and the enum arrives with the property `announce(&str)` could never
have: there is no longer a string in the program that can be announced from outside the catalogue.
