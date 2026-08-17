# wxdragon widget inventory vs PathMaster UI

Type: research
Status: resolved
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

## Answer

Full inventory with file:line evidence: [research/03-widget-inventory.md](../research/03-widget-inventory.md).
Versions: **wxdragon 0.9.18 over wxWidgets 3.3.3**. Three levels of availability are distinguished
throughout — safe Rust API / C-ABI only / absent.

**The UI is expressible, and two things are better than assumed:**

- **A full accessibility API exists and is not feature-gated**: `set_accessibility_label` /
  `_description` / `_value` / `_role` / `_state`, plus an `AccessibleImpl` trait (18 callbacks), a real
  `AccRole`/`AccState` MSAA enum set, and `Accessible::notify_event(...)`. Whether the vendored wxWidgets build
  has `wxUSE_ACCESSIBILITY=1`, and whether NVDA actually speaks any of it, remains tickets 01/02/08.
- **Translations load from embedded memory** via a `TranslationsLoader` trait with a passing unit test, so
  `.mo` catalogs can be `include_bytes!`-ed into the exe. NFR-portable and TC-file-structure hold.

**Seven requirements need rewriting** (details and the smallest fix for each are in the research file):

1. **FR-edit-f2** — in-place editing works, but `EVT_LIST_END_LABEL_EDIT` **cannot be vetoed**
   (`ListCtrlEventData` has no `veto()`, inner `Event` private). Validation must accept-then-revert, or drive
   editing through `edit_label()`, which returns the live `TextCtrl`. → ticket 10.
2. **US-admin-elevation / FR-diag-length** — `wxInfoBar` is **absent from both crates**. The banner becomes a
   hand-built `Panel` + `StaticBitmap`(`ArtId::Warning`) + `StaticText` + `Button`, toggled with
   `show(bool)` + `layout()`. → tickets 08, 09.
3. **US-high-contrast** — `wxSystemSettings::GetColour` is **unbound**, and no High Contrast detection exists.
   The requirement inverts into a prohibition: *never set any colour; native controls inherit the system
   theme*. → ticket 09.
4. **FR-auto-diagnose** — there is no `CallAfter` and no `QueueEvent`/custom events. The named mechanism must
   be worker thread + `mpsc` + drain from `on_idle`/`Timer`. → ticket 13.
5. **FR-listview-columns** — a sub-item **cannot carry an icon** (only column 0 can). Status becomes text-only,
   which NFR-no-color-only wanted anyway and which NVDA reads for free.
6. **FR-menubar** — `wxAcceleratorTable` is absent, so **every shortcut must hang off a menu item**. Cheap here,
   but a standing rule.
7. **FR-statusbar** — a field cannot be styled; the over-limit state is carried by text alone.

**The most dangerous finding — a silent-failure mode to write into the spec.** `WindowHandle` is a `u64` index
into a **thread-local** registry, so `ListCtrl`, `Panel`, `Button` etc. are **auto-`Send`**. Moving one to a
worker thread compiles fine and then every method silently no-ops — no panic, no error, no UI update. The rule:
*widgets may be captured across threads but only called on the UI thread.*

Four items logged as UNKNOWN — needs a spike: `edit_label()` versus native F2; whether `call_after` is
delivered during true idle without `wake_up_idle()`; wx 3.3.3 MSW dark mode versus High Contrast; usability of
the C-ABI-only escapes. All refine wording rather than structure; none blocks the spec.
