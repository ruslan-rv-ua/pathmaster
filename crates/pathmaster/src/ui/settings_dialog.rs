//! Tools → Settings…: the two settings the user may change while the
//! application runs (spec §13, §11 FR-i18n-runtime).
//!
//! It rides the same measured native path the Add/Edit dialog does — a title,
//! labelled controls built immediately after the `StaticText` that names each,
//! our own buttons — and needs no accessibility call of its own (ADR-0003).
//!
//! **The restart notice is the selector's own label.** FR-i18n-runtime says the
//! Interface Language applies after a restart, and the Announcement catalogue
//! is closed at seven, so the only place left to say so is the label of the
//! control that changes it: "Language (takes effect after restart)". Nothing
//! here re-translates a running window, and pressing OK speaks nothing.
//!
//! **The budget is validated on commit, exactly as a path is.** A rejected
//! number must go back to the field it was typed into with the text intact, so
//! the check lives here rather than at the call site; nothing reaches
//! `settings.json` until the dialog closes.
//!
//! **In Read-only Data every control is disabled, the OK button included.**
//! That run has no write path at all, so OK is a button with nothing to do,
//! and a disabled control is how a screen reader is told so (spec §5, §11).
//! What is left is a dialog the settings can still be read out of, and Cancel.

use pathmaster_core::catalogue::Catalogue;
use pathmaster_core::language::LanguageChoice;
use pathmaster_core::msgids;
use pathmaster_core::settings::{parse_max_backups, Choices};
use wxdragon::prelude::*;

use crate::catalog::translate;
use crate::ui::question;

/// The two ways out. Local to this module, like the Add/Edit dialog's:
/// `show_modal` hands one back and nothing else binds them.
const ID_COMMIT: Id = ID_HIGHEST + 121;
const ID_ABANDON: Id = ID_HIGHEST + 122;

/// The selector's width in DIP. Its own fit would size it to the longest item,
/// which is a translated sentence in one language and a single word in
/// another; a fixed width keeps the dialog the same shape in both. It crosses
/// the FFI boundary, where wxdragon applies `FromDIP` for us.
const SELECTOR_WIDTH_DIP: i32 = 320;

/// The budget field's width in DIP. It holds a count, so it is sized for one
/// rather than left to stretch across the dialog like a path.
const BUDGET_WIDTH_DIP: i32 = 90;

/// Opens the dialog over the settings this run is using and answers with what
/// the user committed, or `None` if they abandoned it.
///
/// `writable` is whether this run has a Data Directory it may write — the one
/// thing the dialog does differently in Read-only Data, where it disables
/// everything and so can only ever answer `None`.
///
/// Abandoning leaves nothing behind, and neither does an OK over untouched
/// controls: what the answer is compared against is the caller's business
/// (`SettingsFile::record_choices`), and this dialog reports what the controls
/// say rather than what changed.
pub fn ask_for_settings(
    parent: &dyn WxWidget,
    catalogue: &Catalogue,
    opening: Choices,
    writable: bool,
) -> Option<Choices> {
    let dialog = Dialog::builder(parent, &translate(msgids::DIALOG_SETTINGS)).build();
    let panel = Panel::builder(&dialog).build();

    // Each label is built immediately before the control it names: that order
    // is what the native comctl32 path reads a control's name from, and it is
    // the whole of the labelling (spec §10 — zero `set_accessibility_*` calls).
    let language_label = StaticText::builder(&panel)
        .with_label(&translate(msgids::SETTINGS_LANGUAGE))
        .build();
    let language = Choice::builder(&panel)
        .with_choices(catalogue.language_items())
        .with_size(Size::new(SELECTOR_WIDTH_DIP, -1))
        .build();
    // By position, from the one list that fixes the order (`SELECTABLE`). A
    // choice with no place in it is unreachable — the items were built from
    // that very list — and reads here as no selection at all.
    if let Some(index) = opening.language.selector_index() {
        language.set_selection(index as u32);
    }

    let budget_label = StaticText::builder(&panel)
        .with_label(&translate(msgids::SETTINGS_SNAPSHOTS_TO_KEEP))
        .build();
    let budget = TextCtrl::builder(&panel)
        .with_value(&opening.max_backups.to_string())
        .with_size(Size::new(BUDGET_WIDTH_DIP, -1))
        .build();

    let commit = Button::builder(&panel)
        .with_id(ID_COMMIT)
        .with_label(&translate(msgids::BUTTON_OK))
        .build();
    let abandon = Button::builder(&panel)
        .with_id(ID_ABANDON)
        .with_label(&translate(msgids::BUTTON_DIALOG_CANCEL))
        .build();

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    buttons.add_stretch_spacer(1);
    buttons.add(&commit, 0, SizerFlag::All, 4);
    buttons.add(&abandon, 0, SizerFlag::All, 4);

    let inner = BoxSizer::builder(Orientation::Vertical).build();
    inner.add(&language_label, 0, SizerFlag::Left | SizerFlag::All, 8);
    inner.add(&language, 0, SizerFlag::Expand | SizerFlag::All, 4);
    inner.add(&budget_label, 0, SizerFlag::Left | SizerFlag::All, 8);
    inner.add(&budget, 0, SizerFlag::Left | SizerFlag::All, 4);
    inner.add_sizer(&buttons, 0, SizerFlag::Expand | SizerFlag::All, 8);
    panel.set_sizer(inner, true);

    let outer = BoxSizer::builder(Orientation::Vertical).build();
    outer.add(&panel, 1, SizerFlag::Expand, 0);
    dialog.set_sizer_and_fit(outer, true);
    dialog.centre();

    commit.on_click(move |_| match parse_max_backups(&budget.get_value()) {
        Some(_) => dialog.end_modal(ID_COMMIT),
        None => {
            // A bare lookup: this title carries no number and no name, so
            // there is no composition rule for the Catalogue to hold and it
            // stays here with the other labels (`pathmaster_core::catalogue`).
            question::tell(&dialog, &translate(msgids::REJECTED_SNAPSHOTS_TO_KEEP));
            // The text is never touched — the user goes back to what they
            // typed, one character away from a legal budget.
            budget.set_focus();
            budget.set_insertion_point_end();
        }
    });
    abandon.on_click(move |_| dialog.end_modal(ID_ABANDON));
    dialog.set_escape_id(ID_ABANDON);

    if writable {
        commit.set_default();
        language.set_focus();
    } else {
        // Nothing here can be committed, so nothing here can be typed into
        // either, and each control says so on its own. Cancel takes the
        // default and the focus together, because Windows gives Enter to the
        // *focused* button (`question::choose`) — and it is the only answer
        // this dialog has left to give.
        language.enable(false);
        budget.enable(false);
        commit.enable(false);
        abandon.set_default();
        abandon.set_focus();
    }

    let committed = dialog.show_modal() == ID_COMMIT;
    // Read before the window goes: a destroyed control answers with nothing.
    //
    // Both fallbacks are unreachable — the selector's items are `SELECTABLE`
    // itself, and the only route to `ID_COMMIT` is the handler above, which
    // ends the dialog on a budget that parsed. Answering with what the dialog
    // opened on is what an unreadable control is worth: it says "unchanged",
    // which is the one answer that cannot record something nobody chose.
    let chosen = Choices {
        language: language
            .get_selection()
            .and_then(|index| LanguageChoice::at_selector_index(index as usize))
            .unwrap_or(opening.language),
        max_backups: parse_max_backups(&budget.get_value()).unwrap_or(opening.max_backups),
    };
    dialog.destroy();
    committed.then_some(chosen)
}
