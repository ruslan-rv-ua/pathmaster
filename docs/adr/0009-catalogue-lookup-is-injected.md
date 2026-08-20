# The Catalogue lookup is injected, and the Announcement catalogue is a type

[ADR-0003](0003-no-accessibility-calls-except-announce.md) made `announce()` the application's one voice
and declared the Announcement catalogue closed at seven: adding a message is a contract change, and
over-announcing is a defect equal to silence. Nothing enforced any of that. `announce()` takes a `&str`,
so every string in the program is announceable; and because `translate()` is a wx call, every function
that *composes* an Announcement's text has to live in the binary crate — the one
[ADR-0007](0007-crate-boundary-is-the-test-boundary.md) leaves with no automated tests. So the closed
set was closed by memory, and the composition rules behind it — a placeholder filled, a suffix appended,
the zero msgid chosen over a plural form — were verified by reading. Both are now structural: the lookup
becomes an interface `pathmaster-core` owns, the composition moves down beside the msgids it fills, and
the seven Announcements become a type.

**The seam exists so that rules can be tested, and only where there are rules.** A widget label is a bare
lookup with nothing to get wrong, and those stay exactly where they are, calling the binary's
`catalog::translate` directly. What moves is what *composes*: the Announcements, both StatusBar fields
(one of which joins two halves and one of which conditionally appends a threshold warning), the Status
column's severity-ordered join, and the validation rejection text. This also means there is still exactly
one lookup in the program, as `CONTEXT.md`'s **Catalogue** requires — the injected adapter *is*
`catalog::translate`, wrapping the single lookup rather than adding a second.

**The test adapter answers with the msgid, and that is not a compromise.** The tempting alternative was
to give core real translations: read the shipped catalogues in the test and assert real Ukrainian. It was
rejected, and the crate landscape is the weaker half of the reason. `polib` — already a dev-dependency of
core for the `.po` gate — stores the plural expression as a string and never evaluates it; `gettext-ng`
(0.4.1, August 2026) is pure Rust but describes itself as not yet at feature parity and depends on
`encoding` 0.2, long superseded; `gettext-rs` is an FFI binding to the system library, which NFR-portable
forbids outright.

The decisive reason is not crate quality. **At runtime the plural form is selected by wx**, so a test
that selects it with a different implementation asserts a behaviour the product does not have: it would
pass while wx did something else. And what composition can actually get wrong is language-independent —
whether `{operation}` was filled, whether the ", unsaved changes" suffix was appended, whether zero
Entries took their own msgid rather than a plural form. Real translation stays covered where it belongs:
the `.po` gate for the strings themselves, and one composed sentence end-to-end in the wx smoke test.
Incidentally, the identity adapter is not an invention either — `n == 1 ? singular : plural` is precisely
wxdragon's own documented fallback when no catalogue answers.

**The type is what closes the set.** An enum whose variants carry their own data is a value that can be
built at the moment the thing happens, handed to the one thing that speaks, and counted by a test. With
`Announcer` taking an `Announcement` rather than a `&str`, there is no longer a string in the program
that *can* be announced from outside the catalogue — which is what ADR-0003 asked for and could not have.

## Consequences

- **Six variants stand for seven Announcements.** §10.1's item 5 is item 4's text with the
  ", unsaved changes" suffix, not a message of its own, and `UndoOutcome::crossed_apply` already models
  it exactly. Writing it as a seventh variant would create a second route to one sentence; the count is
  recorded here so the discrepancy is not later "fixed".
- **A platform type may not appear in an Announcement.** `ReadOnlyReason` lives in
  `pathmaster-platform`, and so will impl ticket 13's typed Apply failure; core cannot name either
  without reversing the dependency direction. Both contribute a **msgid** instead, which is what
  `ReadOnlyReason::catalogue_msgid()` already returns.
- **All seven exist before two of them are wired.** Announcements 2 and 3 arrive with impl ticket 13, and
  their variants are defined now — otherwise no test can say "the catalogue is the spec's seven", which
  is the whole reason for the type. It also leaves ticket 13 constructing a value rather than designing
  a variant.
- **`Announcer` is no longer `Copy`.** Its doc claimed the trait so "every closure that speaks can hold
  its own"; no closure ever did — every one holds `Rc<App>` and reaches the Announcer through it.
- **The wx smoke test runs through the adapter**, not past it, so the one line of production glue is
  covered, and it asserts one *composed* Announcement in real Ukrainian. It remains the only test that
  links wxWidgets.
- **Menu and button labels do not move.** `Command::menu_label` appends the accelerator, which ADR-0004
  makes load-bearing, so it is composition and belongs down here eventually — but `Command` lives in the
  binary and moving it is separate work. `Command::enabled`, pure logic over a core type sitting in the
  untested crate, is the same argument and the same deferral.
