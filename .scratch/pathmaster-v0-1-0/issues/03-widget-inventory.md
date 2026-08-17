# wxdragon widget inventory vs PathMaster UI

Type: research
Status: claimed
Blocked by: —

## Question

Which parts of the PRD's interface can wxdragon express today, and which have no binding?

Map every UI element in [spec-input.md](../spec-input.md) to a concrete wxdragon API, or record it as absent.
**Absences reshape requirements**, so list them explicitly rather than softening them.

- `wxListCtrl` report mode: columns, item data, sorting, item images — and specifically **in-place label
  editing** (`wxLC_EDIT_LABELS`, `EVT_LIST_BEGIN_LABEL_EDIT` / `EVT_LIST_END_LABEL_EDIT`). FR-edit-f2 stands
  or falls on this one.
- Notebook / tabs; MenuBar with accelerators and runtime enable/disable; StatusBar with multiple fields; Toolbar.
- `wxDirDialog` (FR-browse-folder); a modal dialog with **three** buttons (FR-close-confirm needs
  Save / Discard / Cancel, which is not a stock message box).
- The PRD's `InlineAlert` and `InlineBanner` are WinUI vocabulary with no wx equivalent. Is `wxInfoBar` bound?
  What else could carry a dismissible in-window warning?
- `wxSystemSettings` colours and High Contrast detection; whether the bound wx version has Windows dark-mode
  support at all.
- Translations: does wxdragon bind `wxLocale` / `wxTranslations` / `.mo` catalogs, or must i18n happen
  Rust-side?
- Threading: how to run diagnostics off the UI thread and post the result back safely — is there a
  `CallAfter` / idle-event equivalent, and what are the `Send`/`Sync` constraints on widget handles?
- Clipboard and drag & drop: record availability only (both are v0.2.0), so the deferred features are not
  discovered to be impossible later.

Findings → `../research/03-widget-inventory.md`, each claim linked to the API item that proves it.
