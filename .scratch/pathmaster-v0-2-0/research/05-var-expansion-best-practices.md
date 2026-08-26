# Research: raw vs expanded PATH-entry display toggle (supports ticket 05)

Web research gathered 2026-08-26. Structured as recommendation-per-question with sources;
"no direct guidance found" is stated where true.

## Q1. Scope & persistence of a raw/computed display toggle

**Recommendation:** scope the toggle to the pane that shows the data — for PathMaster's single
list that means effectively app-wide — and **persist it across restarts** (a saved setting the
toggle mutates, not a per-session flag). Raw stays the default view (it is the stored truth,
matching the Windows System PATH editor and PowerToys' variable list).

- Excel Show Formulas (Ctrl+`), the closest analogue (raw formula vs computed value): scoped to
  the *active worksheet only*, and **saved with the workbook** — it is the `showFormulas`
  attribute of OOXML `sheetView` (per-sheet, persisted in the file):
  [ECMA-376 sheetView](https://c-rex.net/samples/ooxml/e1/Part4/OOXML_P4_DOCX_sheetView_topic_ID0ELLI5.html),
  [datypic schema doc](http://www.datypic.com/sc/ooxml/e-ssml_sheetView-1.html),
  [DevExpress mirror of the semantics](https://docs.devexpress.com/OfficeFileAPI/DevExpress.Spreadsheet.WorksheetView.ShowFormulas)
  ("current worksheet only… saved to a document"). The UI equivalent lives under Excel Options →
  Advanced → *Display options for this worksheet* —
  [Microsoft: Show and print formulas](https://support.microsoft.com/en-us/office/show-and-print-formulas-65a29965-b1b1-40db-9cb7-4fd051da3a5c).
- VS Code `editor.renderWhitespace`: a persisted setting (user or workspace `settings.json`);
  the "Toggle Render Whitespace" command flips the stored setting, so the toggle survives
  restarts — [VS Code settings docs pattern](https://code.visualstudio.com/docs/getstarted/settings),
  [worked examples](https://bobbyhadz.com/blog/render-whitespace-vscode).
- File Explorer view modes: remembered per folder type and persisted (historically per-folder
  "Remember each folder's view settings", up to a saved-views cap) —
  [Microsoft Q&A on per-folder views](https://learn.microsoft.com/en-us/answers/questions/5677317/how-to-set-view-differently-in-each-folder).
  Same pattern: scoped to the thing viewed, persisted.
- Word field codes (raw `{ FIELD }` vs result): Alt+F9 toggles the whole document, backed by the
  option File → Options → Advanced → Show document content → *Show field codes instead of their
  values* — [Microsoft Q&A](https://learn.microsoft.com/en-us/answers/questions/5036589/toggling-field-codes-in-word-not-working).
- Environment-variable editors specifically: **no raw/expanded view toggle found anywhere.**
  PowerToys Environment Variables shows the raw list and additionally pins one "Evaluated Path
  variable value… at the top of the list" (both at once, no mode) —
  [Microsoft Learn](https://learn.microsoft.com/en-us/windows/powertoys/environment-variables);
  internally it deliberately "reads variables directly from registry instead of using Environment
  API to prevent automatic variable expansion" —
  [devdocs](https://github.com/microsoft/PowerToys/blob/main/doc/devdocs/modules/environmentvariables.md).
  RapidEE shows raw values and offers expansion on demand (expand-with-a-click, hints/Inspector
  for expandable strings, expands before path-correctness checks) — no persisted display mode
  documented — [RapidEE history](https://www.rapidee.com/en/history). So a persistent toggle
  would be novel in this niche; the general-Windows precedent above is what carries.

## Q2. Screen-reader announcement of the mode toggle (NVDA)

**Recommendation:** make the menu item a **check item with a constant label** (wxITEM_CHECK,
e.g. "Expanded values" checked/unchecked) — the checked menu item is the canonical, re-visitable
state indicator. On activation, announce the *resulting state* through the existing NotifyWinEvent
channel ("Showing expanded values" / "Showing raw entries"), not the action taken. Verify the
menu's "checked" reading with live NVDA per project practice.

- Native Win32 menu items expose `STATE_SYSTEM_CHECKED` via MSAA —
  [Microsoft: menu item (MSAA)](https://learn.microsoft.com/en-us/windows/win32/winauto/menu-item).
  wxMSW check items are native `HMENU` items, so the state is exposed for free. NVDA's known
  failure to speak "checked" is specific to WinForms `ToolStripMenuItem`
  ([nvaccess/nvda#19281](https://github.com/nvaccess/nvda/issues/19281) — Narrator handles it,
  NVDA doesn't); no NVDA issue found for native menus. *Inference that native works — needs the
  usual live-NVDA proof.* Adjacent caution: ARIA `menuitemcheckbox` as a *first* menu item going
  unspoken ([#14550](https://github.com/nvaccess/nvda/issues/14550)) is web-role-specific.
- ARIA: checkable menu items are their own role with `aria-checked`, and "changes in states or
  properties will result in a notification to assistive technologies" —
  [WAI-ARIA 1.2 menuitemcheckbox](https://www.w3.org/TR/wai-aria-1.2/#menuitemcheckbox). WCAG
  4.1.2 requires the state be programmatically determinable; the check state satisfies that.
- Keep the label constant and flip only the state — never both: changing name and checked state
  in tandem is the documented confusion case —
  [Sarah Higley, Playing with state](https://sarahmhigley.com/writing/playing-with-state/).
  (This argues against a label-swapping "Show expanded" ↔ "Show raw" item.)
- What to announce: WCAG 4.1.3's canonical status-message examples are result-state phrasings
  ("5 results returned"-style, i.e. what is now true, not what was done) —
  [Understanding SC 4.1.3](https://www.w3.org/WAI/WCAG22/Understanding/status-messages.html).
  "Showing expanded values" fits the pattern.
- Practical point (reasoning, not sourced): activating a menu item closes the menu, so the user
  never hears the new checked state at activation time — the app announcement is the only
  feedback in the moment; the checked item is how the mode stays *discoverable* afterwards
  (reopen menu, hear "Expanded values checked"). No source recommends repeating the mode on
  every subsequent action or parking it in a status bar; no direct guidance found beyond 4.1.3.

## Q3. Displaying an entry whose %VAR% is undefined, in expanded mode

**Recommendation:** leave the unexpandable `%VAR%` **literal, in place** — this is the
Windows-owned convention — and keep "something is wrong" in the diagnostics channel (the
existing Missing issue), not in the displayed text. No inline warning-marker precedent found.

- `ExpandEnvironmentStrings`: "If the name is not found, the %variableName% portion is left
  unexpanded" — [Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/processenv/nf-processenv-expandenvironmentstringsw).
  This is the API that expands `REG_EXPAND_SZ` PATH itself, so literal-in-place is what Windows
  does with the real PATH.
- cmd.exe interactive: undefined `%VAR%` is left literal on the command line (batch files expand
  to empty — a divergence to know about, not to copy) —
  [SS64 syntax-variables](https://ss64.com/nt/syntax-variables.html),
  [Rob van der Woude](https://www.robvanderwoude.com/battech_defined.php).
- PowerShell: `$env:VAR` for a nonexistent variable returns `$null` (renders empty) —
  [about_Environment_Variables](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_environment_variables?view=powershell-7.4).
  The empty-string school exists, but it destroys information on screen; the API/cmd literal
  convention preserves it and needs no extra marker.
- RapidEE expands before checking path correctness and flags broken paths via its
  error-highlighting, not by rewriting the displayed value —
  [RapidEE history](https://www.rapidee.com/en/history). PowerToys shows no per-entry expansion
  state at all ([Learn page](https://learn.microsoft.com/en-us/windows/powertoys/environment-variables)).
- Inline markers / a11y guidance against status-in-text: **no direct guidance found.** Note the
  literal-in-place convention has a built-in property that suits a screen reader: an undefined
  variable *sounds different* in expanded mode (you still hear "percent JAVA underline HOME
  percent") — the anomaly is audible without any marker, and the Missing issue already names it.

## Q4. Editing raw text while the expanded view is shown

**Recommendation:** edit-reveals-raw with **no change to the list** while the modal Edit dialog
is open — the dialog showing raw text over a list showing expanded values matches Excel's
formula-bar model exactly. Keep the view all-or-nothing; per-row mixing has precedent (Word)
but nothing recommends it and Excel's toggle is strictly all-or-nothing per sheet.

- Excel: the cell shows the computed value while the formula bar simultaneously shows the raw
  formula for the selected cell; F2 / in-cell edit reveals and edits the raw formula —
  [Lenovo glossary summary of formula-bar behavior](https://www.lenovo.com/us/en/glossary/formula-bar/),
  [F2 edit-mode mechanics](https://excelribbon.tips.net/T006174_Activating_the_Formula_Bar_with_the_Keyboard.html).
  Raw-in-editor coexisting with computed-in-grid is the established, unremarked-on norm — the
  grid does not switch modes when editing starts.
- What the list should do while a modal edit dialog is open: **no direct guidance found.**
  Excel's precedent implies "nothing" — the underlying view keeps its mode; the dialog is the
  raw surface. (On OK, the row re-renders in the current view mode.)
- Mixed display precedent: Word toggles field codes per single field (Shift+F9) vs whole
  document (Alt+F9) —
  [Microsoft Q&A](https://learn.microsoft.com/en-us/answers/questions/5068476/word-field-problem-shift-f9) —
  so per-row mixing exists in one major app; Excel's Ctrl+` has no per-cell variant (a cell
  formatted as Text showing its formula is an accident, not a mode). No guidance found that
  favors mixing; for a list whose rows are announced positionally, uniform rendering is the
  conservative choice.

## Q5. What does search/filter match — displayed or raw text?

**Recommendation:** match the **currently displayed rendering**, so the filter result count and
audible row text are self-consistent with what NVDA reads; document the behavior. (If both-mode
matching ever feels needed, Excel's precedent is an explicit user choice, defaulting to raw —
not silent both-matching.)

- Excel Find exposes the choice directly: "Look in: Formulas / Values / Notes / Comments", and
  on the **Replace** tab only Formulas is available — i.e. Excel refuses to *edit* through the
  computed rendering; mutation binds to raw —
  [Microsoft: Find or replace text and numbers on a worksheet](https://support.microsoft.com/en-us/office/find-or-replace-text-and-numbers-on-a-worksheet-0e304ca5-ecef-4808-b90f-fdb42f892e90)
  ("Formulas, Values, Notes and Comments are available only on the Find tab; only Formulas are
  available on the Replace tab"). Semantics of the two options (raw formula text vs displayed
  result) corroborated by
  [Ablebits](https://www.ablebits.com/office-addins-blog/excel-find-replace/) *(secondary source)*.
- VS Code search runs over the raw file text — which *is* the displayed text (no dual rendering
  exists), so it supports "match what is shown" rather than a raw-vs-displayed split —
  [VS Code code-basics docs](https://code.visualstudio.com/docs/editor/codebasics).
- Accessibility angle: **no direct guidance found** in WCAG on matched-vs-shown consistency.
  Nearest principle is SC 4.1.3's announced-count examples (see the ticket-04 research file):
  the spoken "N results" must be the N of rows the user will actually hear when arrowing —
  which displayed-text matching guarantees by construction and raw-matching can violate in
  expanded mode (a row matching on hidden raw text would be audibly inexplicable).

## Cross-cutting note

Every strong analogue (Excel, Word, VS Code) treats the raw text as the single editable truth
and the computed rendering as a read-only projection with its own persisted visibility flag —
exactly PathMaster's existing model. Nothing found argues for severity classes, inline markers,
or per-session reset.
