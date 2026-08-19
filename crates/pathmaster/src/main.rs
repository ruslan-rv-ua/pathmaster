//! PathMaster — portable Windows PATH editor, built for an NVDA user first.
//!
//! The GUI shell is covered by the Release Checklist, never by automated tests
//! (spec §18, ADR-0007). Accessibility rides the free native comctl32 path:
//! zero `set_accessibility_*` calls anywhere (ADR-0003), and nothing sets a colour.

#![windows_subsystem = "windows"]

mod ui;

fn main() -> std::process::ExitCode {
    // No console to print to (windows subsystem) and no logger yet (its own ticket) —
    // a failed toolkit init can only surface as a nonzero exit code.
    match wxdragon::main(|_| {
        ui::build_main_window();
    }) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(_) => std::process::ExitCode::FAILURE,
    }
}
