//! The one editing surface: a modal dialog with a labelled path field
//! (spec §6, FR-edit-f2, FR-add-delete, FR-browse-folder).
//!
//! F2, Enter and double-click on a row all open it, and so do Add and Edit;
//! `ListCtrlStyle::EditLabels` is deliberately not used, so there is no
//! in-place editor anywhere and no second answer to "how is an Entry typed".
//! The dialog rides the measured native path — a title, a labelled field,
//! buttons — and needs no accessibility call of its own (ADR-0003).
//!
//! Validation lives here rather than in the caller for one reason: a rejected
//! text must go back to *the field it was typed into*, with the text intact.
//! Nothing reaches the Working Copy until the dialog closes, so a rejected
//! edit leaves no Checkpoint — Ctrl+Z into an invalid state is impossible by
//! construction.

use pathmaster_core::msgids::{self, fill};
use pathmaster_core::path::{rejection, Rejection};
use wxdragon::prelude::*;

use crate::catalog::translate;
use crate::ui::question;

/// The two ways out. Local to this module: `show_modal` hands one back and
/// nothing else binds them.
const ID_COMMIT: Id = ID_HIGHEST + 111;
const ID_ABANDON: Id = ID_HIGHEST + 112;

/// The path field's width in DIP. Paths are long; the dialog's own fit would
/// size the field to its initial text, which for Add is nothing at all. It
/// crosses the FFI boundary, where wxdragon applies `FromDIP` for us.
const FIELD_WIDTH_DIP: i32 = 520;

/// Opens the dialog over `initial` and answers with the committed text, or
/// `None` if the user abandoned it.
///
/// `title` is the Catalogue's "Add entry" or "Edit entry" — the same strings
/// Announcement 4 names the operation with, because they name the same thing.
/// Abandoning leaves nothing behind: no Entry, no Checkpoint, no Issue.
pub fn ask_for_entry(parent: &dyn WxWidget, title: &str, initial: &str) -> Option<String> {
    let dialog = Dialog::builder(parent, title).build();
    let panel = Panel::builder(&dialog).build();

    // The label is built immediately before the field it names: that order is
    // what the native comctl32 path reads a control's name from, and it is the
    // whole of the labelling (spec §10 — zero `set_accessibility_*` calls).
    let label = StaticText::builder(&panel)
        .with_label(&translate(msgids::COLUMN_PATH))
        .build();
    let field = TextCtrl::builder(&panel)
        .with_value(initial)
        .with_size(Size::new(FIELD_WIDTH_DIP, -1))
        .build();
    let browse = Button::builder(&panel)
        .with_label(&translate(msgids::BUTTON_BROWSE))
        .build();
    let commit = Button::builder(&panel)
        .with_id(ID_COMMIT)
        .with_label(&translate(msgids::BUTTON_OK))
        .build();
    let abandon = Button::builder(&panel)
        .with_id(ID_ABANDON)
        .with_label(&translate(msgids::BUTTON_DIALOG_CANCEL))
        .build();
    commit.set_default();

    let entry_row = BoxSizer::builder(Orientation::Horizontal).build();
    entry_row.add(&field, 1, SizerFlag::Expand | SizerFlag::All, 4);
    entry_row.add(&browse, 0, SizerFlag::All, 4);

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    buttons.add_stretch_spacer(1);
    buttons.add(&commit, 0, SizerFlag::All, 4);
    buttons.add(&abandon, 0, SizerFlag::All, 4);

    let inner = BoxSizer::builder(Orientation::Vertical).build();
    inner.add(&label, 0, SizerFlag::Left | SizerFlag::All, 8);
    inner.add_sizer(&entry_row, 0, SizerFlag::Expand | SizerFlag::All, 4);
    inner.add_sizer(&buttons, 0, SizerFlag::Expand | SizerFlag::All, 8);
    panel.set_sizer(inner, true);

    let outer = BoxSizer::builder(Orientation::Vertical).build();
    outer.add(&panel, 1, SizerFlag::Expand, 0);
    dialog.set_sizer_and_fit(outer, true);
    dialog.centre();

    browse.on_click(move |_| browse_for_folder(&dialog, &field));
    commit.on_click(move |_| match rejection(&field.get_value()) {
        None => dialog.end_modal(ID_COMMIT),
        Some(reason) => {
            question::tell(&dialog, &rejection_text(reason));
            // The text is never touched — the user goes back to what they
            // typed, one character away from a legal Entry.
            field.set_focus();
            field.set_insertion_point_end();
        }
    });
    abandon.on_click(move |_| dialog.end_modal(ID_ABANDON));
    dialog.set_escape_id(ID_ABANDON);

    field.set_focus();
    field.set_insertion_point_end();
    let committed = dialog.show_modal() == ID_COMMIT;
    // Read before the window goes: a destroyed control answers with nothing.
    let text = field.get_value();
    dialog.destroy();
    committed.then_some(text)
}

/// The error dialog's title, which is the whole of the error (spec §6, §10).
fn rejection_text(reason: Rejection) -> String {
    let message = translate(reason.catalogue_msgid());
    match reason {
        Rejection::Empty => message,
        Rejection::ForbiddenCharacter(character) => {
            fill(&message, &[("character", &character.to_string())])
        }
    }
}

/// Browse: the one native file dialog in the application, and a named
/// exception to "nothing outside our own directory" — `ComDlg32` writes its
/// MRU under HKCU, which the README documents and the release check expects
/// (spec §3, §6 D2).
///
/// The picker seeds from the field only when that text already names a
/// directory; anything else — a path not created yet, a `%VAR%` this side has
/// no business expanding, plain nonsense — leaves the system default alone.
/// The chosen folder replaces the field text, and focus returns to the field.
fn browse_for_folder(parent: &Dialog, field: &TextCtrl) {
    let typed = field.get_value();
    let seed = if std::path::Path::new(&typed).is_dir() {
        typed
    } else {
        String::new()
    };
    let picker = DirDialog::builder(parent, &translate(msgids::DIALOG_CHOOSE_FOLDER), &seed)
        .with_style((DirDialogStyle::Default | DirDialogStyle::MustExist).bits())
        .build();
    if picker.show_modal() == ID_OK {
        if let Some(chosen) = picker.get_path() {
            field.set_value(&chosen);
        }
    }
    // `DirDialog`'s own `Drop` only nulls the pointer — without this the
    // window leaks (ticket 03's measurement).
    picker.destroy();
    field.set_focus();
    field.set_insertion_point_end();
}
