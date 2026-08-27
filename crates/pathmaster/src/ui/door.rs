//! The one modal door (spec §11, ADR-0011): every dialog in the application
//! opens through [`show`], which counts how deep in modal loops the UI thread
//! is, and [`modal_open`] is how the diagnostic Timer's tick knows to be inert.
//!
//! A modal dialog runs a nested event loop, and `WM_TIMER` is dispatched by
//! whatever loop pumps the queue — so without this door the Timer's
//! `collect_pass` runs *inside* an open dialog and takes its borrows there.
//! The gate is on the tick handler, not the Timer: the Timer keeps firing
//! under a dialog, preserving `Pump::request`'s self-healing restart, and a
//! pass that lands mid-dialog is collected by the first tick after it closes,
//! ≤ 100 ms later. Nothing needs to happen "on dialog close".
//!
//! The depth is decremented by a `Drop` guard, so a panic inside a dialog
//! cannot leave the door jammed shut. A source scan
//! (`pathmaster-core/tests/modal_door.rs`) fails the build if `show_modal`
//! appears anywhere in this crate outside this module.

use std::cell::Cell;

use wxdragon::prelude::{Dialog, DirDialog, MessageDialog};

thread_local! {
    /// How many modal loops the UI thread is inside right now. Thread-local
    /// rather than shared: widgets live on the UI thread and so do their
    /// dialogs, and a static would have to pretend otherwise to be `Sync`.
    static DEPTH: Cell<u32> = const { Cell::new(0) };
}

/// Opens a dialog modally, counted: the depth is raised for exactly as long as
/// the dialog's nested event loop runs, and the answer is `show_modal`'s own.
pub fn show(dialog: &impl Modal) -> i32 {
    let _open = Opened::new();
    dialog.run_modal()
}

/// Whether any modal dialog is up — the question the Timer's tick handler asks
/// before doing anything at all.
pub fn modal_open() -> bool {
    DEPTH.with(|depth| depth.get()) > 0
}

/// The raised depth, held for the length of one [`show`]. Its `Drop` is what
/// makes the count panic-safe: an unwind out of a dialog lowers the depth on
/// the way through, where an explicit decrement after the call would be
/// skipped and jam the door shut.
struct Opened;

impl Opened {
    fn new() -> Opened {
        DEPTH.with(|depth| depth.set(depth.get() + 1));
        Opened
    }
}

impl Drop for Opened {
    fn drop(&mut self) {
        DEPTH.with(|depth| depth.set(depth.get() - 1));
    }
}

/// What a dialog is to the door: something that can run a modal loop.
///
/// wxdragon gives each dialog type its own inherent `show_modal`, so the door
/// names the ones this application opens. A new dialog type earns its `impl`
/// here — one line, in the module the source scan already watches — and
/// nothing else about it is hand-checked.
pub trait Modal {
    fn run_modal(&self) -> i32;
}

impl Modal for Dialog {
    fn run_modal(&self) -> i32 {
        self.show_modal()
    }
}

impl Modal for MessageDialog {
    fn run_modal(&self) -> i32 {
        self.show_modal()
    }
}

impl Modal for DirDialog {
    fn run_modal(&self) -> i32 {
        self.show_modal()
    }
}
