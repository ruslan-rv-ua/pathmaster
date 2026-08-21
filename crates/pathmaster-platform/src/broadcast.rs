//! `WM_SETTINGCHANGE`: telling Windows the environment moved (spec §4,
//! TC-wm-settingchange).
//!
//! **It runs on its own thread, and it logs itself.** `SendMessageTimeoutW`
//! blocks per top-level window, so a machine with a few hundred of them can
//! hold the caller for the whole timeout — which on the UI thread is a window
//! NVDA cannot read. And because the call can still be blocked long after the
//! Apply that started it has returned, its one `WARN` line cannot ride that
//! Apply's outcome: the thread appends past the `Logger`, the way
//! [`panic_hook`](crate::panic_hook) already does.
//!
//! **A timeout is not a failure.** Already-open shells never see the change
//! however this call goes — only newly launched processes do — so nothing is
//! surfaced to the user and nothing about the Apply is undone.

use std::io::Write as _;
use std::path::PathBuf;
use std::thread::{self, JoinHandle};

use pathmaster_core::logfmt::{line, Record};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
};

use crate::logwriter::{append_handle, now};

/// What the `lParam` names: the environment block, not one variable. One
/// broadcast covers both Scopes, which is why an Apply Run sends one however
/// many Scopes it wrote.
const AREA: &str = "Environment";

/// How long any one window may take (spec §4's 1,000–2,000 ms). The PRD's
/// 5,000 is a spec bug: the timeout applies *per* top-level window and
/// multiplies — 226 windows × 5,000 ms is nearly nineteen minutes.
const TIMEOUT_MS: u32 = 2_000;

/// Broadcasts the change on a thread of its own, and answers with its handle.
///
/// The handle exists so this module's own test can wait for the call to come
/// back; an Apply Run drops it. Nothing downstream waits on a notification —
/// the registry already holds the new value — and the thread outliving the run
/// is the whole point of it being a thread.
///
/// `log_path` is where the `WARN` goes, or `None` in a run without a log.
pub fn environment_changed(log_path: Option<PathBuf>) -> JoinHandle<()> {
    thread::spawn(move || {
        if !notify() {
            append(log_path.as_deref(), &Record::broadcast_timed_out());
        }
    })
}

/// The call itself. `true` means every window answered inside the timeout.
fn notify() -> bool {
    // NUL-terminated UTF-16, bound to a local that outlives the call — the
    // pointer is read by every window the message reaches, synchronously,
    // before `SendMessageTimeoutW` returns.
    let area: Vec<u16> = AREA.encode_utf16().chain(std::iter::once(0)).collect();
    let mut answered: usize = 0;
    // SAFETY: `area` is a NUL-terminated UTF-16 buffer alive for the whole
    // call; `answered` is a live local the call writes a scalar into.
    let result = unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            area.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            TIMEOUT_MS,
            &mut answered,
        )
    };
    result != 0
}

/// Appends one record straight to the log file, past the `Logger` — the panic
/// hook's own idiom, and for the same reason: this thread holds no `Logger`
/// and must not fail outward whatever the file does.
fn append(log_path: Option<&std::path::Path>, record: &Record) {
    let Some(log_path) = log_path else { return };
    if let Ok(mut handle) = append_handle(log_path) {
        let _ = handle.write_all(line(&now(), record).as_bytes());
    }
}
