//! The `WM_SETTINGCHANGE` broadcast (spec §4, TC-wm-settingchange; ticket
//! impl-13).
//!
//! The call is made for real, against this desktop's own windows — there is no
//! seam here worth inventing, and a mocked `SendMessageTimeoutW` would assert
//! nothing about the one thing that can go wrong, which is a window that does
//! not answer. It is harmless: the message names the environment block and
//! every receiver simply re-reads it, so a broadcast this test sends says
//! truthfully that nothing changed.

#![cfg(windows)]

use std::fs;

use pathmaster_platform::broadcast;

#[test]
fn a_broadcast_runs_on_its_own_thread_and_never_fails_outward() {
    // The handle exists so a caller about to end the process can wait for it;
    // an Apply Run drops it, which is what "off the UI thread" buys.
    broadcast::environment_changed(None)
        .join()
        .expect("the broadcast thread never panics");
}

#[test]
fn a_broadcast_that_lands_says_nothing_at_all() {
    // A `0` return is the only thing worth a line, and it is a `WARN` — so a
    // healthy broadcast must leave the log exactly as it found it. Anything
    // else would be a line per Apply in a five-line healthy skeleton
    // (spec §14).
    let temp = tempfile::tempdir().unwrap();
    let log = temp.path().join("pathmaster.log");
    fs::write(&log, b"").unwrap();

    broadcast::environment_changed(Some(log.clone()))
        .join()
        .expect("the broadcast thread never panics");

    assert_eq!(
        fs::read_to_string(&log).unwrap(),
        "",
        "a broadcast every window answered has nothing to report"
    );
}
