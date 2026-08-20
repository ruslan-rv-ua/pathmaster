//! `announce()`: the application's one voice (spec §10, ADR-0003).
//!
//! This is the only code in the application that fires an accessibility
//! event, and both halves of every announcement happen here so no message can
//! be audio-only: the Banner's `StaticText` shows the text, then
//! `NotifyWinEvent(EVENT_OBJECT_LIVEREGIONCHANGED)` on that control makes NVDA
//! speak it — the one mechanism measured to speak verbatim, every time,
//! repeats included, focus anywhere (wayfinder ticket 08 killed every
//! alternative). The Banner is always visible at a fixed height, so setting
//! its label never reflows the layout and never moves focus.

use std::rc::Rc;

use pathmaster_core::catalogue::{Announcement, Catalogue};
use wxdragon::prelude::*;

use windows_sys::Win32::UI::Accessibility::NotifyWinEvent;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CHILDID_SELF, EVENT_OBJECT_LIVEREGIONCHANGED, OBJID_CLIENT,
};

/// The voice, handed to whatever fires an Announcement: the Banner's widget
/// and the Catalogue that is allowed to put words in it.
///
/// It is not `Copy` — it holds the one Catalogue, shared rather than copied.
/// The trait was claimed so "every closure that speaks can hold its own", and
/// no closure ever did: every one holds an `Rc<App>` and reaches the voice
/// through it.
#[derive(Clone)]
pub struct Announcer {
    banner: StaticText,
    catalogue: Rc<Catalogue>,
}

impl Announcer {
    /// Wraps the Banner's `StaticText` — the caller builds the widget, this
    /// module owns what may be done with it.
    pub fn new(banner: StaticText, catalogue: Rc<Catalogue>) -> Self {
        Announcer { banner, catalogue }
    }

    /// Speaks and shows one Announcement.
    ///
    /// It takes an [`Announcement`] and not a `&str`, which is what closes
    /// ADR-0003's catalogue: the text is composed here, from the Catalogue, so
    /// there is no longer a string in the program that *can* be announced from
    /// outside it.
    pub fn announce(&self, announcement: Announcement) {
        let text = self.catalogue.announcement(announcement);
        self.banner.set_label(&text);
        let hwnd = self.banner.get_handle();
        if hwnd.is_null() {
            return;
        }
        // SAFETY: a fire-and-forget notification on a live window handle; the
        // call takes no pointers that outlive it.
        unsafe {
            NotifyWinEvent(
                EVENT_OBJECT_LIVEREGIONCHANGED,
                hwnd,
                OBJID_CLIENT,
                CHILDID_SELF as i32,
            );
        }
    }
}
