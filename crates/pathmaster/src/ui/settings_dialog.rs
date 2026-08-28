//! Tools → Settings…: the settings the user may change while the application
//! runs (spec §13, §11 FR-i18n-runtime; v0.2.0 §15).
//!
//! It rides the same measured native path the Add/Edit dialog does — a title,
//! labelled controls built immediately after the `StaticText` that names each,
//! our own buttons — and needs no accessibility call of its own (ADR-0003).
//!
//! **A checkbox is its own label.** The two Filtered View toggles carry their
//! text on the control, where the native path reads it as the control's name
//! and speaks the checked state beside it; a `StaticText` before one would be
//! the same name twice. That is the same rule the labelled controls follow —
//! the visible text *is* the accessible name — arriving at a different shape
//! because the widget already has somewhere to put it.
//!
//! **The restart notice is the selector's own label.** FR-i18n-runtime says the
//! Interface Language applies after a restart, and the Announcement catalogue
//! is closed at fourteen, so the only place left to say so is the label of the
//! control that changes it: "Language (takes effect after restart)". Nothing
//! here re-translates a running window, and pressing OK speaks nothing.
//!
//! **The typed numbers are validated on commit, exactly as a path is.** A
//! rejected number must go back to the field it was typed into with the text
//! intact, so the check lives here rather than at the call site; nothing
//! reaches `settings.json` until the dialog closes. With two such fields the
//! rejection also has to name which — the message is the whole of what is
//! spoken (§10), and "must be a whole number" alone would fit both.
//!
//! **In Read-only Data every control is disabled, the OK button included.**
//! That run has no write path at all, so OK is a button with nothing to do,
//! and a disabled control is how a screen reader is told so (spec §5, §11).
//! What is left is a dialog the settings can still be read out of, and Cancel.

use pathmaster_core::catalogue::Catalogue;
use pathmaster_core::language::LanguageChoice;
use pathmaster_core::msgids;
use pathmaster_core::settings::{parse_count_delay, parse_max_backups, Choices};
use wxdragon::prelude::*;

use crate::catalog::translate;
use crate::ui::{door, question};

/// The two ways out. Local to this module, like the Add/Edit dialog's:
/// `door::show` hands one back and nothing else binds them.
const ID_COMMIT: Id = ID_HIGHEST + 121;
const ID_ABANDON: Id = ID_HIGHEST + 122;

/// The selector's width in DIP. Its own fit would size it to the longest item,
/// which is a translated sentence in one language and a single word in
/// another; a fixed width keeps the dialog the same shape in both. It crosses
/// the FFI boundary, where wxdragon applies `FromDIP` for us.
const SELECTOR_WIDTH_DIP: i32 = 320;

/// The typed fields' width in DIP. Each holds a count, so it is sized for one
/// rather than left to stretch across the dialog like a path — and both are
/// sized alike, because two number fields of different widths would read as
/// two kinds of field.
const NUMBER_WIDTH_DIP: i32 = 90;

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
        .with_size(Size::new(NUMBER_WIDTH_DIP, -1))
        .build();

    // The three Filtered View settings, in §15's own order — which is also the
    // order they read in: the toggle that decides whether a count is spoken,
    // the delay that decides when, then the unrelated one about ESC. A
    // checkbox opens on the setting rather than on `false`, so what it says
    // and what the run is doing are the same thing from the first frame.
    let speak_count = CheckBox::builder(&panel)
        .with_label(&translate(msgids::SETTINGS_SPEAK_FILTERED_COUNT))
        .with_value(opening.speak_filtered_count)
        .build();

    let delay_label = StaticText::builder(&panel)
        .with_label(&translate(msgids::SETTINGS_COUNT_DELAY))
        .build();
    // Left enabled while the count is switched off, and not because the two
    // are unrelated: the debounce is what the *rebuild* waits on as well, so
    // the delay still decides when a narrowed list changes under the typist.
    // A control that greys itself out on a neighbour's state would be one more
    // rule for a screen-reader user to discover by walking into it, and here
    // it would also be saying something untrue.
    let delay = TextCtrl::builder(&panel)
        .with_value(&opening.filtered_count_delay_ms.to_string())
        .with_size(Size::new(NUMBER_WIDTH_DIP, -1))
        .build();

    let escape_returns_focus = CheckBox::builder(&panel)
        .with_label(&translate(msgids::SETTINGS_SEARCH_ESCAPE_RETURNS_FOCUS))
        .with_value(opening.search_escape_returns_focus)
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
    // A checkbox takes the label's own margin, not the control's: it is the
    // label, and a row that reads as a caption should sit where the captions
    // do rather than indented under the one above it.
    inner.add(&speak_count, 0, SizerFlag::Left | SizerFlag::All, 8);
    inner.add(&delay_label, 0, SizerFlag::Left | SizerFlag::All, 8);
    inner.add(&delay, 0, SizerFlag::Left | SizerFlag::All, 4);
    inner.add(
        &escape_returns_focus,
        0,
        SizerFlag::Left | SizerFlag::All,
        8,
    );
    inner.add_sizer(&buttons, 0, SizerFlag::Expand | SizerFlag::All, 8);
    panel.set_sizer(inner, true);

    let outer = BoxSizer::builder(Orientation::Vertical).build();
    outer.add(&panel, 1, SizerFlag::Expand, 0);
    dialog.set_sizer_and_fit(outer, true);
    dialog.centre();

    commit.on_click(move |_| {
        // The fields in the order they are laid out, so the first thing wrong
        // is the first thing the user is sent back to — and one at a time,
        // because two stacked rejections would say the second about a field
        // the user has not been shown yet. Both messages are bare lookups:
        // neither title carries a number or a name, so there is no composition
        // rule for the Catalogue to hold (`pathmaster_core::catalogue`).
        let rejected = if parse_max_backups(&budget.get_value()).is_none() {
            Some((budget, msgids::REJECTED_SNAPSHOTS_TO_KEEP))
        } else if parse_count_delay(&delay.get_value()).is_none() {
            Some((delay, msgids::REJECTED_COUNT_DELAY))
        } else {
            None
        };
        match rejected {
            None => dialog.end_modal(ID_COMMIT),
            Some((field, message)) => {
                question::warn(&dialog, &translate(message));
                // The text is never touched — the user goes back to what they
                // typed, one character away from a legal number.
                field.set_focus();
                field.set_insertion_point_end();
            }
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
        speak_count.enable(false);
        delay.enable(false);
        escape_returns_focus.enable(false);
        commit.enable(false);
        abandon.set_default();
        abandon.set_focus();
    }

    let committed = door::show(&dialog) == ID_COMMIT;
    // Read before the window goes: a destroyed control answers with nothing —
    // which for a checkbox is `false`, a value it could legitimately hold, so
    // the order here is what keeps the two apart rather than a check.
    //
    // All three fallbacks are unreachable — the selector's items are
    // `SELECTABLE` itself, and the only route to `ID_COMMIT` is the handler
    // above, which ends the dialog on two numbers that parsed. Answering with
    // what the dialog opened on is what an unreadable control is worth: it
    // says "unchanged", the one answer that cannot record something nobody
    // chose.
    let chosen = Choices {
        language: language
            .get_selection()
            .and_then(|index| LanguageChoice::at_selector_index(index as usize))
            .unwrap_or(opening.language),
        max_backups: parse_max_backups(&budget.get_value()).unwrap_or(opening.max_backups),
        speak_filtered_count: speak_count.is_checked(),
        filtered_count_delay_ms: parse_count_delay(&delay.get_value())
            .unwrap_or(opening.filtered_count_delay_ms),
        search_escape_returns_focus: escape_returns_focus.is_checked(),
    };
    dialog.destroy();
    committed.then_some(chosen)
}
