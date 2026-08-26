# Search bar contract

Type: grilling
Status: resolved (2026-08-26)
Blocked by: 03, 04, 05

## Question

FR-search: Ctrl+F, live substring filter over the active Scope's list, counter, ESC to clear and
return. The filtered-view semantics (03), the NVDA mechanism verdict (04) and the expansion toggle
(05) are in; specify the feature:

- **Search over which text** — raw, expanded, or "whatever the list currently shows" (the expansion
  toggle changes that)? And case folding: plain case-insensitive contains, or the full Normalisation
  reading (quote stripping, slash reconciliation)?
- Placement and construction: the field sits above the ListView (per PRD) — always visible, or
  appearing on Ctrl+F and collapsing on clear? What the prototype (04) said about focus/announcement
  shapes this.
- The counter ("N of M entries") — where it lives given no toolbar-decision assumptions (Banner?
  StatusBar field? beside the field?), and the debounced spoken count's exact wording, both
  languages (a new member of the closed Announcement set).
- ESC semantics: clear text + return focus to the list (PRD) — to which row (the focus rule from 03
  applies)? And what does ESC do when the field is already empty?
- Empty result set: what the list shows, what is spoken, and whether the editing-command rule
  from 03 has anything special to say.
- Per-Scope or shared: does switching tabs keep, share, or clear the search text (03 sets the frame;
  this ticket sets the value).
- Does search state survive Refresh, Restore, Apply?

## Input from ticket 05 (2026-08-26)

Expansion Mode is decided: app-wide, per-Run, display expands via Normalisation's own reading.
On search-over-which-text, [research/05](../research/05-var-expansion-best-practices.md) Q5
recommends matching the **currently displayed** rendering, so the spoken count equals the rows the
user will hear (raw-matching in expanded mode produces audibly inexplicable hits); Excel's
dual-mode Find is an explicit user choice defaulting to raw, and its Replace binds to raw only.
Consequence to weigh here: if search matches displayed text, toggling the mode recomputes Filtered
View membership.

## Resolution (2026-08-26)

**The Search field is a permanent part of each Scope tab; it matches the text the row displays; and
its count is an Announcement whose visible home is the Banner and whose on-demand home is the
StatusBar.** Everything below either follows from that or from a rule already in place.

### What is matched

1. **The currently displayed rendering** (research Q5 of ticket 05, adopted): what the spoken count
   counts is exactly what the arrow keys will read. The cost is real and is paid deliberately —
   **with a Filtered View active, toggling Expansion Mode changes membership**, because
   `%JAVA_HOME%\bin` and `C:\jdk21\bin` are different haystacks. Ticket 05's item 6 said the toggle
   never changes the count; that is now false, and 05 is amended rather than quietly outlived (see
   *Amendment to ticket 05* below). The rejected alternatives, for the record: matching raw always
   keeps 05 intact but lets a row match on text the user cannot see (searching `jdk` finds nothing
   while NVDA is reading "jdk21"); matching raw **or** displayed makes membership mode-independent —
   05 survives untouched — at the price of a count that can exceed the rows visibly containing what
   was typed, which is the failure the displayed-text rule exists to prevent.
2. **Case-insensitive substring, slash-folded, and nothing else.** The line is principled, not
   arbitrary: **case and slash direction are foldings the domain already applies everywhere**
   (Normalisation does both; Everything folds `/`→`\` by default in Windows path search), while
   quote stripping, trailing-`\` trimming and `%VAR%` expansion are the parts of Normalisation that
   change *what text exists* — and those stay out. A search for `"` **must** find the `Quoted`
   entries; that is precisely when a user goes looking for them. Expansion is Expansion Mode's job,
   not the matcher's.
3. **Unicode case folding, never ASCII** (`str::to_lowercase`, both sides). This machine's own PATH
   contains `C:\Users\Руслан`; ASCII folding would make the field silently case-sensitive for every
   Cyrillic path, in the one product that must not be.
4. **The query is never trimmed.** A space is a legitimate character in an Entry, and a
   whitespace-only query honestly matches every Entry containing a space.

### The control and the keyboard

5. **A permanent field, one per Scope tab, built from a native `TextCtrl`** — never `SearchCtrl`,
   which is the *generic composite* on MSW (ticket 01) and so unmeasured with NVDA, while `TextCtrl`
   has the identical event surface and is the exact control ticket 04 proved. Permanent rather than
   summoned by Ctrl+F: this is a filter field (the visible set *is* the output), not a find bar, and
   v0.1.0's layout rule is that the window never reflows under the user (§12). NVDA's own GUI is the
   precedent — the Add-on Store's Search field and the Input Gestures filter are permanent parts of
   their dialogs, reached by an accelerator, filtering live.
6. **Per-tab control, so per-Scope state costs nothing** (ticket 03's frame, given its value): each
   Scope tab carries its own field and its own text, switching tabs keeps both, and the Backups tab
   simply has no field. Layout inside a Scope tab, per the ticket-04 prototype: label + field above
   the list.
7. **The label is constant text with no mnemonic** — "Search:" **[assembly]**. Only menu items carry
   mnemonics in this application (§15), and Ctrl+F with a menu home is the gesture. The label never
   carries the count: a changing label is a `NAMECHANGE`, measured dead in v0.1.0.
8. **Tab order becomes tabs → search field → list → buttons.** One extra Tab stop on every run,
   including runs where nobody searches — accepted, and named as the cost it is; both NVDA dialogs
   cited above pay it.
9. **Ctrl+F focuses the field and selects its whole contents**, so the next keystroke replaces the
   old query and a second Ctrl+F is harmless (the Visual Studio / VS Code find convention). Its
   menu home is **View**, per ticket 02's model; recorded deviation: Windows convention puts Find in
   Edit, and 02 chose "what changes the list's contents lives in View" over that convention
   knowingly. **Disabled on the Backups tab**, which has no Filtered View.
10. **Enter is consumed by the field and does nothing.** Two hazards, both real: an unhandled Enter
    in a wx text field reaches the default button (Add, or Apply), and an Enter that moved focus to
    the list would arm the list's own Enter — Edit Entry — on the very next press. **Down-arrow and
    Tab** are how the user enters the list (ticket 04, proven).
11. **ESC** clears the text and returns focus to the list (ticket 04's measured verdict, default on,
    reversible by setting). Focus lands by ticket 03's rule 4.1 — the Entry that held focus in the
    Filtered View is visible again in the full list, so it keeps focus; if the filtered list was
    empty, the last Entry that held focus, else the first row. **ESC on an already-empty field still
    returns focus to the list and says nothing** — one gesture, one meaning, and nothing changed so
    nothing is announced. (Firefox DevTools' two-tier rule is "clear if non-empty, otherwise let the
    panel handle it"; a permanent field has no panel to close, so the app's own answer to "I am done
    here" is the list.)

### What is spoken

12. **The count Announcement fires when the *view criteria* change** — the search text (debounced)
    or Expansion Mode. Working-Copy changes recompute membership **silently**: ticket 03's rule 3
    stands unextended, and a Delete under a filter still speaks only what v0.1.0 gave it.
13. **Two new catalogue items, six msgids** — the catalogue goes from eight items to ten. (The
    grilling table counted msgids against items and wrote "8 → 14"; the decision it named is
    unchanged.) A short form for the count heard on every typing pause, and a Scope-named form for
    the moment the Scope itself changed:

    | # | When | English msgid | Ukrainian |
    |---|---|---|---|
    | 9 | typing pause, ESC-into-a-still-filtered-view, Expansion toggle | `{n} of {m} entry` / `{n} of {m} entries` | «{n} з {m} запису» / «{n} з {m} записів» / «{n} з {m} записів» |
    | 9 | …and its zero case | `No matching entries` | «Немає збігів» |
    | 10 | Scope tab activation and Refresh, **while that Scope has a Filtered View** | `User PATH: {n} of {m} entry` / `…entries`; `System PATH: …` | «PATH користувача: {n} з {m} записів» (3 forms); «PATH системи: …» |
    | 10 | …and their zero cases | `User PATH: no matching entries`; `System PATH: no matching entries` | «PATH користувача: немає збігів»; «PATH системи: немає збігів» |

    Two Scope-named strings rather than one frame with the Scope filled in — §11's own rule, the one
    that already split "User PATH applied" from "System PATH applied".
    **The plural form is selected by `{m}`, not `{n}`**: the noun is governed by the total in both
    languages ("1 of 1 entry"; «з 1 запису» vs «з 2 записів»). The i18n gate checks that plurals are
    present, not which number chose them, so this is written down here or it is lost.
    Ukrainian «Немає збігів» rather than «Немає відповідних записів»: it is the message heard most
    often — every mistyped query — and terseness outranks vocabulary symmetry there.
14. **When there is no Filtered View, Announcement 1 speaks** — «PATH користувача: 50 записів» —
    whether the query was emptied by ESC or by backspace. Reusing the existing item is exact rather
    than merely thrifty: an empty query restores precisely the state tab activation announces. Once
    ticket 07 lands, "no Filtered View" means an empty query **and** an unnarrowed filter.
15. **The Expansion Mode toggle, with a Filtered View active, speaks twice**: its own message
    ("Showing expanded values") and then item 9's count, routed through the **same debounced path**
    — so the two are separated by exactly the delay the user tuned, and no combined msgid is minted
    (composing translated fragments is what §11 forbids). Whether NVDA lands both is a measurement,
    not an assumption: it joins the round-2 verification ticket below.
16. **Empty result set**: the list shows **zero rows and no placeholder** (v0.1.0's own position for
    its zero case), "No matching entries" is spoken, and Edit / Delete / Copy are disabled because
    there is no focused visible Entry — ticket 03's rule applied, not extended. Focus stays on the
    empty list and never jumps to the field uninvited (03, rule 4.3).
17. **StatusBar field 0** carries the same numbers as passive, on-demand text (`NVDA+End`), which is
    what makes the count re-readable after the Banner has moved on: a Scope with an active Filtered
    View reads "User PATH: {n} of {m} entries ({k} issues)" **[assembly]**. **The parenthetical never
    changes meaning** — it counts that Scope's Issues, not the view's; a filter is a view, and the
    diagnosis is a fact about the data. Field 1 (merged length) is untouched: the merged PATH is a
    property of the Working Copies, not of what is being shown. This is also the home of ticket 07's
    "Filtered view — N of M" reminder; 07 inherits it rather than choosing again.

### State, settings, and the rest

18. **Nothing persists.** The query dies with the Run — ticket 03's signpost confirmed, and the
    Editing Session it belongs to never survives a process boundary anyway.
19. **Refresh, Restore, Apply change nothing about the view** — ticket 03's rule 6, unchanged. What
    they do change is what is *spoken*: Refresh under a Filtered View speaks item 10, not
    Announcement 1.
20. **Read-only Data searches normally**: that Run still reads, diagnoses and lists.
21. **`settings.json` gains three flat `camelCase` fields**, each with its own default — deliberately
    not a record: §13's "geometry falls back as a unit" is a rule for members that have no individual
    defaults, and these three do, so a record would let one typo silently reset all three.

    | Field | Type and domain | Default |
    |---|---|---|
    | `speakFilteredCount` | bool | `true` |
    | `filteredCountDelayMs` | int, `0`–`5000` | `250` |
    | `searchEscapeReturnsFocus` | bool | `true` |

    Named for the **Filtered View** where the setting is about the view (ticket 07's filter changes
    the same count and rides the same two fields) and for **Search** where it is about the field.
    `0` is a legal delay (announce as soon as the keystroke settles); the upper bound exists so a
    typo cannot silently mute a feature the user believes is on. **The failure taxonomy needs no new
    layer**: these are ordinary field-layer members — out-of-domain value → that field's default in
    memory, the file keeps the raw text, one `WARN` line, no dialog, no clamping (§13).
    **All three get Settings-dialog controls** **[assembly]**: every setting the *user* chooses has
    one today (geometry is the app's own), and a v0.2.0 that hid three accessibility preferences in
    a hand-edited file would be the one product that must not.

### Amendment to ticket 05

Item 6 of [05](05-var-expansion-toggle.md) reads "The count is not included — the toggle never
changes it." That holds only while no Filtered View is active, which decision 1 above makes a
condition rather than a fact. Amended: **with a Filtered View active on the visible Scope, the toggle
speaks its mode message and then item 9's count** (decision 15). Everything else in 05 stands —
the mode message itself, its wording, the check menu item, and the count's absence when nothing is
filtered.

**Evidence**: [research/06-search-bar-best-practices.md](../research/06-search-bar-best-practices.md)
(gathered 2026-08-26, before grilling, per standing directive 7), plus
[research/04](../research/04-live-filter-best-practices.md) for the debounce, count wording and NVDA
mechanism, and [research/05](../research/05-var-expansion-best-practices.md) Q5 for displayed-vs-raw.
Notable: NVDA's own GUI settles the permanent-field question better than any style guide, and the
toggle-bar precedent (Thunderbird's Quick Filter) carries an open NVDA defect and a decade-old
"how do I close this" bug.

**New ticket**: [16 — NVDA verification round 2](16-nvda-verification-round-2.md), which collects the
measurement obligations the grilling tickets keep producing (05's check-menu-item read, and this
ticket's two-Announcements-in-succession). It is blocked by every remaining feature contract that
could add an item to it, and blocks the assembly.

**No ADR**: every branch follows either the domain model already in place or uncontested precedent,
and all of it is reversible until the spec locks. **No `CONTEXT.md` change**: *Filtered View* already
names the Search text as one of its two criteria, and this ticket sets that criterion's value rather
than adding a term.

**Consumed by**: 07 (the count's StatusBar home, the item 9/10 split, the two shared settings, and
"no Filtered View" becoming a two-part condition), 11 (there is now a permanent text field in every
Scope tab, so Ctrl+F's neighbour Ctrl+C must be scoped to the list without breaking the field's own
copy), 15 (menu home, accelerator, six msgids, three settings fields, Release Checklist delta), 16
(the verification obligation).
