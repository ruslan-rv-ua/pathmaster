# i18n mechanism

Type: grilling
Status: resolved
Blocked by: 03

## Question

How are translations stored, embedded, and selected?

Settled at charting: language changes take effect **after restart**; `maxBackups` applies immediately;
`settings.json` holds `language` and `maxBackups` only (`theme` is cut). FR-settings-file and FR-i18n-runtime
must both be rewritten to stop contradicting each other.

Open:

- **Mechanism.** wx `.mo` catalogs through `wxLocale` (only if ticket 03 found it bound) versus a Rust-side
  crate (`fluent`, `rust-i18n`) with the catalog embedded in the exe. Which, and why — including how it
  survives the single-exe constraint.
- **One catalog, not two.** Every NVDA-facing string must come from the same catalog as the visible UI, or
  translations will silently diverge from what is spoken. Confirm nothing is announced from a hard-coded
  literal.
- **Default from system locale**: which API, and how `uk-UA`, `uk`, and an unrecognised locale each resolve.
- **Where the catalog lives** in the repo, and the workflow for adding a third language later without touching
  the code.
- **What is deliberately not translated**: registry paths, file names, log lines, and the exact
  `WM_SETTINGCHANGE` payload.
- Do any announced strings need plural forms or interpolation ("3 items", "N of M entries")? That constrains
  the mechanism choice, so decide it here rather than discovering it in v0.2.0.

## Carried in from ticket 09

Announcement texts are translation strings like any other UI text: canonical English in the spec,
Ukrainian shipped as translations. The closed catalogue of Announcements (ticket 09, D3) defines which
strings exist; this ticket owns only how they are stored, embedded and selected.

## Answer

Resolved by grilling, 2026-08-19. Facts were read from the vendored sources
(`wxdragon-0.9.18`, `wxdragon-sys-0.9.18`) and cross-checked against published localisation practice;
sources are cited inline.

### Facts this ticket established

Read from the crate, not assumed:

| Fact | Evidence |
|---|---|
| `TranslationsLoader` serves `.mo` **from memory**; `include_bytes!` is viable | `translations.rs:429`, `set_loader:188`, upstream test `:833` |
| `translate(s)` -> `get_string(s, "")` — **empty domain**, searches every loaded catalog | `translations.rs:537` |
| On miss, `translate` **returns the msgid itself** | `translations.rs:543` |
| `translate_plural(singular, plural, n)` exists; falls back to `n == 1 ? singular : plural` | `translations.rs:559`, `:565` |
| **No interpolation anywhere** in the API — substitution is ours | whole public surface of `translations.rs` |
| `translate` is a **function, not a macro** — no compile-time extraction, no existence check | ticket 03, row 117 |
| **`msgctxt` is not bound at any level** — `GetTranslatedString(ptr, orig, domain, buf, len)` has no context parameter | `translations.rs:226`, `:236`; no context in the C++ shim |
| `Language::Ukrainian = 217`, a single variant — no regional `uk-UA` | `language.rs:458` |
| `add_std_catalog()` is a **separate call** for wx's own strings | `translations.rs:168` |
| `MessageDialog` cannot relabel its buttons — no FFI at all | ticket 03, row 74 |
| Menu label reaches `wxMenu::Append` **verbatim**; wx parses `&` and `\t` out of it | ticket 03, row 56 |
| **`wxAcceleratorTable` is ABSENT at every level** — the label string is the *only* way to register a shortcut | ticket 03, rows 57, 213 |

The last two are the load-bearing pair: **the string we translate is also the keyboard binding.**

### D1. Mechanism: `wxTranslations` + `.mo` embedded through a custom loader

No Rust-side i18n crate. The embedded path is proven end-to-end by an upstream unit test, costs zero
runtime dependencies, brings plural forms, and — decisively — makes **one catalogue structural rather
than disciplinary**: visible labels and Announcements both go through the same `translate()`.

The known cost, accepted: `translate()` is a function, so nothing guarantees a msgid exists. That is
closed by D9, not by choosing a different mechanism.

### D2. `add_std_catalog()` is never called

One catalogue, ours. wx's own "OK"/"Cancel" stay untranslated, which costs nothing in practice: ticket 03
established that `MessageDialog` cannot relabel its buttons at all, so every dialog whose button text
carries meaning (close-confirm `[Save] [Discard] [Cancel]`, Apply's three-way) is already a generic
`Dialog` with **our** buttons. The only stock dialog left is the validation error, whose single button is
`OK` — a word needing no translation.

Not verified, and deliberately made irrelevant by this decision: wxMSW renders `MessageDialog` through
the native Windows API, whose standard buttons are localised by **Windows** per system UI language, not
by us. If translated stock buttons are ever wanted, that is a **measurement**, not a decision, and needs
its own prototype ticket.

### D3. Interface Language resolves through a two-way branch, not locale negotiation

Two languages ship, so nothing needs negotiating:

- `Locale::get_system_language() == Ukrainian` -> `uk`; **everything else, including `Unknown`** -> `en`.
- `settings.json` overrides the system locale; absent or unrecognised -> back to the system locale, no error.
- English is **not the default language** — it is the **fallback**, and that is precisely why msgids are English.

Regional variants need no handling: the wx enum has one `Ukrainian`, so `uk` and `uk-UA` both land on it.

### D4. The log is English, always, and lives outside the Catalogue

Registry paths, file names and the `WM_SETTINGCHANGE` payload are data and are never translated. The open
question was the log, and it resolves against translation: the log is a **diagnostic artifact, not an
interface** — read by a developer, off a machine they cannot see, and it must stay greppable and
independent of catalogue completeness. **No `translate()` call ever appears on a logging path.**

Issue names in the Status column are the opposite case and **are** translated: they are interface
(ticket 09, D1).

### D5. msgids are English source text — and that English text is part of the Catalogue's API

Standard gettext practice, chosen with its cost open: with `msgctxt` unbound, **two identical English
strings with different meanings cannot be disambiguated**. A real instance already exists —

- **Cancel** the command: discard changes back to the Baseline (ticket 06, section 9), itself a Checkpoint;
- **Cancel** the button in `[Save] [Discard] [Cancel]`: do not close, stay open (ticket 06:197).

Different actions, one English word, and Ukrainian needs different renderings. The rule that follows:

> **Where two strings mean different things, their English must differ.** The Catalogue's English text is
> an API surface, not prose — it may be phrased for disambiguation rather than for elegance.

Symbolic keys were rejected for a specific reason: on a miss `translate()` returns the msgid, so a key
would make NVDA speak `dialog.close.cancel` — for this application's primary user that is far worse than
hearing English. Keys would also turn English into a catalogue of its own (`en.mo`), which it is not.

### D6. Placeholders are named braces — `{n}`, `{operation}`

Not `%d`/`%s`, despite that being the gettext idiom and despite Poedit validating `%`-placeholders for
free. The reason is specific to this application: its domain **is** `%PATH%`, `%SystemRoot%`, `%VAR%`, and
UI strings quote that syntax (ticket 06's convert-or-keep dialog is literally about `%VAR%` in a `REG_SZ`
scope). A `%`-placeholder would be indistinguishable from the data being displayed. The validation given
up here is taken back by D9.

Substitution is one explicit helper over `translate()`; wx provides none.

### D7. `.po` is committed; `.mo` is generated at build time by `polib`

Only `.po` files go under version control. `build.rs` compiles them with [`polib`](https://docs.rs/polib)
— pure Rust, writes `.mo`, handles plural forms — into `OUT_DIR`, and `include_bytes!` embeds them from
there.

This reverses an earlier position taken during this session (commit the `.mo`). That position rested on
avoiding a `msgfmt` CI pin, which ticket 04 had shown to be expensive; `polib` removes the objection
entirely, being a **build-dependency** with no effect on the artifact or its size. Committing compiled
catalogues also runs against settled practice ([GNU gettext manual](https://www.gnu.org/software/gettext/manual/html_node/Files-under-Version-Control.html);
[Django #23321](https://code.djangoproject.com/ticket/23321) removed `.mo` from its repository).

The principle that survives intact is ticket 04's: **gate the artifact, not the build config.** D9 still
runs against the `.mo` — now derived rather than committed, so `.po`/`.mo` drift becomes structurally
impossible instead of merely unlikely.

### D8. Startup order, and the Read-only Data edge

Language is needed for the *first* Announcement — the Read-only Data reason (ticket 09, catalogue item 7)
— and it lives in `settings.json`, inside the very directory that announcement is about. There is no
conflict, because Read-only Data is about **writing**; reading `settings.json` still works. Fixed order:

1. locate the Data Directory (ticket 07's resolve rule)
2. read `settings.json` -> Interface Language
3. `Translations::new` -> `set_loader` -> `set_language_str` -> `add_catalog`
4. build the UI
5. determine writability
6. announce

A missing or corrupt settings file falls to the system locale by D3, silently.

**In Read-only Data the language selector is disabled and reads as disabled** — exactly like Apply and
Cancel on a clean Session (ticket 09, D5). Ticket 07's rule ("every write path closed, the reason named")
is satisfied with no new mechanism: the reason was already spoken at startup.

### D9. The completeness gate is a plain `#[test]` over a registry of msgid constants

`translate()` is a function, so the set of msgids is not extractable — unless it is made explicit.
**Every msgid is a constant in one registry module.** The gate then needs no external tooling at all:

- for every entry, assert the translation is **present** under `uk` — via `get_string(s, "").is_some()`,
  **not** `translate(s) != s`, which would falsely flag any string whose Ukrainian equals its English;
- for plural entries, the same through `get_plural_string`;
- **placeholder integrity** — every `{name}` in the msgid appears in the translation, and no unknown
  `{...}` appears in it. This recovers what D6 traded away;
- **mnemonic uniqueness** — parse `&` from every translated label belonging to one menu and assert no
  letter repeats (see D12);
- **self-sensitivity** — a deliberately absent msgid must return `None`, so the gate cannot pass by
  always answering "present".

The registry earns two more things for free: a constant cannot be mistyped, and "one Catalogue" becomes a
structural fact rather than a rule someone must remember.

**Fuzzy entries are handled without any code.** gettext marks a translation fuzzy when it is guessed after
the source changed, and **excludes fuzzy entries from the compiled `.mo`** — so gating the `.mo` reads
them as missing, which is the correct treatment. This matches practice elsewhere (Django's
`check_translations` verifies "fully translated with no fuzzy entries" and exits non-zero; `i18next
status` likewise).

### D10. Catalogue location — and this ticket's own premise, rewritten

This ticket asked for a workflow to add a third language **"without touching the code"**. That is
**impossible by construction**, and impossible because of a constraint chosen deliberately: NFR-portable
puts the catalogue *inside* the binary. No embedded-catalogue scheme escapes it — `i18n-embed`,
`include-po` and `gettext-macros` all require a rebuild. The premise is rewritten rather than pretended
satisfied.

What is achievable, and is the decision: catalogues live in `i18n/` as `<code>.po`, and **`build.rs`
enumerates `i18n/*.po`**, generating both the `.mo` files and the loader's table. Language codes are the
file basenames, so they match what `set_language_str` and `available_translations` exchange.

> **Adding a third language: drop `xx.po` into `i18n/`.** The table, the `.mo` and
> `available_translations` all follow automatically. Exactly one thing is edited by hand — one arm in
> D3's system-locale mapping — plus a rebuild.

### D11. `settings.json` accepts `"auto" | "en" | "uk"`

Absent or unrecognised -> `auto`. `"auto"` must be expressible **explicitly**, because otherwise "go back
to following the system" can only be said by deleting a key — awkward in a UI and invisible in the file.
This mirrors the established shape of per-app language settings, where an explicit "System default" sits
alongside the concrete choices and an app-level choice overrides the system.

**The application writes the user's choice, not its outcome.** Recording `uk` when `auto` was chosen on a
Ukrainian machine would freeze that choice and follow the user onto an English one.

### D12. Accelerators belong to the code; mnemonics belong to the Catalogue

Because `wxAcceleratorTable` is absent, a menu item's label string is the **only** way to register a
keyboard shortcut. If the whole label were translated, a translator would hold the key bindings — and
`"&Скасувати\tCtrl+Я"` would not merely read oddly, it would **destroy the shortcut**, with no fallback
mechanism, in an application whose primary user is keyboard-only.

**(a) The string is split.** The Catalogue holds `"&Undo"`; the code appends `"\tCtrl+Z"`. Accelerators
become untranslatable and unbreakable. This matches general practice — mnemonics are adapted to the
translated text while universal shortcuts such as Ctrl+Z stay constant across locales
([Microsoft](https://learn.microsoft.com/en-us/windows/apps/develop/input/keyboard-accelerators)) — and
the one-letter-per-menu rule is enforced by D9.

**(b) Ukrainian mnemonics keep the Latin letter, in parentheses: `"Файл(&F)"`.** Not `"&Файл"`, which is
the platform norm for Ukrainian Windows. The reason is specific to this application: **PathMaster exists
to edit Latin paths**, so its user sits in the Latin layout most of the time, and a Cyrillic mnemonic
(Alt+Ф) is unreachable from there — every menu access would cost a layout switch. The parenthesised form
is an established pattern for exactly this mismatch between script and keyboard
([Localization Guide](http://docs.translatehouse.org/projects/localization-guide/en/latest/guide/translation/accelerators.html)),
and it costs nothing audibly: NVDA speaks the access key separately (`['Файл', 'підменю', 'Alt+', 'f']`,
ticket 02). Confirmed by the user, who *is* the target user.

Dropping mnemonics in Ukrainian was rejected — ticket 02 measured them being announced; they are free
keyboard access.

Recorded as [ADR-0004](../../../docs/adr/0004-catalogue-text-is-load-bearing.md), together with D5,
because both are cases where the text itself carries mechanism and both invite a well-meaning tidy-up.

### D13. The restart notice rides the selector's own label — the Announcement catalogue stays closed at seven

Language changes take effect after restart (charting decision 6), so the user must be told. But ticket 09's
Announcement catalogue is **closed**, and an eighth item would amend a resolved ticket. It is not needed:
ticket 09 requires every Announcement to have a visible home, not every visible message to be announced.

A separate static caption would not be read — NVDA speaks the **focused control's name**. So the notice
folds into that name: **"Language (takes effect after restart)"**. It is spoken as part of the control,
needs no Announcement, and **ticket 09 is not modified.** `maxBackups` needs no such caption precisely
because it applies immediately.

### D14. Composed strings must compose in Ukrainian — a constraint on the English, not a mechanism

Announcement 4 is `"Undone: {operation}"`, and `{operation}` is itself a Catalogue string. English composes
freely; Ukrainian does not — "Скасовано: Додати запис" is ungrammatical, the genitive is required.

Enumerating every combination (about 20 strings instead of 7) was rejected. Instead: **operation names are
translated as verbal nouns** — "Скасовано: додавання запису". This is a note to the translator, not a
mechanism, and costs nothing.

It does create an obligation under D5: an operation name must be a **different English string** from the
button that performs it, because the two need different Ukrainian forms — "Додати запис" on the button,
"додавання запису" in the announcement. Tickets 09 and 10 already diverge usefully here (`"Add…"` vs
`"Add entry"`); **that is now a requirement, not a happy accident.**

### D15. Languages are listed by endonym

The selector shows "English" and "Українська", each in its own language — never "Англійська"/"Українська".
A user who cannot read the current interface language must still be able to find theirs. Side benefit:
these two strings need no translation and do not depend on Catalogue completeness.

### Requirement rewrites

- **FR-settings-file** and **FR-i18n-runtime** stop contradicting each other: `settings.json` holds
  `language` (`auto|en|uk`) and `maxBackups` only; language applies **after restart**, `maxBackups`
  immediately.
- **US-i18n** "add a language without touching the code" -> D10's rewritten premise: drop a `.po`, edit one
  mapping arm, rebuild.
- **NFR-portable** is unaffected: catalogues are embedded, `polib` is a build-dependency, and nothing is
  read from disk at runtime.

### What this ticket deliberately did not change

- **Ticket 09's Announcement catalogue stays closed at seven items** (D13).
- **Where the language selector lives** — a settings dialog or a menu — belongs to ticket 17.
- **Which exact strings exist** — the Catalogue's contents — is assembled by ticket 16 from the tickets
  that own each message.

### Carried forward

- **Number formatting is a non-issue, for a recorded reason.** Instantiable `wxLocale` is absent (ticket
  03, row 122), so the C locale is never initialised and numbers stay C-formatted. This does not affect
  `wxTranslations`, which is independent of `wxLocale` in modern wx, and the only numbers shown are small
  counts.
- **A `.mo` must carry `Plural-Forms:` for Ukrainian (`nplurals=3`).** Poedit writes it; `polib` preserves
  it. Announcement 1 is the only plural string, and its zero case is a separate msgid ("no entries",
  ticket 09), so `n = 0` never reaches plural selection.
- **New domain terms** **Catalogue** and **Interface Language**: [CONTEXT.md](../../../CONTEXT.md). The
  Announcement entry there was reworded from "a closed catalogue of messages" to "a closed set of
  messages" to stop it colliding with the new term.
