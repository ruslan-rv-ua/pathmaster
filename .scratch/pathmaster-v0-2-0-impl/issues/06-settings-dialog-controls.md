# 06 — Settings-dialog controls for the three view-state fields

**Spec:** [delta-spec §15](../../pathmaster-v0-2-0/spec.md)

**What to build:** The Settings dialog grows three controls for the fields ticket 03 introduced, so the primary user can tune the narrowing behaviour without hand-editing `settings.json`: whether filtered counts speak, the debounce delay, and whether ESC returns focus to the list.

**Blocked by:** 03 (the fields and the behaviour they gate).

**Status:** in review — built and driven live in both languages; the NVDA reading is the one bullet still open, see Comments

- [x] Three controls with the assembly labels (amendable at implementation like v0.1.0's dialog strings were): "Speak filtered entry counts" («Озвучувати кількість відфільтрованих записів»), "Delay before speaking the count (ms)" («Затримка перед озвученням кількості (мс)»), "Escape returns focus to the list" («Escape повертає фокус до списку») — taken unamended
- [x] The dialog's existing rules extend unchanged: only changed settings are written, domains are one rule read twice (0–5000 for the delay, 0 legal), Read-only Data disables the controls and OK
- [x] Changing the delay in the dialog demonstrably changes when the count speaks; turning `speakFilteredCount` off demonstrably silences items 9/10/11 without touching anything else
- [x] New dialog msgids shipped in both languages, i18n gate green
- [ ] NVDA reads the three controls and their states on the free native path

## Comments

**2026-08-27 (implementation)** — `Choices` grows from two settings to five, and that is the whole
of the design: the dialog still reports what its controls say, `record_choices` still compares
field by field, and the choice-not-outcome rule therefore extends to the three new fields for free.
The one thing worth naming is what it protects here — a `filteredCountDelayMs` of `99999` in the
file leaves the dialog *showing* 250, the default standing in for it, so an OK over that untouched
field must not write 250 back over what the hand wrote. Three flat fields rather than a record
(§15's own reason) is what makes "one control moved, one field written" true; a test fixes each of
those.

The two typed numbers now share one reading of what a typed number *is* — `typed_number` — with
`in_backup_budget` and `in_count_delay` as the two domains over it. That keeps "one rule read
twice" literally one rule per field, and the delay's second reading earns its own note: an
out-of-domain delay does not merely fail to persist, it falls back to a default that speaks
*sooner*, so a dialog that took `9000` would leave the user believing they had slowed a count that
is still snapping past them. Rejection is one dialog per press, in layout order — budget, then
delay — because two stacked rejections would say the second about a field the user has not been
shown yet, and each carries its own control's words: the message is the whole of what is spoken
(§10) and "must be a whole number" alone would fit both fields.

**A checkbox is its own label**, so the two toggles carry their text on the control and there is no
`StaticText` before them — the same rule the labelled controls follow (the visible text *is* the
accessible name), arriving at a different shape because the widget already has somewhere to put it.
The delay field is labelled the way the budget is. The delay stays **enabled** while the count is
switched off, and not because the two are unrelated: the debounce is what the row rebuild waits on
as well, so the delay still decides when a narrowed list changes under the typist — a control that
greyed itself out there would be saying something untrue.

**Driven live on a staged copy, both languages** (cross-process probes; the app's own `data\`
never touched). The dialog reads, in creation order: `Static` "Language (takes effect after
restart)" · `ComboBox` · `Static` "Snapshots to keep per PATH" · `Edit` `50` · `Button` "Speak
filtered entry counts" **check=1** · `Static` "Delay before speaking the count (ms)" · `Edit` `250`
· `Button` "Escape returns focus to the list" **check=1** · `OK` · `Cancel` — and the same list in
Ukrainian with the assembly's strings. The three behaviours, each measured against the Banner's own
text (which is what `announce()` sets) on a 45-entry User PATH:

- **250 ms (default)** — one character typed, rows 45 → 38, Banner "38 з 45 записів" at **265 ms**.
- **3000 ms, set in the dialog and no restart** — the same gesture, rows 38 → 29, the same
  announcement at **3019 ms**. The delay is in force from the next keystroke.
- **`speakFilteredCount` cleared** — rows still narrow (29 → 12) and the Banner **never changes**
  in eight seconds. Clearing the query then still speaks Announcement 1 ("PATH користувача: 45
  записів"): the switch silences items 9/10/11 and nothing else, which is the bullet's own wording.
- **`searchEscapeReturnsFocus` cleared** — ESC in the Search field leaves focus in the **field**;
  checked, focus lands on the **list**. Both read back through `GetGUIThreadInfo().hwndFocus`.

Also confirmed: `5001` raises the delay's own rejection and never the budget's, the Settings dialog
stays open, and pressing OK again unchanged raises it again — which is how "the field keeps the
text" is asserted from outside the process (cross-process `GetWindowTextW` reads a stale cache for
an `Edit` written with `WM_SETTEXT`, so reading the field back is not the check it looks like).
With both fields wrong the budget's message comes first. `settings.json` gains only what moved —
`{"speakFilteredCount": false, "filteredCountDelayMs": 3000}` over a file whose `language` and
`maxBackups` were left alone. In the unwritable-`data\` run the selector, both fields, **both
checkboxes** and OK all read disabled with their values still shown, Cancel alone alive.

**Still open: the NVDA reading.** The controls are stock `wxCheckBox` (native `Button` answering
`BM_GETCHECK`) and a `TextCtrl` behind a `StaticText`, with zero accessibility calls — ADR-0003's
free native path by construction — but this project measures rather than assumes, and the
unattended harness needs NVDA's logging level at Input/Output, which logs every keystroke on the
machine while it is on. Left for the user to authorise or to walk at the keyboard.

The Release Checklist steps **this** change falsifies were rewritten here rather than left to
ticket 12, which is not told to touch them: B12a's Tab list now names the three controls and their
states, B18's disabled list grows the two checkboxes, and B14a (the delay's rejection and the
two-field order), B22 (the delay in force with no restart), B23 (the count silenced and nothing
else) and B24 (ESC either way) are new. Ticket 12 re-runs them with the rest.
