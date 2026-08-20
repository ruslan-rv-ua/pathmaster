//! PathMaster — portable Windows PATH editor, built for an NVDA user first.
//!
//! The GUI shell is covered by the Release Checklist, never by automated tests
//! (spec §18, ADR-0007). Accessibility rides the free native comctl32 path:
//! zero `set_accessibility_*` calls anywhere (ADR-0003), and nothing sets a colour.

#![windows_subsystem = "windows"]

mod catalog;
mod ui;

use pathmaster_core::language;
use pathmaster_core::logfmt::Record;
use pathmaster_platform::datadir::{self, DataDirState};
use pathmaster_platform::elevation;
use pathmaster_platform::locale;
use pathmaster_platform::logwriter::Logger;
use pathmaster_platform::panic_hook;
use pathmaster_platform::settings::{self, Source};

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

    // Startup order (spec §11): Data Directory → settings → translations → UI.
    // Settings are read in both data modes; only the set-aside of an unreadable
    // file is withheld in Read-only Data.
    let loaded = settings::read(&data);
    // Interface Language, decided once per run and never again (spec §11) —
    // from the stored choice, which is a choice and not its outcome.
    let language = language::resolve(loaded.file.language(), locale::system_language());

    // The startup line comes first even though the settings lines describe
    // something that happened before it: it names the build, and until it does,
    // nothing under it means anything. Read in order, the pair explains itself
    // — `language: en` above, and below it the reason it is not what the file
    // asked for.
    logger.log(&Record::startup(
        env!("CARGO_PKG_VERSION"),
        elevated,
        data.log_state(),
        language.code(),
    ));
    let settings_unreadable = match &loaded.source {
        // An absent file is a first run, not a failure: no dialog, no log line.
        Source::Absent => false,
        // A bad field is noise, and the log is its only witness (spec §13).
        Source::Read(rejected) => {
            rejected.iter().for_each(|r| logger.log(&r.record()));
            false
        }
        // An unreadable file is "your edit did not take" — the user is owed a
        // dialog, and a developer reading the log is owed the file's whereabouts.
        Source::Unreadable { set_aside } => {
            logger.log(&Record::settings_unreadable(*set_aside));
            true
        }
    };

    // No console to print to (windows subsystem) — a failed toolkit init can
    // only surface as a nonzero exit code (and the panic line, if it panics).
    match wxdragon::main(move |_| {
        catalog::install(language);
        let frame = ui::build_main_window();
        if settings_unreadable {
            ui::show_settings_unreadable(&frame);
        }
    }) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(_) => std::process::ExitCode::FAILURE,
    }
}
