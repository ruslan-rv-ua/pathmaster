# Research: tree browser contract (supports ticket 08)

Web research gathered 2026-08-26, before grilling, per the map's standing directive 7. Structured as
recommendation-per-question with sources; "no direct guidance found" is stated where true. This file
does **not** repeat [01-wxdragon-widget-surface-v0-2-0.md](01-wxdragon-widget-surface-v0-2-0.md)
(wxTreeCtrl is bound and is native `SysTreeView32` on MSW),
[04-live-filter-best-practices.md](04-live-filter-best-practices.md) (rebuild-under-focus hazards,
debounced count), [05-var-expansion-best-practices.md](05-var-expansion-best-practices.md)
(raw-mode-per-Run, undefined `%VAR%` display), or
[06-search-bar-best-practices.md](06-search-bar-best-practices.md) (Search matches the displayed
rendering); all are cited rather than restated.

## Q1. What the tree is for, and what Enter should do

**Recommendation: the tree is a COMPREHENSION surface with direct navigation as its exit — not a
search-input device. Enter on a leaf selects the matching row in the main list and closes the
dialog; Enter on an inner node expands/collapses it (the native default action). Drop the PRD's
"fill the Search bar" coupling entirely: no surveyed tool couples tree activation to a search
field, the pattern's own trap (expanded tree text vs raw-mode list, 06's handoff) makes the
feature's normal result an empty list, and a two-hop indirection (tree → search → list) is strictly
worse for an NVDA user than one hop (tree → row).**

- Every surveyed tree-over-PATH-like-data activates DIRECTLY. Rapid Environment Editor — the
  closest product — "unwraps" PATH into tree branches that are themselves the editing surface:
  select to edit in place, drag to reorder, red highlight for broken paths
  ([rapidee.com/en/path-variable](https://www.rapidee.com/en/path-variable),
  [rarst.net review](https://www.rarst.net/software/rapidee/) *(secondary)*). No search-field
  coupling exists in it.
- The Windows canon for "tree node drives another view" is the two-pane master-detail: "these tree
  views have an associated control that displays the content of the selected container" — Explorer's
  navigation pane, Regedit ([UX guide, Tree Views, usage
  patterns](https://learn.microsoft.com/en-us/windows/win32/uxguide/ctrl-tree-views)). Selection
  navigates; nothing is typed anywhere.
- Folder pickers commit the tree selection as the dialog's result: `SHBrowseForFolder` returns the
  selected item when OK is pressed
  ([SHBrowseForFolderW](https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shbrowseforfolderw)).
  Its optional `BIF_EDITBOX` text field is an *input* the user types into, validated via
  `BFFM_VALIDATEFAILED` — the tree does not exist to fill it
  ([BROWSEINFOW](https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/ns-shlobj_core-browseinfow)).
  **No precedent found anywhere for "activate tree node → populate a search/filter field".**
- The ARIA APG defines Enter's contract: "Activates a node, i.e., performs its default action. For
  parent nodes, one possible default action is to open or close the node"; in single-select trees
  the leaf default is selection
  ([APG Tree View pattern](https://www.w3.org/WAI/ARIA/apg/patterns/treeview/)). Leaf-navigates /
  parent-toggles is the standard split the recommendation adopts.
- Search and tree are peers, not a pipeline: "large, complex trees need to be supplemented with
  other access methods, such as word search, an index, or filtering"
  ([UX guide, Tree Views](https://learn.microsoft.com/en-us/windows/win32/uxguide/ctrl-tree-views)).
  PathMaster already has the Search field per Scope (06); the tree adds the one thing search cannot
  give — the *shape* of a long PATH (which directories carry many entries, what clusters where).
  That is comprehension; the leaf-jump is its exit ramp.
- If "show me every entry under C:\Program Files" is ever wanted, it is a FILTER feature, and 06/07
  already own where filters live and what they announce; wiring it through the visible Search text
  in a different rendering mode than the list is the one design that cannot work (the trap 06
  handed this ticket).

## Q2. NVDA over native SysTreeView32, and per-item status

**Recommendation: rely on what NVDA gives for free — name, expanded/collapsed, "level N", "N of M"
among siblings, child count on expand, arrows/Home/End/numpad-* and native first-letter search —
and carry Issue status as a label suffix using the exact Status-column words ("bin — Missing"),
because the item LABEL is the only speakable channel a native tree item has. Verify live per the
standing NVDA-proof rule.**

- NVDA's `SysTreeView32` support (its own source): position is computed by walking `TVGN_PREVIOUS`/
  `TVGN_NEXT` into `indexInGroup`/`similarItemsInGroup` (spoken as "N of M" among *siblings*);
  level comes from the item's MSAA `accValue` (spoken as "level N" when it changes); expanding a
  node speaks "%s items" from the child count
  ([sysTreeView32.py](https://github.com/nvaccess/nvda/blob/master/source/NVDAObjects/IAccessible/sysTreeView32.py)).
  Note the corollary: `accValue` is spent on level and there is no description/columns — the label
  is the whole audible payload.
- Native keyboard behavior comes free: WM_KEYDOWN moves the caret for "direction keys or the PAGE
  UP, PAGE DOWN, HOME, END" keys and sends NM_RETURN on Enter; numpad * expands a subtree
  recursively, +/- expand/collapse one node
  ([About Tree-View Controls, default message
  processing](https://learn.microsoft.com/en-us/windows/win32/controls/tree-view-controls),
  [TestComplete's key table](https://support.smartbear.com/testcomplete/docs/app-objects/specific-tasks/standard/tree-view/expanding-collapsing-items.html)
  *(secondary)*). Incremental first-letter search is native too — the control keeps an
  incremental-search string
  ([TVM_GETISEARCHSTRING](https://learn.microsoft.com/en-us/windows/win32/controls/tvm-getisearchstring)) —
  and the APG recommends type-ahead "for all trees, especially trees with more than 7 root nodes"
  ([APG Tree View](https://www.w3.org/WAI/ARIA/apg/patterns/treeview/)). A one-root-per-drive tree
  gets typeahead over drive letters and folder names with zero code.
- Known NVDA×tree defects are peripheral, not blocking: wrong expand/collapse phrases during
  *object navigation* ([#2805](https://github.com/nvaccess/nvda/issues/2805), old), UIA event
  floods making NVDA sluggish are a WPF problem
  ([#11109](https://github.com/nvaccess/nvda/issues/11109)), and NVDA's own Input Gestures dialog
  is itself a wx tree view ([#6349](https://github.com/nvaccess/nvda/issues/6349) complains only
  about its size) — the toolkit's tree is daily-driven by NVDA's own users. No issue found against
  wxTreeCtrl/SysTreeView32 announcements themselves.
- Label-suffix legitimacy: Microsoft's Name guidance forbids only role/type words in the Name
  ("must not include the control role or type information, such as 'button' or 'list'")
  ([Expose basic accessibility
  information](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/basic-accessibility-information));
  "Missing" is data, not role. The anti-precedent shows why the suffix is needed: Device Manager
  conveys per-item problem state by icon overlay (yellow badge)
  ([Dell KB on unknown devices](https://www.dell.com/support/kbdoc/en-us/000151898/how-to-identify-an-unknown-device-in-device-manager)
  *(secondary)*), and native overlays are exactly the visual-only channel a tree item's speech
  never carries. Using the same closed-catalogue Status words the main list already speaks (04/06)
  keeps one vocabulary across both surfaces. Direct "suffix is the accepted pattern" guidance for
  native trees: **no direct guidance found — judgement** from the channel analysis above.

## Q3. Entries that don't fit the filesystem shape

**Recommendation: a separate top-level group node per class — e.g. "Unresolved variables" holding
undefined-`%VAR%` entries as leaves (shown literal, per 05 Q3) and "Relative entries" holding
relative ones — never exclusion. A tree titled over "all PATH entries" that silently drops the
pathological ones would hide precisely the entries a diagnostics tool exists to surface.**

- The out-of-hierarchy group node is a settled Windows pattern: Device Manager parks everything it
  cannot classify under the root-level category "Other devices"
  ([Dell KB](https://www.dell.com/support/kbdoc/en-us/000151898/how-to-identify-an-unknown-device-in-device-manager),
  [Microsoft Q&A example](https://learn.microsoft.com/en-us/answers/questions/3889027/unrecognized-devices-in-device-manager)).
- WinDirStat inserts bracket-marked pseudo-items `<Free Space>` and `<Unknown>` alongside real
  directories rather than distorting the hierarchy or dropping the data
  ([WinDirStat legend](https://documentation.help/WinDirStat/legend.htm),
  ["What is Unknown?"](https://blog.windirstat.net/20061013/unknown-space/)). The bracket styling
  is a usable convention for "not a real folder" — though for NVDA the group's *name* does the
  work, not punctuation.
- Visual Studio's Solution Explorer does the same for files outside the project model: a
  "Miscellaneous Files" folder, present but distinct
  ([Work with miscellaneous files](https://learn.microsoft.com/en-us/visualstudio/ide/miscellaneous-files?view=vs-2022)).
- The UX guide's tree-organization rule backs one-home-per-item: "place each object under the
  single most appropriate container", and "determine if you really need a root node" — the groups
  sit at top level next to the drive roots; no artificial "PATH" super-root is needed
  ([UX guide, Tree Views](https://learn.microsoft.com/en-us/windows/win32/uxguide/ctrl-tree-views)).
  Group labels are user-visible strings and belong in the Catalogue like every other message.

## Q4. Live diagnostics under a modal: snapshot vs live

**Recommendation: snapshot at open. The dialog is a short-lived, one-off navigation/comprehension
surface — the modal pattern's own definition — and every mechanism that would make it live is
either silent to a screen reader (native trees have no free live-region channel), an announcement-
churn hazard, or the v0.1.0 timer-in-modal borrow hazard again. The main list stays the authority
on current Issue labels; activating a leaf lands the user there anyway. No refresh affordance
needed; reopening is the refresh.**

- Modal dialogs are for "critical or infrequent, one-off tasks that require completion before
  continuing" ([UX guide, Dialog
  Boxes](https://learn.microsoft.com/en-us/windows/win32/uxguide/win-dialog-box)) — a surface the
  user is expected to finish and leave, not monitor. No guidance found that requires (or even
  discusses) refreshing a modal's data mid-flight; **no direct guidance found** on
  snapshot-vs-live for dialogs specifically.
- Making updates *audible* in Win32 is opt-in work, not a default: a plain label change in a native
  control announces nothing; the app must raise `EVENT_OBJECT_LIVEREGIONCHANGED` with a LiveSetting
  property, or a UIA Notification event
  ([How to have important changes in your Win32 UI announced by
  Narrator](https://learn.microsoft.com/en-us/archive/blogs/winuiautomation/how-to-have-important-changes-in-your-win32-ui-announced-by-narrator),
  [UIA Notification event](https://learn.microsoft.com/en-us/archive/blogs/winuiautomation/can-your-desktop-app-leverage-the-new-uia-notification-event-in-order-to-have-narrator-say-exactly-what-your-customers-need)).
  So "live labels" in this tree are, to the focused NVDA user, silent mutations — the worst of both
  worlds: churn risk without information.
- Where live updates DO reach the screen reader, the guidance is all about restraint: assertive
  interruptions are "extremely annoying and disruptive", and "if there is a focus change to the
  element being updated, then a live region is not needed"
  ([MDN, ARIA live regions](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Guides/Live_regions),
  [HTMHell live-regions deep dive](https://www.htmhell.dev/adventcalendar/2023/22/) *(secondary)*).
  The event-flood failure mode is on record: NVDA "bombarded" by tree UIA events becomes sluggish
  ([nvaccess/nvda#11109](https://github.com/nvaccess/nvda/issues/11109)); and 04 Q3 already
  documents the sibling hazard of rebuilding a control the user is standing in.
- Focus stability is the other half: mutating tree structure (an entry gaining/losing an Issue does
  not move nodes, but a diagnostics-driven rebuild would) risks the focused item vanishing under
  NVDA mid-arrow-stroke — the exact class of failure 04's rebuild rules exist to prevent. A
  snapshot makes the whole class unreachable, and also keeps ticket 14's mechanism decision fully
  decoupled from this dialog (no timer needs to tick inside its event loop — the v0.1.0 hazard).

## Q5. Dialog buttons, Enter conflicts, and the shortcut

**Recommendation: commit buttons "Go to entry" (default) + Cancel, Esc closes; no OK. The
Enter-vs-default-button conflict resolves itself in wx: wxTreeCtrl eats unmodified Enter and turns
it into item-activation, so Enter-in-tree runs the activation logic and the default button fires
only when focus is on it — but per the UX guide the Go-to button must exist anyway as the
redundant, visible form of the double-click/Enter action. For the shortcut: NOT Alt+T — Windows
guidance reserves Alt+letter for access keys; use a Ctrl letter from the recommended
non-conflicting set (Ctrl+T is in it) on the View-menu item that opens the dialog.**

- Button set: for modal choice dialogs the pattern is "OK/Cancel or [Do it]/Cancel", with the
  strong preference for "positive commit buttons that are specific responses to the main
  instruction, instead of generic labels such as OK"; Cancel must exist as the explicit exit;
  "pressing the Esc key always closes an active dialog box"; OK/Cancel/Close get no access keys
  because Enter and Esc are their access keys
  ([UX guide, Dialog Boxes](https://learn.microsoft.com/en-us/windows/win32/uxguide/win-dialog-box)).
  A browse dialog whose whole point is a navigation commit is a [Do it]/Cancel dialog, not a Close
  dialog — "never use Close for dialogs that have settings" (here: a selection).
- The raw-Win32 baseline: the tree answers `WM_GETDLGCODE` with only "DLGC_WANTARROWS and
  DLGC_WANTCHARS"
  ([About Tree-View Controls](https://learn.microsoft.com/en-us/windows/win32/controls/tree-view-controls)),
  so a stock dialog manager gives Enter to the default button while focus sits in the tree — the
  `SHBrowseForFolder` model, where Enter = OK = commit the current tree selection. A control that
  wants Enter must claim it via dialog-code flags
  ([Old New Thing on WM_GETDLGCODE](https://blogs.msdn.microsoft.com/oldnewthing/20061012-06/?p=29413),
  [KB Q83302](https://jeffpar.github.io/kbarchive/kb/083/Q83302/)).
- The wx reality PathMaster actually ships: wxMSW's tree intercepts unmodified VK_RETURN in
  `MSWShouldPreProcessMessage` — "we need VK_RETURN to generate wxEVT_TREE_ITEM_ACTIVATED" — and
  raises the activation event instead of letting the default button fire
  ([src/msw/treectrl.cpp](https://github.com/wxWidgets/wxWidgets/blob/master/src/msw/treectrl.cpp),
  [wxTreeCtrl docs](https://docs.wxwidgets.org/3.2/classwx_tree_ctrl.html)). So Enter-on-leaf →
  activation handler → navigate+close, with no double-fire; both models converge on the same user
  experience, and the activated handler is the single place the commit logic lives.
- The button is still mandatory: "Consider providing double-click behavior… Make double-click
  behavior redundant. There should always be a command button or context menu command that has the
  same effect"
  ([UX guide, Tree Views](https://learn.microsoft.com/en-us/windows/win32/uxguide/ctrl-tree-views)).
  "Go to entry" disabled while an inner node is selected is the discoverable statement of the
  leaf-only commit rule.
- Alt+T: "Don't use Alt+alphanumeric key combinations for shortcut keys. Such shortcut keys may
  conflict with access keys"; shortcut keys "primarily use Ctrl and Function key sequences", and
  Ctrl+G/J/K/L/M/Q/R/**T** are the recommended conflict-free letters
  ([UX guide, Keyboard](https://learn.microsoft.com/en-us/windows/win32/uxguide/inter-keyboard)).
  No precedent found for Alt+T opening a tree/browse dialog in any surveyed app; the PRD's choice
  appears to be an invention, and it would shadow any future menu whose mnemonic is T.

## Q6. Building the prefix tree: chains, sort, raw form

**Recommendation: compress single-child chains into one node whose LABEL is the joined segment text
("Program Files\Java\jdk-21"), sort siblings alphabetically case-insensitively, and carry an
entry's raw form (when it differs from the displayed expansion) as a leaf label suffix — not a
tooltip, which NVDA cannot reliably reach. Compression is what keeps a per-segment tree of a long
PATH inside the UX guide's depth budget.**

- Chain compression is precedented and default-on in the biggest tree UI shipping today: VS Code's
  `explorer.compactFolders` "renders single child folder sequences in a single tree row", enabled
  by default ([VS Code v1.41 release notes](https://code.visualstudio.com/updates/v1_41),
  [test plan #85928](https://github.com/microsoft/vscode/issues/85928)). The UX guide independently
  commands it: "eliminate unnecessary or combine redundant intermediate-level containers" and
  "prefer breadth over depth. Ideally, a tree should have no more than four levels"
  ([UX guide, Tree Views](https://learn.microsoft.com/en-us/windows/win32/uxguide/ctrl-tree-views)) —
  an uncompressed `C:\Program Files\Java\jdk-21\bin` is already five.
- The screen-reader caveat travels with the precedent: VS Code's compact rows had NVDA "sometimes
  skip reading folder names… read different folders instead"
  ([microsoft/vscode#107235](https://github.com/microsoft/vscode/issues/107235)). That defect lived
  in their custom web tree's speech plumbing; in a native tree the compressed text IS the item
  label, the very thing NVDA speaks from `SysTreeView32` (Q2) — the failure mode is
  unconstructible here, but it is the thing the live-NVDA pass must confirm.
- Sort: "Within a container, sort the items in a logical order. Sort names in alphabetical order"
  ([UX guide, Tree Views](https://learn.microsoft.com/en-us/windows/win32/uxguide/ctrl-tree-views));
  the control's own `TVI_SORT`/`TVM_SORTCHILDREN` inserts "in alphabetical order based on the text
  of the item labels"
  ([About Tree-View Controls](https://learn.microsoft.com/en-us/windows/win32/controls/tree-view-controls)).
  Alphabetical also makes native first-letter search (Q2) predictable. PATH's *order* semantics
  cannot survive prefix-merging in any case — order belongs to the main list, and RapidEE, the tool
  whose tree does preserve order, achieves that only by NOT merging prefixes (its tree is
  variable → values, one flat level per variable —
  [rapidee.com](https://www.rapidee.com/en/path-variable)).
- Raw form: the UX guide's own device for per-item elaboration is the infotip
  ([UX guide, Tree Views](https://learn.microsoft.com/en-us/windows/win32/uxguide/ctrl-tree-views)),
  but tooltips are the channel NVDA users demonstrably lose: unreadable in whole app classes
  ([nvaccess/nvda#3314](https://github.com/nvaccess/nvda/issues/3314),
  [#8118](https://github.com/nvaccess/nvda/issues/8118)) and unreachable without a mouse — the
  keyboard-summon request is still open
  ([#10320](https://github.com/nvaccess/nvda/issues/10320)). The label suffix decided in Q2 for
  Issue status is the same channel the raw form should use, e.g.
  `bin (%JAVA_HOME%\bin) — Missing`; a leaf whose raw and expanded forms are identical gets no
  parenthetical, keeping labels short — the same show-only-when-different logic 05 Q3 applied.
