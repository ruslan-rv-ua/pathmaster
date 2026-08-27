//! THROWAWAY PROTOTYPE — wayfinder v0.2.0 ticket 16, "NVDA verification round 2".
//!
//! One window, one listening session, seven obligations accumulated by the feature
//! contracts (tickets 05-09). Everything here is placeholder wording and hardcoded
//! data — the contracts own the real sentences; this window only exists so NVDA's
//! behaviour is a measurement instead of an argument.
//!
//! The seven probes (numbering matches the ticket):
//!   1. View → "Expanded values" (Ctrl+E): does NVDA say "checked"/"not checked"
//!      when arrowing onto a wxITEM_CHECK menu item in both states?
//!   2. With a narrowed Filter, Ctrl+E speaks the mode message and then, after the
//!      tuned debounce (Options menu, 250/500/750/1000 ms), the count — do both
//!      land, or does the second cut the first off? Record the shortest reliable
//!      separation.
//!   3. View → Filter: seven wxITEM_RADIO items — does NVDA name the selected one?
//!      Switch tabs with different Filters and re-open the menu: does the checked
//!      item follow the active Scope?
//!   4. In the tree (Ctrl+T): does a compressed chain node ("Program Files\Java\
//!      jdk-21") speak its whole joined label, its level, and "N of M"?
//!   5. Does a three-part leaf ("bin (%JAVA_HOME%\bin) — Missing") speak in full,
//!      %VAR% text included?
//!   6. Enter on a leaf / the "Go to entry" button: modal closes, the Entry's row
//!      is selected in the main list — does NVDA speak the landed row with no dead
//!      silence and no dialog-title residue? And after Cancel/Esc, the restored
//!      focus?
//!   7. In the Fix Issues dialog (Ctrl+I): native LVS_EX_CHECKBOXES rows enabled
//!      through the raw-LVM_* hatch — does NVDA read "checked"/"not checked" per
//!      row, does Space announce the toggle, and does Space toggle the native
//!      state image at all with the wx event layer silent?
//!
//! Fidelity notes: Expansion Mode is app-wide (ticket 05), the Filter is per-Scope
//! (ticket 07), the tree is hardcoded to the exact shapes the probes need rather
//! than merged from the entries (the merge algorithm is not under test), and "Fix
//! selected" changes nothing — it only reads the native check states back and
//! announces the count (Announcement 12's placeholder).

#![windows_subsystem = "windows"]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use wxdragon::id::ID_EXIT;
use wxdragon::prelude::*;
use wxdragon::timer::Timer;
use wxdragon::widgets::item_data::HasItemData;
use wxdragon::widgets::statusbar::StatusBar;

use windows_sys::Win32::UI::Accessibility::NotifyWinEvent;
use windows_sys::Win32::UI::Controls::{
    LVITEMW, LVIS_STATEIMAGEMASK, LVM_GETITEMSTATE, LVM_SETEXTENDEDLISTVIEWSTYLE,
    LVM_SETITEMSTATE, LVS_EX_CHECKBOXES,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SendMessageW, CHILDID_SELF, EVENT_OBJECT_LIVEREGIONCHANGED, OBJID_CLIENT,
};

const ID_EXPANDED: Id = 7201;
const ID_TREE: Id = 7202;
const ID_FIX: Id = 7203;
// Filter radio ids: ID_FILTER_BASE + Filter index (0 = All .. 6 = Empty).
const ID_FILTER_BASE: Id = 7210;
const ID_DEBOUNCE_250: Id = 7221;
const ID_DEBOUNCE_500: Id = 7222;
const ID_DEBOUNCE_750: Id = 7223;
const ID_DEBOUNCE_1000: Id = 7224;
// Dialog exits.
const ID_GO: Id = 7231;
const ID_DIALOG_CANCEL: Id = 7232;

// ---------------------------------------------------------------------------
// Data: two Scopes of fake entries. Issue strings are the product's own
// (msgids ISSUE_*); the tree specs below hand-carve exactly the node shapes
// the probes need and point each leaf at its entry by index.
// ---------------------------------------------------------------------------

struct Entry {
    raw: &'static str,
    expanded: &'static str,
    issue: Option<&'static str>,
}

const fn e(raw: &'static str, expanded: &'static str, issue: Option<&'static str>) -> Entry {
    Entry { raw, expanded, issue }
}

const USER_ENTRIES: &[Entry] = &[
    e(r"%JAVA_HOME%\bin", r"C:\Program Files\Java\jdk-21\bin", Some("Missing")),
    e(r"C:\Users\Ruslan\.cargo\bin", r"C:\Users\Ruslan\.cargo\bin", None),
    e(r"C:\scoop\shims", r"C:\scoop\shims", None),
    e(r"C:\scoop\apps\python\current", r"C:\scoop\apps\python\current", None),
    e(r"C:\scoop\apps\git\current\usr\bin", r"C:\scoop\apps\git\current\usr\bin", None),
    e(r"C:\scoop\shims", r"C:\scoop\shims", Some("Duplicate")),
    e(r#""C:\Tools\Ripgrep""#, r#""C:\Tools\Ripgrep""#, Some("Quoted")),
    e("", "", Some("Empty")),
    e(r".\tools", r".\tools", Some("Relative")),
    e(r"%FOO%\bin", r"%FOO%\bin", Some("Missing")),
];

const SYSTEM_ENTRIES: &[Entry] = &[
    e(r"C:\WINDOWS\system32", r"C:\WINDOWS\system32", None),
    e(r"C:\WINDOWS", r"C:\WINDOWS", None),
    e(r"%SystemRoot%\System32\Wbem", r"C:\WINDOWS\System32\Wbem", None),
    e(r"C:\Program Files\PowerShell\7", r"C:\Program Files\PowerShell\7", None),
    e(r"C:\Old\Removed\Tool", r"C:\Old\Removed\Tool", Some("Missing")),
    e(r"C:\WINDOWS", r"C:\WINDOWS", Some("Duplicate")),
];

/// Preorder tree spec: (depth, label, Some(entry index) for a leaf).
/// Probe 4 lives on the compressed "Program Files\Java\jdk-21" node; probe 5 on
/// its three-part leaf. Group nodes ("Unresolved variables", "Relative entries")
/// are ticket 08's misfit homes.
type TreeSpec = &'static [(u8, &'static str, Option<usize>)];

const USER_TREE: TreeSpec = &[
    (0, "C:", None),
    (1, r"Program Files\Java\jdk-21", None),
    (2, r"bin (%JAVA_HOME%\bin) — Missing", Some(0)),
    (1, r"Users\Ruslan\.cargo", None),
    (2, "bin", Some(1)),
    (1, "scoop", None),
    (2, "shims", Some(2)),
    (2, "shims — Duplicate", Some(5)),
    (2, r"apps\python", None),
    (3, "current", Some(3)),
    (2, r"apps\git\current\usr", None),
    (3, "bin", Some(4)),
    (1, "Tools", None),
    (2, "Ripgrep (\"C:\\Tools\\Ripgrep\") — Quoted", Some(6)),
    (0, "Unresolved variables", None),
    (1, r"%FOO%\bin — Missing", Some(9)),
    (0, "Relative entries", None),
    (1, r".\tools — Relative", Some(8)),
];

const SYSTEM_TREE: TreeSpec = &[
    (0, "C:", None),
    (1, "WINDOWS", None),
    (2, "system32", Some(0)),
    (2, "System32", None),
    (3, r"Wbem (%SystemRoot%\System32\Wbem)", Some(2)),
    (1, "WINDOWS", Some(1)),
    (1, "WINDOWS — Duplicate", Some(5)),
    (1, r"Program Files\PowerShell", None),
    (2, "7", Some(3)),
    (1, r"Old\Removed", None),
    (2, "Tool — Missing", Some(4)),
];

/// The seven Filter states of ticket 07, in the product's canonical Issue order.
const FILTERS: &[&str] = &["All", "With issues", "Missing", "Relative", "Quoted", "Duplicate", "Empty"];

// ---------------------------------------------------------------------------

/// The v0.1.0 voice, verbatim: label the Banner, then fire LIVEREGIONCHANGED.
fn announce(banner: &StaticText, text: &str) {
    banner.set_label(text);
    let hwnd = banner.get_handle();
    if hwnd.is_null() {
        return;
    }
    // SAFETY: fire-and-forget notification on a live window handle.
    unsafe {
        NotifyWinEvent(
            EVENT_OBJECT_LIVEREGIONCHANGED,
            hwnd,
            OBJID_CLIENT,
            CHILDID_SELF as i32,
        );
    }
}

fn visible_for(entries: &[Entry], filter: usize) -> Vec<usize> {
    (0..entries.len())
        .filter(|&i| match filter {
            0 => true,
            1 => entries[i].issue.is_some(),
            f => entries[i].issue == Some(FILTERS[f]),
        })
        .collect()
}

/// One Scope tab: its list, its data, its own Filter (per-Scope, ticket 07).
struct ScopePage {
    name: &'static str,
    list: ListCtrl,
    entries: &'static [Entry],
    tree: TreeSpec,
    visible: RefCell<Vec<usize>>,
    filter: Cell<usize>,
}

impl ScopePage {
    /// Rebuild to the current filter + expansion mode; keep focus by the
    /// ticket-03 rule (same entry if it survived, else same position clamped).
    fn rebuild(&self, expanded: bool) {
        let new_visible = visible_for(self.entries, self.filter.get());
        let old_visible = self.visible.borrow().clone();
        let prev_row = (0..old_visible.len() as i64)
            .find(|&row| self.list.get_item_state(row, ListItemState::Focused))
            .unwrap_or(0);
        let keep = old_visible.get(prev_row as usize).copied();

        self.list.delete_all_items();
        for (row, &original) in new_visible.iter().enumerate() {
            let entry = &self.entries[original];
            self.list.insert_item(row as i64, &format!("{}", original + 1), None);
            self.list.set_item_text_by_column(
                row as i64,
                1,
                if expanded { entry.expanded } else { entry.raw },
            );
            self.list.set_item_text_by_column(row as i64, 2, entry.issue.unwrap_or(""));
        }
        if !new_visible.is_empty() {
            let target = keep
                .and_then(|orig| new_visible.iter().position(|&v| v == orig))
                .map(|row| row as i64)
                .unwrap_or_else(|| prev_row.clamp(0, new_visible.len() as i64 - 1));
            self.list.set_item_state(
                target,
                ListItemState::Focused | ListItemState::Selected,
                ListItemState::Focused | ListItemState::Selected,
            );
            self.list.ensure_visible(target);
        }
        *self.visible.borrow_mut() = new_visible;
    }

    /// Ticket 07's count pair (placeholder wording).
    fn count_text(&self) -> String {
        let shown = self.visible.borrow().len();
        let total = self.entries.len();
        if self.filter.get() == 0 {
            format!("All {} entries", total)
        } else {
            format!("{}: {} of {} entries", FILTERS[self.filter.get()], shown, total)
        }
    }

    /// Select `original` in the list (it must be visible) and hand it focus.
    fn focus_entry(&self, original: usize) {
        if let Some(row) = self.visible.borrow().iter().position(|&v| v == original) {
            self.list.set_focus();
            self.list.set_item_state(
                row as i64,
                ListItemState::Focused | ListItemState::Selected,
                ListItemState::Focused | ListItemState::Selected,
            );
            self.list.ensure_visible(row as i64);
        }
    }
}

fn build_page(notebook: &Notebook, name: &'static str, entries: &'static [Entry], tree: TreeSpec) -> ScopePage {
    let panel = Panel::builder(notebook).build();
    let list = ListCtrl::builder(&panel)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel)
        .build();
    list.insert_column(0, "#", ListColumnFormat::Right, 48);
    list.insert_column(1, "Path", ListColumnFormat::Left, 560);
    list.insert_column(2, "Issue", ListColumnFormat::Left, 140);

    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 4);
    panel.set_sizer(sizer, true);
    notebook.add_page(&panel, name, false, None);

    ScopePage {
        name,
        list,
        entries,
        tree,
        // Starts empty — the list holds no rows yet, and rebuild()'s focus scan
        // asks the control about every row it thinks is there.
        visible: RefCell::new(Vec::new()),
        filter: Cell::new(0),
    }
}

// ---------------------------------------------------------------------------
// Probes 4-6: the tree dialog. Hardcoded shape, modal, Enter-on-leaf goes,
// Enter-on-inner toggles, "Go to entry" default button, Cancel/Esc restores.
// ---------------------------------------------------------------------------

fn show_tree_dialog(frame: &Frame, page: &ScopePage) -> Option<usize> {
    let dialog = Dialog::builder(frame, &format!("PATH tree — {}", page.name)).build();
    let panel = Panel::builder(&dialog).build();

    let tree = TreeCtrl::builder(&panel)
        .with_style(
            TreeCtrlStyle::HasButtons
                | TreeCtrlStyle::LinesAtRoot
                | TreeCtrlStyle::HideRoot
                | TreeCtrlStyle::Single,
        )
        .with_size(Size::new(560, 380))
        .build();

    let root = tree
        .add_root("PATH", None, None)
        .expect("prototype: tree root");
    // Build the hardcoded spec with a per-depth parent stack.
    let mut stack: Vec<TreeItemId> = Vec::new();
    let mut first: Option<TreeItemId> = None;
    for &(depth, label, entry) in page.tree {
        let parent = if depth == 0 { &root } else { &stack[depth as usize - 1] };
        let item = tree
            .append_item(parent, label, None, None)
            .expect("prototype: tree item");
        if let Some(original) = entry {
            tree.set_custom_data_direct(&item, original);
        }
        if first.is_none() {
            first = Some(item.clone());
        }
        stack.truncate(depth as usize);
        stack.push(item);
    }
    tree.expand_all();

    let go = Button::builder(&panel).with_id(ID_GO).with_label("&Go to entry").build();
    let cancel = Button::builder(&panel)
        .with_id(ID_DIALOG_CANCEL)
        .with_label("Cancel")
        .build();
    go.set_default();

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    buttons.add_stretch_spacer(1);
    buttons.add(&go, 0, SizerFlag::All, 4);
    buttons.add(&cancel, 0, SizerFlag::All, 4);

    let inner = BoxSizer::builder(Orientation::Vertical).build();
    inner.add(&tree, 1, SizerFlag::Expand | SizerFlag::All, 8);
    inner.add_sizer(&buttons, 0, SizerFlag::Expand | SizerFlag::All, 8);
    panel.set_sizer(inner, true);

    let outer = BoxSizer::builder(Orientation::Vertical).build();
    outer.add(&panel, 1, SizerFlag::Expand, 0);
    dialog.set_sizer_and_fit(outer, true);
    dialog.centre();

    // Leaf → chosen entry index, read back after the modal closes.
    let chosen: Rc<Cell<Option<usize>>> = Rc::new(Cell::new(None));

    let leaf_of = move |tree: &TreeCtrl, item: &TreeItemId| -> Option<usize> {
        tree.get_custom_data(item)
            .and_then(|data| data.downcast_ref::<usize>().copied())
    };

    {
        let chosen = chosen.clone();
        tree.on_item_activated(move |event| {
            if let Some(item) = event.get_item() {
                match leaf_of(&tree, &item) {
                    Some(original) => {
                        chosen.set(Some(original));
                        dialog.end_modal(ID_GO);
                    }
                    // Enter on an inner node expands/collapses (ticket 08).
                    None => tree.toggle(&item),
                }
            }
        });
    }
    {
        let chosen = chosen.clone();
        go.on_click(move |_| {
            if let Some(item) = tree.get_selection() {
                if let Some(original) = leaf_of(&tree, &item) {
                    chosen.set(Some(original));
                    dialog.end_modal(ID_GO);
                }
            }
        });
    }
    cancel.on_click(move |_| dialog.end_modal(ID_DIALOG_CANCEL));
    dialog.set_escape_id(ID_DIALOG_CANCEL);

    if let Some(item) = &first {
        tree.select_item(item);
        tree.set_focused_item(item);
    }
    tree.set_focus();
    dialog.show_modal();
    let result = chosen.get();
    dialog.destroy();
    result
}

// ---------------------------------------------------------------------------
// Probe 7: the Fix Issues dialog. Native LVS_EX_CHECKBOXES through the raw
// LVM_* hatch (in-process — the cross-process crash rule does not apply).
// Disk-Cleanup defaults: safe rows on, Missing rows off when the raw text
// carries %VAR%. "Fix selected" only reads the states back and reports.
// ---------------------------------------------------------------------------

fn set_row_checked(hwnd: *mut core::ffi::c_void, row: usize, checked: bool) {
    // SAFETY: in-process SendMessage on a live SysListView32; LVITEMW lives on
    // this stack for the duration of the synchronous call.
    unsafe {
        let mut item: LVITEMW = std::mem::zeroed();
        item.stateMask = LVIS_STATEIMAGEMASK;
        item.state = (if checked { 2 } else { 1 }) << 12;
        SendMessageW(hwnd, LVM_SETITEMSTATE, row, &item as *const LVITEMW as isize);
    }
}

fn row_checked(hwnd: *mut core::ffi::c_void, row: usize) -> bool {
    // SAFETY: value-carrying message, in-process.
    let state = unsafe { SendMessageW(hwnd, LVM_GETITEMSTATE, row, LVIS_STATEIMAGEMASK as isize) };
    ((state as u32 & LVIS_STATEIMAGEMASK) >> 12) == 2
}

/// Answers with (checked, flagged) counts, or None on Cancel.
fn show_fix_dialog(frame: &Frame, page: &ScopePage) -> Option<(usize, usize)> {
    // The four fixable Issue types (ticket 09): Missing, Duplicate, Empty, Quoted.
    let rows: Vec<usize> = (0..page.entries.len())
        .filter(|&i| {
            matches!(page.entries[i].issue, Some("Missing" | "Duplicate" | "Empty" | "Quoted"))
        })
        .collect();

    let dialog = Dialog::builder(frame, &format!("Fix issues — {}", page.name)).build();
    let panel = Panel::builder(&dialog).build();

    let list = ListCtrl::builder(&panel)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel)
        .with_size(Size::new(680, 240))
        .build();
    list.insert_column(0, "#", ListColumnFormat::Right, 48);
    list.insert_column(1, "Path", ListColumnFormat::Left, 380);
    list.insert_column(2, "Issue", ListColumnFormat::Left, 100);
    list.insert_column(3, "Action", ListColumnFormat::Left, 130);
    for (row, &original) in rows.iter().enumerate() {
        let entry = &page.entries[original];
        let issue = entry.issue.unwrap_or("");
        list.insert_item(row as i64, &format!("{}", original + 1), None);
        list.set_item_text_by_column(row as i64, 1, entry.raw);
        list.set_item_text_by_column(row as i64, 2, issue);
        list.set_item_text_by_column(
            row as i64,
            3,
            if issue == "Quoted" { "Remove quotes" } else { "Remove entry" },
        );
    }

    // The hatch: enable native checkboxes, then set the Disk-Cleanup defaults.
    let hwnd = list.get_handle();
    if !hwnd.is_null() {
        // SAFETY: in-process, value-carrying.
        unsafe {
            SendMessageW(
                hwnd,
                LVM_SETEXTENDEDLISTVIEWSTYLE,
                LVS_EX_CHECKBOXES as usize,
                LVS_EX_CHECKBOXES as isize,
            );
        }
        for (row, &original) in rows.iter().enumerate() {
            let entry = &page.entries[original];
            let on = !(entry.issue == Some("Missing") && entry.raw.contains('%'));
            set_row_checked(hwnd, row, on);
        }
    }
    if !rows.is_empty() {
        list.set_item_state(
            0,
            ListItemState::Focused | ListItemState::Selected,
            ListItemState::Focused | ListItemState::Selected,
        );
    }

    let fix = Button::builder(&panel).with_id(ID_GO).with_label("&Fix selected").build();
    let cancel = Button::builder(&panel)
        .with_id(ID_DIALOG_CANCEL)
        .with_label("Cancel")
        .build();
    fix.set_default();

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    buttons.add_stretch_spacer(1);
    buttons.add(&fix, 0, SizerFlag::All, 4);
    buttons.add(&cancel, 0, SizerFlag::All, 4);

    let inner = BoxSizer::builder(Orientation::Vertical).build();
    inner.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 8);
    inner.add_sizer(&buttons, 0, SizerFlag::Expand | SizerFlag::All, 8);
    panel.set_sizer(inner, true);

    let outer = BoxSizer::builder(Orientation::Vertical).build();
    outer.add(&panel, 1, SizerFlag::Expand, 0);
    dialog.set_sizer_and_fit(outer, true);
    dialog.centre();

    // Read the native states back *inside* the click handler, while the list
    // lives; stash the count for after the modal closes.
    let outcome: Rc<Cell<Option<(usize, usize)>>> = Rc::new(Cell::new(None));
    {
        let outcome = outcome.clone();
        let total = rows.len();
        fix.on_click(move |_| {
            let hwnd = list.get_handle();
            let checked = if hwnd.is_null() {
                0
            } else {
                (0..total).filter(|&row| row_checked(hwnd, row)).count()
            };
            outcome.set(Some((checked, total)));
            dialog.end_modal(ID_GO);
        });
    }
    cancel.on_click(move |_| dialog.end_modal(ID_DIALOG_CANCEL));
    dialog.set_escape_id(ID_DIALOG_CANCEL);

    list.set_focus();
    dialog.show_modal();
    let result = outcome.get();
    dialog.destroy();
    result
}

// ---------------------------------------------------------------------------

fn main() {
    let _ = wxdragon::main(|_| {
        let frame = Frame::builder()
            .with_title("PathMaster — NVDA round 2 prototype (ticket 16)")
            .with_size(Size::new(880, 620))
            .build();
        frame.set_min_size(Size::new(760, 560));

        // Menus. View carries the two probe surfaces (check item, radio submenu);
        // Options carries the debounce tuning for probe 2.
        let file_menu = Menu::builder()
            .append_item(ID_EXIT, "E&xit\tAlt+F4", "Close the prototype")
            .build();
        let edit_menu = Menu::builder()
            .append_item(ID_FIX, "&Fix issues…\tCtrl+I", "Open the checkbox dialog (probe 7)")
            .build();
        let filter_menu = Menu::builder().build();
        for (i, name) in FILTERS.iter().enumerate() {
            filter_menu.append(
                ID_FILTER_BASE + i as Id,
                name,
                "Filter the active Scope (probe 3)",
                ItemKind::Radio,
            );
        }
        let view_menu = Menu::builder()
            .append_check_item(ID_EXPANDED, "E&xpanded values\tCtrl+E", "Toggle Expansion Mode (probes 1 and 2)")
            .build();
        view_menu.append_separator();
        view_menu.append_submenu(filter_menu, "&Filter", "Per-Scope filter (probe 3)");
        view_menu.append_separator();
        view_menu.append(ID_TREE, "PATH &tree…\tCtrl+T", "Open the tree browser (probes 4-6)", ItemKind::Normal);
        let options_menu = Menu::builder()
            .append_radio_item(ID_DEBOUNCE_250, "Debounce &250 ms\tCtrl+1", "Mode-to-count separation (probe 2)")
            .append_radio_item(ID_DEBOUNCE_500, "Debounce &500 ms\tCtrl+2", "Mode-to-count separation (probe 2)")
            .append_radio_item(ID_DEBOUNCE_750, "Debounce &750 ms\tCtrl+3", "Mode-to-count separation (probe 2)")
            .append_radio_item(ID_DEBOUNCE_1000, "Debounce 1&000 ms\tCtrl+4", "Mode-to-count separation (probe 2)")
            .build();
        let menu_bar = MenuBar::builder()
            .append(file_menu, "&File")
            .append(edit_menu, "&Edit")
            .append(view_menu, "&View")
            .append(options_menu, "&Options")
            .build();
        // The Filter items are needed at runtime (probe 3's per-Scope sync and
        // the tree's fall-back-to-All), and `set_menu_bar` consumes the bar —
        // so grab them first.
        let filter_items: Rc<Vec<MenuItem>> = Rc::new(
            (0..FILTERS.len())
                .map(|i| {
                    menu_bar
                        .find_item(ID_FILTER_BASE + i as Id)
                        .expect("prototype: filter item")
                })
                .collect(),
        );
        for id in [ID_FILTER_BASE, ID_DEBOUNCE_250] {
            if let Some(item) = menu_bar.find_item(id) {
                item.check(true);
            }
        }
        frame.set_menu_bar(menu_bar);

        let root = Panel::builder(&frame).build();

        // Banner: the announcement voice's live region, fixed height, empty label.
        let banner = StaticText::builder(&root).with_label("").build();

        let notebook = Notebook::builder(&root).build();
        let pages: Rc<Vec<ScopePage>> = Rc::new(vec![
            build_page(&notebook, "User", USER_ENTRIES, USER_TREE),
            build_page(&notebook, "System", SYSTEM_ENTRIES, SYSTEM_TREE),
        ]);

        let root_sizer = BoxSizer::builder(Orientation::Vertical).build();
        root_sizer.add(&banner, 0, SizerFlag::Expand | SizerFlag::All, 6);
        root_sizer.add(&notebook, 1, SizerFlag::Expand | SizerFlag::All, 6);
        root.set_sizer(root_sizer, true);

        let status_bar: StatusBar = frame.create_status_bar(2, 0, ID_ANY as Id, "");
        status_bar.set_status_widths(&[-3, -1]);

        let expanded = Rc::new(Cell::new(false));
        let debounce_ms = Rc::new(Cell::new(250i32));

        let active = {
            let pages = pages.clone();
            move || {
                let index = notebook.selection().max(0) as usize;
                index.min(pages.len() - 1)
            }
        };

        let show_state = {
            let pages = pages.clone();
            let expanded = expanded.clone();
            let debounce_ms = debounce_ms.clone();
            let active = active.clone();
            move || {
                let page = &pages[active()];
                status_bar.set_status_text(
                    &format!(
                        "{} · {} values · Filter: {} · debounce {} ms",
                        page.name,
                        if expanded.get() { "expanded" } else { "raw" },
                        FILTERS[page.filter.get()],
                        debounce_ms.get(),
                    ),
                    0,
                );
                status_bar.set_status_text(
                    &format!("{} of {}", page.visible.borrow().len(), page.entries.len()),
                    1,
                );
            }
        };

        for page in pages.iter() {
            page.rebuild(false);
        }
        show_state();

        // Probe 2's second voice: one-shot timer, fired by the Ctrl+E handler.
        let timer = Timer::new(&frame);
        {
            let pages = pages.clone();
            let active = active.clone();
            timer.on_tick(move |_| {
                announce(&banner, &pages[active()].count_text());
            });
        }
        let timer = Rc::new(timer);

        {
            let pages = pages.clone();
            let active = active.clone();
            let show_state = show_state.clone();
            let filter_items = filter_items.clone();
            notebook.on_page_changed(move |_| {
                // The menu is shared; the checked Filter radio must follow the
                // active Scope (probe 3's second half).
                let page = &pages[active()];
                filter_items[page.filter.get()].check(true);
                show_state();
            });
        }

        let frame_for_menu = frame; // Frame is Copy
        let pages_in_menu = pages.clone();
        let pages = pages.clone();
        frame.on_menu(move |event| {
            let pages = &pages_in_menu;
            let id = event.get_id();
            match id {
                ID_EXIT => {
                    frame_for_menu.close(true);
                    return;
                }
                ID_EXPANDED => {
                    // Probe 1 (checked state) + probe 2 (mode message, then count
                    // after the debounce, but only when a Filtered View is active).
                    expanded.set(!expanded.get());
                    let page = &pages[active()];
                    page.rebuild(expanded.get());
                    announce(
                        &banner,
                        if expanded.get() { "Showing expanded values" } else { "Showing raw values" },
                    );
                    if page.filter.get() != 0 {
                        timer.start(debounce_ms.get(), true);
                    }
                }
                ID_TREE => {
                    // Probes 4-6. If the chosen entry is filtered out, fall back
                    // to All first so the landing row exists.
                    let page = &pages[active()];
                    if let Some(original) = show_tree_dialog(&frame_for_menu, page) {
                        if !page.visible.borrow().contains(&original) {
                            page.filter.set(0);
                            filter_items[0].check(true);
                            page.rebuild(expanded.get());
                        }
                        page.focus_entry(original);
                    }
                }
                ID_FIX => {
                    // Probe 7. Nothing changes; the count is the read-back proof.
                    let page = &pages[active()];
                    if let Some((checked, flagged)) = show_fix_dialog(&frame_for_menu, page) {
                        page.list.set_focus();
                        // Announcement 12's placeholder.
                        announce(&banner, &format!("Fixed {} entries", checked));
                        status_bar.set_status_text(
                            &format!("Would fix {} of {} flagged rows (prototype: list unchanged)", checked, flagged),
                            0,
                        );
                        return; // keep the "would fix" status line visible
                    }
                }
                ID_DEBOUNCE_250 => debounce_ms.set(250),
                ID_DEBOUNCE_500 => debounce_ms.set(500),
                ID_DEBOUNCE_750 => debounce_ms.set(750),
                ID_DEBOUNCE_1000 => debounce_ms.set(1000),
                _ if (ID_FILTER_BASE..ID_FILTER_BASE + FILTERS.len() as Id).contains(&id) => {
                    // Probe 3: per-Scope radio filter; count spoken immediately.
                    let page = &pages[active()];
                    page.filter.set((id - ID_FILTER_BASE) as usize);
                    page.rebuild(expanded.get());
                    announce(&banner, &page.count_text());
                }
                _ => return,
            }
            show_state();
        });

        frame.centre();
        frame.show(true);
        pages[0].list.set_focus();
    });
}
