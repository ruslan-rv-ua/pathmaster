//! THROWAWAY PROTOTYPE — wayfinder ticket 02, "NVDA baseline for a stock wxdragon shell".
//!
//! This exists to be *listened to*, not read. It carries PathMaster's real shape — menubar with
//! accelerators and a disabled item, a three-tab notebook, a report-mode ListCtrl with Path/Status
//! columns, an empty list, four buttons, a two-field status bar — and **deliberately contains no
//! accessibility code of any kind**. Not one `set_accessibility_*` call, no `set_name`, no
//! `Accessible`, no `NotifyWinEvent`.
//!
//! That prohibition is the whole point. Per ticket 01, `wxWindow::CreateAccessible()` returns
//! `nullptr` by default and no wx control overrides it, so `WM_GETOBJECT` goes unhandled and
//! comctl32's *own* IAccessible serves the list rows. The first `set_accessibility_*` call on a
//! widget flips it onto the wx-mediated path. So anything added here would destroy the measurement.
//!
//! Nothing is wired up except Exit — buttons and menu items are inert on purpose. The question is
//! what NVDA *says*, not what the app does.

#![windows_subsystem = "windows"]

use wxdragon::id::{ID_ABOUT, ID_EXIT};
use wxdragon::prelude::*;

// Custom command ids. Values are arbitrary and above ID_HIGHEST's usual range.
const ID_APPLY_CHANGES: Id = 6001;
const ID_REFRESH: Id = 6002;
const ID_UNDO: Id = 6003;
const ID_REDO: Id = 6004;
const ID_ADD: Id = 6005;
const ID_DELETE: Id = 6006;
const ID_MOVE_UP: Id = 6007;
const ID_MOVE_DOWN: Id = 6008;
const ID_EXPAND_VARS: Id = 6009;
const ID_DIAGNOSE: Id = 6010;
const ID_SETTINGS: Id = 6011;
const ID_SHOW_STATUSBAR: Id = 6012;

/// User PATH rows. Deliberately mixed: clean entries, a duplicate, a missing folder, an unexpanded
/// variable, a relative path, and one **empty** path cell — an empty sub-item is a known screen
/// reader edge and costs nothing to include.
const USER_ROWS: &[(&str, &str)] = &[
    (r"C:\Users\Ruslan\AppData\Local\Microsoft\WindowsApps", "OK"),
    (r"C:\Program Files\Git\cmd", "OK"),
    (r"C:\scoop\shims", "OK"),
    (r"%USERPROFILE%\.cargo\bin", "OK"),
    (r"C:\Program Files\nodejs", "OK"),
    (r"C:\scoop\shims", "Warning: Duplicate"),
    (r"C:\Tools\NoSuchFolder", "Error: Path does not exist"),
    (r".\relative\bin", "Warning: Relative path"),
    ("", "Error: Empty entry"),
    (r"C:\Program Files\PowerShell\7", "OK"),
    (r"C:\Program Files\dotnet", "OK"),
];

/// System PATH rows — shorter, so the two lists are distinguishable by ear.
const SYSTEM_ROWS: &[(&str, &str)] = &[
    (r"C:\Windows\system32", "OK"),
    (r"C:\Windows", "OK"),
    (r"C:\Windows\System32\Wbem", "OK"),
    (r"C:\Windows\System32\WindowsPowerShell\v1.0", "OK"),
    (r"C:\Windows\System32\OpenSSH\", "OK"),
    (r"C:\Program Files\Docker\Docker\resources\bin", "Error: Path does not exist"),
];

/// The Backups tab. Empty on purpose — question 4 of the ticket is what NVDA says when focus lands
/// on a list with no rows at all.
const BACKUP_ROWS: &[(&str, &str)] = &[];

fn main() {
    let _ = wxdragon::main(|_| {
        let frame = Frame::builder()
            .with_title("PathMaster — NVDA baseline prototype")
            .with_size(Size::new(920, 620))
            .build();
        frame.set_min_size(Size::new(800, 600));

        build_menu_bar(&frame);

        let root = Panel::builder(&frame).build();
        let notebook = Notebook::builder(&root).build();

        let user_page = build_path_page(&notebook, USER_ROWS);
        let system_page = build_path_page(&notebook, SYSTEM_ROWS);
        let backups_page = build_path_page(&notebook, BACKUP_ROWS);

        notebook.add_page(&user_page, "User PATH", true, None);
        notebook.add_page(&system_page, "System PATH", false, None);
        notebook.add_page(&backups_page, "Backups", false, None);

        let root_sizer = BoxSizer::builder(Orientation::Vertical).build();
        root_sizer.add(&notebook, 1, SizerFlag::Expand | SizerFlag::All, 6);
        root.set_sizer(root_sizer, true);

        // Two fields, per the ticket. Text mirrors FR-statusbar's real content.
        let status_bar = frame.create_status_bar(2, 0, ID_ANY as Id, "");
        status_bar.set_status_widths(&[-3, -2]);
        status_bar.set_status_text("User PATH: 11 entries (4 issues)", 0);
        status_bar.set_status_text("Total length: 486 chars", 1);

        let frame_for_menu = frame; // Frame is Copy
        frame.on_menu(move |event| {
            if event.get_id() == ID_EXIT {
                frame_for_menu.close(true);
            }
        });

        frame.centre();
        frame.show(true);
    });
}

fn build_menu_bar(frame: &Frame) {
    let file_menu = Menu::builder()
        .append_item(ID_APPLY_CHANGES, "&Apply\tCtrl+S", "Write the changes to the registry")
        .append_item(ID_REFRESH, "&Refresh\tF5", "Re-read PATH from the registry")
        .append_separator()
        .append_item(ID_EXIT, "E&xit\tAlt+F4", "Close PathMaster")
        .build();

    let edit_menu = Menu::builder()
        .append_item(ID_UNDO, "&Undo\tCtrl+Z", "Undo the last change")
        .append_item(ID_REDO, "&Redo\tCtrl+Y", "Redo the last undone change")
        .append_separator()
        .append_item(ID_ADD, "&Add Entry…\tInsert", "Add a new PATH entry")
        .append_item(ID_DELETE, "&Delete Entry\tDelete", "Delete the selected entry")
        .build();

    let view_menu = Menu::builder()
        .append_check_item(ID_EXPAND_VARS, "Expand &%VAR%", "Show expanded values")
        .append_check_item(ID_SHOW_STATUSBAR, "Show &Status Bar", "Toggle the status bar")
        .build();

    let tools_menu = Menu::builder()
        .append_item(ID_DIAGNOSE, "Run &Diagnostics\tCtrl+D", "Re-run all diagnostics")
        .append_separator()
        .append_item(ID_SETTINGS, "&Settings…", "Open the settings dialog")
        .build();

    let help_menu = Menu::builder()
        .append_item(ID_ABOUT, "&About PathMaster", "Version and licence information")
        .build();

    let menu_bar = MenuBar::builder()
        .append(file_menu, "&File")
        .append(edit_menu, "&Edit")
        .append(view_menu, "&View")
        .append(tools_menu, "&Tools")
        .append(help_menu, "&Help")
        .build();

    // The deliberately disabled item (question 5) and a checked check-item, so the user can hear
    // whether "dimmed" and "checked" are spoken. Both must happen before set_menu_bar takes
    // ownership of the MenuBar.
    menu_bar.enable_item(ID_UNDO, false);
    menu_bar.check_item(ID_SHOW_STATUSBAR, true);

    frame.set_menu_bar(menu_bar);
}

/// One notebook page: a report-mode list above a row of four buttons.
///
/// `SingleSel` is deliberate, not stock-by-omission: Delete / Move Up / Move Down all act on one
/// entry, so single selection is the app's real shape. It changes what NVDA says about selection
/// state, so it is called out here and in the findings rather than left implicit.
fn build_path_page(notebook: &Notebook, rows: &[(&str, &str)]) -> Panel {
    let page = Panel::builder(notebook).build();

    let list = ListCtrl::builder(&page)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel)
        .build();
    list.insert_column(0, "Path", ListColumnFormat::Left, 560);
    list.insert_column(1, "Status", ListColumnFormat::Left, 260);
    for (index, (path, status)) in rows.iter().enumerate() {
        list.insert_item(index as i64, path, None);
        list.set_item_text_by_column(index as i64, 1, status);
    }

    let button_sizer = BoxSizer::builder(Orientation::Horizontal).build();
    for (id, label) in [
        (ID_ADD, "&Add…"),
        (ID_DELETE, "&Delete"),
        (ID_MOVE_UP, "Move &Up"),
        (ID_MOVE_DOWN, "Move D&own"),
    ] {
        let button = Button::builder(&page).with_id(id).with_label(label).build();
        button_sizer.add(&button, 0, SizerFlag::Left, 6);
    }

    let page_sizer = BoxSizer::builder(Orientation::Vertical).build();
    page_sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 6);
    page_sizer.add_sizer(&button_sizer, 0, SizerFlag::AlignRight | SizerFlag::All, 6);
    page.set_sizer(page_sizer, true);

    page
}
