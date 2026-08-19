//! THROWAWAY PROTOTYPE — wayfinder ticket 08, "Live announcement mechanism".
//!
//! One question: how does a transient message — "PATH refreshed", "Copied to clipboard", the
//! PATH-length banner — reach NVDA when no focus change carries it? This rig wires every candidate
//! rung to its own accelerator so `tools/nvda-drive.ps1` can fire them while focus sits in the
//! list, exactly where a real user's focus would be:
//!
//!   Ctrl+0  T0  control case: banner shown + label set, NO accessibility call   → expect silence
//!   Ctrl+1  T1  raw NotifyWinEvent(EVENT_OBJECT_NAMECHANGE)  on banner text
//!   Ctrl+2  T2  raw NotifyWinEvent(EVENT_OBJECT_LIVEREGIONCHANGED) on banner text
//!   Ctrl+3  T3  raw NotifyWinEvent(EVENT_SYSTEM_ALERT)       on banner text
//!   Ctrl+4  T4  UIA UiaRaiseNotificationEvent via UiaHostProviderFromHwnd
//!   Ctrl+5  T5  status bar: set_status_text + raw NAMECHANGE on the status bar HWND
//!   Ctrl+6  T6  wx route: set_accessibility_role(Alert) once + Accessible::notify_event(ALERT)
//!               — on its OWN StaticText, so flipping that widget onto the wx-mediated
//!               WM_GETOBJECT path can never contaminate the raw-rung targets
//!   Ctrl+7  T7  design-away: banner is a focusable read-only TextCtrl; move focus to it
//!   Ctrl+8  T8  design-away: modal MessageDialog (close with Enter)
//!   Ctrl+9  T9  raw NotifyWinEvent(EVENT_OBJECT_SHOW) as the banner is shown
//!   Ctrl+H      hide the banner again (reset between cases)                     → expect silence
//!
//! Every message carries a serial number (`#N`) so repeats are never deduplicated away and every
//! utterance in NVDA's log attributes to exactly one trigger.
//!
//! The app's shape is copied from the ticket-02 baseline (menubar, notebook, report list, buttons,
//! two-field status bar) so results transfer. The baseline itself stays untouched — this copy
//! exists precisely so accessibility calls never land there.

#![windows_subsystem = "windows"]

use std::cell::Cell;
use std::rc::Rc;

use wxdragon::accessible::{AccObjectType, AccRole, Accessible};
use wxdragon::id::{ID_ABOUT, ID_EXIT};
use wxdragon::prelude::*;
use wxdragon::widgets::statusbar::StatusBar;

use windows::core::BSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{
    NotificationKind_ActionCompleted, NotificationProcessing_ImportantAll, NotifyWinEvent,
    UiaHostProviderFromHwnd, UiaRaiseNotificationEvent,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CHILDID_SELF, EVENT_OBJECT_LIVEREGIONCHANGED, EVENT_OBJECT_NAMECHANGE, EVENT_OBJECT_SHOW,
    EVENT_SYSTEM_ALERT, OBJID_CLIENT,
};

const ID_APPLY_CHANGES: Id = 6001;
const ID_REFRESH: Id = 6002;
const ID_ADD: Id = 6005;
const ID_DELETE: Id = 6006;
const ID_MOVE_UP: Id = 6007;
const ID_MOVE_DOWN: Id = 6008;

// Announcement triggers. Ctrl+digit so the harness can fire them without moving focus.
const ID_T0_CONTROL: Id = 7000;
const ID_T1_NAMECHANGE: Id = 7001;
const ID_T2_LIVEREGION: Id = 7002;
const ID_T3_ALERT: Id = 7003;
const ID_T4_UIA_NOTIFY: Id = 7004;
const ID_T5_STATUSBAR: Id = 7005;
const ID_T6_WX_ALERT: Id = 7006;
const ID_T7_FOCUS: Id = 7007;
const ID_T8_DIALOG: Id = 7008;
const ID_T9_SHOW: Id = 7009;
const ID_HIDE_BANNER: Id = 7010;
const ID_T2R_REPEAT: Id = 7011;

/// Same mixed rows as the baseline, so the surrounding soundscape is identical.
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

fn main() {
    let _ = wxdragon::main(|_| {
        let frame = Frame::builder()
            .with_title("PathMaster — announcement prototype")
            .with_size(Size::new(920, 620))
            .build();
        frame.set_min_size(Size::new(800, 600));

        build_menu_bar(&frame);

        let root = Panel::builder(&frame).build();

        // --- The banner: the PRD's InlineAlert, hand-built (no wxInfoBar in the binding). ---
        // Hidden until a trigger shows it. No background colour — ticket 04's standing rule.
        // Three message targets so no rung's side effects touch another rung's widget.
        let banner = Panel::builder(&root).build();
        let raw_text = StaticText::builder(&banner).with_label("").build(); // T0–T3, T9
        let wx_text = StaticText::builder(&banner).with_label("").build(); // T6 only
        let focus_text = TextCtrl::builder(&banner)
            .with_style(TextCtrlStyle::ReadOnly)
            .build(); // T7 only
        let banner_sizer = BoxSizer::builder(Orientation::Horizontal).build();
        banner_sizer.add(&raw_text, 1, SizerFlag::Expand | SizerFlag::All, 4);
        banner_sizer.add(&wx_text, 1, SizerFlag::Expand | SizerFlag::All, 4);
        banner_sizer.add(&focus_text, 1, SizerFlag::Expand | SizerFlag::All, 4);
        banner.set_sizer(banner_sizer, true);
        banner.show(false);

        let notebook = Notebook::builder(&root).build();
        let user_page = build_path_page(&notebook, USER_ROWS);
        notebook.add_page(&user_page, "User PATH", true, None);

        let root_sizer = BoxSizer::builder(Orientation::Vertical).build();
        root_sizer.add(&banner, 0, SizerFlag::Expand | SizerFlag::All, 6);
        root_sizer.add(&notebook, 1, SizerFlag::Expand | SizerFlag::All, 6);
        root.set_sizer(root_sizer, true);

        let status_bar: StatusBar = frame.create_status_bar(2, 0, ID_ANY as Id, "");
        status_bar.set_status_widths(&[-3, -2]);
        status_bar.set_status_text("User PATH: 11 entries (4 issues)", 0);
        status_bar.set_status_text("Total length: 486 chars", 1);

        // Serial number stamped into every message; the wx role is set exactly once.
        let serial = Rc::new(Cell::new(0u32));
        let wx_role_set = Rc::new(Cell::new(false));

        let frame_for_menu = frame; // Frame is Copy
        frame.on_menu(move |event| {
            let id = event.get_id();
            if id == ID_EXIT {
                frame_for_menu.close(true);
                return;
            }
            let n = serial.get() + 1;
            serial.set(n);

            let show_banner = |text: &str| {
                raw_text.set_label(text);
                if !banner.is_shown() {
                    banner.show(true);
                    root.layout();
                }
            };

            match id {
                ID_T0_CONTROL => {
                    // Control case: everything visual happens, no accessibility call at all.
                    show_banner(&format!("T0 #{n}: PATH refreshed — no event fired"));
                }
                ID_T1_NAMECHANGE => {
                    show_banner(&format!("T1 #{n}: PATH refreshed — namechange"));
                    unsafe {
                        NotifyWinEvent(
                            EVENT_OBJECT_NAMECHANGE,
                            HWND(raw_text.get_handle()),
                            OBJID_CLIENT.0,
                            CHILDID_SELF as i32,
                        );
                    }
                }
                ID_T2_LIVEREGION => {
                    show_banner(&format!("T2 #{n}: Copied to clipboard — liveregion"));
                    unsafe {
                        NotifyWinEvent(
                            EVENT_OBJECT_LIVEREGIONCHANGED,
                            HWND(raw_text.get_handle()),
                            OBJID_CLIENT.0,
                            CHILDID_SELF as i32,
                        );
                    }
                }
                ID_T3_ALERT => {
                    show_banner(&format!(
                        "T3 #{n}: PATH is 2168 characters, near the 2047 limit — alert"
                    ));
                    unsafe {
                        NotifyWinEvent(
                            EVENT_SYSTEM_ALERT,
                            HWND(raw_text.get_handle()),
                            OBJID_CLIENT.0,
                            CHILDID_SELF as i32,
                        );
                    }
                }
                ID_T4_UIA_NOTIFY => {
                    // The modern, documented "announce without focus change" API. The provider is
                    // the host provider Windows itself builds for the HWND — no custom UIA code.
                    let msg = format!(
                        "T4 #{n}: Settings file was corrupted and has been reset — uia notification"
                    );
                    show_banner(&msg);
                    let outcome = unsafe {
                        UiaHostProviderFromHwnd(HWND(raw_text.get_handle())).and_then(|provider| {
                            UiaRaiseNotificationEvent(
                                &provider,
                                NotificationKind_ActionCompleted,
                                NotificationProcessing_ImportantAll,
                                &BSTR::from(msg.as_str()),
                                &BSTR::from("PathMasterTransient"),
                            )
                        })
                    };
                    // Surface failure where -Probe can read it, without firing any event.
                    if let Err(e) = outcome {
                        raw_text.set_label(&format!("T4 #{n}: UIA call FAILED: {e}"));
                    }
                }
                ID_T5_STATUSBAR => {
                    status_bar.set_status_text(
                        &format!("T5 #{n}: PATH refreshed — statusbar namechange"),
                        0,
                    );
                    unsafe {
                        NotifyWinEvent(
                            EVENT_OBJECT_NAMECHANGE,
                            HWND(status_bar.get_handle()),
                            OBJID_CLIENT.0,
                            CHILDID_SELF as i32,
                        );
                    }
                }
                ID_T6_WX_ALERT => {
                    // The in-toolkit route, per research/01's spike order. First call moves
                    // wx_text (and only wx_text) onto the wx-mediated WM_GETOBJECT path.
                    wx_text.set_label(&format!("T6 #{n}: PATH refreshed — wx alert"));
                    if !banner.is_shown() {
                        banner.show(true);
                        root.layout();
                    }
                    if !wx_role_set.get() {
                        wx_text.set_accessibility_role(AccRole::Alert);
                        wx_role_set.set(true);
                    }
                    Accessible::notify_event(EVENT_SYSTEM_ALERT, &wx_text, AccObjectType::Alert, 0);
                }
                ID_T7_FOCUS => {
                    focus_text.set_value(&format!("T7 #{n}: PATH refreshed — focus moved here"));
                    if !banner.is_shown() {
                        banner.show(true);
                        root.layout();
                    }
                    focus_text.set_focus();
                }
                ID_T8_DIALOG => {
                    let dialog = MessageDialog::builder(
                        &frame_for_menu,
                        &format!("T8 #{n}: Settings file was corrupted and has been reset to defaults."),
                        "PathMaster",
                    )
                    .build();
                    dialog.show_modal();
                }
                ID_T9_SHOW => {
                    // Show-event variant: hide, relabel, show, then announce the appearance.
                    banner.show(false);
                    raw_text.set_label(&format!(
                        "T9 #{n}: PATH is 2168 characters, near the 2047 limit — show"
                    ));
                    banner.show(true);
                    root.layout();
                    unsafe {
                        NotifyWinEvent(
                            EVENT_OBJECT_SHOW,
                            HWND(raw_text.get_handle()),
                            OBJID_CLIENT.0,
                            CHILDID_SELF as i32,
                        );
                    }
                }
                ID_T2R_REPEAT => {
                    // Same text, event fired again: does NVDA speak identical repeats?
                    // ("Copied to clipboard" twice in a row is a real product case.)
                    unsafe {
                        NotifyWinEvent(
                            EVENT_OBJECT_LIVEREGIONCHANGED,
                            HWND(raw_text.get_handle()),
                            OBJID_CLIENT.0,
                            CHILDID_SELF as i32,
                        );
                    }
                }
                ID_HIDE_BANNER => {
                    banner.show(false);
                    root.layout();
                }
                _ => {}
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

    // The rig. `\t` accelerators are load-bearing: the harness fires these by keystroke.
    let announce_menu = Menu::builder()
        .append_item(ID_T0_CONTROL, "T0 control (silent?)\tCtrl+0", "Banner only, no event")
        .append_item(ID_T1_NAMECHANGE, "T1 raw namechange\tCtrl+1", "NotifyWinEvent NAMECHANGE")
        .append_item(ID_T2_LIVEREGION, "T2 raw liveregion\tCtrl+2", "NotifyWinEvent LIVEREGIONCHANGED")
        .append_item(ID_T3_ALERT, "T3 raw alert\tCtrl+3", "NotifyWinEvent EVENT_SYSTEM_ALERT")
        .append_item(ID_T4_UIA_NOTIFY, "T4 UIA notification\tCtrl+4", "UiaRaiseNotificationEvent")
        .append_item(ID_T5_STATUSBAR, "T5 statusbar namechange\tCtrl+5", "Status text + NAMECHANGE")
        .append_item(ID_T6_WX_ALERT, "T6 wx alert route\tCtrl+6", "Accessible::notify_event")
        .append_item(ID_T7_FOCUS, "T7 focus the banner\tCtrl+7", "Read-only TextCtrl takes focus")
        .append_item(ID_T8_DIALOG, "T8 modal dialog\tCtrl+8", "MessageDialog")
        .append_item(ID_T9_SHOW, "T9 raw show event\tCtrl+9", "NotifyWinEvent EVENT_OBJECT_SHOW")
        .append_separator()
        .append_item(ID_T2R_REPEAT, "&Repeat liveregion event\tCtrl+R", "Same text, event again")
        .append_item(ID_HIDE_BANNER, "&Hide banner\tCtrl+H", "Hide the banner again")
        .build();

    let help_menu = Menu::builder()
        .append_item(ID_ABOUT, "&About PathMaster", "Version and licence information")
        .build();

    let menu_bar = MenuBar::builder()
        .append(file_menu, "&File")
        .append(announce_menu, "A&nnounce")
        .append(help_menu, "&Help")
        .build();

    frame.set_menu_bar(menu_bar);
}

/// One notebook page, same shape as the baseline: report list above four buttons.
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
