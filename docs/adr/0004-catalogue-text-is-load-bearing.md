# Catalogue text and markup are load-bearing, not prose

Two things in PathMaster's Catalogue look like sloppy writing and are not: some English strings are
phrased awkwardly to stay distinct from each other, and Ukrainian menu labels carry a Latin mnemonic in
parentheses — `Файл(&F)`, not `&Файл`. Both invite a well-meaning tidy-up that would silently break
something. The reasoning is recorded so the next reader does not perform it.

**The English text is an API surface, because there is no way to disambiguate.** gettext's `msgctxt`
exists precisely so two identical source strings can mean different things — and it is **not bound at any
level** in wxdragon (`GetTranslatedString(ptr, orig, domain, buf, len)` has no context parameter, and the
C++ shim adds none). So identical English collapses to a single translation, whether or not the two uses
mean the same thing. A real case already exists: **Cancel** the command discards changes back to the
Baseline (ADR-0001's Checkpoint model), while **Cancel** in `[Save] [Discard] [Cancel]` means "do not
close" — «Відхилити зміни» versus «Скасувати». The rule is therefore that where two strings mean different
things, **their English must differ**, even when uniform English would read better.

Symbolic keys (`dialog.close.cancel`) would remove the collision, and were rejected: on a miss
`translate()` returns the msgid itself, so a missing translation would make NVDA speak the key aloud. For
an application built for a screen-reader user first, degrading to readable English beats degrading to
punctuation.

**The mnemonic is in parentheses because the label string is also the keyboard binding.** `wxAcceleratorTable`
is absent at every level in wxdragon, so a menu item's label is the *only* place a shortcut can be
registered — wx parses both `&` and `\t` out of the string it is given. Two consequences follow. The
accelerator is split out and owned by the code (the Catalogue holds `"&Undo"`; the code appends
`"\tCtrl+Z"`), because a translated `"\tCtrl+Я"` would not misread — it would delete the shortcut, with
nothing to fall back to. And the mnemonic letter stays Latin: **PathMaster exists to edit Latin paths**, so
its user sits in a Latin keyboard layout most of the time, and a Cyrillic mnemonic (`&Файл` → Alt+Ф) is
simply unreachable from there. `Файл(&F)` is an established pattern for a mismatch between script and
keyboard, and costs nothing audibly — NVDA speaks the access key as a separate utterance.

## Consequences

- **Do not "fix" the English for consistency.** Two Catalogue strings that differ only awkwardly are
  probably differing on purpose. Merging them merges their translations.
- **Do not "fix" `Файл(&F)` to `&Файл`.** It is not a mis-transcription; reversing it removes keyboard
  access to the menus whenever the user is in a Latin layout — an accessibility regression that is
  invisible to anyone testing in a Cyrillic one.
- **A keyboard shortcut may never be typed into a translated string.** Every accelerator is appended by
  code; a `\t` inside a Catalogue entry is a defect.
- **The completeness gate enforces what a reader cannot see.** It checks placeholder integrity and that
  no mnemonic letter repeats within one menu — both of which a translator can break without the text
  looking wrong.
- **This constrains the source language, which is unusual.** Adding a new string means checking that its
  English does not already exist with another meaning, because the format offers no other way to say so.
