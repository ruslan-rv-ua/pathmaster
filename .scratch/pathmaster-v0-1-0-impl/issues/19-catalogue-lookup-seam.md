# 19 — The Catalogue lookup seam, and Announcements as a type

**Spec:** [spec §11 (FR-i18n), §10 + §10.1 (the closed seven), §12 (StatusBar), §7 (Status column), §18 (test tiers)](../../pathmaster-v0-1-0/spec.md) · ADR-0003, ADR-0004, ADR-0007, [ADR-0009](../../../docs/adr/0009-catalogue-lookup-is-injected.md)

**What to build:** The Catalogue stops being a wx call that every composed string has to reach past. A lookup interface `pathmaster-core` owns, a `Catalogue` holding it, every composing function moved down beside the msgids it fills, and §10.1's seven Announcements as an enum the one voice takes — so ADR-0003's closed set is closed by the compiler instead of by memory, and the composition rules become testable without linking wxWidgets.

**Blocked by:** 06 (the Catalogue mechanism), 12 (the StatusBar and Status column texts this moves) — both resolved.

**Blocks:** 13, so that its Announcements 2–3 and the five taxonomy texts land in a tested crate rather than being retrofitted.

**Take before 14, though nothing enforces it.** The frontier rule is "open, unblocked, first by number wins", and it has no way to express priority — with 13 blocked, it hands out 14 next. But 14's Backups list composes its own rows (a date and Scope read off the file name, plus Corrupted), so taken first it adds to exactly the pile this ticket exists to clear. 16 is safe either way: its dialog strings are plain labels.

**Status:** resolved

- [x] A lookup interface in `pathmaster-core` with two adapters: the binary's, calling the free `catalog::translate` / `translate_plural` and never holding a `Translations` (`set_global` transfers ownership to wx); and the tests', answering with the msgid and picking `n == 1 ? singular : plural` — wxdragon's own documented no-catalogue fallback
- [x] A `Catalogue` in core holding the injected lookup, with composition as its methods rather than free functions each taking an adapter — `CONTEXT.md`'s "there is exactly one". Never a global: a global is the trap being left
- [x] `Announcement` is a data-carrying enum — **six variants for §10.1's seven**, because item 5 is item 4's ", unsaved changes" suffix and `UndoOutcome::crossed_apply` already models it. Announcements 2 and 3 are defined here and wired by ticket 13
- [x] No platform type appears in a variant: `ReadOnlyReason` and ticket 13's typed Apply failure contribute a **msgid**, which `catalogue_msgid()` already returns
- [x] `Announcer::announce` takes an `Announcement`, not a `&str`, so nothing outside the catalogue can be spoken. It loses `Copy`, which no closure ever used — every one holds `Rc<App>`
- [x] Everything that composes moves: the six builders at the tail of `ui/mod.rs`, `status_text` in `scope_page.rs`, `rejection_text` in `entry_dialog.rs`. Bare widget lookups stay on `catalog::translate` — a label has no rule to test
- [x] Core tests cover what composition can get wrong, all through the identity adapter and linking no wx: `{operation}` filled, the suffix appended only across the Apply barrier, the zero msgid chosen over a plural form, the Status column's severity-ordered join, StatusBar field 1's conditional threshold warning, and field 0's Read-only substitution
- [x] One test asserts the catalogue is §10.1's seven and nothing else
- [x] The wx smoke test runs **through** the wx adapter rather than past it, and asserts one composed Announcement in real Ukrainian — the undo line with its suffix. It remains the only test that links wxWidgets
- [x] No new dependency: `gettext-ng`, `gettext-rs` and a hand-rolled plural evaluator were all considered and rejected ([ADR-0009](../../../docs/adr/0009-catalogue-lookup-is-injected.md))
- [x] No Catalogue text changes and no `.po` changes; the completeness gate passes unchanged. This ticket moves code, not strings
- [x] Spec §17's `pathmaster-core` module list gains this ticket's Catalogue module
- [x] Deferred, not built here: `Command::menu_label` (which appends the accelerator, so it *is* composition) and `Command::enabled` (pure logic over a core type, in the untested crate) both stay in the binary — moving them means moving `Command`, which is separate work

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

---

Implemented 2026-08-21 on `feature/catalogue-lookup-seam`. `pathmaster-core::catalogue` now owns the
`Lookup` interface, the `Catalogue` that holds one, the `Announcement` type and every composition
rule that used to be pinned to the wx-linking crate. The three UI files are 97 lines lighter —
`ui/mod.rs` alone loses 89 — and core gains a 318-line module with 22 tests behind it, none of which
link wxWidgets.

**The property the enum was built for is real now, with one edge named.** `Announcer::announce`
takes an `Announcement`: every announcement site hands over a value of a closed type, and the
sentence is composed inside the one voice, so no *composed text* can reach the Banner from outside
the Catalogue. The closed-set test is a `match` rather than a count — an eighth Announcement fails
to compile before it fails an assertion, which is the only gate memory was ever going to lose to.
What the compiler does not yet check is the msgid three variants carry: `announce(ApplyFailed {
msgid: "anything" })` compiles, and a lookup miss returns that string verbatim. Every msgid in play
is registered and gated, and closing the gap properly means a msgid type, which is nobody's ticket
yet — ADR-0009's headline is true of the message, not yet of the string.

**Announcement 2 carries a msgid, exactly as 3 does, and that is the one shape the ticket left
open.** §10.1 item 2 is two per-Scope strings ("User PATH applied" / "System PATH applied") and
neither is registered yet; a `Applied { scope }` variant could not compose without adding them, and
this ticket adds no Catalogue text. So both Apply Announcements carry the msgid their caller chose,
which is also what keeps ticket 13's typed failure — a `pathmaster-platform` type core cannot name —
out of the enum. **Read it as a deferral rather than as the shape ADR-0009 specified**: the ADR's
msgid rule covers the two announcements whose cause is a platform type, and item 2's cause is a
`Scope`, which is core's own. When ticket 13 registers the two strings it may narrow the variant to
`Applied { scope }`; what is fixed here is that both exist, that they are Announcements, and that
neither can smuggle a platform type into core.

**Three smaller choices, each with a reason a reviewer would otherwise have to reconstruct:**

- **The identity adapter lives in the test file**, not in `src`. The ticket calls it one of the
  interface's two adapters and it is — but `tests/diagnostics.rs` already keeps `Fs` and `Env` this
  way, and a production crate that ships a lookup answering with msgids would be shipping a second
  Catalogue nobody asked for. The binary's adapter, `catalog::Installed`, is production and is where
  the ticket puts it.
- **The tests are `tests/composition.rs`**, because `tests/catalogue.rs` is the `.po` completeness
  gate. The two are about different things — the strings themselves, and what is built out of them
  — and the file header says so in both directions.
- **`UndoStep` rather than a `bool`.** `undo_text(redo, outcome)` took a bare `true` at its call
  site, which this codebase already refuses elsewhere ("the direction is the command, so it travels
  as the command"). The direction now travels as a named value, and `undo_redo` matches on the same
  value it announces with.

**Verified live, in Ukrainian, on this machine's real PATH.** Launched the debug build, drove it
cross-process, and read the results back: the Status column composes «Дублікат»/«Відсутній»,
StatusBar field 0 reads «PATH користувача: 42 записи (20 проблем) | PATH системи: 19 записів (9
проблем)» — User first, both counts, and the Ukrainian plural forms selected by wx — and field 1
«Об'єднаний PATH: 2229 символів» with no threshold warning, correctly. Add with a `;` in the text
raised «Запис містить заборонений символ: ;» as the dialog's title, so `{character}` reaches the one
place NVDA will read it; adding `C:\Windows` and then invoking Undo put «Скасовано: Додавання запису»
in the Banner, with no suffix because no Apply had happened. The window closed cleanly, exit code 0.

**What the wx smoke test now asserts** is one composed Announcement end-to-end: the undo line with
its across-the-barrier suffix, «Скасовано: Видалення запису, незбережені зміни» — a translated
template, a translated operation name filled into it, and a translated suffix appended, all through
`Installed`. That is the assertion core's identity adapter cannot make, and it is the one line of
production glue that no pure test can cover. It remains the only test that links wxWidgets.

### The review

Both axes ran against `develop`. **Standards** found one documented breach and three judgement
calls; **Spec** confirmed the pure-move check clean — all eight moved functions are logic-identical
to their pre-move versions, and the only shape change is `general_status`, which now reaches
Announcement 7's text through `announcement()` instead of duplicating it.

Four findings applied:

- **`UndoStep` was on CONTEXT.md's `_Avoid_` list.** The **Checkpoint** entry keeps "Undo step" off
  the vocabulary, and the type's own doc made exactly the conflation the list guards against. It is
  `UndoDirection { Undo, Redo }` now, named for the walk rather than for the Checkpoint it lands on
  — which is also more honest: the direction is Undo, the sentence it earns is "Undone".
- **Repeated Switches in `undo_redo`.** Two cascades over one distinction, ten lines apart: one
  chose the direction, the other chose `undo()` or `redo()`. They are one `match command` now, so
  the history cannot be walked one way and announced the other.
- **`general_status`'s order was load-bearing but unenforced.** The pre-move signature named `user`
  and `system`; an array does not, and `[system, user]` compiled and read backwards — which is not
  hypothetical, because a *pass* evaluates System first and a caller reaching for that order would
  have reversed the sentence. §12's "User first" is the Catalogue's rule now, applied to whatever
  order arrives, with a test that hands it the pass's order and gets the tabs' order back. Each
  `ScopeCounts` still carries its own Scope, so ordering can never mispair a count with a name.
- **Two undocumented test helpers**, now documented like their siblings.

**One finding declined.** The Standards axis read `catalog::Installed` as a Mysterious Name against
neighbours like `Announcer` and `ScopePage`. Its actual neighbour is in the same file: `Embedded`,
the loader that serves the catalogues out of the executable's bytes. `Embedded` and `Installed` are
a deliberate pair — where the catalogues come from, and how the installed one is asked — and the
alternatives read worse at the one call site that matters (`Catalogue::new(catalog::Installed)`).

**Two non-findings the Standards axis raised and answered itself**, worth keeping because they will
be raised again: `Applied`/`ApplyFailed` are constructed nowhere in production (Speculative
Generality) and `Installed` is pure delegation (Middle Man). ADR-0009 asks for both on purpose —
all seven Announcements exist before two are wired, so a test can say "the catalogue is the spec's
seven", and the adapter delegates because wrapping the single lookup is what keeps it single.
