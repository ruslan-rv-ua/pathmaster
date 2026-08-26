# wxdragon widget surface for v0.2.0

Type: research
Status: resolved
Blocked by: —

## Question

Which of the widgets and mechanisms v0.2.0's features assume does wxdragon (the version pinned in
`Cargo.toml`, or the newest release if an upgrade is on the table) actually bind, and with what API
surface? Facts only — no design. Specifically:

- **Tree control**: is `wxTreeCtrl` bound? Item append/expand APIs, per-item data, keyboard events,
  and whether it is the native `SysTreeView32` (NVDA reads that for free). Anything known about
  accessible names on tree items.
- **List checkboxes**: does the bound `wxListCtrl` support `EnableCheckBoxes` (comctl32
  `LVS_EX_CHECKBOXES`)? Check-state events? If not bound, what raw-handle escape hatch exists —
  v0.1.0 already sends raw Win32 messages where needed.
- **Drag & Drop within a list**: what does wxdragon bind for D&D (`wxDropSource`/`wxDropTarget`,
  list item drag events)? Is an in-list reorder drag feasible at all through the bindings?
- **Search input**: is `wxSearchCtrl` bound, or is a plain `wxTextCtrl` the material? Text-change
  events, and how ESC in a text control reaches the app.
- **Clipboard**: what clipboard API does wxdragon expose (`wxClipboard` or otherwise) for putting
  plain text on it?
- **Radio/toggle groups**: what is bound for a row of mutually-exclusive toggle buttons
  (`wxRadioButton`, `wxToggleButton`) — the filter bar's material.
- **List redraw under filter**: any bound support for hiding rows without rebuilding
  (`wxListCtrl` has none natively — confirm the rebuild path: `DeleteAllItems` + reinsert, and
  whether wxdragon exposes `Freeze`/`Thaw`).

Findings to `../research/01-wxdragon-widget-surface-v0-2-0.md`, with file+line permalinks into the
wxdragon source as v0.1.0's research tickets did.

## Answer

Full findings with citations: [../research/01-wxdragon-widget-surface-v0-2-0.md](../research/01-wxdragon-widget-surface-v0-2-0.md).
Target is the pinned **wxdragon 0.9.18** (Cargo.lock; crate source read from the registry); newest
release is 0.9.20, whose delta adds nothing the seven areas ask about.

- **Tree control**: fully bound (`TreeCtrl`) — append/insert/expand/collapse/select/traversal, hit-test,
  and per-item data via a Rust-side `HasItemData` registry. On MSW it is the native `SysTreeView32`
  (`MSWCreateControl(WC_TREEVIEW, …)` in wxWidgets 3.3.3), so NVDA reads items for free; item label text
  is each item's MSAA name, and no per-item accessible-name API is bound. Keyboard: `WindowEvents`
  `on_key_down`/`on_char`, plus `EventType::TREE_KEY_DOWN` (bindable via `bind_internal`, no sugar).
- **List checkboxes**: NOT bound — no `EnableCheckBoxes`/`CheckItem` anywhere in the crate or its C++
  wrapper, and no check events in the closed `EventType` enum. wxWidgets itself has all of it natively
  since 3.1.0 (on MSW it is one `LVM_SETEXTENDEDLISTVIEWSTYLE`/`LVS_EX_CHECKBOXES` call). Escape hatch:
  `get_handle()` + `windows-sys` `SendMessageW` `LVM_*` (v0.1.0's announce.rs pattern); check-state
  *events* cannot arrive through wxdragon — toggles are observable via Space (`LIST_KEY_DOWN`) and
  clicks + `hit_test` + `LVM_GETITEMSTATE`. `TreeListCtrl` has bound checkboxes but is not native.
- **D&D**: `DropSource` (`do_drag_drop`) and `TextDropTarget`/`FileDropTarget` (full callback builders)
  are bound; payloads limited to text/file/bitmap data objects. `LIST_BEGIN_DRAG` is bound; there is no
  list end-drag event and no reorder helper in wxdragon or wxWidgets. An in-list reorder is assemblable
  from bound parts only: begin-drag + (OLE loop or `bind_internal(MOTION/LEFT_UP)`) + `hit_test` +
  `DropHilited` + delete/reinsert. `RearrangeList` (buttons-based reorder) is also bound.
- **Search input**: `SearchCtrl` is bound (value, search/cancel buttons, `on_text_updated`,
  `on_enter_pressed`, `ProcessEnter`), but on MSW wxSearchCtrl is the *generic composite*, not native;
  `TextCtrl` has the identical event surface. ESC reaches the app only via `on_key_down`/`on_char` +
  `WXK_ESCAPE` (`CHAR_HOOK` exists only from 0.9.19).
- **Clipboard**: bound — `Clipboard::get().set_text(&str)`, plus `get_text`, `flush`, RAII locker.
- **Radio/toggle**: both bound — `RadioButton` with `first_in_group()` (`RB_GROUP`) and `on_selected`;
  `ToggleButton` with `on_toggle`; `RadioBox` too. Toggle-group exclusivity is app-side.
- **Filter redraw**: wxListCtrl has no row-hiding (confirmed in the 3.3.3 interface header) — rebuild is
  `delete_all_items` + reinsert; `freeze()`/`thaw()` ARE bound (WxWidget trait). Virtual mode is also
  bound (`Virtual` style, `set_item_count`, text callback) as a no-reinsert alternative.

Delta worth knowing: pinned 0.9.18 has a `ListCtrl::get_item_text` bug (last character truncated,
multi-byte UTF-8 corrupted) fixed in 0.9.20; 0.9.19 added `CHAR_HOOK` and `SYS_COLOUR_CHANGED`.
