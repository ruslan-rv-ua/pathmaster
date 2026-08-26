# Research: search bar contract (supports ticket 06)

Web research gathered 2026-08-26, before grilling, per the map's standing directive 7. Structured as
recommendation-per-question with sources; "no direct guidance found" is stated where true. This file
does **not** repeat [04-live-filter-best-practices.md](04-live-filter-best-practices.md) (debounce
value, count wording, NVDA rebuild behaviour, focus-into-results) or
[05-var-expansion-best-practices.md](05-var-expansion-best-practices.md) Q5 (match displayed vs raw);
both are load-bearing here and are cited rather than restated.

## Q1. Persistent search field vs a find bar that appears on Ctrl+F

**Recommendation: a persistent field, always visible above the list, reached by an accelerator.**
The two precedents split by *kind*, not by taste: a **find bar** (transient, "go to the next match",
nothing changes while it is closed) hides and closes — Firefox, VS Code, Thunderbird's Quick Filter;
a **filter field** over a list (the visible set *is* the feature's output) stays put. PathMaster's is
the second kind. Two further reasons are local rather than sourced: v0.1.0's layout rule is that the
window never reflows under the user (spec §12), and a field that appears and disappears is a
Tab-order that changes shape.

- **NVDA's own GUI is the strongest precedent available for this product** — screen-reader-first,
  wxPython, list-plus-filter. The Add-on Store: "To search, press `alt+s` to jump to the 'Search'
  field and type the text to search for… The list updates while typing the search terms" — a
  permanent field with a mnemonic, live filtering, no toggle
  ([NVDA User Guide](https://download.nvaccess.org/documentation/userGuide.html)).
  The Input Gestures dialog does the same: Shift+Tab or **Alt+F** moves to a filter edit field that
  is simply part of the dialog (same guide;
  [nvaccess/nvda#4458](https://github.com/nvaccess/nvda/issues/4458) is the request that built it,
  [PR #10307](https://github.com/nvaccess/nvda/pull/10307) made its filtering async for
  speed-typing — the same latency concern ticket 04's debounce answers).
- The toggle pattern's own bug list argues against it for a keyboard-first app: Thunderbird's Quick
  Filter is shown by Ctrl+Shift+K and hidden by Esc, but the shortcut does **not** toggle it back
  ([Quick Filter Toolbar](https://support.mozilla.org/en-US/kb/quick-filter-toolbar)), closing it is
  a known usability bug ([bug 587478](https://bugzilla.mozilla.org/show_bug.cgi?id=587478)), users
  report being stranded in a filtered mailbox without knowing why
  ([support thread](https://support.mozilla.org/en-US/questions/1172959)), and NVDA has an open
  navigation defect against that very bar
  ([nvaccess/nvda#17657](https://github.com/nvaccess/nvda/issues/17657)).
- Discoverability of hidden controls is the standing UX finding — hidden search features go unused
  when users cannot find them, and users spend a large share of their time locating filters at all
  ([NN/g filters course](https://www.nngroup.com/contents/self-paced-courses/filters-and-sorting-the-complete-design-guide/),
  [UXPin filter UI](https://www.uxpin.com/studio/blog/filter-ui-and-ux/) *(secondary)*). The
  persistent-panel pattern is recommended for data-heavy screens where filters are adjusted often;
  the drawer pattern for space-constrained ones.
- Cost, stated honestly: a permanent field is one extra Tab stop before the list on every run,
  including runs where nobody searches. No source weighs that against the alternative — it is a
  judgement, not a finding.

## Q2. What Ctrl+F does when the field already holds text

**Recommendation: focus the field and select its whole contents**, so the next keystroke replaces the
old query and Ctrl+F is idempotent. No sourced rule was found for *filter* fields; the find-dialog
convention is to pre-fill and select.

- Visual Studio pre-populates "Find what" from the caret's word on Ctrl+F
  ([Finding and replacing text](https://learn.microsoft.com/en-us/previous-versions/visualstudio/visual-studio-2017/ide/finding-and-replacing-text?view=vs-2017);
  [Additional Tips](https://www.oreilly.com/library/view/coding-faster-getting/9780735662155/apbs05.html)
  *(secondary)*). VS Code seeds its find widget from the selection
  ([microsoft/vscode#95692](https://github.com/microsoft/vscode/issues/95692) discusses the adjacent
  find-in-selection behaviour). Both replace, rather than append to, what was there.

## Q3. ESC when the field is already empty

**Recommendation: one gesture, one meaning — ESC always leaves the field.** With text: clear it,
then return focus to the list. Empty: return focus to the list and say nothing (nothing changed).

- The nearest documented convention is Firefox DevTools', and it is explicitly two-tier: "If the
  input has focus and a non-empty value, clear that value. If the input has focus and an empty
  value, let the Esc key be handled by the panel (e.g. to close a modal or toggle the Split
  Console)" ([bug 1561585](https://bugzilla.mozilla.org/show_bug.cgi?id=1561585)). ESC-clears in
  `<input type=search>` is Chrome/Safari behaviour and was requested for Firefox in
  [bug 1055085](https://bugzilla.mozilla.org/show_bug.cgi?id=1055085) and
  [bug 1490916](https://bugzilla.mozilla.org/show_bug.cgi?id=1490916).
- The second tier assumes there is something to close. In a persistent field (Q1) there is not —
  so the honest translation of "let the panel handle it" is the app's own answer to "I am done
  here", which for a list-plus-filter is the list. Windows 8.1's SearchBox self-describes "escape
  to clear text", confirming ESC-clears as the native convention (via ticket 04's research, Q5).
- ESC-to-list vs ESC-stays-in-field was settled by measurement, not by this file: ticket 04's NVDA
  session chose ESC-to-list as the default, with a setting to reverse it.

## Q4. Matching semantics for Windows paths

**Recommendation: case-insensitive substring over the text the row displays, and nothing more —
with one candidate exception, `/`→`\` folding, which has real Windows-search precedent.**
Normalising the *haystack* (stripping quotes, trimming a trailing `\`) would break the property that
makes the count trustworthy: that every matched row visibly contains what was typed. A search for
`"` must find the `Quoted` entries — that is exactly when a user goes looking for them.

- Slash folding is standard in Windows path search: Everything replaces forward slashes with
  backslashes, and that option is on by default in current releases
  ([voidtools forum](https://www.voidtools.com/forum/viewtopic.php?t=15831),
  [Options](https://mail.voidtools.com/support/everything/options/),
  [ignore-punctuation thread](https://www.voidtools.com/forum/viewtopic.php?t=10042)). It is
  input-forgiveness, not hidden-text matching: the row still contains the typed segment, spelled
  with the other slash.
- Case-insensitivity by default is the settled default for user-facing search; "smart case"
  (case-sensitive only when the query contains an uppercase letter) is ripgrep's `-S` and remains
  an unimplemented request in VS Code
  ([microsoft/vscode#41119](https://github.com/microsoft/vscode/issues/41119),
  [ripgrep discussion #1594](https://github.com/BurntSushi/ripgrep/discussions/1594)). Nothing
  found argues for smart case in a *filter* field, where queries are short and typed in lowercase.
- Case folding must be Unicode-aware, not ASCII: this machine's own PATH contains
  `C:\Users\Руслан`. Rust's `str::to_lowercase` is full Unicode; ASCII-only folding is wrong here.
  (No external source needed — it is a property of the data.)
- Which text is the haystack — displayed or raw — is answered in
  [05's Q5](05-var-expansion-best-practices.md): match the currently displayed rendering, so the
  spoken count equals the rows the user will hear.

## Q5. The counter — wording and where it lives

**Recommendation: the Banner carries the spoken count (it is an Announcement, and v0.1.0 forbids
audio-only ones); the StatusBar carries the same "N of M" as passive, on-demand text.** No new
widget and no new Tab stop; `NVDA+End` re-reads it whenever the Banner has moved on to something
else.

- Persisting the count somewhere re-readable is the practitioner recommendation — bake it into a
  visible label rather than leaving it only in the live region
  ([Scott O'Hara, dynamic results](https://www.scottohara.me/blog/2022/02/05/dynamic-results.html)).
- "Showing X of Y" is a recognised pattern precisely because the sighted user watches the number
  fall while the screen-reader user hears nothing unless it is announced
  ([Sara Soueidan, live regions part 2](https://www.sarasoueidan.com/blog/accessible-notifications-with-aria-live-regions-part-2/)).
  Announcing a count on *every* keystroke is what the same source warns against — ticket 04's
  debounce is the answer to that.
- WCAG SC 4.1.3's own examples are "5 results returned" / "No results returned"; GOV.UK ships
  "{N} results are available" and "No results found"
  ([Understanding 4.1.3](https://www.w3.org/WAI/WCAG21/Understanding/status-messages.html); via
  ticket 04's research Q2). None of them carry the total — "of M" is PathMaster's own addition,
  and it is defensible here because M is the count the same Scope announces on activation.
- Ukrainian trap, no source needed: in «{n} з {m} записів» the noun is governed by **{m}**, not
  {n}, so the plural form must be selected by the total. The i18n gate checks plural presence, not
  which number picked the form.
- NVDA does not supply this for free: "Search fields: announce suggestion count"
  ([nvaccess/nvda#7330](https://github.com/nvaccess/nvda/issues/7330)) is an open request, and a
  vendor bug of exactly this shape is on record
  ([ServiceNow KB1005128](https://support.servicenow.com/kb?id=kb_article_view&sysparm_article=KB1005128)).

## Q6. The empty result set

**Recommendation: an empty list — no placeholder row — plus a distinct spoken message; never a bare
"0", never silence.**

- Empty states should *replace* the content rather than decorate it, and empty list items confuse
  screen reader users ([Carbon empty states](https://carbondesignsystem.com/patterns/empty-states-pattern/),
  [Telerik ListView a11y](https://www.telerik.com/design-system/docs/components/listview/accessibility/)).
- An empty state that appears as a result of the user's action is a status change and must be
  announced
  ([Semrush widget-empty a11y](https://developer.semrush.com/intergalactic/components/widget-empty/widget-empty-a11y),
  [ICDS empty state](https://design.sis.gov.uk/components/feedback-progress/empty-state/accessibility),
  [Accessibility.build on empty states](https://accessibility.build/blog/accessible-loading-empty-error-states)).
- Every search state — including "successful with no results" — needs a visual message, a
  programmatic message, and a **focus decision** (same source). PathMaster's focus decision already
  exists: ticket 03's rule 4.3 — focus stays on the empty list and never jumps to the field
  uninvited.
- v0.1.0 already took this position for its own zero case: "User PATH: no entries" is its own
  msgid, and there are no placeholder rows (spec §10.1).

## Q7. Does the query survive the run?

**No direct guidance found**, and the domain answers it anyway: an Editing Session never survives a
process boundary (`CONTEXT.md`), and ticket 03 recorded non-persistence as the natural consequence.
The only external precedent found points the same way — a find bar's contents are per-tab and
per-session, never per-installation.

## Q8. Settings field naming and validation

**Recommendation: flat `camelCase` fields, one per setting, each with its own default.** The
existing file is `language`, `maxBackups`, `window` — and §13's rule that geometry falls back "as a
unit under that one name" is a rule *for records whose members have no individual defaults*. These
three do have their own defaults, so a record would let an unrelated typo silently reset all three.

- The invalid-value contract already exists and needs no new precedent: known field, out-of-domain
  value → that field's default **in memory**, the file keeps the raw text, one `WARN` line, no
  dialog, no clamping (spec §13). What this ticket owes is the *domain* of each new field.
- No external source prescribes a debounce range. The bounds worth writing down are behavioural:
  `0` is meaningful (announce as soon as the keystroke settles) and an upper bound exists so a
  typo cannot silently mute a feature the user believes is on.
