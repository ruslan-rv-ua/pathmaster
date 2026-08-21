# 16 — Settings dialog

**Spec:** [spec §13 (dialog), §11 (FR-i18n-runtime)](../../pathmaster-v0-1-0/spec.md)

**What to build:** Tools → Settings… lets the user change the Interface Language and the backup budget: a modal dialog with the language selector (whose own label carries the restart notice, keeping the Announcement catalogue closed) and the `maxBackups` field, writing settings.json atomically on OK.

**Blocked by:** 07 (settings semantics), 08 (UI shell, Tools menu home).

**Status:** resolved

- [x] Dialog holds the language selector labelled "Language (takes effect after restart)" with endonym items ("English", "Українська") plus the auto choice, and the `maxBackups` field (valid domain ≥ 1); our own OK/Cancel buttons, never stock ones
- [x] The file records the choice, not its outcome (`"auto" | "en" | "uk"`); language applies after restart — no live re-translation, no extra Announcement
- [x] `maxBackups` applies immediately (next rotation uses it)
- [x] OK writes settings.json via the atomic-replace helper, preserving unknown fields; a field previously invalid in the file is replaced only when the user changes that setting
- [x] In Read-only Data the dialog's controls are disabled and read as disabled (no write path)
- [x] Escape/Cancel leaves the file untouched; focus returns to the control that opened the dialog
- [x] All strings in the Catalogue with Ukrainian translations; the completeness gate passes

## Comments

Implemented 2026-08-21 on `feature/settings-dialog`.

The binary gains one module — `ui::settings_dialog` — and the Tools menu its first item. Everything
worth a test moved down a tier first (ADR-0007): `pathmaster-core::settings` gained the pair the
dialog opens on and answers with and the rule that records it, and `pathmaster-core::catalogue` the
selector's items. Thirteen new tests, none of which link wxWidgets.

**"Only what the user changed" is the whole ticket, and it is one comparison.** `settings.json` may
be holding a `language` this version cannot read — the field layer falls back *in memory* and leaves
the raw value in the file (§13's choice-not-outcome rule) — and the dialog necessarily shows the
fallback, because that is what the run is doing. So an OK that wrote both settings would replace
`"fr"` with `"auto"` every time the dialog was so much as opened, and the rule that a v0.2 value
survives a v0.1 run would be true only until the user changed the backup budget.
`SettingsFile::record_choices` therefore compares setting by setting and calls only the setters
whose value moved — which is also why it is one method in core rather than two calls in the window:
the rule has a place to be written down and a place to be tested from.

**It answers whether anything changed, and nothing is written when nothing did.** Dirty is a
comparison and not a record that something happened (`CONTEXT.md`), and the same reading applies to
a dialog: a user who retypes the number already in the field has changed no setting. The alternative
— writing on every OK — would reformat a hand-edited file for the crime of being looked at, and give
a first run a `settings.json` holding `{}`, which records nothing and claims to be a file the user
made choices in.

**One domain, read twice.** What the field accepts and what the file accepts are now the same
predicate — `in_backup_budget`, whole and ≥ 1 — with `parse_max_backups` in front of it for typed
text. A dialog with its own idea of the domain would eventually accept a budget the file rejects:
a value the user chose, watched being written, and lost at the next start with a `WARN` line as its
only witness. A `wxSpinCtrl` was considered for the field and rejected on the same ground — it is
`i32`, the domain is `u32`, and a budget past `i32::MAX` would come back from the control changed
without anybody changing it. So the field is a plain text field validated on commit exactly as a
path is (§6): the text and the focus stay where the user left them, and nothing reaches the file.

**In Read-only Data the OK button is disabled with the two controls.** The ticket says "the dialog's
controls", and OK is the one that writes; a run with no write path leaves it a button with nothing
to do, and a disabled control is how this application says so. What is left is a dialog the settings
can still be *read* out of — which in that run is exactly what is still possible — with Cancel
holding the focus, the default and Escape together. The menu item itself stays **available in every
state** for the same reason: an item reading as unavailable would say the settings cannot be looked
at.

**The selector's order is written down once, in `LanguageChoice::SELECTABLE`.** The dialog reads its
answer back by position, so the labels and the choices they stand for cannot be two lists;
`Catalogue::language_items` composes the labels from that very array. The auto item is Catalogue
text and the two languages are not — an endonym translated into the current Interface Language would
be the one item a user who cannot read that language could not find, which is the whole reason
endonyms are outside the Catalogue.

**Live pass, driven cross-process against a staged copy** (its own `data\`, so nothing on this
machine was touched):

- Tools reads `Налаштування(&S)…` first, then Open Backups Folder — §15's order.
- Over a hand-written `{"zeta": {...}, "language": "fr", "maxBackups": 0, "alpha": 2}`: the dialog
  opens on the auto choice and 50 (the two in-memory fallbacks), `0` in the field earns the
  rejection dialog whose title is the message, and OK on ` 12 ` leaves the file with `zeta`,
  `alpha`, the key order **and `"fr"`** intact, `maxBackups` now 12.
- Choosing English and pressing OK replaces `"fr"` with `"en"` and leaves `maxBackups: 0` — the
  other invalid value, which the user did not touch — exactly where it was.
- Cancel, and an OK over untouched controls, each leave the file byte-for-byte unchanged.
- Escape closes the dialog and focus returns to the control it came from.
- Committing a new language leaves the running window's menu bar, tabs and Banner untouched and
  says nothing; the log gains no line, because the write succeeded.
- With `data\` denied write, the selector, the field and OK all read as disabled, the settings are
  still shown, and Cancel holds the focus.

`maxBackups` applying immediately needed no code: the window holds the settings and every Apply Run
reads the budget from them at the moment it runs (ADR-0010), so recording the change *is* applying
it. It is the one criterion with no live check — the success path of an Apply writes this machine's
own `PATH` — and it is left to the Checklist.

Release Checklist steps **B12–B21** cover the dialog: the Tools order, its labels and buttons, the
endonyms, the rejection, the two "only what changed" cases, the no-op OK, Cancel and Escape, the
Read-only Data pass, the budget taking effect without a restart, clearing an unreadable `language`,
and a failed write.

## Review

Two axes, run against `develop`. Both found something real.

**The failure path was wrong, and it was wrong twice.** The first draft recorded the choices in
memory and then wrote the file, saying no more about a failure than one `WARN settings:` line — the
rule ticket 15 fixed for the geometry writer. The Spec axis took that apart. The justification
("the setting they chose is in force this run either way; what they lose is that it does not
survive a restart") is **false for half the dialog**: a language change only ever takes effect
through the file, so a failed write loses it outright. And recording before writing left the change
**unretryable** — memory already held the new values, so a second OK compared equal and wrote
nothing, on a failure whose cause (§3: the other instance holding the file) is a *designed* state.

So the order is now the other way round: the document is amended on a copy, written, and adopted
into the run **only if the file took it**. The file and the run can never disagree, and a write
that failed leaves the very difference that makes the next OK a change again. With nothing adopted,
silence became indefensible — the user pressed OK and got a dialog closing — so a failed OK now
earns **"Settings could not be written — nothing was changed"**, the mirror of the unreadable-file
dialog §13 already has, beside the `WARN` line. The shutdown writer keeps its silence, and now for
a reason that is only true there: nobody asked for it, and the window is already going. Both
writers share one `App::record_settings`, which is where the rule is written down; the Standards
axis had flagged the duplicated write-and-log as a smell independently.

Verified live against a staged copy with `settings.json` held exclusively: the dialog appears, the
log gains exactly one `WARN settings: … (os error 5)`, reopening Settings shows the **old** value,
and releasing the file lets the same change through on the next OK.

**A name the glossary reserves.** `SETTINGS_MAX_BACKUPS` / `REJECTED_MAX_BACKUPS` named a count of
Snapshot files, which `CONTEXT.md` does not let "Backup" mean — and the doc comment directly above
them made exactly that argument about the label text before naming the constants the other way.
Now `SETTINGS_SNAPSHOTS_TO_KEEP` / `REJECTED_SNAPSHOTS_TO_KEEP`. The private `in_backup_budget`
keeps its name: `develop`'s own `DEFAULT_MAX_BACKUPS` doc already says "the per-Scope backup
budget", and renaming only the new helper would leave it disagreeing with the constant above it.

**A rule left in the crate no test can reach.** `SELECTABLE`'s order had moved down a tier, but
both halves of the position↔choice mapping stayed in the dialog — so the round trip the module
calls load-bearing had no test, and this file's own "everything worth a test moved down" was an
over-claim. `LanguageChoice::selector_index` and `::at_selector_index` now sit beside the array
they walk, with a round-trip test over every choice and one past the end.

**Named rather than fixed:** with `"language": "fr"` in the file, a user who *wants* the auto choice
changes nothing by selecting it — it is already selected, being the fallback — so `"fr"` stands and
its `WARN` recurs at every start. That is the "only what the user changed" rule working, not
failing, and the alternative is writing the language on every OK, which deletes the rule. It is now
written into §13 and given Checklist step **B20**, which is also how it is cleared: choose another
language, OK, reopen, choose back.

**Declined, with reasons:** a shared modal skeleton between this dialog and `entry_dialog` (the
genuinely identical part is ~13 lines of sizer plumbing, and a helper parameterised over three
dialogs' differing validation, focus and disabled rules would say less than the explicit versions —
though the colliding `FIELD_WIDTH_DIP` became `BUDGET_WIDTH_DIP`); a newtype for the budget
carrying "≥ 1" (real, but it ripples through `ApplyRun`, rotation and their tests for a value
already checked at both entry points); and renaming `Choices`, whose sibling is `LanguageChoice`
and whose module is `settings`.
