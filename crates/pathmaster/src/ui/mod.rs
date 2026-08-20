//! The main window shell (spec §12): one vertical sizer — Banner above the notebook —
//! with the native status bar attached to the frame outside the sizer.

use pathmaster_core::msgids;
use wxdragon::prelude::*;

use crate::catalog::translate;

/// Status column width in DIP — the app's single deliberate pixel constant (spec §12 D2).
/// Status text is of predictable length (comma-joined one-word Issue types) while paths
/// are unbounded, so Status is fixed and Path takes all remaining width.
const STATUS_COLUMN_DIP: i32 = 220;

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

/// Builds and shows the main window, and hands it back so a startup dialog has
/// a parent to sit on and a window to hand focus back to.
pub fn build_main_window() -> Frame {
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

    let notebook = Notebook::builder(&root).build();
    let user_page = build_scope_page(&notebook);
    let system_page = build_scope_page(&notebook);
    // The Backups tab is not a Scope; its Snapshot list arrives with the backups ticket.
    let backups_page = Panel::builder(&notebook).build();
    notebook.add_page(&user_page, &translate(msgids::TAB_USER), true, None);
    notebook.add_page(&system_page, &translate(msgids::TAB_SYSTEM), false, None);
    notebook.add_page(&backups_page, &translate(msgids::TAB_BACKUPS), false, None);

    let root_sizer = BoxSizer::builder(Orientation::Vertical).build();
    root_sizer.add(&banner, 0, SizerFlag::Expand | SizerFlag::All, 4);
    root_sizer.add(&notebook, 1, SizerFlag::Expand | SizerFlag::All, 4);
    root.set_sizer(root_sizer, true);

    // Command-only (NVDA+End), absent from the Tab order: field 0 general status,
    // field 1 the passive merged-length field (spec §12 D10). Text arrives with diagnostics.
    let status_bar = frame.create_status_bar(2, 0, ID_ANY as Id, "");
    status_bar.set_status_widths(&[-3, -2]);

    frame.centre();
    frame.show(true);
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

/// One Scope tab: a report-mode list with exactly two columns, Path and Status —
/// no index column, no icons (spec §7, §10). Empty until data loading lands.
fn build_scope_page(notebook: &Notebook) -> Panel {
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
