//! Where the window opens (spec §12, impl ticket 15; ADR-0010).
//!
//! Everything here drives the pure half — the arithmetic over monitor
//! rectangles — because that is the half with rules. The other half is one
//! `EnumDisplayMonitors` call, which is the OS fact a test cannot make fail
//! and so is exactly what the seam is drawn around: `place` is handed the work
//! areas rather than asking for them, and a test can therefore arrange a
//! second monitor left of the primary, a monitor that has been unplugged since
//! the window was last closed, and a run that can see no monitors at all.
//!
//! The rule being protected is small and unforgiving: **a window must never
//! open where the user cannot reach it.** A remembered place is a convenience;
//! a window off the edge of every monitor is an application that has to be
//! killed from Task Manager.

#![cfg(windows)]

use pathmaster_core::settings::Window;
use pathmaster_platform::geometry::{place, Placement, WorkArea};

/// A single 1080p monitor with the taskbar taking the bottom 40 pixels — the
/// shape every one-monitor test starts from.
fn primary() -> WorkArea {
    WorkArea {
        x: 0,
        y: 0,
        width: 1920,
        height: 1040,
    }
}

/// A remembered window that was not maximised. The flag is the subject of
/// exactly one test, so every other test says nothing about it.
fn remembered(x: i32, y: i32, width: i32, height: i32) -> Window {
    Window {
        x,
        y,
        width,
        height,
        maximised: false,
    }
}

#[test]
fn a_run_with_nothing_remembered_opens_at_the_default_place() {
    // First run: 900×650, centred (spec §12). The file records choices, and
    // nobody chose this one.
    assert_eq!(place(None, &[primary()]), Placement::Centred);
}

#[test]
fn a_window_that_still_fits_where_it_was_is_put_back_exactly_there() {
    let was = remembered(100, 80, 900, 650);

    assert_eq!(place(Some(was), &[primary()]), Placement::Remembered(was));
}

#[test]
fn a_window_hanging_off_the_edge_is_moved_back_onto_the_work_area() {
    // Its size is kept — the user chose that — and only its position moves,
    // by the least that puts the whole window on screen.
    assert_eq!(
        place(Some(remembered(1700, 900, 900, 650)), &[primary()]),
        Placement::Remembered(remembered(1020, 390, 900, 650)),
    );
}

#[test]
fn a_window_larger_than_the_work_area_is_shrunk_to_it() {
    // The monitor the window was closed on is gone and a smaller one has taken
    // its place. A window taller than the work area would put its own title
    // bar out of reach, which is the one thing this clamp exists to prevent.
    assert_eq!(
        place(Some(remembered(0, 0, 2400, 1400)), &[primary()]),
        Placement::Remembered(remembered(0, 0, 1920, 1040)),
    );
}

#[test]
fn a_window_left_where_no_monitor_is_opens_at_the_default_place() {
    // The second monitor it was closed on has been unplugged (spec §12).
    assert_eq!(
        place(Some(remembered(2400, 100, 900, 650)), &[primary()]),
        Placement::Centred,
    );
}

#[test]
fn a_window_merely_touching_the_edge_is_off_screen_like_any_other() {
    // x = 1920 on a work area of 0..1920 shares an edge and no pixels. There
    // is nothing of it to see, so it is not "mostly on" that monitor — it is
    // off it.
    assert_eq!(
        place(Some(remembered(1920, 0, 900, 650)), &[primary()]),
        Placement::Centred,
    );
}

#[test]
fn a_window_straddling_two_monitors_clamps_onto_the_one_it_shows_most_of() {
    // 120 columns of it on the primary, 280 on the second: the window belongs
    // to the monitor the user is mostly looking at it on, and clamping it to
    // the other would move it away from them.
    let second = WorkArea {
        x: 1920,
        y: 0,
        width: 1920,
        height: 1040,
    };

    assert_eq!(
        place(Some(remembered(1800, 100, 400, 300)), &[primary(), second]),
        Placement::Remembered(remembered(1920, 100, 400, 300)),
    );
}

#[test]
fn a_monitor_left_of_the_primary_is_a_real_place_to_be() {
    // Negative coordinates are ordinary: a monitor arranged left of or above
    // the primary has them, which is why the stored `x` and `y` are signed.
    let left_of_primary = WorkArea {
        x: -1920,
        y: 0,
        width: 1920,
        height: 1040,
    };
    let was = remembered(-1800, 50, 900, 650);

    assert_eq!(
        place(Some(was), &[left_of_primary, primary()]),
        Placement::Remembered(was),
    );
}

#[test]
fn the_maximised_state_rides_through_the_clamp_untouched() {
    // Where a maximised window sits is still worth clamping — it is where it
    // goes the moment the user restores it — but whether it was maximised is
    // not a fact about any monitor.
    let was = Window {
        maximised: true,
        ..remembered(1700, 900, 900, 650)
    };

    assert_eq!(
        place(Some(was), &[primary()]),
        Placement::Remembered(Window {
            maximised: true,
            ..remembered(1020, 390, 900, 650)
        }),
    );
}

#[test]
fn a_run_that_can_see_no_monitors_opens_at_the_default_place() {
    // Not reachable on a machine with a screen, and the answer still has to be
    // the one that cannot strand a window: centred is wx's own problem to
    // solve, and it solves it against whatever it can see.
    assert_eq!(
        place(Some(remembered(0, 0, 900, 650)), &[]),
        Placement::Centred
    );
}
