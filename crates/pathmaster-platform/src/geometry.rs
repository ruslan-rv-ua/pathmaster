//! Where the window opens: the geometry `settings.json` remembered, measured
//! against the monitors that are actually plugged in (spec §12, §13).
//!
//! **A remembered place is a convenience; a window off the edge of every
//! monitor is an application that has to be killed from Task Manager.** That
//! asymmetry is the whole of this module: nothing here trusts the file, and
//! every answer it cannot justify against a real monitor falls back to the
//! default place, which is the one place that cannot be wrong.
//!
//! The seam is [`work_areas`] — the one `EnumDisplayMonitors` call a test
//! cannot make fail — and everything downstream of it is [`place`], which is
//! arithmetic (ADR-0010). A test then arranges a second monitor left of the
//! primary, a monitor unplugged since the window was last closed, and a run
//! that can see none at all, without needing a machine that has them.
//!
//! **Physical pixels throughout.** wxdragon routes a *builder's* position and
//! size through an implicit `FromDIP` and `set_size_with_pos`, `get_position`
//! and `get_size` through nothing at all — so a geometry round trip is in the
//! same units Windows reports its monitors in, and neither end scales what the
//! other did not. That is why the restore path sets the size on the built
//! frame rather than handing it to the builder: the builder would scale it
//! once more per run (spec §12's implicit-`FromDIP` note).

use pathmaster_core::settings::Window;
use windows_sys::Win32::Foundation::{LPARAM, RECT};
use windows_sys::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
};

/// One monitor's **work area**: the part of it a window may occupy, which is
/// the monitor less the taskbar and any other appbar. Never the monitor's full
/// rectangle — a window restored under the taskbar has its own title bar out
/// of reach.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkArea {
    /// May be negative: a monitor arranged left of or above the primary is a
    /// real place, and so are its coordinates.
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl WorkArea {
    /// How much of `window` this monitor would show, in pixels of area. Zero
    /// covers both "nowhere near" and "sharing an edge and no pixels", which
    /// are the same thing to a user looking for their window.
    ///
    /// **Widened to `i64` before anything is added.** The monitor's own edges
    /// cannot overflow — Windows reported them — but the window's come out of
    /// a hand-editable file that §13 deliberately does not clamp, so `x` and
    /// `width` may each be anything an `i32` can hold and their sum may not
    /// be. A panic here is a hand edit turning into an application that will
    /// not start; widening makes the far edge of the type an ordinary answer
    /// (no overlap) instead of an event.
    fn overlap(&self, window: &Window) -> i64 {
        let span = |low: i64, high: i64, other_low: i64, other_high: i64| {
            (high.min(other_high) - low.max(other_low)).max(0)
        };
        let far = |start: i32, length: i32| i64::from(start) + i64::from(length);
        span(
            i64::from(self.x),
            far(self.x, self.width),
            i64::from(window.x),
            far(window.x, window.width),
        ) * span(
            i64::from(self.y),
            far(self.y, self.height),
            i64::from(window.y),
            far(window.y, window.height),
        )
    }

    /// `window` moved and shrunk by the least that puts all of it on this
    /// monitor. The size is cut only where it does not fit at all — the user
    /// chose that size, and a window narrower than the screen keeps it.
    fn clamp(&self, window: &Window) -> Window {
        let width = window.width.min(self.width);
        let height = window.height.min(self.height);
        Window {
            x: window.x.clamp(self.x, self.x + self.width - width),
            y: window.y.clamp(self.y, self.y + self.height - height),
            width,
            height,
            maximised: window.maximised,
        }
    }
}

/// Where the window is to open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// The default size, centred on the primary monitor (spec §12): a first
    /// run, a window whose remembered place no longer exists, or a run that
    /// can see no monitors at all.
    ///
    /// It carries no numbers because it decides none — 900×650 is the frame's
    /// own builder size and centring is wx's, both of which already happen
    /// without asking this module anything.
    Centred,
    /// What the file remembered, clamped onto the work area showing most of
    /// it.
    Remembered(Window),
}

/// Decides where the window opens, from what the file remembered and what is
/// plugged in now (spec §12).
///
/// The clamp is to **one** monitor — the one showing most of the window —
/// rather than to the union of them all, because the union has holes: two
/// monitors of different heights leave a region that is inside the bounding
/// box and on no screen, and a window clamped into it would be exactly the
/// unreachable window this exists to prevent.
///
/// A remembered `width` or `height` is positive by the time it reaches here
/// (`settings::Window` reads the rest as one invalid field), and nothing here
/// depends on that: a window with no area overlaps nothing and takes the
/// default place like any other window that is nowhere. Nothing bounds them
/// from *above*, though — §13 does not clamp what the file says — which is why
/// [`WorkArea::overlap`] widens before it adds.
pub fn place(remembered: Option<Window>, work_areas: &[WorkArea]) -> Placement {
    let Some(window) = remembered else {
        return Placement::Centred;
    };
    work_areas
        .iter()
        .map(|area| (area.overlap(&window), area))
        .filter(|(overlap, _)| *overlap > 0)
        .max_by_key(|(overlap, _)| *overlap)
        .map_or(Placement::Centred, |(_, area)| {
            Placement::Remembered(area.clamp(&window))
        })
}

/// Every connected monitor's work area, in whatever order Windows enumerates
/// them — [`place`] asks about overlap, not about order.
///
/// A failure to enumerate reads as no monitors, which is the same answer as a
/// machine with none: the default place. There is nothing to log and nothing
/// to tell the user, because nothing has gone wrong from where they sit — the
/// window opens where a first run's window opens.
pub fn work_areas() -> Vec<WorkArea> {
    let mut found: Vec<WorkArea> = Vec::new();
    // SAFETY: the callback runs synchronously, for the duration of this call
    // only, and `found` outlives it — so the pointer it is handed is valid for
    // every invocation.
    unsafe {
        EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null(),
            Some(collect),
            &mut found as *mut Vec<WorkArea> as LPARAM,
        );
    }
    found
}

/// One monitor, appended to the `Vec<WorkArea>` behind `data`. A monitor that
/// will not describe itself is skipped rather than guessed at.
///
/// # Safety
///
/// `data` must be the `*mut Vec<WorkArea>` [`work_areas`] passes, valid for
/// the duration of the enumeration.
unsafe extern "system" fn collect(
    monitor: HMONITOR,
    _hdc: HDC,
    _clip: *mut RECT,
    data: LPARAM,
) -> windows_sys::core::BOOL {
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if GetMonitorInfoW(monitor, &mut info) != 0 {
        let work = info.rcWork;
        (*(data as *mut Vec<WorkArea>)).push(WorkArea {
            x: work.left,
            y: work.top,
            width: work.right - work.left,
            height: work.bottom - work.top,
        });
    }
    // Keep going: every monitor counts, and one that could not be read is not
    // a reason to stop asking about the rest.
    1
}
