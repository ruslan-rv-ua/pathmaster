# Research: NVDA-friendly live filter (supports tickets 04 and 06)

Web research gathered 2026-08-26, before building the ticket-04 prototype. Structured as
recommendation-per-question with sources; "no direct guidance found" is stated where true.

## Q1. Debounce delay for the spoken result count

**Recommendation:** ~1–1.5 s after the last keystroke; the best-sourced concrete number is
**1400 ms**. Make the announcement latest-wins (a new keystroke cancels a pending one); reserve
interruption only for the no-results case.

- GOV.UK accessible-autocomplete: `statusDebounceMillis: 1400` default, chosen so typing echo
  finishes before the count speaks, and fixing NVDA announcing stale counts —
  [PR #348](https://github.com/alphagov/accessible-autocomplete/pull/348),
  [repo](https://github.com/alphagov/accessible-autocomplete).
- [aria-autocomplete](https://github.com/mynamesleon/aria-autocomplete): `srDelay` 1400 ms default,
  auto-clears afterwards — corroborates 1400 ms as de-facto standard.
- Scott O'Hara: live region "remain[s] empty until a long enough delay since the last key press";
  no universally correct number (typing speeds differ) —
  [Considering dynamic search results](https://www.scottohara.me/blog/2022/02/05/dynamic-results.html).
- Sara Soueidan: polite for counts, assertive only for "no results"; clear the announcement text
  after ~350–500 ms to avoid duplicate re-announcement —
  [Accessible notifications part 2](https://www.sarasoueidan.com/blog/accessible-notifications-with-aria-live-regions-part-2/).
- Counterpoint: Microsoft Reading List (Win 8.1) announced on every keystroke via assertive live
  region with a settle delay, and still called it "not 100% reliable" —
  [case study part 5](https://learn.microsoft.com/en-us/archive/blogs/winuiautomation/an-accessibility-case-study-reading-list-part-5-live-regions).
  PathMaster's direct NotifyWinEvent channel is more reliable than their winEvent flood, but the
  debounce remains the practitioner recommendation.

## Q2. Wording of the count announcement

**Recommendation:** "N results" / "N results found" with singular/plural handled; a distinct
worded empty state ("No results found") — never silence, never a bare "0".

- GOV.UK: "{N} results are available", empty: "No results found" / "No search results".
- WCAG 2.1 Understanding SC 4.1.3: "5 results returned", "No results returned" as canonical
  examples; announcing a visually-shown "no results" message is effectively AA-required —
  [Understanding 4.1.3](https://www.w3.org/WAI/WCAG21/Understanding/status-messages.html).
- Microsoft Reading List: "{N} results found" / "0 results found".
- O'Hara: announce the no-results state immediately (the one interrupt-worthy case); also bake
  the count into a persistent visible label so it can be re-read on demand.
- No source recommends announcing rows themselves — counts only.

## Q3. NVDA vs. SysListView32 rebuilt while focus is elsewhere

**Recommendation:** background rebuilds should be silent and stale-text-free, but item identity
is positional — never let NVDA's focus/navigator sit on an index that changed meaning. Verify by
prototype; treat "silent during background rebuild" as expected-but-unverified.

- NVDA fetches SysListView32 item text live at query time (in-process helper or
  `VirtualAllocEx` + `LVM_GETITEMTEXTW`), so post-rebuild reads are fresh —
  [sysListView32.py](https://github.com/nvaccess/nvda/blob/master/source/NVDAObjects/IAccessible/sysListView32.py).
  Caveat: properties cached on a live NVDAObject within a speech cycle can still speak stale.
- MSAA childID = 1-based row index → the real "stale row" hazard is identity, not text.
- NVDA propagates list-item state changes only for the focused item, and generally ignores
  background winEvents for speech → mass delete/reinsert under an unfocused list should not chatter.
  No nvaccess/nvda issue found reporting such chatter (searched directly).
- Adjacent known bugs — list mutation can wedge NVDA's focus tracking, recovery = Tab out/in:
  [#5713](https://github.com/nvaccess/nvda/issues/5713),
  [#8825](https://github.com/nvaccess/nvda/issues/8825); robustness context:
  [#2693](https://github.com/nvaccess/nvda/issues/2693),
  [#18706](https://github.com/nvaccess/nvda/issues/18706),
  [#8328](https://github.com/nvaccess/nvda/issues/8328),
  [#13735](https://github.com/nvaccess/nvda/issues/13735).

## Q4. Focus from the field into the results

**Recommendation:** focus stays in the field while typing (never auto-jumps); the user moves
deliberately — Down-arrow (combobox model) and/or Tab. The list must always carry
`LVIS_FOCUSED|LVIS_SELECTED` on some row, re-established immediately after every rebuild, so the
landing reads.

- Microsoft: default focus on the first result item so the user can move through immediately —
  [case study part 3](https://learn.microsoft.com/en-us/archive/blogs/winuiautomation/an-accessibility-case-study-reading-list-part-3-keyboard-accessibility);
  [APG Combobox](https://www.w3.org/WAI/ARIA/apg/patterns/combobox/).
- No NVDA issue found for "focus a just-inserted row reads nothing" specifically; the plausible
  failure (unsourced, prototype-flagged): re-focusing the *same index* after a rebuild may raise
  no focus winEvent → silence. #5713/#8825 show the category is real.
- If prototyping shows silence, announcing the focused row through the app's own channel is the
  fallback — but test first; double-speaking is the commoner failure.

## Q5. ESC-to-clear

**Recommendation:** ESC clears the text and restores the unfiltered list; per Windows/ARIA
convention focus **stays in the field** (the PRD wants return-to-list — prototype tests both);
announce the restored state (e.g. "Filter cleared, N entries").

- APG Combobox: clearing on ESC is optional; [w3c/aria-practices #1066](https://github.com/w3c/aria-practices/issues/1066)
  argues against universal clearing.
- Windows 8.1 SearchBox self-describes "escape to clear text" — ESC-clears is the native convention.
- What to announce after clearing: no direct guidance found; announcing the restored count is
  consistent with SC 4.1.3.

## Q6. wxMSW frequent wxListCtrl updates

**Recommendation:** `Freeze()`/`Thaw()` around the whole batch; `DeleteAllItems()` + reinsert
(one bulk event, not N per-item events); `LVS_EX_DOUBLEBUFFER` for flicker; re-establish the
focused item before Thaw. Virtual mode avoids rebuilds but changes the a11y surface — prototype
before committing.

- [wxWidgets forum on flicker](https://forums.wxwidgets.org/viewtopic.php?t=37101),
  [Flicker-Free Drawing wiki](https://wiki.wxwidgets.org/Flicker-Free_Drawing).
- `DeleteAllItems` deliberately sends a single `wxEVT_LIST_DELETE_ALL_ITEMS` — less winEvent noise.
- Freeze/Thaw = `WM_SETREDRAW`, which suppresses painting, not accessibility events; no
  documentation found on NVDA behaviour under it — low-risk but unverified, hence the prototype's
  plain-vs-Freeze/Thaw toggle.

## Order of operations per rebuild (cross-cutting)

Freeze → rebuild rows → set `LVIS_FOCUSED|LVIS_SELECTED` on the target row → Thaw → (debounced)
announce count, latest-wins.
