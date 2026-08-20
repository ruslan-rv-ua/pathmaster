//! The main window shell (spec §12): one vertical sizer — Banner above the notebook —
//! with the native status bar attached to the frame outside the sizer.
//!
//! The tab order is the whole map: tabs → list (→ buttons, with the tickets that
//! add them). Nothing here moves focus — `announce()` speaks without touching it,
//! and the status bar is command-only (`NVDA+End`), absent from the Tab order.

use std::cell::RefCell;
use std::rc::Rc;

use pathmaster_core::msgids::{self, fill};
use pathmaster_core::session::{Scope, Session};
use pathmaster_platform::datadir::ReadOnlyReason;
use wxdragon::prelude::*;

use crate::announce::Announcer;
use crate::catalog::{translate, translate_plural};

/// Status column width in DIP — the app's single deliberate pixel constant (spec §12 D2).
/// Status text is of predictable length (comma-joined one-word Issue types) while paths
/// are unbounded, so Status is fixed and Path takes all remaining width.
const STATUS_COLUMN_DIP: i32 = 220;

/// The notebook's page order (spec §12): the two Scopes, then Backups —
/// which is not a Scope, so activating it announces nothing.
const TAB_INDEX_USER: i32 = 0;
const TAB_INDEX_SYSTEM: i32 = 1;

/// The app's single explicit FromDIP conversion (spec §12 D4). wxdragon applies FromDIP
/// implicitly to sizes crossing the FFI boundary, but ListCtrl column widths cross it raw,
/// so the one hardcoded pixel value is scaled here against the live DPI.
fn from_dip(widget: &ListCtrl, dip: i32) -> i32 {
    let dc = ClientDC::new(widget);
    let (ppi_x, _) = dc.get_ppi();
    if ppi_x > 0 {
        dip * ppi_x / 96
    } else {
        dip
    }
}

/// Builds and shows the main window over the two loaded Sessions, and hands it
/// back so a startup dialog has a parent to sit on and a window to hand focus
/// back to. A Read-only Data run passes its reason; announcing it is the last
/// step of startup (spec §11: … → UI → writability → announce).
pub fn build_main_window(
    user: Rc<RefCell<Session>>,
    system: Rc<RefCell<Session>>,
    readonly: Option<ReadOnlyReason>,
) -> Frame {
    let frame = Frame::builder()
        .with_title("PathMaster")
        // Crosses the FFI boundary through the implicit FromDIP → 900×650 DIP (spec §12 D2).
        .with_size(Size::new(900, 650))
        .build();
    frame.set_min_size(Size::new(800, 600));

    let root = Panel::builder(&frame).build();

    // The Banner: always visible, fixed height, its StaticText empty at rest — the layout
    // never reflows under the user when announce() sets a message (spec §12 D1, §10).
    // get_char_height() and set_min_size are both physical pixels: SetMinSize is one of the
    // FFI calls wxdragon does NOT route through its implicit FromDIP, so no double scaling.
    let banner = StaticText::builder(&root).with_label("").build();
    banner.set_min_size(Size::new(-1, banner.get_char_height()));
    let announcer = Announcer::new(banner);

    let notebook = Notebook::builder(&root).build();
    let user_page = build_scope_page(&notebook, &user.borrow());
    let system_page = build_scope_page(&notebook, &system.borrow());
    // The Backups tab is not a Scope; its Snapshot list arrives with the backups ticket.
    let backups_page = Panel::builder(&notebook).build();
    notebook.add_page(&user_page, &translate(msgids::TAB_USER), true, None);
    notebook.add_page(&system_page, &translate(msgids::TAB_SYSTEM), false, None);
    notebook.add_page(&backups_page, &translate(msgids::TAB_BACKUPS), false, None);

    // Announcement 1 (spec §10.1): activating a Scope tab speaks its entry
    // count. The count is read at activation time, not captured — Refresh and
    // editing change it under the same handler.
    let user_for_tabs = Rc::clone(&user);
    let system_for_tabs = Rc::clone(&system);
    notebook.on_page_changed(move |event| {
        let session = match event.get_selection() {
            Some(TAB_INDEX_USER) => Some(user_for_tabs.borrow()),
            Some(TAB_INDEX_SYSTEM) => Some(system_for_tabs.borrow()),
            // The Backups tab, or no selection at all: silence.
            _ => None,
        };
        if let Some(session) = session {
            announcer.announce(&entry_count_text(session.scope(), session.entries().len()));
        }
        event.base.skip(true);
    });

    let root_sizer = BoxSizer::builder(Orientation::Vertical).build();
    root_sizer.add(&banner, 0, SizerFlag::Expand | SizerFlag::All, 4);
    root_sizer.add(&notebook, 1, SizerFlag::Expand | SizerFlag::All, 4);
    root.set_sizer(root_sizer, true);

    // Command-only (NVDA+End), absent from the Tab order: field 0 general status,
    // field 1 the passive merged-length field (spec §12 D10) — text arrives with
    // diagnostics. No field is ever styled: text carries everything.
    let status_bar = frame.create_status_bar(2, 0, ID_ANY as Id, "");
    status_bar.set_status_widths(&[-3, -2]);
    status_bar.set_status_text(
        &general_status(&user.borrow(), &system.borrow(), readonly.as_ref()),
        0,
    );

    frame.centre();
    frame.show(true);

    // Announcement 7 (spec §10.1), once at startup: a Read-only Data run names
    // its reason. Fired after show so the Banner's window exists to speak from.
    if let Some(reason) = &readonly {
        announcer.announce(&readonly_text(reason));
    }

    frame
}

/// The one startup dialog `settings.json` can earn: it could not be read, so
/// this run is on defaults (spec §13).
///
/// Everything it says is in the title, because NVDA speaks a `MessageDialog`'s
/// title and buttons and never its body (spec §10, D6) — the body repeats the
/// title for the eyes rather than carrying anything of its own. The stock [OK]
/// is left stock: it is the one button in the application whose text carries no
/// meaning we would have to own (spec §11).
///
/// Shown after the main window rather than before it, so that dismissing it
/// leaves focus in the window the user came for.
pub fn show_settings_unreadable(parent: &Frame) {
    let title = translate(msgids::DIALOG_SETTINGS_UNREADABLE);
    MessageDialog::builder(parent, &title, &title)
        .with_style(MessageDialogStyle::OK | MessageDialogStyle::IconWarning)
        .build()
        .show_modal();
}

/// Announcement 1's text: the Scope's entry count, with the zero case as its
/// own msgid — "no entries" is better speech than "0", and Ukrainian's three
/// plural forms have no zero form to give it (spec §10.1 item 1).
fn entry_count_text(scope: Scope, count: usize) -> String {
    let (none, singular, plural) = match scope {
        Scope::User => (
            msgids::ENTRIES_USER_NONE,
            msgids::ENTRIES_USER,
            msgids::ENTRIES_USER_PLURAL,
        ),
        Scope::System => (
            msgids::ENTRIES_SYSTEM_NONE,
            msgids::ENTRIES_SYSTEM,
            msgids::ENTRIES_SYSTEM_PLURAL,
        ),
    };
    if count == 0 {
        translate(none)
    } else {
        fill(
            &translate_plural(singular, plural, count as u32),
            &[("n", &count.to_string())],
        )
    }
}

/// Announcement 7's text, which is also StatusBar field 0 in Read-only Data:
/// the mode and its reason, both halves Catalogue text (spec §10.1 item 7).
fn readonly_text(reason: &ReadOnlyReason) -> String {
    fill(
        &translate(msgids::READONLY),
        &[("reason", &translate(reason.catalogue_msgid()))],
    )
}

/// StatusBar field 0, the general status (spec §12): the two entry counts —
/// issue counts join with the diagnostics ticket — or, in Read-only Data, the
/// mode and its reason in their place.
fn general_status(user: &Session, system: &Session, readonly: Option<&ReadOnlyReason>) -> String {
    match readonly {
        Some(reason) => readonly_text(reason),
        None => format!(
            "{} | {}",
            entry_count_text(user.scope(), user.entries().len()),
            entry_count_text(system.scope(), system.entries().len()),
        ),
    }
}

/// One Scope tab: a report-mode list with exactly two columns, Path and Status —
/// no index column, no icons (spec §7, §10). Each Entry renders its raw text in
/// the Path column; Status stays empty until diagnostics land. Zero Entries is
/// an empty list — no placeholder rows (spec §10.1 item 1).
fn build_scope_page(notebook: &Notebook, session: &Session) -> Panel {
    let page = Panel::builder(notebook).build();

    // SingleSel is the app's real shape: Delete / Move Up / Move Down act on one entry.
    let list = ListCtrl::builder(&page)
        .with_style(ListCtrlStyle::Report | ListCtrlStyle::SingleSel)
        .build();
    let status_width = from_dip(&list, STATUS_COLUMN_DIP);
    // Path's width is never a constant: the fit below sets it on the initial layout and
    // on every resize, so it is inserted at zero.
    list.insert_column(
        0,
        &translate(msgids::COLUMN_PATH),
        ListColumnFormat::Left,
        0,
    );
    list.insert_column(
        1,
        &translate(msgids::COLUMN_STATUS),
        ListColumnFormat::Left,
        status_width,
    );
    for (index, entry) in session.entries().iter().enumerate() {
        list.insert_item(index as i64, entry.raw(), None);
    }

    let sizer = BoxSizer::builder(Orientation::Vertical).build();
    sizer.add(&list, 1, SizerFlag::Expand | SizerFlag::All, 4);
    page.set_sizer(sizer, true);

    // Path takes all remaining width (spec §12 D2). Lay the page out first so the list's
    // client size is current, then hand the event on. The zero floor is unreachable at the
    // 800×600 window minimum; it only guards degenerate sizes during construction.
    page.on_size(move |event| {
        page.layout();
        let path_width = list.get_client_size().width - status_width;
        list.set_column_width(0, path_width.max(0));
        event.skip(true);
    });

    page
}
