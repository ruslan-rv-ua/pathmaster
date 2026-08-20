//! The two shapes a modal message takes (spec §10 dialog discipline, §11).
//!
//! NVDA never speaks a `MessageDialog`'s body, so **everything a dialog has to
//! say is in its title and its buttons**. Both functions here take the title as
//! the message; the body only repeats it, for the eyes.
//!
//! [`ask`] builds its own buttons because a `MessageDialog` cannot relabel its
//! own and `add_std_catalog()` is never called — wx's built-in "Yes"/"No" would
//! stay English in a Ukrainian run. [`tell`] is the one place a stock button
//! survives: a lone OK carries no meaning of its own to lose.

use wxdragon::prelude::*;

/// The two buttons' ids. They matter only inside this module: `show_modal`
/// hands one of them straight back, and nothing else in the application binds
/// them.
const ID_AFFIRMATIVE: Id = ID_HIGHEST + 101;
const ID_NEGATIVE: Id = ID_HIGHEST + 102;

/// Asks a two-button question. `true` means the affirmative button.
///
/// The negative button holds the default, the initial focus and the Escape
/// key, in every use: it is always the outcome that changes least, so the
/// reflexes — Escape, Enter on a dialog whose buttons have not been read yet
/// — land on the safe side. Closing the dialog by its close box is the same
/// answer. Focus and default are set together deliberately: Windows gives
/// Enter to the *focused* button, so a default the focus does not sit on is
/// not the answer Enter gives.
pub fn ask(parent: &dyn WxWidget, title: &str, affirmative: &str, negative: &str) -> bool {
    let dialog = Dialog::builder(parent, title).build();
    let panel = Panel::builder(&dialog).build();
    let body = StaticText::builder(&panel).with_label(title).build();
    let yes = Button::builder(&panel)
        .with_id(ID_AFFIRMATIVE)
        .with_label(affirmative)
        .build();
    let no = Button::builder(&panel)
        .with_id(ID_NEGATIVE)
        .with_label(negative)
        .build();
    no.set_default();

    let buttons = BoxSizer::builder(Orientation::Horizontal).build();
    buttons.add_stretch_spacer(1);
    buttons.add(&yes, 0, SizerFlag::All, 4);
    buttons.add(&no, 0, SizerFlag::All, 4);

    let inner = BoxSizer::builder(Orientation::Vertical).build();
    inner.add(&body, 0, SizerFlag::All, 12);
    inner.add_sizer(&buttons, 0, SizerFlag::Expand | SizerFlag::All, 4);
    panel.set_sizer(inner, true);

    let outer = BoxSizer::builder(Orientation::Vertical).build();
    outer.add(&panel, 1, SizerFlag::Expand, 0);
    dialog.set_sizer_and_fit(outer, true);
    dialog.centre();

    yes.on_click(move |_| dialog.end_modal(ID_AFFIRMATIVE));
    no.on_click(move |_| dialog.end_modal(ID_NEGATIVE));
    dialog.set_escape_id(ID_NEGATIVE);

    no.set_focus();
    let answer = dialog.show_modal() == ID_AFFIRMATIVE;
    dialog.destroy();
    answer
}

/// States something with a single OK — the whole of it in the title.
///
/// This is the one dialog left with a stock button (spec §11): "OK" is the
/// same word in both shipped languages and carries no meaning we would have to
/// own. The body repeats the title so the message is visible as well as spoken.
pub fn tell(parent: &dyn WxWidget, title: &str) {
    MessageDialog::builder(parent, title, title)
        .with_style(MessageDialogStyle::OK | MessageDialogStyle::IconWarning)
        .build()
        .show_modal();
}
