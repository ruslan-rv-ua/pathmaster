# wxdragon widget surface for v0.2.0

Research for ticket `01-wxdragon-widget-surface-v0-2-0`. Target: **wxdragon 0.9.18** — the version
`Cargo.lock` resolves (`Cargo.lock:1573-1575`; `crates/pathmaster/Cargo.toml:21` requests `"0.9.17"`).
Read from the actual crate source in the cargo registry:

- `C:\scoop\persist\rustup\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\wxdragon-0.9.18\`
- `C:\scoop\persist\rustup\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\wxdragon-sys-0.9.18\`

(`CARGO_HOME` on this machine is `C:\scoop\persist\rustup\.cargo`, per the v0.1.0 research note.)

The crate's files map 1:1 onto the repo at tag `v0.9.18` under `rust/wxdragon/src/` — line numbers
verified to match (e.g. `append_item` at treectrl.rs:375 both locally and on GitHub). Permalinks below
use that tag. wxdragon-sys pins and compiles **wxWidgets 3.3.3** from source, so wxWidgets citations use
the `v3.3.3` tag; `docs.wxwidgets.org` pages are generated from the cited `interface/wx/*.h` headers
(the site 403s automated fetchers; the headers are the same text).

| Source | What it grounds |
|---|---|
| [rust/wxdragon/src/widgets/treectrl.rs](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/treectrl.rs) | TreeCtrl binding |
| [rust/wxdragon/src/widgets/list_ctrl.rs](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/list_ctrl.rs) | ListCtrl binding, its closed event set |
| [rust/wxdragon/src/event/tree_events.rs](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/event/tree_events.rs), [event/mod.rs](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/event/mod.rs), [event/window_events.rs](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/event/window_events.rs), [event/text_events.rs](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/event/text_events.rs), [event/button_events.rs](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/event/button_events.rs) | event surface |
| [rust/wxdragon/src/dnd/*](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/dnd/mod.rs), [data_object.rs](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/data_object.rs) | drag & drop |
| [rust/wxdragon/src/widgets/search_ctrl.rs](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/search_ctrl.rs), [widgets/textctrl.rs](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/textctrl.rs) | search input |
| [rust/wxdragon/src/clipboard.rs](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/clipboard.rs) | clipboard |
| [rust/wxdragon/src/widgets/radio_button.rs](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/radio_button.rs), [widgets/togglebutton.rs](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/togglebutton.rs) | radio/toggle groups |
| [rust/wxdragon/src/window.rs](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/window.rs) | Freeze/Thaw, `get_handle` |
| [wxWidgets v3.3.3 interface/wx/listctrl.h](https://github.com/wxWidgets/wxWidgets/blob/v3.3.3/interface/wx/listctrl.h), [src/msw/listctrl.cpp](https://github.com/wxWidgets/wxWidgets/blob/v3.3.3/src/msw/listctrl.cpp) | native `EnableCheckBoxes`, check events ([docs page](https://docs.wxwidgets.org/3.3/classwx_list_ctrl.html)) |
| [wxWidgets v3.3.3 src/msw/treectrl.cpp](https://github.com/wxWidgets/wxWidgets/blob/v3.3.3/src/msw/treectrl.cpp), [interface/wx/treectrl.h](https://github.com/wxWidgets/wxWidgets/blob/v3.3.3/interface/wx/treectrl.h) | tree is native `SysTreeView32` ([docs page](https://docs.wxwidgets.org/3.3/classwx_tree_ctrl.html)) |
| [wxWidgets v3.3.3 interface/wx/srchctrl.h](https://github.com/wxWidgets/wxWidgets/blob/v3.3.3/interface/wx/srchctrl.h) | wxSearchCtrl is generic on MSW ([docs page](https://docs.wxwidgets.org/3.3/classwx_search_ctrl.html)) |
| [AllenDang/wxDragon CHANGELOG](https://github.com/AllenDang/wxDragon/blob/main/CHANGELOG.md), [releases](https://github.com/AllenDang/wxDragon/releases) | 0.9.19 / 0.9.20 delta |
| `crates/pathmaster/src/announce.rs`, `crates/pathmaster/Cargo.toml` | v0.1.0's raw-Win32 escape-hatch precedent |

---

## Upgrade delta first: newest release is 0.9.20

Two releases exist beyond the pinned 0.9.18 (newest **v0.9.20**, 2026-08-25). **Neither adds anything the
seven areas below ask about** — no list checkboxes, no new D&D, no SearchCtrl/clipboard/radio/toggle/
Freeze changes. What they do carry (facts, not a recommendation):

- **0.9.19**: `CHAR_HOOK` event type — "catching keyboard shortcuts regardless of focused child" (#185);
  `SYS_COLOUR_CHANGED` event (#187); several memory-safety fixes (double-destroy races, leaks).
  `CHAR_HOOK` does not exist in 0.9.18's `EventType`.
- **0.9.20**: fixes `ListCtrl::get_item_text` "truncating the last character and replacing it with a NUL
  byte; multi-byte UTF-8 text is no longer corrupted" (#205) — i.e. **the pinned 0.9.18 has this bug**
  in the very method a search/filter feature would call to read rows back
  ([list_ctrl.rs:429](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/list_ctrl.rs#L429)).
  Also two build fixes around `WXWIDGETS_DIR` (#207, #208).

---

## 1. Tree control — bound, and it is the native `SysTreeView32`

**`wxTreeCtrl` is fully bound** as `TreeCtrl`
([treectrl.rs:279](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/treectrl.rs#L279),
builder at [:292](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/treectrl.rs#L292)).

Item append/expand surface (all `pub fn` on `TreeCtrl`, taking/returning `TreeItemId`,
[treectrl.rs:140](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/treectrl.rs#L140)):

| Method | Line | Signature gist |
|---|---|---|
| `add_root` | [:333](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/treectrl.rs#L333) | `(&str, Option<i32>, Option<i32>) -> Option<TreeItemId>` |
| `append_item` | [:375](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/treectrl.rs#L375) | `(&TreeItemId, &str, image, sel_image) -> Option<TreeItemId>` |
| `insert_item` / `insert_item_before` / `prepend_item` | [:898](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/treectrl.rs#L898) / :928 / :950 | positional insertion |
| `expand` / `expand_all` / `collapse` / `collapse_all` / `toggle` | [:452](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/treectrl.rs#L452) / :658 / :687 / :696 / :723 | |
| `get_selection` / `select_item` / `set_focused_item` / `get_focused_item` | :431 / :441 / :649 / :677 | |
| `ensure_visible` / `scroll_to` / `hit_test` | :640 / :997 / [:1024](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/treectrl.rs#L1024) | hit_test returns `(Option<TreeItemId>, TreeHitTestFlags)` |
| `delete` / `delete_children` / `delete_all_items` / `sort_children` / `set_item_has_children` | :418 / :979 / :970 / :1006 / :1015 | |
| navigation: `get_root_item`, `get_first_child`+cookie, `get_next_child`, `get_next_sibling`, `get_prev_sibling`, `get_item_parent`, `get_children_count` | :483-:814 | full traversal |

Styles bound: `Default`, `HasButtons`, `LinesAtRoot`, `NoLines`, `Single`, `HideRoot`, `EditLabels`
([treectrl.rs:79-96](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/treectrl.rs#L79-L96));
`TR_MULTIPLE`/`TR_FULL_ROW_HIGHLIGHT` are explicitly commented out as not yet exposed (:90-93).

**Per-item data**: two mechanisms, both bound. `append_item_with_data<T: Any + Send + Sync>` /
`add_root_with_data` ([:354](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/treectrl.rs#L354),
[:403](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/treectrl.rs#L403)), and the
`HasItemData` trait (`set_custom_data`/`get_custom_data` returning `Arc<dyn Any + Send + Sync>`,
[treectrl.rs:1057](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/treectrl.rs#L1057);
registry in [widgets/item_data.rs:21-36](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/item_data.rs#L21-L36)).
The data lives in a Rust-side map keyed by a u64 stored in the item — it never crosses into wx.

**Events**: `TreeEvents` is implemented for `TreeCtrl`
([treectrl.rs:1325](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/treectrl.rs#L1325)) with
`on_selection_changed`, `on_selection_changing`, `on_item_activated`, `on_item_expanding/-ed`,
`on_item_collapsing/-ed`, `on_item_right_click`, `on_begin_drag`, `on_end_drag`,
`on_begin/end_label_edit` ([tree_events.rs:90-103](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/event/tree_events.rs#L90-L103));
`TreeEventData::get_item() -> Option<TreeItemId>` ([tree_events.rs:54](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/event/tree_events.rs#L54)).

**Keyboard**: no `on_key_down` in `TreeEvents`, but (a) `TreeCtrl` implements `WindowEvents`
([treectrl.rs:1302](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/treectrl.rs#L1302)) —
`on_key_down` / `on_key_up` / `on_char` with `get_key_code()`
([window_events.rs:319-321](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/event/window_events.rs#L319-L321),
[event/mod.rs:680](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/event/mod.rs#L680)); and (b)
`EventType::TREE_KEY_DOWN` **exists** ([event/mod.rs:284](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/event/mod.rs#L284))
and is mapped to `wxEVT_TREE_KEY_DOWN` in the C++ layer
([wxdragon-sys cpp/src/event.cpp:1299-1300](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon-sys/cpp/src/event.cpp#L1299-L1300)) —
no sugar method, but bindable with the public-trait default `WxEvtHandler::bind_internal(EventType, callback)`
([event/mod.rs:828](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/event/mod.rs#L828)).

**Native control**: wxMSW's wxTreeCtrl creates `WC_TREEVIEW` — i.e. `SysTreeView32` —
via `MSWCreateControl(WC_TREEVIEW, ...)`
([wxWidgets v3.3.3 src/msw/treectrl.cpp](https://github.com/wxWidgets/wxWidgets/blob/v3.3.3/src/msw/treectrl.cpp),
`wxTreeCtrl::Create`), and translates `TVN_KEYDOWN` into `wxEVT_TREE_KEY_DOWN`
(`MSWHandleTreeKeyDownEvent`). So NVDA reads it as a native tree for free, same standing fact as the
v0.1.0 ListCtrl.

**Accessible names on tree items**: nothing per-item is bound. wxdragon's accessibility setters are
whole-window only (`set_accessibility_label` etc.,
[window.rs:1636](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/window.rs#L1636); v0.1.0 research 01
covers them). On the native control each item's **label text is its MSAA name** via comctl32's own
`IAccessible` — no API needed. There is no bound way to give an item an accessible name different from
its visible text.

---

## 2. List checkboxes — NOT bound; native wxWidgets has it; the escape hatch is raw `LVM_*`

**Not bound.** Grep of the whole `wxdragon-0.9.18` crate and the `wxdragon-sys-0.9.18` C++ wrapper for
`EnableCheckBoxes` / `LVS_EX_CHECKBOXES` / `IsItemChecked` / `CheckItem` / `ITEM_CHECKED` finds **zero**
ListCtrl hits — the only checkbox APIs are on menus, `CheckListBox`, `RearrangeList`, and
**`TreeListCtrl`** (which does bind `check_item`, `uncheck_item`, and an `ItemChecked` event —
[treelistctrl.rs:540](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/treelistctrl.rs#L540),
[:813](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/treelistctrl.rs#L813) — but
wxTreeListCtrl is a generic wx-drawn control, not the native `SysListView32`). `ListCtrl`'s event enum
is closed and has no checked/unchecked variants
([list_ctrl.rs:117-152](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/list_ctrl.rs#L117-L152));
`EventType` has `TREELIST_ITEM_CHECKED` but no `LIST_ITEM_CHECKED`
([event/mod.rs:148](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/event/mod.rs#L148)).

**wxWidgets itself has it natively**, unbound: `wxListCtrl::EnableCheckBoxes(bool)` /
`HasCheckBoxes` / `CheckItem` / `IsItemChecked`, all `@since 3.1.0`, with events
`wxEVT_LIST_ITEM_CHECKED` / `wxEVT_LIST_ITEM_UNCHECKED`
([interface/wx/listctrl.h @ v3.3.3](https://github.com/wxWidgets/wxWidgets/blob/v3.3.3/interface/wx/listctrl.h),
[docs](https://docs.wxwidgets.org/3.3/classwx_list_ctrl.html)). On MSW `EnableCheckBoxes` is exactly one
call: `ListView_SetExtendedListViewStyleEx(GetHwnd(), LVS_EX_CHECKBOXES, enable ? LVS_EX_CHECKBOXES : 0)`
([src/msw/listctrl.cpp @ v3.3.3](https://github.com/wxWidgets/wxWidgets/blob/v3.3.3/src/msw/listctrl.cpp)).

**Escape hatch** (the v0.1.0 precedent, described in §8 below): `WxWidget::get_handle()`
([window.rs:1605](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/window.rs#L1605)) gives the
`SysListView32` HWND; `windows-sys` `SendMessageW` with `LVM_SETEXTENDEDLISTVIEWSTYLE`
(`LVS_EX_CHECKBOXES`), `LVM_SETITEMSTATE` / `LVM_GETITEMSTATE` (`LVIS_STATEIMAGEMASK`,
`INDEXTOSTATEIMAGEMASK(1|2)`) sets/reads check state — the same messages wxMSW itself sends.

**Check-state events through the hatch — the sharp edge**: wxMSW's `LVN_ITEMCHANGED` handler translates
state-image changes into `wxEVT_LIST_ITEM_CHECKED`/`UNCHECKED` **without** a `HasCheckBoxes()` gate
(verified in `src/msw/listctrl.cpp @ v3.3.3`), so the wx events fire even for raw-enabled checkboxes —
**but wxdragon 0.9.18 cannot deliver them**: its `EventType` enum has no `LIST_ITEM_CHECKED` value to
bind, and `bind_internal` only accepts enumerated `EventType`s mapped in
[cpp/src/event.cpp](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon-sys/cpp/src/event.cpp). Bound
material that can observe toggles instead: `on_key_down` (`LIST_KEY_DOWN`,
[list_ctrl.rs:1121](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/list_ctrl.rs#L1121), key code
via `ListCtrlEventData::get_key_code`, :212 — Space toggles a checked list item), mouse clicks +
`hit_test(point) -> (item, flags, subitem)`
([list_ctrl.rs:615](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/list_ctrl.rs#L615); the flags
include `LVHT_ONITEMSTATEICON`), then `LVM_GETITEMSTATE` to read the result. Anything richer means
native subclassing (`SetWindowSubclass`) — entirely outside wxdragon.

---

## 3. Drag & drop — `DropSource`/`DropTarget` bound; no in-list reorder helper anywhere

Bound, in `wxdragon::dnd`:

- **`DropSource`** — `new<W: WxWidget>(&W)`, `set_data<D: DataObject>(&D)`,
  `do_drag_drop(allow_move: bool) -> DragResult` (blocking until drop/cancel)
  ([dnd/dropsource.rs:11-47](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/dnd/dropsource.rs#L11-L47)).
- **`DragResult`**: `None|Copy|Move|Link|Cancel|Error`
  ([dnd/mod.rs:21-39](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/dnd/mod.rs#L21-L39)).
- **Drop targets** — only two concrete kinds: **`TextDropTarget`** and **`FileDropTarget`**, each a
  builder over callbacks `with_on_enter` / `with_on_drag_over(x, y, DragResult) -> DragResult` /
  `with_on_leave` / `with_on_drop(x, y) -> bool` / `with_on_drop_text(&str, x, y) -> bool` (resp.
  `with_on_drop_files(Vec<String>, ...)`)
  ([dnd/droptarget.rs:35-164](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/dnd/droptarget.rs#L35-L164),
  [:184-313](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/dnd/droptarget.rs#L184-L313)). A generic
  `wxDropTarget` over arbitrary formats is **not** bound; payloads are limited to the bound data
  objects: `TextDataObject`, `FileDataObject`, `BitmapDataObject`
  ([data_object.rs:91](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/data_object.rs#L91), :160, :251 —
  no custom-bytes object).
- **List item drag events**: `LIST_BEGIN_DRAG` / `LIST_BEGIN_RDRAG` are bound as `on_begin_drag` /
  `on_begin_right_drag`
  ([list_ctrl.rs:1117-1118](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/list_ctrl.rs#L1117-L1118)),
  with `get_item_index()` (:167) and `get_position()` (:207) on the event data. The tree has
  `on_begin_drag`/`on_end_drag` ([tree_events.rs:101-102](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/event/tree_events.rs#L101-L102)); **the list has no end-drag event** — wxWidgets defines none for wxListCtrl.

**In-list reorder through the bindings — the material that exists** (facts, no design): start on
`on_begin_drag`; either run the OLE loop (`DropSource` + `TextDropTarget` on the same list, row index as
the text payload — `do_drag_drop` blocks, drop lands in `on_drop_text` with coordinates) or track the
mouse manually. For the manual route `ListCtrl` does **not** implement `WindowEvents`
(no `on_mouse_motion`/`on_left_up` sugar; grep of list_ctrl.rs finds no `WindowEvents` impl), but it is
a `WxEvtHandler` (via the widget macro,
[macros.rs:164](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/macros.rs#L164)), so
`bind_internal(EventType::MOTION / LEFT_UP, ...)` is available. Row-under-cursor comes from
`hit_test` ([list_ctrl.rs:615](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/list_ctrl.rs#L615));
a drop-position indicator from `ListItemState::DropHilited` + `set_item_state`
([list_ctrl.rs:85](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/list_ctrl.rs#L85),
[:485](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/list_ctrl.rs#L485)); the reorder itself is
delete+reinsert (nothing moves an existing row). Neither wxdragon nor wxWidgets ships an in-list
reorder-drag helper for wxListCtrl. Separately, **`RearrangeList` is bound** — a checklistbox with
`move_current_up/down`, `get_current_order`
([rearrangelist.rs:119-175](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/rearrangelist.rs#L119-L175)) —
reordering by buttons, not by drag, and not a report-mode list.

---

## 4. Search input — `wxSearchCtrl` IS bound; on MSW it is the generic composite, not a native control

**Bound** as `SearchCtrl`
([search_ctrl.rs:87](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/search_ctrl.rs#L87)):
builder (:94), `set_value`/`get_value` (:160/:171), `show_search_button`/`show_cancel_button`
(:120/:140), style `ProcessEnter` (`WXD_TE_PROCESS_ENTER`, :15-23). Events: its own
`on_search_button_clicked` / `on_cancel_button_clicked`
([search_ctrl.rs:253-259](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/search_ctrl.rs#L253-L259)),
plus `TextEvents` (:196) — `on_text_updated` (`EventType::TEXT`) and `on_enter_pressed` (`TEXT_ENTER`)
([text_events.rs:41-42](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/event/text_events.rs#L41-L42)) —
plus `WindowEvents` (:217).

**Platform fact**: wxSearchCtrl "is implemented natively under macOS and GTK 3.6 or later and
generically for all the other platforms"
([interface/wx/srchctrl.h @ v3.3.3](https://github.com/wxWidgets/wxWidgets/blob/v3.3.3/interface/wx/srchctrl.h),
[docs](https://docs.wxwidgets.org/3.3/classwx_search_ctrl.html)) — on MSW it is a wx-composite (text
entry plus drawn buttons), **not** a single native EDIT control. A plain **`TextCtrl` is equally
bound** with the same `ProcessEnter` style, `TextEvents` and `WindowEvents`
([textctrl.rs:29](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/textctrl.rs#L29), :737, :758).

**How ESC reaches the app**: no dedicated event. The bound route is `WindowEvents::on_key_down` /
`on_char` on the control ([window_events.rs:319-321](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/event/window_events.rs#L319-L321))
and `get_key_code()` ([event/mod.rs:680](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/event/mod.rs#L680))
compared against `WXK_ESCAPE`, with `skip()` to pass unhandled keys on. A frame-wide catch-all
(`CHAR_HOOK`) exists only from **0.9.19** (#185) — not in the pinned version.

---

## 5. Clipboard — bound, plain text is two lines

**Bound** as `Clipboard` over `wxClipboard`
([clipboard.rs:27](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/clipboard.rs#L27)):
`Clipboard::get()` (:33), **`set_text(&str) -> bool`** (:154, opens/sets/closes internally),
`get_text() -> Option<String>` (:173), `flush() -> bool` (:131 — keeps data available after app exit),
`open`/`close`/`is_opened` (:39-:60), `set_data`/`add_data<T: DataObject + TransferOwnership>`
(:89/:67), and an RAII `ClipboardLocker` (:188-:210). Nothing missing for FR-copy-entry.

---

## 6. Radio / toggle groups — both bound, with events

- **`RadioButton`** ([radio_button.rs](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/radio_button.rs)):
  builder (:35) with **`first_in_group()`** setting `RB_GROUP` (:133); `get_value`/`set_value`
  (:58/:69); event `on_selected` (`COMMAND_RADIOBUTTON_SELECTED`, :171-172); `WindowEvents` (:97).
  Native `BUTTON` controls on MSW.
- **`ToggleButton`** ([togglebutton.rs](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/togglebutton.rs)):
  builder (:61), `get_value`/`set_value` (:111/:121), `set_label`/`get_label` (:131/:144);
  `ButtonEvents` (:165) provides **`on_toggle`** (`COMMAND_TOGGLEBUTTON_CLICKED`,
  [button_events.rs:42](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/event/button_events.rs#L42)) and
  `on_click`; `WindowEvents` (:190). No mutual-exclusion logic — a toggle "group" is app-side state.
- Also bound, same family: `RadioBox` (widgets/radiobox.rs) and `BitmapToggleButton`
  (widgets/bitmaptogglebutton.rs).

---

## 7. List redraw under filter — no row hiding (confirmed); `Freeze`/`Thaw` bound; virtual mode bound

- **wxListCtrl has no hide-a-row API** — the v3.3.3 interface header defines no such method
  ([interface/wx/listctrl.h](https://github.com/wxWidgets/wxWidgets/blob/v3.3.3/interface/wx/listctrl.h)); the rebuild path is
  the only one: `delete_all_items()`
  ([list_ctrl.rs:574](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/list_ctrl.rs#L574)) +
  `insert_item` (:352) + `set_item_text_by_column` (:390).
- **`freeze()` / `thaw()` / `is_frozen()` are bound** as `WxWidget` trait methods — available on every
  widget including `ListCtrl`
  ([window.rs:1326](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/window.rs#L1326),
  [:1334](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/window.rs#L1334),
  [:1342](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/window.rs#L1342)).
- **Bound alternative**: virtual mode. `ListCtrlStyle::Virtual`
  ([list_ctrl.rs:32](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/widgets/list_ctrl.rs#L32)),
  `set_item_count(i64)` (:771), `set_virtual_text_callback(Fn(i64, i32) -> String)` (:806),
  `refresh_item(s)` (:781/:791) — filtering then means shrinking the backing vector and calling
  `set_item_count`, no per-row inserts at all. (Note §0's 0.9.18 `get_item_text` truncation bug if rows
  are ever read back from the widget rather than the model.)

---

## 8. The existing raw-Win32 escape hatch in PathMaster (v0.1.0 precedent)

Precisely one widget-level raw call exists, and it fixes the pattern:

- `crates\pathmaster\src\announce.rs:58` — `let hwnd = self.banner.get_handle();` (wxdragon's
  `WxWidget::get_handle`, [window.rs:1605](https://github.com/AllenDang/wxDragon/blob/v0.9.18/rust/wxdragon/src/window.rs#L1605),
  returns `*mut c_void` = the HWND on MSW, null-checked at :59).
- `announce.rs:64-71` — `unsafe { NotifyWinEvent(EVENT_OBJECT_LIVEREGIONCHANGED, hwnd, OBJID_CLIENT,
  CHILDID_SELF as i32) }` via **`windows-sys` 0.61** (not the `windows` crate), features
  `Win32_Foundation`, `Win32_UI_Accessibility`, `Win32_UI_WindowsAndMessaging`
  (`crates\pathmaster\Cargo.toml:24-29`, with the comment "announce() only (ADR-0003)").
- The only other raw Win32 message is not widget-related: `SendMessageTimeoutW(HWND_BROADCAST,
  WM_SETTINGCHANGE, ...)` in `crates\pathmaster-platform\src\broadcast.rs:63`.

So the established hatch is: fetch the HWND from wxdragon at the point of use, make a `windows-sys`
call in one dedicated module, never cache the handle (wxdragon's doc note: valid only for the widget's
lifetime). A ListCtrl-checkbox hatch would extend the `windows-sys` feature list with `LVM_*`
constants (`Win32_UI_Controls`) and follow the same shape.

---

## Summary table

| Ticket area | Bound in 0.9.18? | Where | If not — nearest hatch |
|---|---|---|---|
| wxTreeCtrl | **yes, fully** (native SysTreeView32) | treectrl.rs, tree_events.rs | — (per-item a11y names: not bound; native item text serves as MSAA name) |
| ListCtrl `EnableCheckBoxes` | **no** (nothing in crate or C++ wrapper) | — | `get_handle()` + `LVM_SETEXTENDEDLISTVIEWSTYLE`/`LVM_GET/SETITEMSTATE`; check events unreceivable via wxdragon — observe Space/`hit_test` clicks instead; `TreeListCtrl` has bound checkboxes but is non-native |
| Drag & drop | **partly**: DropSource + Text/File drop targets, `LIST_BEGIN_DRAG` | dnd/*, list_ctrl.rs:1117 | no list end-drag event, no custom data object, no reorder helper; manual route via `bind_internal(MOTION/LEFT_UP)` + `hit_test` + `DropHilited` |
| wxSearchCtrl | **yes** (generic composite on MSW) | search_ctrl.rs | ESC = `on_key_down`/`on_char` + `WXK_ESCAPE`; `CHAR_HOOK` only in ≥0.9.19 |
| Clipboard | **yes** | clipboard.rs (`set_text`, `flush`) | — |
| Radio/toggle | **yes** | radio_button.rs (`first_in_group`), togglebutton.rs (`on_toggle`) | — |
| Hide rows w/o rebuild | **no such wx API** (confirmed) | — | rebuild under `freeze()`/`thaw()` (bound, window.rs:1326/1334), or Virtual style + `set_item_count` (bound) |

Upgrade delta (0.9.19/0.9.20): `CHAR_HOOK`, `SYS_COLOUR_CHANGED`, memory-safety fixes, and the
`get_item_text` UTF-8 truncation fix — nothing else the seven areas ask about.
