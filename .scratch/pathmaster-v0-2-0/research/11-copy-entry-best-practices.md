# Research: Ctrl+C copy entry — clipboard, scoping and confirmation facts

Supporting ticket [11-copy-entry-contract](../issues/11-copy-entry-contract.md).
Researched 2026-08-26, per the map's standing directive 7 (research before grilling).

## 1. Raw vs shown: what the analogues actually put on a plain-text clipboard

Ticket 05 carried over the finding "Excel copies the formula" — that is true of Excel's *internal*
clipboard formats. The nuance that matters here: Excel's clipboard is **multi-format**, and the
plain-text format (`CF_UNICODETEXT`) — what Notepad receives — carries the **displayed value**,
not the formula ([e.g.](https://www.extendoffice.com/documents/excel/2747-excel-copy-cell-as-text.html);
pasting formulas externally requires the Ctrl+`​ show-formulas mode first). So Excel is precedent
for *both* readings at once: raw for in-app mutation, shown for plain-text export. PathMaster's
clipboard is plain-text-only (`Clipboard::set_text`, ticket 01), so it must choose one — the
multi-format dodge is not available (wxdragon binds no custom `DataObject` composition worth the
cost here).

What the copied text is *for* cuts both ways:

- **cmd.exe** and the **Explorer address bar** expand `%VAR%` themselves on use — raw text works there.
- **PowerShell** does not expand `%VAR%` (its syntax is `$env:VAR`) — raw text pastes broken there;
  expanded text works everywhere.

Project-internal precedents pull in both directions too: 05 bound Edit/Add (mutation) to raw;
06 bound Search (a display surface) to the **currently displayed** rendering. The decision is which
family Copy belongs to — extraction-for-reuse (raw) or the-list-as-shown (follows Expansion Mode).

Windows' own analogues: **Task Manager** Details tab Ctrl+C copies the row *as displayed*, silently
([Winhelponline](https://www.winhelponline.com/blog/copy-process-details-clipboard-windows-8-taskmgr/));
**Explorer's Copy as path** (Ctrl+Shift+C) copies the path text *plus added quotes* — a deliberate
transform for command-line consumers, always quoted
([How-To Geek](https://www.howtogeek.com/how-to-copy-file-and-folder-paths-on-windows-11/));
Windows' own PATH editor has no per-entry copy at all. Adding quotes would break Entry fidelity
(an Entry's raw text may already carry its own quotes), so Explorer's transform is a
counter-example, not a model.

## 2. Ctrl+C ownership: the platform already scopes it (decisive finding)

wxdragon has no `wxAcceleratorTable` at any level — a shortcut exists only as a menu item's
`"\tCtrl+C"` label suffix ([command.rs](../../../crates/pathmaster/src/ui/command.rs), ADR-0004).
The feared conflict — a frame-level Ctrl+C accelerator stealing copy from the Search `TextCtrl` —
**does not happen on wxMSW**. In the pinned wxWidgets 3.3.3 source
(`src/msw/textentry.cpp`, `wxMSWTextEntryShouldPreProcessMessage`), every text entry claims
Ctrl+A/C/V/X, Ctrl/Shift+Insert, Ctrl/Shift+Del/Home/End/arrows **before** accelerator
translation, with the comment: *"if we don't do it and the parent frame uses them as accelerators,
they wouldn't work at all, so we disable usual preprocessing for them."* This is MSW-specific —
priority differs per platform ([wxWidgets#22630](https://github.com/wxWidgets/wxWidgets/issues/22630)) —
and PathMaster is Windows-only, so it holds unconditionally here.

Consequences:

- The menu-label accelerator is *already* correctly scoped: with focus in the Search field (or any
  dialog's text field — modal dialogs run their own loop anyway), Ctrl+C never reaches the menu;
  the native EDIT control copies the query. No focus-checking handler, no dynamic tables.
- Everywhere else on a Scope tab (list, buttons), the accelerator fires frame-wide. v0.1.0
  precedent: every Entry command acts on the active Scope's focused Entry regardless of which
  control has focus, and is a **silent no-op** when there is none
  (`edit`/`delete` in [mod.rs](../../../crates/pathmaster/src/ui/mod.rs) both
  `let Some(...) else { return }`).
- The Backups tab is answered by the existing availability model: `session: None` closes every
  Entry command; Edit → Copy disables the same way Edit → Edit/Delete already do (its list has no
  Entries to copy — Restore is the only thing a Snapshot row is for).
- Menu home: ticket 02 put Working-Copy commands in **Edit** — Copy reads rather than changes the
  Working Copy, but Edit is where every Windows app keeps Copy; no rival home exists.
- CUA's secondary copy chord **Ctrl+Insert** would need a second (hidden-duplicate) menu item,
  since one item carries one accelerator and there is no table to add another — cost without
  evidence of need; NVDA users use Ctrl+C.

## 3. Confirmation: screen readers hear nothing unless the app speaks

NVDA core does **not** announce application-side copies — the request is open since 2010
([nvda#75](https://github.com/nvaccess/nvda/issues/75)), behavior across apps is inconsistent
([nvda#16375](https://github.com/nvaccess/nvda/issues/16375)), and the gap is filled by add-ons
([Clipspeak](https://addons.nvda-project.org/addons/clipspeak.en.html),
[clipboard-announcer](https://github.com/HBM2001/clipboard-announcer)) an app cannot assume. So the
PRD's spoken confirmation is sound practice, and it must come from PathMaster itself — one new
closed-set Announcement.

Wording best practice across design systems is a **short, fixed** confirmation that does not read
the payload back: GOV.UK's copy-to-clipboard component announces a brief confirmation via a live
region ([component guide](https://components.publishing.service.gov.uk/component-guide/copy_to_clipboard)),
PatternFly swaps the tooltip to "Successfully copied to clipboard!"
([accessibility notes](https://www.patternfly.org/components/clipboard-copy/accessibility/)),
JupyterLab's a11y issue converged on the same live-region pattern
([jupyterlab#14827](https://github.com/jupyterlab/jupyterlab/issues/14827)). Echoing the entry
text would re-speak strings that are routinely 60+ characters and that focus already reads.

No-selection case: only reachable over an empty (possibly empty-because-filtered) list. v0.1.0's
precedent is the silent no-op (see §2); the alternatives — a "nothing selected" Announcement, or
selection-tracking menu enablement — are both new machinery v0.1.0 deliberately avoided.

## 4. Clipboard failure: transient, retriable, detectable

`CLIPBRD_E_CANT_OPEN` is the canonical transient failure — another process (clipboard managers,
Dropbox, VM clipboard bridges) holds the clipboard open for a fraction of a second
([Raymond Chen](https://devblogs.microsoft.com/oldnewthing/20240410-00/?p=109632)). Industry
default: **retry with delay** — .NET's `Clipboard.SetDataObject` retries 10 × 100 ms out of the box
([background](https://www.w3tutorials.net/blog/clipbrd-e-cant-open-error-when-setting-the-clipboard-from-net/)).
wxWidgets does not retry for us; wxdragon's `set_text` returns `bool`, so failure is detectable and
a short app-side retry is cheap. After retries fail, the v0.1.0 taxonomy says name it or rule it
out loud — and for a blind user an Announcement is the only channel; a copy that silently did
nothing reads as success withheld.

**Separate fact, easy to miss:** wx clipboard contents are owned by the app and vanish at exit
unless `flush()` is called (bound in wxdragon, ticket 01). PathMaster is exactly the tool a user
copies from *and then closes* — flush-after-copy is the difference between the paste working and
the clipboard being empty.

## 5. Multi-select

Single-select is load-bearing across v0.1.0 and this effort (ticket 03: "the list is
single-selection and every allowed command touches exactly one visible Entry"; the v0.1.0 NVDA
baseline chose it deliberately). Explorer and Task Manager copy multi-selections one-item-per-line,
quoted or columned — a convention that matters only if multi-select ever arrives, which nothing in
v0.2.0 asks for.
