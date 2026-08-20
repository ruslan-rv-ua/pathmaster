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

use wxdragon::prelude::*;

use windows_sys::Win32::UI::Accessibility::NotifyWinEvent;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CHILDID_SELF, EVENT_OBJECT_LIVEREGIONCHANGED, OBJID_CLIENT,
};

/// The voice, handed to whatever fires an Announcement. `Copy` like the
/// widget it wraps, so every closure that speaks can hold its own.
#[derive(Clone, Copy)]
pub struct Announcer {
    banner: StaticText,
}

impl Announcer {
    /// Wraps the Banner's `StaticText` — the caller builds the widget, this
    /// module owns what may be done with it.
    pub fn new(banner: StaticText) -> Self {
        Announcer { banner }
    }

    /// Announcement text is Catalogue output, already translated and filled.
    pub fn announce(&self, text: &str) {
        self.banner.set_label(text);
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
