//! The panic hook (spec §14): one `ERROR panic:` line, appended directly to
//! the log file, best-effort. It deliberately bypasses the `Logger` — no
//! shared state, no lost-record accounting — so a panic anywhere, including
//! inside logging, cannot recurse into it. `set_hook` runs before unwinding
//! and before abort alike, which is how the line survives `panic=abort`.

use std::io::Write as _;
use std::path::PathBuf;

use pathmaster_core::logfmt::{line, Record};

use crate::logwriter::{append_handle, now};

/// Installs the process-wide hook, aimed at the log file the run opened.
/// Installed only when a log exists — a run without a log has nowhere for a
/// panic line either. Every failure inside the hook is swallowed: a panic
/// must abort exactly as it would have, never worse.
pub fn install(log_path: PathBuf) {
    std::panic::set_hook(Box::new(move |info| {
        let message = payload_text(info.payload());
        let (file, line_no) = match info.location() {
            Some(location) => (location.file(), location.line()),
            None => ("unknown", 0),
        };
        let text = line(&now(), &Record::panic(message, file, line_no));
        if let Ok(mut handle) = append_handle(&log_path) {
            let _ = handle.write_all(text.as_bytes());
        }
    }));
}

/// The panic message, when it is a string at all — `panic!` produces `&str`
/// or `String`; anything else (a typed payload) has no text to carry.
fn payload_text(payload: &dyn std::any::Any) -> &str {
    if let Some(text) = payload.downcast_ref::<&str>() {
        text
    } else if let Some(text) = payload.downcast_ref::<String>() {
        text
    } else {
        "non-string panic payload"
    }
}
