//! The command line's two dialogs (v0.2.0 §10).
//!
//! A GUI application has no console to print to, so its answer to a
//! command-line question is a dialog — Firefox's own GUI-build help is
//! literally a message box, and this follows it. Both dialogs carry the same
//! usage line in their body, from one Catalogue string, so they can never
//! describe two different command lines.
//!
//! They are the application's whole argument posture towards the user: an
//! argument it does not recognise is named and then ignored, and a request for
//! help is answered and then obeyed by exiting. Neither is a refusal to start,
//! and neither is silence.

use pathmaster_core::catalogue::Catalogue;
use pathmaster_core::msgids;
use wxdragon::prelude::*;

use crate::catalog::{self, translate};
use crate::ui::question;

/// The dialog an unknown argument earns, over the window that started anyway
/// (v0.2.0 §10): the message in the title, the usage line in the body, one OK,
/// and then a normal Run.
///
/// It names **the first** unknown argument, where the log names every one of
/// them. A line of arguments a launcher garbled would otherwise stack a dialog
/// per token in front of a screen-reader user at startup; the dialog's job is
/// to point at the usage line, and the log's is to be the inventory.
pub fn show_unknown_argument(parent: &Frame, argument: &str) {
    // A `Catalogue` over `Installed` holds nothing — it is a view onto the
    // translations wx already has, not a second store (ADR-0009) — so building
    // one here is not a second Catalogue, and the composition rule stays where
    // a test without wx can reach it.
    let catalogue = Catalogue::new(catalog::Installed);
    question::warn_with_body(
        parent,
        &catalogue.unknown_argument_dialog(argument),
        &translate(msgids::USAGE),
    );
}

/// `--help` and `-?`: the usage line, and then exit — a query, not a launch
/// (v0.2.0 §10).
///
/// This Run has no main window, because a query must not start an application,
/// and a `MessageDialog` needs a parent all the same. An unshown [`Frame`] is
/// that parent, and destroying it is also what ends the event loop: wx exits
/// when its last top-level window goes, which is the same door every other exit
/// leaves by.
///
/// Nothing is located, created or read on this path — in particular no Data
/// Directory, which is why the language is the system's rather than the stored
/// choice. The stored choice lives in a directory this query would have to go
/// looking for, and `--data-dir` may have pointed that directory somewhere that
/// does not exist yet: creating it to answer a question about switches is
/// exactly the rudeness "a query, not a launch" rules out.
pub fn show_usage() {
    let frame = Frame::builder()
        .with_title(&translate(msgids::DIALOG_COMMAND_LINE))
        .build();
    question::inform_with_body(
        &frame,
        &translate(msgids::DIALOG_COMMAND_LINE),
        &translate(msgids::USAGE),
    );
    frame.destroy();
}
