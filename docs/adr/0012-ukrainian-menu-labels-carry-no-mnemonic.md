# Ukrainian menu labels carry no mnemonic, and the measurement that kept them was of another string

[ADR-0004](0004-catalogue-text-is-load-bearing.md) settled that a Ukrainian menu label keeps its Latin
mnemonic in parentheses — `Файл(&F)` — and told the next reader not to tidy it away. It was right about
the alternative it examined and wrong about the one it shipped, because the measurement it rested on was
taken from a string the application never displayed.

**wx removes the `&` and nothing else.** Given `&File` it draws "File" with the F underlined; given
`&Файл` it draws "Файл" with the Ф underlined; given `Файл(&F)` it draws **"Файл(F)"** — the parentheses
and the letter are ordinary text and stay in the label. The three forms therefore differ in what a screen
reader is handed, not only in which key opens the menu.

**The recorded measurement was of `&Файл`.** ADR-0004 claims the parenthesised form "costs nothing
audibly — NVDA speaks the access key as a separate utterance", citing ticket 02's
`['Файл', 'підменю', 'Alt+', 'f']`. That utterance list has a clean "Файл" in it, which only the
`&`-prefixed form produces. Re-measured on the form that actually shipped, NVDA says
**"Файл(F) підменю Alt+ f"**: the letter once inside the name and once as the access key. Every menu bar
title and every menu item said its letter twice, in the application built for a screen-reader user first,
for as long as ADR-0004 stood — and the record said the opposite, so nobody went to check.

**The Ukrainian labels therefore carry no mnemonic at all.** All twenty-eight of them lose the
parenthesised letter. The catalogue keeps the `&` on the English side, where it is invisible by
construction, and every `msgstr` in `uk.po` is now plain text.

**Moving the letter onto Cyrillic was not the fix, for ADR-0004's own reason.** `&Файл` reads cleanly and
binds Alt+Ф — unreachable from the Latin layout this application's user sits in, because PathMaster
exists to edit Latin paths. That argument is untouched by the re-measurement; it is why the choice was
between the doubled letter and no mnemonic, with no third form that keeps both.

**So this is a real loss of keyboard access, taken deliberately.** A Ukrainian run reaches its menus with
Alt or F10 and the arrow keys, and by no other gesture. The trade was made by the person who is the
target user: a letter spoken twice on every menu item, against a shortcut that only ever worked in one
keyboard layout.

**English keeps its mnemonics, because the defect is the parenthesised form and not the mnemonic.**
Nothing is doubled in an English run — wx eats the `&` — and no complaint was ever about `&File`.
Deleting the mechanism as well would have cost a working shortcut to buy nothing.

## Consequences

- **The gate splits in two rather than relaxing.** `the_source_menus_keep_one_mnemonic_letter_per_item`
  keeps the old rule for the source language: every menu item has a mnemonic, unique within its menu.
  `no_translation_carries_a_mnemonic_outside_the_source_language` is its inverse, over every registered
  msgid of every language that is not the source — not only the menu items, because an `&` is a defect
  wherever a translation puts one. The weaker "if there is a mnemonic it must be unique" was rejected:
  it would have stopped catching both of the things the original caught.
- **`mnemonic()` still reads `Файл(&F)`, and that is now load-bearing in the other direction.** The
  function's ability to see a parenthesised mnemonic is exactly how the gate notices one restored. Its
  unit test stays for that reason, not as a record of the convention.
- **`Файл(&F)` is what a translator will write next.** It is the platform norm for Ukrainian Windows and
  reads as ordinary text, so no reviewer of the Ukrainian would catch it — which is why the rule is
  gated and stated in `uk.po`'s own header rather than left to this file.
- **Release Checklist steps 31 and 72 name a gesture that no longer exists in Ukrainian.** Both open the
  Help menu with `Alt+H`; both now say so for the English interface and give F10 and the arrows for the
  Ukrainian one.
- **ADR-0004 is superseded only here.** Its other half — that identical English collapses to one
  translation, so two meanings must differ in English — is untouched, as is the rule that an accelerator
  is appended by code and never typed into the Catalogue. The paragraph and the consequence this record
  overturns are marked there rather than rewritten: the reasoning was sound and the measurement was not,
  and erasing it would erase the only interesting thing that happened.
- **Measure the string you ship, not the one in the argument.** ADR-0004 compared `Файл(&F)` against
  `&Файл`, and then justified the winner with a reading of the loser. Nothing about the file's shape made
  that visible; only speaking the shipped label does.
