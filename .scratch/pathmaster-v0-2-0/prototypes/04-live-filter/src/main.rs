//! THROWAWAY PROTOTYPE — wayfinder v0.2.0 ticket 04, "Live-filtered list under NVDA".
//!
//! One question: does the Search bar's core mechanism work under real NVDA? A text field sits
//! above a `SysListView32` of 50 fake PATH entries; typing live-filters the list (rows deleted
//! and reinserted while focus stays in the field) and a debounced count is spoken through the
//! v0.1.0 announcement mechanism (`NotifyWinEvent(EVENT_OBJECT_LIVEREGIONCHANGED)` on the
//! Banner's StaticText — copied verbatim from the app's announce.rs).
//!
//! What the user listens for (the ticket's four questions):
//!   1. Silence while rows are rebuilt under the unfocused list — or chatter / the deaf-list
//!      signature from v0.1.0?
//!   2. Does the debounced spoken count arrive reliably while focus is in the field, and does
//!      further typing interrupt it acceptably?
//!   3. Tab from the field into the filtered list: does focus land on a sensible row and is it
//!      read?
//!   4. ESC in the field: clear + return focus to the list — what does NVDA say?
//!
//! Runtime toggles (Options menu; accelerators work with focus anywhere, so the user can switch
//! mid-listen):
//!   Ctrl+1/2/3/4  debounce 250 / 500 / 1000 / 1400 ms (1400 is GOV.UK accessible-autocomplete's
//!                 measured default — long enough for typing echo to finish; see research below)
//!   Ctrl+5/6      rebuild plain / wrapped in Freeze..Thaw (state set before Thaw, per research)
//!   Ctrl+7        speak the result count on/off (off isolates question 1: pure rebuild silence)
//!   Ctrl+8        ESC returns focus to the list (PRD) / keeps it in the field (Windows/ARIA
//!                 convention) — the two candidate contracts, both listenable
//!
//! Down-arrow in the field also moves focus into the list (the combobox-model gesture research
//! recommends alongside Tab). The status bar mirrors every mode change and the current match
//! count, so nothing is audio-only. Wording of the spoken count is a PLACEHOLDER — ticket 06
//! owns the real wording; research anchors: "N results" + worded "No results found" (WCAG 4.1.3,
//! GOV.UK), never a bare zero or silence.

#![windows_subsystem = "windows"]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use wxdragon::id::ID_EXIT;
use wxdragon::prelude::*;
use wxdragon::timer::Timer;
use wxdragon::widgets::statusbar::StatusBar;

use windows_sys::Win32::UI::Accessibility::NotifyWinEvent;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CHILDID_SELF, EVENT_OBJECT_LIVEREGIONCHANGED, OBJID_CLIENT,
};

const ID_DEBOUNCE_250: Id = 7101;
const ID_DEBOUNCE_500: Id = 7102;
const ID_DEBOUNCE_1000: Id = 7103;
const ID_DEBOUNCE_1400: Id = 7104;
const ID_REBUILD_PLAIN: Id = 7111;
const ID_REBUILD_FREEZE: Id = 7112;
const ID_SPEAK_COUNT: Id = 7121;
const ID_ESC_TO_LIST: Id = 7122;

const WXK_ESCAPE: i32 = 27;
const WXK_DOWN: i32 = 317;

/// 50 fake entries. Substring clusters are deliberate so filtering is interesting:
/// "git" (4 rows), "python" (5), "node" (2), "scoop" (8), "Program Files" (many),
/// "ruby" (2), and "zz" matches nothing.
const ENTRIES: &[&str] = &[
    r"C:\Users\Ruslan\AppData\Local\Microsoft\WindowsApps",
    r"C:\Program Files\Git\cmd",
    r"C:\Program Files\Git\mingw64\bin",
    r"C:\scoop\apps\git\current\usr\bin",
    r"C:\scoop\shims",
    r"%USERPROFILE%\.cargo\bin",
    r"C:\Program Files\nodejs",
    r"C:\Users\Ruslan\AppData\Roaming\npm",
    r"C:\scoop\apps\nodejs-lts\current\bin",
    r"C:\Python312\Scripts",
    r"C:\Python312",
    r"C:\Users\Ruslan\AppData\Local\Programs\Python\Launcher",
    r"C:\scoop\apps\python\current",
    r"C:\scoop\apps\python\current\Scripts",
    r"C:\Program Files\PowerShell\7",
    r"C:\Program Files\dotnet",
    r"C:\Program Files (x86)\Windows Kits\10\Windows Performance Toolkit",
    r"C:\WINDOWS\system32",
    r"C:\WINDOWS",
    r"C:\WINDOWS\System32\Wbem",
    r"C:\WINDOWS\System32\WindowsPowerShell\v1.0",
    r"C:\WINDOWS\System32\OpenSSH",
    r"C:\Program Files\Docker\Docker\resources\bin",
    r"C:\Program Files\Microsoft VS Code\bin",
    r"C:\Users\Ruslan\AppData\Local\Programs\Microsoft VS Code Insiders\bin",
    r"C:\Program Files\7-Zip",
    r"C:\Program Files\CMake\bin",
    r"C:\scoop\apps\ruby\current\bin",
    r"C:\scoop\apps\ruby\current\gems\bin",
    r"C:\Program Files\Go\bin",
    r"%USERPROFILE%\go\bin",
    r"C:\Program Files\LLVM\bin",
    r"C:\Program Files\Java\jdk-21\bin",
    r"C:\Gradle\gradle-8.5\bin",
    r"C:\Program Files\Apache\maven\bin",
    r"C:\tools\vim\vim91",
    r"C:\Program Files\Neovim\bin",
    r"C:\ProgramData\chocolatey\bin",
    r"C:\Program Files\GitHub CLI",
    r"C:\Users\Ruslan\.dotnet\tools",
    r"C:\Program Files\Perforce",
    r"C:\Strawberry\perl\bin",
    r"C:\Strawberry\c\bin",
    r"C:\Program Files\PostgreSQL\16\bin",
    r"C:\Program Files\MySQL\MySQL Shell 8.0\bin",
    r"C:\Program Files\WireGuard",
    r"C:\Program Files\NVIDIA Corporation\NVSMI",
    r"C:\Users\Ruslan\AppData\Local\Pandoc",
    r"C:\texlive\2025\bin\windows",
    r"C:\Program Files\ffmpeg\bin",
];

/// The v0.1.0 voice, verbatim: label the Banner, then fire LIVEREGIONCHANGED on it.
fn announce(banner: &StaticText, text: &str) {
    banner.set_label(text);
    let hwnd = banner.get_handle();
    if hwnd.is_null() {
        return;
    }
    // SAFETY: fire-and-forget notification on a live window handle; the call takes no
    // pointers that outlive it.
    unsafe {
        NotifyWinEvent(
            EVENT_OBJECT_LIVEREGIONCHANGED,
            hwnd,
            OBJID_CLIENT,
            CHILDID_SELF as i32,
        );
    }
}

/// Which original entry currently carries the list's focus rectangle, if any.
fn focused_original(list: &ListCtrl, visible: &[usize]) -> Option<usize> {
    (0..visible.len() as i64)
        .find(|&row| list.get_item_state(row, ListItemState::Focused))
        .map(|row| visible[row as usize])
}

/// Rebuild the list to show exactly `visible`, then place focus+selection per the ticket-03
/// rule (concerned entry if it survived, else same visual position clamped, else nothing).
fn rebuild(
    list: &ListCtrl,
    visible: &[usize],
    keep_original: Option<usize>,
    prev_row: i64,
    use_freeze: bool,
) {
    // Research order-of-operations: Freeze → rebuild → set focused+selected → Thaw, so the
    // list never sits visible in a "no item focused" state.
    if use_freeze {
        list.freeze();
    }
    list.delete_all_items();
    for (row, &original) in visible.iter().enumerate() {
        list.insert_item(row as i64, &format!("{}", original + 1), None);
        list.set_item_text_by_column(row as i64, 1, ENTRIES[original]);
    }
    if !visible.is_empty() {
        let target = keep_original
            .and_then(|orig| visible.iter().position(|&v| v == orig))
            .map(|row| row as i64)
            .unwrap_or_else(|| prev_row.clamp(0, visible.len() as i64 - 1));
        list.set_item_state(
            target,
            ListItemState::Focused | ListItemState::Selected,
            ListItemState::Focused | ListItemState::Selected,
        );
        list.ensure_visible(target);
    }
    if use_freeze {
        list.thaw();
    }
}

fn matches(query: &str) -> Vec<usize> {
    let needle = query.to_lowercase();
    (0..ENTRIES.len())
        .filter(|&i| needle.is_empty() || ENTRIES[i].to_lowercase().contains(&needle))
        .collect()
}

fn main() {
    let _ = wxdragon::main(|_| {
        let frame = Frame::builder()
            .with_title("PathMaster — live filter prototype (ticket 04)")
            .with_size(Size::new(920, 620))
            .build();
        frame.set_min_size(Size::new(800, 600));

        build_menu_bar(&frame);

        let root = Panel::builder(&frame).build();

        // Banner: always visible at fixed height, exactly like the app — setting its label
        // never reflows the layout and never moves focus.
        let banner = StaticText::builder(&root).with_label("").build();

        let field_label = StaticText::builder(&root).with_label("&Filter:").build();
        let field = TextCtrl::builder(&root).build();

        let list = ListCtrl::builder(&root)
            .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel)
            .build();
        list.insert_column(0, "#", ListColumnFormat::Right, 60);
        list.insert_column(1, "Path", ListColumnFormat::Left, 720);

        let field_sizer = BoxSizer::builder(Orientation::Horizontal).build();
        field_sizer.add(&field_label, 0, SizerFlag::AlignCenterVertical | SizerFlag::All, 4);
        field_sizer.add(&field, 1, SizerFlag::Expand | SizerFlag::All, 4);

        let root_sizer = BoxSizer::builder(Orientation::Vertical).build();
        root_sizer.add(&banner, 0, SizerFlag::Expand | SizerFlag::All, 6);
        root_sizer.add_sizer(&field_sizer, 0, SizerFlag::Expand | SizerFlag::All, 2);
        root_sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 6);
        root.set_sizer(root_sizer, true);

        let status_bar: StatusBar = frame.create_status_bar(2, 0, ID_ANY as Id, "");
        status_bar.set_status_widths(&[-3, -2]);

        // --- Mode state, all runtime-switchable. ---
        let debounce_ms = Rc::new(Cell::new(1400i32));
        let use_freeze = Rc::new(Cell::new(false));
        let speak_count = Rc::new(Cell::new(true));
        let esc_to_list = Rc::new(Cell::new(true));
        let visible: Rc<RefCell<Vec<usize>>> = Rc::new(RefCell::new((0..ENTRIES.len()).collect()));
        let suppress_text_event = Rc::new(Cell::new(false));

        let show_mode = {
            let debounce_ms = debounce_ms.clone();
            let use_freeze = use_freeze.clone();
            let speak_count = speak_count.clone();
            let esc_to_list = esc_to_list.clone();
            move || {
                status_bar.set_status_text(
                    &format!(
                        "Debounce {} ms · {} rebuild · count {} · ESC → {}",
                        debounce_ms.get(),
                        if use_freeze.get() { "Freeze/Thaw" } else { "plain" },
                        if speak_count.get() { "spoken" } else { "silent" },
                        if esc_to_list.get() { "list" } else { "field" },
                    ),
                    0,
                );
            }
        };
        let show_count = move |shown: usize| {
            status_bar.set_status_text(&format!("{} of {} shown", shown, ENTRIES.len()), 1);
        };

        rebuild(&list, &visible.borrow(), None, 0, false);
        show_mode();
        show_count(ENTRIES.len());

        // --- The debounce: one-shot timer, restarted on every keystroke. ---
        let timer = Timer::new(&frame);
        {
            let visible = visible.clone();
            let use_freeze = use_freeze.clone();
            let speak_count = speak_count.clone();
            timer.on_tick(move |_| {
                let query = field.get_value();
                let new_visible = matches(query.trim());
                let keep = focused_original(&list, &visible.borrow());
                let prev_row = (0..visible.borrow().len() as i64)
                    .find(|&row| list.get_item_state(row, ListItemState::Focused))
                    .unwrap_or(0);
                rebuild(&list, &new_visible, keep, prev_row, use_freeze.get());
                let shown = new_visible.len();
                *visible.borrow_mut() = new_visible;
                show_count(shown);
                if speak_count.get() {
                    // PLACEHOLDER wording — ticket 06 decides the real sentence.
                    let text = if query.trim().is_empty() {
                        format!("All {} entries", ENTRIES.len())
                    } else if shown == 0 {
                        "No matching entries".to_string()
                    } else {
                        format!("{} of {} entries", shown, ENTRIES.len())
                    };
                    announce(&banner, &text);
                }
            });
        }

        // Timer isn't Clone and its Drop would destroy it; the Rc keeps the one timer alive
        // inside every closure that needs it.
        let timer = Rc::new(timer);
        {
            let timer = timer.clone();
            let debounce_ms = debounce_ms.clone();
            let suppress = suppress_text_event.clone();
            field.on_text_changed(move |_| {
                if suppress.get() {
                    return;
                }
                timer.start(debounce_ms.get(), true);
            });
        }

        // ESC in the field: stop the pending rebuild, clear, restore the full list, announce
        // the restored state, and send focus wherever the current mode says — then listen to
        // what NVDA does with the landing. Down-arrow: the combobox-model way into the list.
        {
            let timer = timer.clone();
            let visible = visible.clone();
            let use_freeze = use_freeze.clone();
            let speak_count = speak_count.clone();
            let esc_to_list = esc_to_list.clone();
            let suppress = suppress_text_event.clone();
            field.on_key_down(move |event| {
                let key = match &event {
                    wxdragon::event::WindowEventData::Keyboard(k) => k.get_key_code(),
                    _ => None,
                };
                match key {
                    Some(WXK_DOWN) => {
                        list.set_focus();
                    }
                    Some(WXK_ESCAPE) => {
                        timer.stop();
                        let keep = focused_original(&list, &visible.borrow());
                        suppress.set(true);
                        field.set_value("");
                        suppress.set(false);
                        let full: Vec<usize> = (0..ENTRIES.len()).collect();
                        rebuild(&list, &full, keep, 0, use_freeze.get());
                        *visible.borrow_mut() = full;
                        show_count(ENTRIES.len());
                        if speak_count.get() {
                            // PLACEHOLDER wording, research anchor: announce the restored state.
                            announce(&banner, &format!("Filter cleared, {} entries", ENTRIES.len()));
                        }
                        if esc_to_list.get() {
                            list.set_focus();
                        }
                    }
                    _ => event.skip(true),
                }
            });
        }

        // --- Mode switches. Radio items check themselves; we just mirror the state. ---
        let frame_for_menu = frame; // Frame is Copy
        frame.on_menu(move |event| {
            match event.get_id() {
                ID_EXIT => frame_for_menu.close(true),
                ID_DEBOUNCE_250 => debounce_ms.set(250),
                ID_DEBOUNCE_500 => debounce_ms.set(500),
                ID_DEBOUNCE_1000 => debounce_ms.set(1000),
                ID_DEBOUNCE_1400 => debounce_ms.set(1400),
                ID_REBUILD_PLAIN => use_freeze.set(false),
                ID_REBUILD_FREEZE => use_freeze.set(true),
                ID_SPEAK_COUNT => speak_count.set(!speak_count.get()),
                ID_ESC_TO_LIST => esc_to_list.set(!esc_to_list.get()),
                _ => return,
            }
            show_mode();
        });

        frame.centre();
        frame.show(true);
        field.set_focus();
    });
}

fn build_menu_bar(frame: &Frame) {
    let file_menu = Menu::builder()
        .append_item(ID_EXIT, "E&xit\tAlt+F4", "Close the prototype")
        .build();

    let options_menu = Menu::builder()
        .append_radio_item(ID_DEBOUNCE_250, "Debounce &250 ms\tCtrl+1", "Speak count 250 ms after typing stops")
        .append_radio_item(ID_DEBOUNCE_500, "Debounce &500 ms\tCtrl+2", "Speak count 500 ms after typing stops")
        .append_radio_item(ID_DEBOUNCE_1000, "Debounce 1&000 ms\tCtrl+3", "Speak count 1000 ms after typing stops")
        .append_radio_item(ID_DEBOUNCE_1400, "Debounce &1400 ms (GOV.UK default)\tCtrl+4", "Speak count 1400 ms after typing stops")
        .append_separator()
        .append_radio_item(ID_REBUILD_PLAIN, "&Plain rebuild\tCtrl+5", "DeleteAllItems + insert, no Freeze/Thaw")
        .append_radio_item(ID_REBUILD_FREEZE, "&Freeze/Thaw rebuild\tCtrl+6", "Wrap the rebuild in Freeze..Thaw")
        .append_separator()
        .append_check_item(ID_SPEAK_COUNT, "Speak result &count\tCtrl+7", "Announce the debounced count (off = test rebuild silence alone)")
        .append_check_item(ID_ESC_TO_LIST, "&ESC moves focus to list\tCtrl+8", "Off = ESC clears but focus stays in the field (Windows/ARIA convention)")
        .build();

    let menu_bar = MenuBar::builder()
        .append(file_menu, "&File")
        .append(options_menu, "&Options")
        .build();

    // Radio/check defaults: 1400 ms, plain rebuild, count spoken, ESC → list (the PRD's shape).
    for id in [ID_DEBOUNCE_1400, ID_SPEAK_COUNT, ID_ESC_TO_LIST] {
        if let Some(item) = menu_bar.find_item(id) {
            item.check(true);
        }
    }

    frame.set_menu_bar(menu_bar);
}
