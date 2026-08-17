# wxdragon widget inventory vs PathMaster UI

Resolves [issues/03-widget-inventory.md](../issues/03-widget-inventory.md).

## What was inventoried

| | |
|---|---|
| Crate | `wxdragon` **0.9.18** (latest on crates.io as of 2026-08-18), plus `wxdragon-sys` 0.9.18 |
| Underlying toolkit | **wxWidgets 3.3.3** — `wxdragon-sys/build.rs:1-2` (`WX_SRC_URL`, `const WX_VERSION: &str = "3.3.3"`), downloaded as a hash-pinned release zip and built via CMake |
| Method | `cargo add wxdragon` + `cargo fetch` into a throwaway crate; read the vendored source. **Nothing was built** (that belongs to another ticket). Cross-checked against the upstream `examples/rust/` tree on GitHub, which is *not* shipped in the published crate. |

**Path abbreviations used below.** `wxdragon/…` = `C:\scoop\persist\rustup\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\wxdragon-0.9.18\…`; `wxdragon-sys/…` = same registry path with `wxdragon-sys-0.9.18`. Upstream examples are cited as `examples/rust/<name>` under <https://github.com/AllenDang/wxdragon>.

**One structural fact that shapes everything.** `wxdragon-sys` runs bindgen over the C++ shim headers in `wxdragon-sys/cpp/include/` (`wxdragon-sys/build.rs:18`, `src/lib.rs:4`). So there are *three* levels of "present", and this report distinguishes them:

1. **Safe Rust API** — wrapped in `wxdragon`. This is what "PRESENT" means unless stated otherwise.
2. **C ABI only** — declared in `wxdragon-sys/cpp/include/`, reachable as an `unsafe` `wxdragon_sys::…` call but with no safe wrapper. Marked *"sys-only"*.
3. **Absent everywhere** — no shim, no binding. Marked ABSENT.

---

## Inventory

### A. ListCtrl report mode — FR-listview-columns, FR-edit-f2, FR-backup-ui

| UI need | wxdragon API | Verdict | Evidence |
|---|---|---|---|
| Report (multi-column) mode | `ListCtrlStyle::Report` | **PRESENT** | `wxdragon/src/widgets/list_ctrl.rs:39`; constant `WXD_LC_REPORT = 32` in `wxdragon-sys/src/generated_constants/wx_msw_constants.rs:203` |
| Columns | `ListCtrl::insert_column(col, heading, ListColumnFormat, width)`, `set_column_width`, `get_column_width`, `get_column_count` | **PRESENT** | `list_ctrl.rs:311`, `:322`, `:332`, `:342` |
| Row insert / cell text | `insert_item(index, label, image)`, `set_item_text(index, text)`, `set_item_text_by_column(index, col, text)`, `get_item_text(index, col)`, `get_item_count()` | **PRESENT** | `list_ctrl.rs:352`, `:364`, `:390`, `:429`, `:457` |
| Arbitrary per-row Rust data | `HasItemData::set_custom_data<T: Any + Send + Sync>` / `get_custom_data` — stores a registry id in the native item data slot | **PRESENT** | `list_ctrl.rs:897-946` (trait impl); backing store `wxdragon/src/widgets/item_data.rs` |
| Selection / focus state | `set_item_state`, `get_item_state`, `get_next_item`, `get_first_selected_item`, `get_selected_item_count`, `ensure_visible` | **PRESENT** | `list_ctrl.rs:485`, `:520`, `:531`, `:541`, `:594`, `:604` |
| Delete / clear | `delete_item`, `delete_all_items`, `clear_all` | **PRESENT** | `list_ctrl.rs:564`, `:574`, `:584` |
| Item image (icon) on **column 0** | `set_item_image(item, image_index)` + `set_image_list(ImageList, image_list_type::SMALL)` | **PRESENT** | `list_ctrl.rs:554`, `:856`; type constants `:219-226` |
| Item image on a **sub-item** (a status icon in a column other than 0) | — | **ABSENT** | `set_item_text_by_column` hardcodes `image = -1` and `mask = WXD_LIST_MASK_TEXT` (`list_ctrl.rs:404-423`), so it can never carry an image. No other sub-item setter exists. |
| **In-place label editing** (`wxLC_EDIT_LABELS`) | `ListCtrlStyle::EditLabels` | **PRESENT** | `list_ctrl.rs:33`; `WXD_LC_EDIT_LABELS = 1024` in `wx_msw_constants.rs:209` (matches wx's real value) |
| Start editing programmatically | `ListCtrl::edit_label(item) -> TextCtrl` — returns the live in-place editor | **PRESENT** | `list_ctrl.rs:634`; shim calls `wxListCtrl::EditLabel` and returns the `wxTextCtrl*` at `wxdragon-sys/cpp/src/list_ctrl.cpp:255-262` |
| BEGIN / END label-edit events | `on_begin_label_edit`, `on_end_label_edit` (`EventType::LIST_BEGIN_LABEL_EDIT` / `LIST_END_LABEL_EDIT`) | **PRESENT** | `list_ctrl.rs:1115-1116` |
| Read the new label; detect Escape | `ListCtrlEventData::get_label() -> Option<String>`, `is_edit_cancelled() -> Option<bool>`, `get_item_index()` | **PRESENT** | `list_ctrl.rs:184`, `:198`, `:167` |
| **Veto / reject an invalid label edit** | — | **ABSENT** | `ListCtrlEventData` exposes only the 6 getters at `list_ctrl.rs:160-215`. It has **no** `veto()`, no `skip()`, and its inner `event: Event` field is **private** (`list_ctrl.rs:157`) with no accessor. The capability exists in the crate but was not wired here: `Event::veto()` is at `wxdragon/src/event/mod.rs:786`, `NotebookPageChangedEvent` exposes `pub base: Event` (`notebook.rs:313`), and `MenuEventData` has its own `veto()` (`event/menu_events.rs:88`). |
| Reach the editor during a *user-initiated* edit (F2 / double-click) | — | **ABSENT** | `wxListCtrl::GetEditControl` is not bound at any level. Only `wxd_TreeCtrl_GetEditControl` exists (`wxdragon-sys/cpp/include/widgets/wxd_treectrl.h:262`). The editor is reachable only as the return value of your own `edit_label()` call. |
| Sorting by comparator | `wxd_ListCtrl_SortItems(self, cmpFunc, data)` | **sys-only** | Declared `wxdragon-sys/cpp/include/widgets/wxd_listctrl.h:104`, implemented `cpp/src/list_ctrl.cpp:509-520`. **No safe wrapper** — `sort_items` has zero hits in `wxdragon/src`. |
| Sort-style flags / column-click event | `ListCtrlStyle::{SortAscending, SortDescending, NoSort}`, `on_column_click` | **PRESENT** | `list_ctrl.rs:30-31`, `:50`, `:1112` |
| Virtual (owner-data) mode | `ListCtrlStyle::Virtual`, `set_item_count`, `set_virtual_text_callback`, `refresh_item(s)` | **PRESENT** | `list_ctrl.rs:32`, `:771`, `:806`, `:781`, `:791` |
| Per-row colours | `set_item_background_colour`, `set_item_text_colour` | **PRESENT** | `list_ctrl.rs:653`, `:663` — but see §E: there is no way to source a *system* colour to pass in |

### B. Structural chrome — FR-view-tabs, FR-menubar, FR-statusbar

| UI need | wxdragon API | Verdict | Evidence |
|---|---|---|---|
| Notebook / 3 tabs | `Notebook::builder`, `add_page(page, text, select, image_id)`, `set_selection`, `change_selection`, `get_page_count`, `get_page`, `advance_selection` | **PRESENT** | `wxdragon/src/widgets/notebook.rs:59`, `:89`, `:143`, `:154`, `:214`, `:224`, `:164` |
| Tab change events | `on_page_changed`, `on_page_changing`; `NotebookPageChangedEvent::{get_selection, get_old_selection}`; veto via `pub base: Event` | **PRESENT** | `notebook.rs:344-348`, `:324`, `:334`, `:313` |
| Rename a tab after creation | — | **ABSENT** | No `set_page_text`/`get_page_text` on `Notebook` at any level (not in `notebook.rs`, not in `wxdragon-sys/cpp/include/widgets/wxd_notebook.h`). Tab text is fixed at `add_page` time. `Treebook::set_page_text` (`treebook.rs:212`) and `AuiNotebook::set_page_text` (`aui_notebook.rs:197`) do exist. **Harmless here** — language change is restart-only (map decision 6). |
| MenuBar / Menu / MenuItem | `MenuBar::builder().append(menu, title)`, `Menu::builder().append_item/append_check_item/append_radio_item/append_separator`, `Frame::set_menu_bar` | **PRESENT** | `menus/menubar.rs:21`, `:207`; `menus/menu.rs:93`, `:423-456`; `widgets/frame.rs:286` |
| Accelerator text on menu items (`"&Undo\tCtrl+Z"`) | Label string is passed through verbatim to `wxMenu::Append`, so wx's own `\t` and `&` parsing applies | **PRESENT (by pass-through)** | `menus/menu.rs:373` → `wxdragon-sys/cpp/src/menu.cpp:151` (`wx_menu->Append(id, wxString::FromUTF8(item), …)`). `CString::new` only fails on interior NUL (`menu.rs:371`), so `\t` survives. |
| **`wxAcceleratorTable` — shortcuts not attached to a menu item** | — | **ABSENT** | Case-insensitive search for `accelerator` / `AcceleratorEntry` / `wxACCEL` / `SetAcceleratorTable` across both crates (Rust, C++ headers, C++ sources, generated constants) yields only `webview.rs:820/830` (WebView-only) and `dc.cpp:328` (a draw helper). `Window::set_accelerator_table` does not exist. |
| Runtime enable/disable/check of menu items | `MenuBar::enable_item(id, bool)` / `is_item_enabled` / `check_item` / `is_item_checked`; same four on `Menu`; `MenuItem::enable/check`; `MenuBar::enable_top(pos, bool)` | **PRESENT** | `menubar.rs:73`, `:78`, `:83`, `:88`, `:143`; `menu.rs:183-198`; `menus/menuitem.rs:174-206` |
| Menu command events + item id | `MenuEvents::on_menu_selected` → `MenuEventData::get_id()`; per-item `MenuItem::on_click`; frame-wide `Frame::on_menu` | **PRESENT** | `event/menu_events.rs:101`, `:73`; `menuitem.rs:114`; `frame.rs:477` |
| StatusBar with multiple fields | `StatusBar::set_fields_count(usize)`, `set_status_widths(&[i32])`, `set_status_text(text, field_index)`, `push_status_text`, `pop_status_text`; builder `with_fields_count` / `with_status_widths` | **PRESENT** | `widgets/statusbar.rs:95`, `:126`, `:107`, `:138`, `:149`, `:231`, `:237`. Note argument order: **text first, index second**. |
| Read a status bar field back; style one field differently | — | **ABSENT** | The shim exports exactly six functions (`Create`, `SetFieldsCount`, `SetStatusText`, `SetStatusWidths`, `PushStatusText`, `PopStatusText`) — `wxdragon-sys/cpp/include/widgets/wxd_statusbar.h`. No `GetStatusText`, no `SetStatusStyles`, no `GetFieldRect`. `StatusBarStyle` has a single variant `Default = 0` (`statusbar.rs:21-28`); the real style constants exist only in sys (`wx_msw_constants.rs:186-191`) and are reachable solely through `Frame::create_status_bar`'s raw `i64` parameter (`frame.rs:354`). |
| Toolbar | `Frame::create_tool_bar(style, id)`, then `add_tool`, `add_check_tool`, `add_radio_tool`, `add_separator`, `add_control`, `realize`, `enable_tool`, `toggle_tool`, `is_tool_enabled`, `get_tool_state` | **PRESENT** | `frame.rs:336`; `widgets/toolbar.rs:156`, `:169`, `:183`, `:197`, `:210`, `:222`, `:232`, `:244`, `:256`, `:266` |
| Toolbar caveats | Only creatable via `Frame::create_tool_bar` (no `ToolBar::builder`); text labels need `ToolBarStyle::Text`; tools deliver `EventType::MENU`, sharing an id namespace with the menu | — | `toolbar.rs:21`, `:88`, `:409-414` |
| Minimum window size (NFR-window-sizing 800×600) | `WxWidget::set_min_size(Size)` | **PRESENT** | `wxdragon/src/window.rs:440` |

### C. Dialogs — FR-browse-folder, FR-close-confirm, FR-apply, FR-cancel

| UI need | wxdragon API | Verdict | Evidence |
|---|---|---|---|
| Folder picker (`wxDirDialog`) | `DirDialog::builder(parent, message, default_path).build()`, `show_modal()`, `get_path() -> Option<String>`, `set_path`, `DirDialogStyle::{MustExist, ChangeDir}` | **PRESENT** | `wxdragon/src/dialogs/dir_dialog.rs:28`, `:33`, `:38`, `:49`, `:7-16`. **Leak caveat:** `impl Drop` at `:82-90` only nulls the pointer and never destroys — call `.destroy()` (`window.rs:516`) after `show_modal()`. |
| Inline folder-picker widget | `DirPickerCtrl` | **PRESENT** | `widgets/dir_picker_ctrl.rs:94`, `:129`, `:145`, `:186` |
| Stock 2-button message box | `MessageDialog::builder(parent, message, caption).with_style(...)`, `MessageDialogStyle::{OK, Cancel, YesNo, IconWarning, …}` | **PRESENT** | `dialogs/message_dialog.rs:47`, `:10-28`, `:54` |
| Stock 3-button Yes/No/Cancel | `MessageDialogStyle::YesNo \| MessageDialogStyle::Cancel` (it is a real bitflags type) | **PRESENT** | `message_dialog.rs:10-28`; bitflags machinery `wxdragon/src/macros.rs:278-299` |
| **Custom button *labels* on a message box** ("Save"/"Discard"/"Cancel") | — | **ABSENT** | No `SetYesNoLabels` / `SetOKCancelLabels` / `SetYesNoCancelLabels` anywhere. Confirmed at the shim: the *only* MessageDialog entry point is `wxd_MessageDialog_Create` — `wxdragon-sys/cpp/include/dialogs/wxd_dialogs.h:44-45`, `cpp/src/message_dialog.cpp:9`. There are no setters of any kind. |
| **Hand-built modal dialog with three buttons** | `Dialog::builder(parent, title).build()`, `Dialog::show_modal() -> i32`, `Dialog::end_modal(code)`, `set_escape_id`, `set_affirmative_id`, `get_return_code` | **PRESENT** | `dialogs/mod.rs:226`, `:110`, `:122`, `:151`, `:178`, `:212`. Proven by the upstream runnable example `examples/rust/generic_dialog_test/src/main.rs:17-55` (builds a `Dialog`, adds a `Button`, calls `dialog.end_modal(ID_OK)`, then `show_modal()` and `destroy()`). |
| Standard-order dialog buttons | `StdDialogButtonSizer` + `add_button`, `realize()`, and `set_affirmative_button` / `set_negative_button` / `set_cancel_button` for non-standard ids | **PRESENT** | `sizers/std_dialog_button_sizer.rs:45`, `:53`, `:120`, `:131`, `:137` |
| `ID_SAVE` constant | — | **ABSENT from the safe API** | `wxdragon/src/id.rs:9-34` exports only `ID_ANY, ID_NONE, ID_HIGHEST, ID_EXIT, ID_ABOUT, ID_OK, ID_CANCEL, ID_YES, ID_NO, ID_APPLY, ID_HELP`. `WXD_ID_SAVE = 5003` exists in sys (`wx_msw_constants.rs:35`). Use `ID_YES`/`ID_NO`/`ID_CANCEL` or the sys constant. |
| Other stock dialogs available | `FileDialog`, `ColourDialog`, `FontDialog`, `ProgressDialog`, `SingleChoiceDialog`, `MultiChoiceDialog`, `TextEntryDialog`, `AboutDialogInfo` + `show_about_box` | **PRESENT** | `wxdragon/src/dialogs/` — one file each; all re-exported at `prelude.rs:198-209` |

### D. Dismissible in-window banner — the PRD's "InlineAlert" / "InlineBanner"

| UI need | wxdragon API | Verdict | Evidence |
|---|---|---|---|
| **`wxInfoBar`** | — | **ABSENT** | Case-insensitive search for `infobar` across **both** crates in full (`src/`, `cpp/include/`, `cpp/src/`, `build.rs`, `generated_constants/`) returns **zero matches**. There is no `wxd_infobar.h` in `wxdragon-sys/cpp/include/widgets/` (58 widget headers, none for InfoBar). |
| `wxNotificationMessage` | `NotificationMessage::builder()`, `show(timeout)`, `add_action` | **PRESENT but wrong shape** | `widgets/notification_message.rs:89`, `:101`, `:164`. It is an **OS-level toast/balloon, not an in-window control** — it has no `WxWidget` impl and cannot go in a sizer; the source says so at `:210`. |
| Hand-built banner: a `Panel` holding a `StaticBitmap` + `StaticText` + dismiss `Button`, shown/hidden at runtime | `Panel::builder`, `WxWidget::show(bool)` / `hide()` / `is_shown()`, `layout()` (hidden children collapse to zero height), `Button::on_click`, `StaticText::set_label` / `wrap` | **PRESENT — this is the substitute** | `widgets/panel.rs:59`; `window.rs:393`, `:1043`, `:1031`, `:371`; `widgets/button.rs:345`; `widgets/static_text.rs:83`, `:111`. Note `Sizer` itself has no show/hide-item API (`sizers/base.rs` exposes only `add`/`add_sizer`/`add_spacer`), so the toggle must be `panel.show(false); parent.layout();` |
| Warning icon for the banner | `ArtProvider::get_bitmap(ArtId::Warning, ArtClient::MessageBox, size)` / `get_bitmap_bundle`, rendered via `StaticBitmap` | **PRESENT** | `wxdragon/src/art_provider.rs:16`, `:73`, `:167`, `:200`; `widgets/static_bitmap.rs:70`, `:137` |
| `CollapsiblePane` as an alternative | `CollapsiblePane::builder`, `expand`, `collapse`, `get_pane` | **PRESENT but poor fit** | `widgets/collapsible_pane.rs:52`, `:106`, `:119`, `:130` — a disclosure triangle with no dismiss affordance; the header never goes away |

### E. System colours, High Contrast, dark mode — US-high-contrast, NFR-accessibility-wcag

| UI need | wxdragon API | Verdict | Evidence |
|---|---|---|---|
| **`wxSystemSettings::GetColour`** — read a system colour to paint with | — | **ABSENT** | The only `wxSystemSettings` symbol exported anywhere is `wxd_SystemSettings_GetAppearance()` (`wxdragon-sys/cpp/include/core/wxd_app.h:59-60`). There are **zero** `wxSYS_COLOUR_*` / `SYS_` constants in `wxdragon-sys/src/generated_constants/wx_msw_constants.rs`. `wxdragon/src/color.rs` is a static struct plus a hardcoded Tailwind-style palette (`color.rs:14`, `:114`, `:202+`) — nothing system-derived. |
| **High Contrast detection** | — | **ABSENT** | Zero hits for `HighContrast` / `high_contrast` / `HOTLIGHT` across both crates. And because `GetColour` is unbound, even the classic `SYS_COLOUR_WINDOW == SYS_COLOUR_BUTTONFACE` heuristic is unavailable. Detecting HCM requires your own Win32 call (`SystemParametersInfo(SPI_GETHIGHCONTRAST)`). |
| Nearest workaround — sample the theme off a live control | `WxWidget::get_background_color()` / `get_foreground_color()` | **PRESENT** | `wxdragon/src/window.rs:942`, `:929` (setters at `:401`, `:916`). Note the crate spells these `color`, not `colour`. |
| Dark-mode detection | `appearance::get_system_appearance()`, `SystemAppearance::is_dark()` / `is_using_dark_background()`, `appearance::is_system_dark_mode()` | **PRESENT** | `wxdragon/src/appearance.rs:193`, `:123`, `:134`, `:219` |
| Opt into MSW dark mode | `app::set_appearance(Appearance::{Light, Dark, System})` → `wxApp::SetAppearance()`; `AppearanceResult::{Ok, Failure, CannotChange}` | **PRESENT** | `wxdragon/src/app.rs:389`; `appearance.rs:43`, `:58`, `:257`, `:276`. Real because the vendored wxWidgets is 3.3.3 (`wxdragon-sys/build.rs:2`); the shim guards it with `#if wxCHECK_VERSION(3, 3, 0)` (`cpp/src/app.cpp`). Demonstrated by `examples/rust/dark_mode_demo`. |
| DPI scaling query (`FromDIP` / `GetDPIScaleFactor`) | — | **ABSENT from the safe API** | No DPI methods on `Window` or `Font`. `FromDIP` is applied *implicitly and invisibly* to every position/size crossing the FFI boundary — `wxdragon-sys/cpp/src/wxd_utils.h:47`, `:60`. You cannot query or bypass the factor. `DeviceContext::get_ppi()` (`dc/mod.rs:947`) and `BitmapBundle` (`bitmap_bundle.rs:182`) are the only adjacent tools. |

### F. Accessibility — US-accessibility, FR-diag-* announcements

| UI need | wxdragon API | Verdict | Evidence |
|---|---|---|---|
| Accessible name / description / value | `WxWidget::set_accessibility_label(&str)`, `set_accessibility_description(&str)`, `set_accessibility_value(&str)` | **PRESENT** | `wxdragon/src/window.rs:1636`, `:1655`, `:1674` — Windows + macOS; no-op on GTK |
| Accessible role / state | `set_accessibility_role(AccRole)`, `set_accessibility_state(AccState)` | **PRESENT (Windows-only)** | `window.rs:1694`, `:1711` — both `#[cfg(target_os = "windows")]` |
| `wxWindow::SetName` (MSAA fallback) | `WxWidget::set_name(&str)` / `get_name()` | **PRESENT** | `window.rs:1086`, `:1105` |
| Full custom `wxAccessible` provider | `AccessibleImpl` trait (18 overridable callbacks), `Accessible::new(&window, impl)`, `Accessible::notify_event(...)`; enums `AccRole` (full MSAA `ROLE_SYSTEM_*` set), `AccState`, `AccStatus`, `NavDir` | **PRESENT** | `wxdragon/src/accessible.rs:226`, `:291`, `:333`, `:98`, `:178`, `:13`, `:33` |
| Feature gate needed? | **No** — `mod accessible` is unconditional (`wxdragon/src/lib.rs:7`); the crate's only features are `aui, media-ctrl, richtext, stc, webview, xrc` (`wxdragon/Cargo.toml:47-53`) | **PRESENT** | as cited |
| Stale doc warning | `window.rs:1633` references a `hide_from_accessibility` method that **does not exist** anywhere in the crate | — | grep: the only occurrence is that doc comment |

### G. i18n — US-i18n, FR-i18n-runtime, NFR-portable

| UI need | wxdragon API | Verdict | Evidence |
|---|---|---|---|
| `wxTranslations` | `Translations::new()`, `Translations::get()`, `set_global()`, `set_language(Language)` / `set_language_str("uk")`, `add_catalog(domain)`, `is_loaded`, `get_available_translations` | **PRESENT** | `wxdragon/src/translations.rs:89`, `:72`, `:106`, `:116`, `:126`, `:143`, `:199`, `:365` |
| gettext lookup (`_()` equivalent) | free fn `translations::translate(&str) -> String`; `Translations::get_string(orig, domain)` | **PRESENT** | `translations.rs:537`, `:217`. It is a **plain function, not a macro** — no compile-time extraction; `xgettext` would need `--keyword=translate`. |
| Plural forms | `translate_plural(singular, plural, n)`; `Translations::get_plural_string(...)` | **PRESENT** | `translations.rs:559`, `:251` |
| Ukrainian in the language enum | `Language::Ukrainian = 217` (enum has 235 variants mirroring `wxLanguage`) | **PRESENT** | `wxdragon/src/language.rs:458`; enum at `:21` |
| **Load `.mo` catalogs from embedded memory** (single-file portable exe) | `TranslationsLoader` trait — `load_catalog(domain, lang) -> Option<Cow<'_, [u8]>>` and `available_translations(domain)`; installed via `Translations::set_loader(loader)` **before** `add_catalog` | **PRESENT — this is the key finding for NFR-portable** | Trait `translations.rs:429`, methods `:439`, `:444`; installer `:188`. C++ side builds the catalog with `wxMsgCatalog::CreateFromData` over a non-owning buffer — `wxdragon-sys/cpp/src/translations.cpp:12`, `:42`. There is a passing unit test proving the embedded path end-to-end: `translations.rs:833` (`rust_loader_serves_embedded_catalog`), with a hand-built `.mo` at `:767`. |
| Load `.mo` from disk | free fn `add_catalog_lookup_path_prefix(prefix)`, expecting `<prefix>/<lang>/LC_MESSAGES/<domain>.mo` | **PRESENT** | `translations.rs:518` → `cpp/src/translations.cpp:339`. This is a static on the *file* loader and has no effect once a custom Rust loader is installed (wx keeps one loader). Layout confirmed by `examples/rust/translations_demo/locale/{de,es,fr,ja,ru,zh_CN}/LC_MESSAGES/`. |
| `wxLocale` (instantiable, C-locale/number formatting) | — | **ABSENT** | `translations::Locale` is a zero-sized namespace struct exposing only static helpers — `get_language_name` (`:651`), `get_language_canonical_name` (`:669`), `find_language_info` (`:687`), `get_language_info` (`:694`), `get_system_language()` (`:700`). No `wxLocale::Init`. `UILocale` is read-only (`:715`, `:721`, `:743`). |
| Re-translate a built UI in place | — | **ABSENT** | No re-translate/relayout hook; you must re-set every label yourself. **Harmless here** — language change is restart-only (map decision 6). `get_system_language()` covers the "default = system locale" rule. |
| `wxResourceTranslationsLoader` | — | **ABSENT** (superseded by the Rust loader above) | zero hits in either crate |

### H. Threading — FR-auto-diagnose ("асинхронно, не блокує UI")

| UI need | wxdragon API | Verdict | Evidence |
|---|---|---|---|
| Post a closure to the UI thread | `wxdragon::call_after(Box<F>) where F: FnOnce() + Send + 'static` | **PRESENT, but it is *not* `wxEvtHandler::CallAfter`** | `wxdragon/src/app.rs:37`. It is a pure-Rust global queue: `static MAIN_THREAD_QUEUE: LazyLock<Arc<Mutex<VecDeque<Box<dyn FnOnce() + Send>>>>>` at `app.rs:16` — genuinely safe to call from a worker. |
| How the queue is drained | `process_rust_callbacks()` (`app.rs:83`) called from the app's **idle handler** | **PRESENT, with two limits** | `WxdApp::OnIdle` at `wxdragon-sys/cpp/src/app.cpp:100-111`, bound to `wxEVT_IDLE` at `cpp/src/app.cpp:84`. Limit 1: at most **10 callbacks per idle tick** (`app.rs:63`). Limit 2: it only calls `event.RequestMore()` when it actually processed something (`app.cpp:107-111`). |
| Waking a sleeping event loop | `wxdragon::app::wake_up_idle()` → `wxWakeUpIdle()` | **PRESENT** | `app.rs:318` → `wxdragon-sys/cpp/include/core/wxd_app.h:20`. **`call_after` does not call it** — `app.rs:37-43` only locks and pushes. A callback queued while the app is genuinely idle may therefore not run until some UI activity occurs. |
| `QueueEvent` / `AddPendingEvent` / custom user events with a payload | — | **ABSENT** | No binding in either crate. The whole FFI event surface is `Bind` / `BindWithId` / `Unbind` / `UnbindAll` (`wxdragon-sys/cpp/include/events/wxd_event_api.h:14,19,25,30`). `EventType` is a **closed** set of predefined wx types (`wxdragon/src/event/mod.rs`); there is no `wxNewEventType()` and no way to construct and post an event carrying data. |
| Idle event hook | `WindowEvents::on_idle(...)`; `IdleEventData::request_more(bool)` / `more_requested()`; `IdleEvent::set_mode(IdleMode::{ProcessAll, ProcessSpecified})` | **PRESENT** | `event/window_events.rs:329`, `:276`, `:281`; `event/mod.rs:534`, `:511` |
| Timer (poll a channel from the UI thread) | `Timer::new(owner)`, `on_tick`, `start(ms, one_shot)`, `stop`, `is_running` | **PRESENT** | `wxdragon/src/timer.rs:52`, `:65`, `:90`, `:98`, `:106`. Caveats: `on_tick` binds `EventType::TIMER` on the **owner's** handler, not the timer (`timer.rs:70-76`), so two timers sharing an owner cross-fire; and `Timer` stops on `Drop` (`timer.rs:130`), so it must be kept alive. |
| **The upstream-recommended pattern** | worker → `tokio::sync::mpsc` → drained inside `frame.on_idle(...)` with `event.request_more(has_more)`, plus `IdleEvent::set_mode(IdleMode::ProcessSpecified)` and `frame.set_extra_style(ExtraWindowStyle::ProcessIdle)` | **PRESENT** | `examples/rust/tokio_async_demo/src/main.rs:1-9` (header calls it "the recommended pattern"), `:93`, `:136-150`, `:227` |
| **`Send`/`Sync` on widget handles** | Widget structs are `{ handle: WindowHandle }` where `WindowHandle(u64)` — so most widgets are **auto-`Send`/`Sync`** | **PRESENT but a silent footgun — see the warning below** | `WindowHandle(u64)` at `window.rs:49`; `ListCtrl { handle: WindowHandle }` at `list_ctrl.rs:258-262`; same single-field shape for `StaticText` (`:28`), `Panel` (`:52`), `Notebook` (`:52`), `StatusBar` (`:51`), `Button` (`:47`), `TextCtrl` (`:102`) |
| …but some widgets hold raw pointers and are `!Send` | `Frame { handle, parent_ptr: *mut …, _marker }`; `Window(*mut …)`; `Timer` | — | `widgets/frame.rs:100-108`; `window.rs:258` |
| Explicit `unsafe impl Send/Sync` in the crate (exhaustive) | `App` (`app.rs:288-289`), `SystemAppearance` (`appearance.rs:166-167`), `Cursor` (`cursor.rs:316-317`), `Font` (`font.rs:71`, Send only), `Sound` (`sound.rs:73`, Send only), `NotificationMessage` (`notification_message.rs:84-85`). The `impl_refcounted_object!(send_sync …)` arm (`macros.rs:732-733`) is **never invoked**. | — | as cited; `wxdragon-sys` has zero `unsafe impl` lines |

> **⚠ The single most dangerous finding in this report.** `WindowHandle` is a `u64` index into a **`thread_local!`** registry (`window.rs:20-27`), resolved by `get_ptr()` at `window.rs:91-96`. Because the handle is a plain integer, `ListCtrl`, `Panel`, `StatusBar`, `Button` etc. are **auto-`Send`** — the compiler will happily let you move one into `thread::spawn`. On the worker thread the registry lookup **misses**, returns `None`, and every method takes its "widget has been destroyed" branch and becomes a **silent no-op** (e.g. `list_ctrl.rs:313-314`, `:366-368`). No panic, no error, no compile failure — the UI simply never updates.
>
> Using a widget inside a `call_after` closure is **safe and correct**, because the closure body executes on the UI thread where the registry lookup succeeds. The rule to write into the spec is: *widgets may be **captured** across threads but only **called** on the UI thread.*

### I. Clipboard and drag & drop — availability only (both v0.2.0)

| UI need | wxdragon API | Verdict | Evidence |
|---|---|---|---|
| Clipboard text (FR-copy-entry) | `Clipboard::get()`, `set_text(&str) -> bool`, `get_text() -> Option<String>`; RAII `Clipboard::locker()`; `clear`, `flush` | **PRESENT** | `wxdragon/src/clipboard.rs:33`, `:154`, `:173`, `:188`, `:123`, `:131`. Example: `examples/rust/clipboard_test`. Minor bug: `is_using_primary_selection()` is hardcoded `false` (`clipboard.rs:147`). |
| Clipboard data objects | `TextDataObject`, `FileDataObject`, `BitmapDataObject`; `add_data`/`set_data`/`get_data` | **PRESENT** | `wxdragon/src/data_object.rs:97`, `:166`, `:257`; `clipboard.rs:67`, `:89`, `:115`. No custom/private clipboard format can be registered. `BitmapDataObject` leaks — its `Drop` body is empty (`data_object.rs:293-303`). |
| Drag **source** | `DropSource::new(window)`, `set_data(&DataObject)`, `do_drag_drop(allow_move) -> DragResult` | **PRESENT** | `wxdragon/src/dnd/dropsource.rs:17`, `:23`, `:43`; `DragResult` at `dnd/mod.rs:21` |
| Drop **target** | `TextDropTarget::builder(&window)` (requires `with_on_drop_text`) and `FileDropTarget::builder(&window)` (requires `with_on_drop_files`); optional `with_on_enter` / `with_on_drag_over` / `with_on_leave` / `with_on_data` | **PRESENT (two flavours only)** | `wxdragon/src/dnd/droptarget.rs:161`, `:111`, `:310`, `:260`, `:65-101`. They self-install at `build()`; there is **no** `Window::set_drop_target`. |
| List-reorder DnD helpers (insertion marker, auto-scroll, `EnableDragSource`) | — | **ABSENT** | No `EnableDragSource` / `EnableDropTarget` / `InsertionMark` anywhere. Must be hand-rolled: start from `ListCtrl::on_begin_drag` (`list_ctrl.rs:1117`), carry the source index in a `TextDataObject`, map `(x, y)` → row in `with_on_drag_over` via `ListCtrl::hit_test` (`list_ctrl.rs:615`). Example: `examples/rust/dnd_advanced`. |

---

## Requirements at risk

Ordered by severity. Each names the spec requirement, what the binding cannot do, and the smallest rewrite that restores it.

### 1. FR-edit-f2 🔴 — validation cannot reject an edit *(rewrite required)*

The spec says: *"При підтвердженні (Enter) виконується валідація… У разі помилки — **поле підсвічується** і відображається текстова підказка."* That wording requires staying in the editor with the bad text highlighted.

In-place editing itself is fine (`ListCtrlStyle::EditLabels`, `list_ctrl.rs:33`; BEGIN/END events, `:1115-1116`; `get_label()`, `:184`). **What is missing is the veto.** `ListCtrlEventData` has no `veto()`/`skip()` and its inner `Event` is private (`list_ctrl.rs:157`), so `EVT_LIST_END_LABEL_EDIT` cannot be cancelled — the crate wires veto for notebooks (`notebook.rs:313`) and menus (`menu_events.rs:88`) but not here. `wxListCtrl::GetEditControl` is also unbound, so the live editor is unreachable during a user-initiated F2/double-click edit.

**Rewrite to one of:**
- **(a) Accept-then-revert.** On `on_end_label_edit`, validate `get_label()`; if invalid, `set_item_text(idx, old_value)` to restore and report the error in the status bar / banner. Cheap, but the bad value visibly commits for one frame and the user loses their typing.
- **(b) Drive editing yourself.** Do not rely on the control's native F2; handle `on_key_down`/`on_item_activated` and call `edit_label(idx)` (`list_ctrl.rs:634`), which **returns the `TextCtrl`** — bind live validation to it. Needs a spike (below).

Either way the acceptance criterion must be reworded away from "the field is highlighted and stays in edit mode".

### 2. US-admin-elevation / FR-uac-elevation 🔴 and FR-diag-length 🔴 — "InlineAlert" / "InlineBanner" do not exist *(rewrite required)*

`wxInfoBar` is **absent from both crates entirely** — zero matches, no shim header. `NotificationMessage` is an OS toast, not an in-window control, and cannot be placed in a sizer (`notification_message.rs:210`).

**Rewrite:** replace the WinUI vocabulary with a concrete wx construction — *"a `Panel` at the top of the tab body containing an `ArtProvider::get_bitmap(ArtId::Warning, …)` in a `StaticBitmap`, a `StaticText` message, an action `Button` (for "Run as Administrator"), and a dismiss `Button`; toggled with `panel.show(bool)` followed by `parent.layout()`."* All pieces verified present (§D).

This has an **accessibility consequence that belongs to ticket 08 (Live announcement mechanism), not here**: a hand-built Panel is not a live region, so showing it announces nothing to NVDA. The spec's *"NVDA оголошує попередження при появі банера"* (FR-diag-length) is exactly the "transient, non-focus message" problem the map already routes to its own ticket. Flagging it so it is not lost: **the banner mechanism chosen here must be verified against real NVDA.**

### 3. US-high-contrast 🔴 — system colours are unreadable *(rewrite required)*

The acceptance criterion is *"застосунок використовує системні кольори (не жорстко задані HEX-кольори)"*. `wxSystemSettings::GetColour` is **unbound at every level**, and there are zero `wxSYS_COLOUR_*` constants in the generated tables. There is also **no High Contrast detection** of any kind.

The requirement is still satisfiable, but only by inverting it into a **prohibition** rather than an instruction:

> **Rewrite:** *"The application never calls `set_background_color` / `set_foreground_color` / `set_item_text_colour` / `set_item_background_colour`. All controls are native Win32 and inherit system colours — including High Contrast — automatically. Status is carried by text and by an icon from `ArtProvider`, never by colour (this already follows from NFR-no-color-only)."*

Two consequences to write down:
- The hand-built banner from risk #2 must **not** set a background colour, or it will punch a hard-coded rectangle through High Contrast. If a distinct banner background is judged essential, the only in-crate source is sampling `get_background_color()` off a live control (`window.rs:942`) — or a Win32 `SPI_GETHIGHCONTRAST` FFI call of your own.
- Delete any remaining notion of app-controlled theming. This is consistent with map decision 8 (the `theme` setting is already cut), but **FR-settings-file still lists `theme` (system/high-contrast) among its parameters** — that line is now doubly dead and should go.

### 4. FR-auto-diagnose 🔴 — "asynchronous, does not block the UI" needs a named mechanism *(specify, then spike)*

The primitives exist but none of them is `wxEvtHandler::CallAfter`, and the obvious approach has a silent-failure mode:

- `call_after` (`app.rs:37`) is a Rust-side queue drained **only from the idle handler** (`cpp/src/app.cpp:100-111`), max 10 per tick, and it **does not** call `wake_up_idle()`.
- `QueueEvent` / `AddPendingEvent` / custom events with a payload are **absent**; `EventType` is a closed set.
- Widget handles are auto-`Send` but resolve through a **thread-local** registry, so calling one off the UI thread compiles and silently does nothing (see the boxed warning in §H).

**Rewrite the acceptance criterion to name the mechanism**, e.g.: *"Diagnostics run on a `std::thread`; results return over an `mpsc::channel`; the UI drains the channel from `on_idle` (per the upstream `tokio_async_demo` pattern) or from a `Timer`. No widget method is ever called from the worker thread."*

Note the 1-second budget for ≤200 entries is dominated by `os.Stat`-style filesystem checks (FR-diag-nonexistent) and is unaffected by any of this.

### 5. FR-listview-columns 🔴 — no icon in the Status column *(minor rewrite)*

*"Статусні значення: `OK`, `Warning`, `Error` з відповідними іконками."* Item images work only on **column 0** (`set_item_image`, `list_ctrl.rs:554`); `set_item_text_by_column` hardcodes `image = -1` and a text-only mask (`list_ctrl.rs:404-423`), so no sub-item can carry an icon. Column 0 is the Path.

**Rewrite to one of:** (a) status is **text-only** in the Status column — which NFR-no-color-only already demands and which NVDA reads for free from the native `SysListView32`; or (b) move the status icon to column 0 and the path to column 1, if a visual icon is judged necessary. Option (a) is the smaller change and costs nothing in accessibility.

### 6. FR-menubar 🔴 — every shortcut must hang off a menu item *(constraint to record)*

`wxAcceleratorTable` is **absent** at every level, so there is no way to register a keyboard shortcut that is not attached to a menu item. Menu items themselves accept accelerators normally, because the label string reaches `wxMenu::Append` verbatim (`menu.rs:373` → `cpp/src/menu.cpp:151`), so `"&Undo\tCtrl+Z"` works and wx builds the accelerator table itself.

In practice this costs almost nothing — FR-menubar already mandates a menu covering every command, and F5 (FR-refresh), Ctrl+Z/Ctrl+Y (FR-undo-redo), Ctrl+F (FR-search) are all menu items. F2 is handled natively by the list control. **Record it as a standing rule:** *"every keyboard shortcut in PathMaster must correspond to a MenuBar item carrying its `\t` accelerator; shortcuts with no menu home are not expressible."* Anything left over must be caught by a manual key-event handler.

### 7. FR-statusbar — a field cannot be styled *(minor rewrite)*

Multi-field status bars work (`set_fields_count`, `set_status_widths`, `set_status_text(text, index)` — `statusbar.rs:95`, `:126`, `:107`). But the shim exports only six functions and **no** `SetStatusStyles`, so *"секція 'Total length' **виділяється**"* is not expressible, and there are no getters to read a field back.

**Rewrite:** the over-limit state is conveyed by the field's **text only** — `"Total length: N chars ⚠ Exceeds limit"`. (The spec's own fallback wording already says the section "містить текст '⚠ Exceeds limit'", so only the word *виділяється* needs to go.)

### 8. Requirements that are fine, contrary to the ticket's suspicion

Worth stating explicitly so they are not re-litigated:

- **FR-close-confirm 🔴 (Save/Discard/Cancel) — satisfiable.** `MessageDialog` genuinely cannot relabel its buttons (no FFI for it at all), but the generic `Dialog` + `end_modal(code)` path is real and proven by a runnable upstream example (`examples/rust/generic_dialog_test/src/main.rs:17-55`). Same for FR-apply's `[Overwrite] [Refresh and discard] [Cancel]`. Only caveat: `ID_SAVE` is missing from `id.rs` — use `ID_YES`/`ID_NO`/`ID_CANCEL` or `wxdragon_sys::WXD_ID_SAVE`.
- **US-i18n / FR-i18n-runtime + NFR-portable — satisfiable, and better than expected.** The `TranslationsLoader` trait (`translations.rs:429`, `set_loader` at `:188`) serves `.mo` bytes **from memory**, with a passing unit test for the embedded path (`translations.rs:833`). Catalogs can be `include_bytes!`-ed straight into the exe; no `locale/` directory on disk is needed, so TC-file-structure and NFR-portable hold. Ukrainian is `Language::Ukrainian = 217` (`language.rs:458`), and `Locale::get_system_language()` (`:700`) supplies the "default = system locale" rule.
- **FR-view-tabs 🔴 — satisfiable.** `Notebook` is complete for this use. Tabs cannot be renamed after creation, which is irrelevant because language change is restart-only (map decision 6).
- **FR-browse-folder 🔴 — satisfiable.** `DirDialog` is complete; just remember to call `.destroy()` (its `Drop` does not, `dir_dialog.rs:82-90`).
- **Accessibility plumbing — present.** `set_accessibility_label` / `_description` / `_role` / `_state` and a full custom `wxAccessible` provider all exist, unconditionally compiled. Whether NVDA actually *announces* what we need is the business of tickets 02 and 08, not an API gap.
- **v0.2.0 deferrals are not blocked.** Clipboard (FR-copy-entry) is fully present. Drag & drop source *and* target are present (FR-reorder-dnd); only the reorder ergonomics — insertion marker, auto-scroll — must be hand-rolled.

---

## UNKNOWN — needs a spike

1. **Does `edit_label()` interoperate with native F2?** `wxLC_EDIT_LABELS` makes the control handle F2 and double-click itself. Whether you can suppress that and drive editing entirely through `edit_label()` (to get the `TextCtrl` back for live validation, per risk #1 option b) — or whether the two race — is not answerable from source. Decides FR-edit-f2's final wording.
2. **Does a worker-queued `call_after` run while the app is idle?** `call_after` does not call `wake_up_idle()` (`app.rs:37-43`), and the idle handler only re-requests when it processed something (`app.cpp:107-111`). Whether a callback queued during true idle is delivered promptly, or waits for the next mouse move, needs measuring. `wxWakeUpIdle()` is documented thread-safe by wxWidgets and is the expected fix, but that pairing is not proven here.
3. **Is the wxWidgets 3.3.3 MSW dark-mode path safe to leave at `Appearance::System`?** `set_appearance` is bound and `dark_mode_demo` exists, but 3.3.x MSW dark mode is young and its interaction with High Contrast is exactly the collision US-high-contrast cares about. Cheap to check once something is running.
4. **Do the C-ABI-only escapes work?** `wxd_ListCtrl_SortItems` (`cpp/include/widgets/wxd_listctrl.h:104`) and `WXD_ID_SAVE` are reachable only as raw `wxdragon_sys::` calls. Neither is needed for v0.1.0 as currently scoped — PATH order is semantic, so column sorting would be actively wrong — but if either is ever wanted, its usability is unproven.

*(All four are cheap and none blocks writing the spec — they refine wording, not structure.)*
