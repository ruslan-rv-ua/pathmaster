//! PathMaster — portable Windows PATH editor, built for an NVDA user first.
//!
//! The GUI shell is covered by the Release Checklist, never by automated tests
//! (spec §18, ADR-0007). Accessibility rides the free native comctl32 path:
//! zero `set_accessibility_*` calls anywhere (ADR-0003), and nothing sets a colour.

#![windows_subsystem = "windows"]

mod ui;

use pathmaster_core::logfmt::Record;
use pathmaster_platform::datadir::{self, DataDirState};
use pathmaster_platform::elevation;
use pathmaster_platform::logwriter::Logger;
use pathmaster_platform::panic_hook;

fn main() -> std::process::ExitCode {
    // The startup facts, each decided exactly once per run (spec §3, §9):
    // the Data Directory state and elevation are properties of the run.
    let data = datadir::startup();
    let elevated = elevation::is_elevated();

    // The log lives in the Data Directory, so Read-only Data is a run without
    // a log — and an unopenable log stays a run without a log, never
    // Read-only Data (spec §14).
    let mut logger = match &data {
        DataDirState::Writable(dir) => Logger::open(dir),
        DataDirState::ReadOnly(_) => Logger::disabled(),
    };
    if let Some(log_path) = logger.path() {
        panic_hook::install(log_path.to_path_buf());
    }
    // Interface Language is decided by the settings ticket; until it exists
    // every run speaks the default.
    logger.log(&Record::startup(
        env!("CARGO_PKG_VERSION"),
        elevated,
        data.log_state(),
        "en",
    ));

    // No console to print to (windows subsystem) — a failed toolkit init can
    // only surface as a nonzero exit code (and the panic line, if it panics).
    match wxdragon::main(|_| {
        ui::build_main_window();
    }) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(_) => std::process::ExitCode::FAILURE,
    }
}
