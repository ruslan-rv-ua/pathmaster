//! The shapes a modal message takes (spec §10 dialog discipline, §11).
//!
//! NVDA never speaks a `MessageDialog`'s body, so **everything a dialog has to
//! say is in its title and its buttons**. Every function here takes the title
//! as the message; the body only repeats it, for the eyes.
//!
//! [`choose`] builds its own buttons because a `MessageDialog` cannot relabel
//! its own and `add_std_catalog()` is never called — wx's built-in "Yes"/"No"
//! would stay English in a Ukrainian run. [`warn`] and [`inform`] are the one
//! place a stock button survives: a lone OK carries no meaning of its own to
//! lose. Those two are named for the only thing that differs between them —
//! whether what they say is a warning — since their shape is identical.

use wxdragon::prelude::*;

/// Where this module's button ids start. They matter only inside it:
/// `show_modal` hands one of them straight back, and nothing else in the
/// application binds them.
const ID_FIRST_BUTTON: Id = ID_HIGHEST + 101;

/// Asks a question whose answers are its buttons, and answers with the index
/// of the one chosen.
///
/// **The last button holds the default, the initial focus and the Escape key**,
/// in every use: it is always the outcome that changes least, so the reflexes
/// — Escape, Enter on a dialog whose buttons have not been read yet — land on
/// the safe side. Closing the dialog by its close box is the same answer, and
/// so is any answer this module does not recognise. Focus and default are set
/// together deliberately: Windows gives Enter to the *focused* button, so a
/// default the focus does not sit on is not the answer Enter gives.
///
/// One button is a legal question: the over-length hard cap has nothing to
/// offer but Cancel, and saying so with the same dialog is what keeps that
/// from becoming a second kind of modal (spec §7).
pub fn choose(parent: &dyn WxWidget, title: &str, labels: &[&str]) -> usize {
    let dialog = Dialog::builder(parent, title).build();
    let panel = Panel::builder(&dialog).build();
    let body = StaticText::builder(&panel).with_label(title).build();

    let row = BoxSizer::builder(Orientation::Horizontal).build();
    row.add_stretch_spacer(1);
    let buttons: Vec<Button> = labels
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let button = Button::builder(&panel)
                .with_id(ID_FIRST_BUTTON + index as Id)
                .with_label(label)
                .build();
            row.add(&button, 0, SizerFlag::All, 4);
            button
        })
        .collect();

    let inner = BoxSizer::builder(Orientation::Vertical).build();
    inner.add(&body, 0, SizerFlag::All, 12);
    inner.add_sizer(&row, 0, SizerFlag::Expand | SizerFlag::All, 4);
    panel.set_sizer(inner, true);

    let outer = BoxSizer::builder(Orientation::Vertical).build();
    outer.add(&panel, 1, SizerFlag::Expand, 0);
    dialog.set_sizer_and_fit(outer, true);
    dialog.centre();

    for (index, button) in buttons.iter().enumerate() {
        let id = ID_FIRST_BUTTON + index as Id;
        button.on_click(move |_| dialog.end_modal(id));
    }
    let last = labels.len().saturating_sub(1);
    dialog.set_escape_id(ID_FIRST_BUTTON + last as Id);
    if let Some(safe) = buttons.get(last) {
        safe.set_default();
        safe.set_focus();
    }

    let answer = dialog.show_modal();
    dialog.destroy();
    usize::try_from(answer - ID_FIRST_BUTTON)
        .ok()
        .filter(|index| *index < labels.len())
        .unwrap_or(last)
}

/// Asks a two-button question. `true` means the affirmative button.
pub fn ask(parent: &dyn WxWidget, title: &str, affirmative: &str, negative: &str) -> bool {
    choose(parent, title, &[affirmative, negative]) == 0
}

/// Reports something that went wrong, with a single OK — the whole of it in
/// the title.
///
/// This and [`inform`] are the one shape left with a stock button (spec §11):
/// "OK" is the same word in both shipped languages and carries no meaning we
/// would have to own. The body repeats the title so the message is visible as
/// well as spoken.
pub fn warn(parent: &dyn WxWidget, title: &str) {
    single_ok(parent, title, MessageDialogStyle::IconWarning);
}

/// States something that is simply true, in the same shape.
///
/// The icon is the only difference from [`warn`], and it is not decoration:
/// Windows plays a different sound for each, and About is not a warning about
/// anything. The two are named for that difference rather than for their
/// shape, which is identical.
pub fn inform(parent: &dyn WxWidget, title: &str) {
    single_ok(parent, title, MessageDialogStyle::IconInformation);
}

fn single_ok(parent: &dyn WxWidget, title: &str, icon: MessageDialogStyle) {
    MessageDialog::builder(parent, title, title)
        .with_style(MessageDialogStyle::OK | icon)
        .build()
        .show_modal();
}
